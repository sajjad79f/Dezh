use contracts::{Module, ModuleDescriptor};

pub struct VpnModule;

impl VpnModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for VpnModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "vpn",
            name: "VPN",
            version: "0.1.0",
        }
    }

    fn initialize(&self) {
        println!("[VPN] initialized");
    }

    fn start(&self) {
        println!("[VPN] started");
    }

    fn stop(&self) {
        println!("[VPN] stopped");
    }
}