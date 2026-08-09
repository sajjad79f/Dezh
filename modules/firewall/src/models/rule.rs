use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleAction {
    Allow,
    Deny,
    Drop,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleDirection {
    Inbound,
    Outbound,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Any,
    Tcp,
    Udp,
    Icmp,
}

#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub id: Uuid,
    pub name: String,
    pub action: RuleAction,
    pub direction: RuleDirection,
    pub protocol: Protocol,
    pub source: String,      // IP یا CIDR یا "any"
    pub destination: String, // IP یا CIDR یا "any"
    pub port: Option<u16>,
    pub enabled: bool,
    pub priority: u32,       // عدد کمتر = اولویت بالاتر
}

impl FirewallRule {
    pub fn new(
        name: impl Into<String>,
        action: RuleAction,
        direction: RuleDirection,
        protocol: Protocol,
        source: impl Into<String>,
        destination: impl Into<String>,
        port: Option<u16>,
        priority: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            action,
            direction,
            protocol,
            source: source.into(),
            destination: destination.into(),
            port,
            enabled: true,
            priority,
        }
    }
}