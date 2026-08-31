use contracts::{Module, ModuleDescriptor};

pub struct AccountingModule;

impl AccountingModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for AccountingModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "accounting",
            name: "Accounting",
            version: "0.1.0",
        }
    }

    fn initialize(&self) {
        println!("[Accounting] initialized");
    }

    fn start(&self) {
        println!("[Accounting] started");
    }

    fn stop(&self) {
        println!("[Accounting] stopped");
    }
}