pub trait Logger: Send + Sync {

    fn info(&self, message: &str);

    fn warn(&self, message: &str);

    fn error(&self, message: &str);
}