#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub destination: String, // e.g. default | 10.0.0.0/8
    pub gateway: Option<String>,
    pub device: Option<String>,
    pub proto: Option<String>,
    pub metric: Option<u32>,
}