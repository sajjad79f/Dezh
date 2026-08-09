use std::{
    collections::HashMap,
    time::SystemTime,
};

use contracts::Event;

#[derive(Debug, Clone)]
pub struct EventContext {

    pub event: Event,

    pub received_at: SystemTime,

    pub dispatched_at: Option<SystemTime>,

    pub correlation_id: Option<String>,

    pub cancelled: bool,

    pub metadata: HashMap<String, String>,
}

impl EventContext {

    pub fn new(
        event: Event,
    ) -> Self {

        Self {

            event,

            received_at: SystemTime::now(),

            dispatched_at: None,

            correlation_id: None,

            cancelled: false,

            metadata: HashMap::new(),
        }
    }

    pub fn cancel(
        &mut self,
    ) {

        self.cancelled = true;
    }

    pub fn dispatched(
        &mut self,
    ) {

        self.dispatched_at = Some(SystemTime::now());
    }

    pub fn insert<K, V>(
        &mut self,
        key: K,
        value: V,
    )

    where
        K: Into<String>,
        V: Into<String>,
    {

        self.metadata.insert(
            key.into(),
            value.into(),
        );
    }

    pub fn get(
        &self,
        key: &str,
    ) -> Option<&String> {

        self.metadata.get(key)
    }
}