use std::collections::HashMap;

use contracts::EventHandler;

pub struct SubscriptionRegistry {

    subscriptions: HashMap<
        String,
        Vec<Box<dyn EventHandler>>,
    >,
}

impl SubscriptionRegistry {

    pub fn new() -> Self {

        Self {

            subscriptions: HashMap::new(),
        }
    }

    pub fn subscribe<H>(
        &mut self,
        topic: impl Into<String>,
        handler: H,
    )

    where
        H: EventHandler + 'static,
    {

        self.subscriptions
            .entry(topic.into())
            .or_default()
            .push(Box::new(handler));
    }

    pub fn handlers(
        &self,
        topic: &str,
    ) -> Option<&Vec<Box<dyn EventHandler>>> {

        self.subscriptions.get(topic)
    }

    pub fn topic_count(
        &self,
    ) -> usize {

        self.subscriptions.len()
    }

    pub fn subscription_count(
        &self,
    ) -> usize {

        self.subscriptions
            .values()
            .map(Vec::len)
            .sum()
    }
}