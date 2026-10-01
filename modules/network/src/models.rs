#[derive(Debug, Clone)]
pub struct ZoneInfo {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub accounting: bool,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub name: String,
    pub zone: String,
    pub up: bool,
    pub addresses: Vec<String>,
    pub enabled: bool,
    pub ipv4_mode: String,
    pub address_cidr: Option<String>,
    pub gateway: Option<String>,
    pub description: String,
}