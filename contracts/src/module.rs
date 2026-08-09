#[derive(Clone, Debug)]
pub struct ModuleDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

pub trait Module: Send + Sync {
    fn descriptor(&self) -> ModuleDescriptor;

    fn initialize(&self) {}

    fn start(&self) {}

    fn stop(&self) {}
}