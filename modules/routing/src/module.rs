use contracts::{Module, ModuleDescriptor};

pub struct RoutingModule;

impl RoutingModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for RoutingModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "routing",
            name: "Routing",
            version: "0.1.0",
        }
    }

    fn initialize(&self) {
        println!("[Routing] initialized");
    }

    fn start(&self) {
        println!("[Routing] started");
    }

    fn stop(&self) {
        println!("[Routing] stopped");
    }
}