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

    /// Agent: کاربر فعال → identity + session (+ nft accounting)
    pub async fn user_active(
        &self,
        username: &str,
        hostname: Option<&str>,
        ip_address: Option<&str>,
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

        if let Ok(Some(existing_id)) = acc_repo.find_open_session(identity_id, "agent").await {
            return Ok(UserActiveOutcome {
                session_id: existing_id,
                identity_id,
                reused: true,
                accounting_error: None,
            });
        }

        let session_id = acc_repo
            .start_session(Some(identity_id), "agent", ip_address)
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
                serde_json::json!({ "ip": ip_address }),
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

        let fw = self.firewall.read().expect("accounting fw lock").clone();
        let mut closed = 0u64;

        for sid in open_ids {
            let (bin, bout) = if let Some(ref fw) = fw {
                match fw.stop_accounting(sid) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[accounting] stop_accounting {sid}: {e}");
                        (bytes_in, bytes_out)
                    }
                }
            } else {
                (bytes_in, bytes_out)
            };

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