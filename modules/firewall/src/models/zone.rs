#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Lan,
    Wan,
    Dmz,
}

impl Zone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Zone::Lan => "lan",
            Zone::Wan => "wan",
            Zone::Dmz => "dmz",
        }
    }

    pub fn parse(s: &str) -> Option<Zone> {
        match s.to_lowercase().as_str() {
            "lan" => Some(Zone::Lan),
            "wan" => Some(Zone::Wan),
            "dmz" => Some(Zone::Dmz),
            _ => None,
        }
    }
}

impl std::fmt::Display for Zone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub name: String,
    pub zone: Zone,
    pub up: bool,
    pub addresses: Vec<String>,
}