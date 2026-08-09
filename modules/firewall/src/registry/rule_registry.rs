use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::error::{FirewallError, FirewallResult};
use crate::models::FirewallRule;

#[derive(Clone, Default)]
pub struct RuleRegistry {
    rules: Arc<RwLock<Vec<FirewallRule>>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, rule: FirewallRule) -> Uuid {
        let id = rule.id;
        let mut rules = self.rules.write().expect("firewall registry lock poisoned");
        rules.push(rule);
        rules.sort_by_key(|r| r.priority);
        id
    }

    pub fn list(&self) -> Vec<FirewallRule> {
        self.rules
            .read()
            .expect("firewall registry lock poisoned")
            .clone()
    }

    pub fn get(&self, id: Uuid) -> Option<FirewallRule> {
        self.rules
            .read()
            .expect("firewall registry lock poisoned")
            .iter()
            .find(|r| r.id == id)
            .cloned()
    }

    pub fn remove(&self, id: Uuid) -> FirewallResult<()> {
        let mut rules = self.rules.write().expect("firewall registry lock poisoned");
        let before = rules.len();
        rules.retain(|r| r.id != id);
        if rules.len() == before {
            return Err(FirewallError::RuleNotFound(id));
        }
        Ok(())
    }

    pub fn set_enabled(&self, id: Uuid, enabled: bool) -> FirewallResult<()> {
        let mut rules = self.rules.write().expect("firewall registry lock poisoned");
        let rule = rules
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(FirewallError::RuleNotFound(id))?;
        rule.enabled = enabled;
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.rules
            .read()
            .expect("firewall registry lock poisoned")
            .len()
    }

    pub fn clear(&self) {
        self.rules
            .write()
            .expect("firewall registry lock poisoned")
            .clear();
    }
}