use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use contracts::{Event, EventHandler};

use super::Subscription;

#[derive(Default)]
pub struct SubscriptionRegistry {

    subscriptions:

        RwLock<HashMap<String, Vec<Subscription>>>,
}

impl SubscriptionRegistry {

    pub fn new() -> Self {

        Self::default()
    }

    pub fn register(

        &self,

        topic: impl Into<String>,

        handler: Arc<dyn EventHandler>,

    ) {

        let topic = topic.into();

        let mut registry =

            self.subscriptions.write().unwrap();

        registry

            .entry(topic.clone())

            .or_default()

            .push(

                Subscription::new(

                    topic,

                    handler,
                ),
            );
    }

    pub fn handlers(

        &self,

        event: &Event,

    ) -> Vec<Arc<dyn EventHandler>> {

        self.subscriptions

            .read()

            .unwrap()

            .get(event.topic.as_str())

            .map(|subs| {

                subs.iter()

                    .map(|s| s.handler())

                    .collect()

            })

            .unwrap_or_default()
    }

    pub fn topic_count(&self) -> usize {

        self.subscriptions

            .read()

            .unwrap()

            .len()
    }

    pub fn handler_count(&self) -> usize {

        self.subscriptions

            .read()

            .unwrap()

            .values()

            .map(Vec::len)

            .sum()
    }
}