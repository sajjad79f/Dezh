use crate::{
    metrics::EventMetrics,
    subscription_registry::SubscriptionRegistry,
};

pub struct DefContext {

    registry: SubscriptionRegistry,

    metrics: EventMetrics,
}

impl DefContext {

    pub fn new() -> Self {

        Self {

            registry: SubscriptionRegistry::new(),

            metrics: EventMetrics::new(),
        }
    }

    pub fn registry(
        &self,
    ) -> &SubscriptionRegistry {

        &self.registry
    }

    pub fn registry_mut(
        &mut self,
    ) -> &mut SubscriptionRegistry {

        &mut self.registry
    }

    pub fn metrics(
        &self,
    ) -> &EventMetrics {

        &self.metrics
    }
}