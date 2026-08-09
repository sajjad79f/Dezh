#[derive(Debug, Clone)]
pub struct MonitoringStatus {
    pub healthy: bool,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub cpu_cores: usize,
    pub total_memory: String,
    pub used_memory: String,
}