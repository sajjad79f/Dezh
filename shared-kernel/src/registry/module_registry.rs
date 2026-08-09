use contracts::{Module, ModuleDescriptor};

pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn register<M>(&mut self, module: M)
    where
        M: Module + 'static,
    {
        self.modules.push(Box::new(module));
    }

    /// تعداد ماژول‌ها
    pub fn count(&self) -> usize {
        self.modules.len()
    }
    pub fn initialize_all(&self) {
        for module in &self.modules {
            module.initialize();
        }
    }

    pub fn start_all(&self) {
        for module in &self.modules {
            module.start();
        }
    }

    pub fn stop_all(&self) {
        for module in self.modules.iter().rev() {
            module.stop();
        }
    }

    pub fn modules(&self) -> &[Box<dyn Module>] {
        &self.modules
    }

    /// فقط اطلاعات ماژول‌ها
    pub fn descriptors(&self) -> Vec<ModuleDescriptor> {
        self.modules
            .iter()
            .map(|m| m.descriptor())
            .collect()
    }

}