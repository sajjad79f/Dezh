pub trait CoreService: Send + Sync {
    fn name(&self) -> &'static str;

    fn initialize(&self) {}

    fn shutdown(&self) {}
}