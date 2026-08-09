use contracts::Logger;

pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn info(&self, message: &str) {
        println!("[INFO ] {}", message);
    }

    fn warn(&self, message: &str) {
        println!("[WARN ] {}", message);
    }

    fn error(&self, message: &str) {
        println!("[ERROR] {}", message);
    }
}