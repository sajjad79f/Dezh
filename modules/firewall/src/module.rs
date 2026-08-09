use contracts::{Module, ModuleDescriptor};

pub struct FirewallModule;

impl FirewallModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for FirewallModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "firewall",
            name: "Firewall",
            version: "0.1.0",
        }
    }

    fn initialize(&self) {
        println!("[Firewall] initialized");
    }

    fn start(&self) {
        println!("[Firewall] started");
    }

    fn stop(&self) {
        println!("[Firewall] stopped");
    }
}