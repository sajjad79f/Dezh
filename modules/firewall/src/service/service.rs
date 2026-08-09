use std::sync::Arc;
use uuid::Uuid;

use crate::error::{FirewallError, FirewallResult};
use crate::models::{FirewallRule, Protocol, RuleAction, RuleDirection};
use crate::registry::RuleRegistry;

pub struct FirewallService {
    registry: Arc<RuleRegistry>,
}

impl FirewallService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RuleRegistry::new()),
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
        Ok(self.registry.add(rule))
    }

    pub fn list_rules(&self) -> Vec<FirewallRule> {
        self.registry.list()
    }

    pub fn get_rule(&self, id: Uuid) -> Option<FirewallRule> {
        self.registry.get(id)
    }

    pub fn remove_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.remove(id)
    }

    pub fn enable_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.set_enabled(id, true)
    }

    pub fn disable_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.set_enabled(id, false)
    }

    pub fn count(&self) -> usize {
        self.registry.count()
    }

    /// convenience: block an IP (inbound drop)
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

    /// remove all rules that match this source IP
    pub fn unblock_ip(&self, ip: &str) -> FirewallResult<usize> {
        let matching: Vec<Uuid> = self
            .list_rules()
            .into_iter()
            .filter(|r| r.source == ip)
            .map(|r| r.id)
            .collect();

        let n = matching.len();
        for id in matching {
            self.registry.remove(id)?;
        }
        Ok(n)
    }
}

impl Clone for FirewallService {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}