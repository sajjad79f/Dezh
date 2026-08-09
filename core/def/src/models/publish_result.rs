use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PublishResult {
    pub delivered: usize,
    pub failed: usize,
    pub duration: Duration,
}

impl PublishResult {
    pub fn success(delivered: usize, duration: Duration) -> Self {
        Self {
            delivered,
            failed: 0,
            duration,
        }
    }

    pub fn failed(delivered: usize, failed: usize, duration: Duration) -> Self {
        Self {
            delivered,
            failed,
            duration,
        }
    }

    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}