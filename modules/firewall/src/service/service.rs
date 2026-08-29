use std::sync::{Arc, RwLock};
use uuid::Uuid;

use storage::{DbPool, FirewallRepo, FirewallRuleRow};

use crate::error::{FirewallError, FirewallResult};
use crate::models::{FirewallRule, Protocol, RuleAction, RuleDirection};
use crate::registry::RuleRegistry;

pub struct FirewallService {
    registry: Arc<RuleRegistry>,
    pool: Arc<RwLock<Option<DbPool>>>,
}

impl FirewallService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RuleRegistry::new()),
            pool: Arc::new(RwLock::new(None)),
        }
    }

    /// بعد از ثبت DbPool در dcm صدا زده می‌شود
    pub fn attach_pool(&self, pool: DbPool) {
        // بارگذاری رول‌های ذخیره‌شده
        if let Err(e) = self.load_from_db(&pool) {
            eprintln!("[firewall] load from db failed: {e}");
        }
        *self.pool.write().expect("firewall pool lock") = Some(pool);
        eprintln!("[firewall] persistence attached");
    }

    fn block_on<F, T>(f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(f)),
            Err(_) => {
                // اگر runtime نبود (نادر): runtime موقت
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("temp runtime");
                rt.block_on(f)
            }
        }
    }

    fn load_from_db(&self, pool: &DbPool) -> Result<(), String> {
        let repo = FirewallRepo::new(pool.inner());
        let rows = Self::block_on(repo.list()).map_err(|e| e.to_string())?;

        self.registry.clear();
        for row in rows {
            let rule = row_to_rule(&row)?;
            self.registry.add(rule);
        }
        Ok(())
    }

    fn persist_insert(&self, rule: &FirewallRule) {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else { return };

        let row = rule_to_row(rule);
        let repo = FirewallRepo::new(pool.inner());
        if let Err(e) = Self::block_on(repo.insert(&row)) {
            eprintln!("[firewall] persist insert failed: {e}");
        }
    }

    fn persist_delete(&self, id: Uuid) {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else { return };

        let repo = FirewallRepo::new(pool.inner());
        if let Err(e) = Self::block_on(repo.delete(id)) {
            eprintln!("[firewall] persist delete failed: {e}");
        }
    }

    pub fn add_rule(
        &self,
        name: impl Into<String>,
        action: RuleAction,
        direction: RuleDirection,
        protocol: Protocol,
        source: impl Into<String>,
        destination: impl Into<String>,
        port: Option<u16>,
        priority: u32,
    ) -> FirewallResult<Uuid> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(FirewallError::InvalidRule("name cannot be empty".into()));
        }

        let rule = FirewallRule::new(
            name, action, direction, protocol, source, destination, port, priority,
        );
        let id = rule.id;
        self.registry.add(rule.clone());
        self.persist_insert(&rule);
        Ok(id)
    }

    pub fn list_rules(&self) -> Vec<FirewallRule> {
        self.registry.list()
    }

    pub fn get_rule(&self, id: Uuid) -> Option<FirewallRule> {
        self.registry.get(id)
    }

    pub fn remove_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.remove(id)?;
        self.persist_delete(id);
        Ok(())
    }

    pub fn enable_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.set_enabled(id, true)
        // TODO: persist enabled flag (نیاز به update در FirewallRepo)
    }

    pub fn disable_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.set_enabled(id, false)
    }

    pub fn count(&self) -> usize {
        self.registry.count()
    }

    pub fn block_ip(&self, ip: &str) -> FirewallResult<Uuid> {
        self.add_rule(
            format!("block-{ip}"),
            RuleAction::Drop,
            RuleDirection::Inbound,
            Protocol::Any,
            ip,
            "any",
            None,
            10,
        )
    }

    pub fn unblock_ip(&self, ip: &str) -> FirewallResult<usize> {
        let matching: Vec<Uuid> = self
            .list_rules()
            .into_iter()
            .filter(|r| r.source == ip)
            .map(|r| r.id)
            .collect();

        let n = matching.len();
        for id in matching {
            self.remove_rule(id)?;
        }
        Ok(n)
    }
}

impl Clone for FirewallService {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            pool: Arc::clone(&self.pool),
        }
    }
}

fn rule_to_row(r: &FirewallRule) -> FirewallRuleRow {
    FirewallRuleRow {
        id: r.id,
        name: r.name.clone(),
        action: format!("{:?}", r.action).to_lowercase(),
        direction: format!("{:?}", r.direction).to_lowercase(),
        protocol: format!("{:?}", r.protocol).to_lowercase(),
        source: r.source.clone(),
        destination: r.destination.clone(),
        port: r.port.map(|p| p as i32),
        enabled: r.enabled,
        priority: r.priority as i32,
    }
}

fn row_to_rule(row: &FirewallRuleRow) -> Result<FirewallRule, String> {
    let action = match row.action.as_str() {
        "allow" => RuleAction::Allow,
        "deny" => RuleAction::Deny,
        "drop" => RuleAction::Drop,
        "reject" => RuleAction::Reject,
        other => return Err(format!("unknown action: {other}")),
    };
    let direction = match row.direction.as_str() {
        "inbound" | "in" => RuleDirection::Inbound,
        "outbound" | "out" => RuleDirection::Outbound,
        "both" => RuleDirection::Both,
        other => return Err(format!("unknown direction: {other}")),
    };
    let protocol = match row.protocol.as_str() {
        "tcp" => Protocol::Tcp,
        "udp" => Protocol::Udp,
        "icmp" => Protocol::Icmp,
        "any" => Protocol::Any,
        other => return Err(format!("unknown protocol: {other}")),
    };

    Ok(FirewallRule {
        id: row.id,
        name: row.name.clone(),
        action,
        direction,
        protocol,
        source: row.source.clone(),
        destination: row.destination.clone(),
        port: row.port.and_then(|p| u16::try_from(p).ok()),
        enabled: row.enabled,
        priority: row.priority as u32,
    })
}