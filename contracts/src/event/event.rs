#[derive(Debug, Clone)]
pub struct Event {
    pub topic: String,
    pub payload: String,
}

impl Event {
    pub fn new(
        topic: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
        }
    }
}