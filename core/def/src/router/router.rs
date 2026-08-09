use std::sync::Arc;

use contracts::{Event, EventHandler};

use crate::registry::SubscriptionRegistry;

pub struct Router {

    registry:

        Arc<SubscriptionRegistry>,
}

impl Router {

    pub fn new(

        registry: Arc<SubscriptionRegistry>,

    ) -> Self {

        Self {

            registry,
        }
    }

    pub fn resolve(

        &self,

        event: &Event,

    ) -> Vec<Arc<dyn EventHandler>> {

        self.registry

            .handlers(event)
    }

    pub fn registry(

        &self,

    ) -> &Arc<SubscriptionRegistry> {

        &self.registry
    }
}