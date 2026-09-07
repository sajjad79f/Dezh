use std::sync::{Arc, RwLock};

use firewall::FirewallService;
use storage::{AccountingRepo, AuditRepo, DbPool, IdentityRepo};
use uuid::Uuid;

use crate::error::{AccountingError, AccountingResult};

pub struct AccountingService {
    pool: Arc<RwLock<Option<DbPool>>>,
    firewall: Arc<RwLock<Option<FirewallService>>>,
}

impl AccountingService {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            firewall: Arc::new(RwLock::new(None)),
        }
    }

    pub fn attach_pool(&self, pool: DbPool) {
        *self.pool.write().expect("accounting pool lock") = Some(pool);
        eprintln!("[accounting] database attached");
    }

    pub fn attach_firewall(&self, fw: FirewallService) {
        *self.firewall.write().expect("accounting fw lock") = Some(fw);
        eprintln!("[accounting] firewall attached");
    }

    fn pool(&self) -> AccountingResult<DbPool> {
        self.pool
            .read()
            .expect("accounting pool lock")
            .clone()
            .ok_or(AccountingError::DbUnavailable)
    }

    /// بستن یک نشست با بهترین بایت موجود:
    /// ۱) counterهای nft (خواندن + پاک‌سازی)  ۲) آخرین interim ثبت‌شده در DB
    async fn final_bytes(&self, repo: &AccountingRepo<'_>, sid: Uuid) -> (i64, i64) {
        if let Some(fw) = self.firewall.read().expect("accounting fw lock").clone() {
            match fw.stop_accounting(sid) {
                Ok(v) => return v,
                Err(e) => eprintln!("[accounting] stop_accounting {sid}: {e}"),
            }
        }
        repo.get_bytes(sid).await.unwrap_or((0, 0))
    }

    /// Agent: کاربر فعال → identity + session (+ nft accounting)
    pub async fn user_active(
        &self,
        username: &str,
        hostname: Option<&str>,
        ip_address: Option<&str>,
        os: Option<&str>,
    ) -> AccountingResult<UserActiveOutcome> {
        let username = username.trim();
        if username.is_empty() {
            return Err(AccountingError::Message("username required".into()));
        }

        let pool = self.pool()?;
        let id_repo = IdentityRepo::new(pool.inner());
        let acc_repo = AccountingRepo::new(pool.inner());

        let identity_id = match id_repo.find_by_username(username).await {
            Ok(Some(row)) => {
                if !row.enabled {
                    return Err(AccountingError::IdentityDisabled);
                }
                row.id
            }
            Ok(None) => id_repo
                .create(username, hostname, "agent")
                .await
                .map_err(|e| AccountingError::Message(e.to_string()))?,
            Err(e) => return Err(AccountingError::Message(e.to_string())),
        };

        // reuse هوشمند: نشست بازِ همان IP → reuse؛ IP متفاوت → بستن با بایت واقعی و نشست جدید
        if let Ok(Some((existing_id, existing_ip))) =
            acc_repo.find_open_session_with_ip(identity_id, "agent").await
        {
            let same_host = match (&existing_ip, ip_address) {
                (Some(a), Some(b)) => a == b,
                (None, None) => true,
                _ => false,
            };
            if same_host {
                return Ok(UserActiveOutcome {
                    session_id: existing_id,
                    identity_id,
                    reused: true,
                    accounting_error: None,
                });
            }

            let (bin, bout) = self.final_bytes(&acc_repo, existing_id).await;
            let _ = acc_repo
                .end_session(existing_id, bin, bout, Some("ip-changed"))
                .await;
        }

        let metadata = serde_json::json!({
            "hostname": hostname,
            "os": os,
        });

        let session_id = acc_repo
            .start_session_meta(Some(identity_id), "agent", ip_address, metadata)
            .await
            .map_err(|e| AccountingError::Message(e.to_string()))?;

        let mut accounting_error = None;
        if let Some(ip) = ip_address {
            if let Some(fw) = self.firewall.read().expect("accounting fw lock").as_ref() {
                if let Err(e) = fw.start_accounting(session_id, ip) {
                    accounting_error = Some(e.to_string());
                    eprintln!("[accounting] start_accounting failed: {e}");
                }
            }
        }

        let _ = AuditRepo::new(pool.inner())
            .log(
                Some(username),
                "agent_user_active",
                Some(&session_id.to_string()),
                serde_json::json!({ "ip": ip_address, "hostname": hostname, "os": os }),
            )
            .await;

        Ok(UserActiveOutcome {
            session_id,
            identity_id,
            reused: false,
            accounting_error,
        })
    }

    /// Agent: کاربر غیرفعال → stop nft + end sessions
    pub async fn user_inactive(
        &self,
        username: &str,
        bytes_in: i64,
        bytes_out: i64,
    ) -> AccountingResult<u64> {
        let username = username.trim();
        if username.is_empty() {
            return Err(AccountingError::Message("username required".into()));
        }

        let pool = self.pool()?;
        let id_repo = IdentityRepo::new(pool.inner());
        let acc_repo = AccountingRepo::new(pool.inner());

        let identity_id = match id_repo.find_by_username(username).await {
            Ok(Some(row)) => row.id,
            Ok(None) => return Err(AccountingError::IdentityNotFound),
            Err(e) => return Err(AccountingError::Message(e.to_string())),
        };

        let open_ids = acc_repo
            .list_open_for_identity(identity_id)
            .await
            .map_err(|e| AccountingError::Message(e.to_string()))?;

        let mut closed = 0u64;

        for sid in open_ids {
            let (bin, bout) = self.final_bytes(&acc_repo, sid).await;
            let bin = if bin == 0 && bytes_in != 0 { bytes_in } else { bin };
            let bout = if bout == 0 && bytes_out != 0 { bytes_out } else { bout };

            if acc_repo
                .end_session(sid, bin, bout, Some("agent-inactive"))
                .await
                .is_ok()
            {
                closed += 1;
            }
        }

        let _ = AuditRepo::new(pool.inner())
            .log(
                Some(username),
                "agent_user_inactive",
                Some(&identity_id.to_string()),
                serde_json::json!({ "closed_sessions": closed }),
            )
            .await;

        Ok(closed)
    }

    /// یک دور interim: بایت‌های زنده nft همه نشست‌های باز را در DB می‌نویسد
    pub async fn interim_once(&self) -> usize {
        let Ok(pool) = self.pool() else { return 0 };
        let acc_repo = AccountingRepo::new(pool.inner());
        let Ok(open) = acc_repo.list_open_ids().await else { return 0 };
        let Some(fw) = self.firewall.read().expect("accounting fw lock").clone() else {
            return 0;
        };

        let mut updated = 0;
        for sid in open {
            if let Ok((bin, bout)) = fw.read_accounting(sid) {
                if acc_repo.update_interim(sid, bin, bout).await.is_ok() {
                    updated += 1;
                }
            }
        }
        updated
    }

    /// بستن نشست‌های بی‌صاحب که مدتی interim update نداشته‌اند
    pub async fn close_stale_sessions(&self, max_idle_secs: u64) -> u64 {
        let Ok(pool) = self.pool() else { return 0 };
        let acc_repo = AccountingRepo::new(pool.inner());

        let Ok(stale) = acc_repo
            .close_stale(max_idle_secs.min(i32::MAX as u64) as i32, "lost-accounting")
            .await
        else {
            return 0;
        };

        let n = stale.len() as u64;
        if n > 0 {
            if let Some(fw) = self.firewall.read().expect("accounting fw lock").clone() {
                for sid in stale {
                    let _ = fw.stop_accounting(sid);
                }
            }
            eprintln!("[accounting] closed {n} stale session(s)");
        }
        n
    }
}

#[derive(Debug, Clone)]
pub struct UserActiveOutcome {
    pub session_id: Uuid,
    pub identity_id: Uuid,
    pub reused: bool,
    pub accounting_error: Option<String>,
}

impl Default for AccountingService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AccountingService {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            firewall: Arc::clone(&self.firewall),
        }
    }
}