use std::sync::Arc;

use contracts::{Event, EventHandler};

#[derive(Clone)]
pub struct Subscription {

    topic: String,

    handler: Arc<dyn EventHandler>,
}

impl Subscription {

    pub fn new(

        topic: impl Into<String>,

        handler: Arc<dyn EventHandler>,

    ) -> Self {

        Self {

            topic: topic.into(),

            handler,
        }
    }

    pub fn topic(&self) -> &str {

        &self.topic
    }

    pub fn handler(&self) -> Arc<dyn EventHandler> {

        Arc::clone(&self.handler)
    }

    pub fn accepts(

        &self,

        event: &Event,

    ) -> bool {

        self.topic == event.topic.as_str()
    }
}