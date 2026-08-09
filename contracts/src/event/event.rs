use std::collections::HashMap;
use std::time::SystemTime;

use super::{
    CorrelationId, EventId, EventPayload, EventSource, Metadata, Severity, Topic,
};

#[derive(Clone, Debug)]
pub struct Event {
    pub id: EventId,
    pub topic: Topic,
    pub payload: EventPayload,
    pub source: EventSource,
    pub severity: Severity,
    pub correlation_id: CorrelationId,
    pub timestamp: SystemTime,
    pub metadata: Metadata,
}

impl Event {
    pub fn new(
        topic: impl Into<String>,
        payload: impl Into<String>,
        source: EventSource,
    ) -> Self {
        Self {
            id: EventId::new(),
            topic: Topic::new(topic),
            payload: payload.into(),
            source,
            severity: Severity::Info,
            correlation_id: CorrelationId::new(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// نسخه ساده‌تر وقتی source مهم نیست
    pub fn simple(topic: impl Into<String>, payload: impl Into<String>) -> Self {
        Self::new(
            topic,
            payload,
            EventSource::new("system", "unknown"),
        )
    }
}