use contracts::{Module, ModuleDescriptor};

pub struct NetworkModule;

impl NetworkModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for NetworkModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "network",
            name: "Network",
            version: "0.1.0",
        }
    }

    fn initialize(&self) {
        println!("[Network] initialized");
    }

    fn start(&self) {
        println!("[Network] started");
    }

    fn stop(&self) {
        println!("[Network] stopped");
    }
}