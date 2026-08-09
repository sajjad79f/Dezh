#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetType {
    Endpoint,
    Server,
    VirtualMachine,
    Firewall,
    VpnGateway,
    User,
    Other(String),
}