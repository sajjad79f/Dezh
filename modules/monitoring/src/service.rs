use crate::models::SystemInfo;

pub struct MonitoringService;

impl MonitoringService {
    pub fn new() -> Self {
        Self
    }

    pub fn start(&self) {
        println!("Monitoring service is running.");
    }

    pub fn system_info() -> SystemInfo {
        SystemInfo {
            hostname: std::env::var("COMPUTERNAME")
                .or_else(|_| std::env::var("HOSTNAME"))
                .unwrap_or_else(|_| "Unknown".to_string()),

            os: std::env::consts::OS.to_string(),

            kernel: "Unknown".to_string(),

            cpu_cores: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),

            total_memory: "Unknown".to_string(),

            used_memory: "Unknown".to_string(),
        }
    }
}