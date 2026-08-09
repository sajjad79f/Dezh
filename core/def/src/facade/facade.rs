use std::sync::Arc;

use contracts::{Event, EventHandler};

use crate::{
    error::*,
    models::*,
    registry::MiddlewareRef,
    service::DefService,
};

pub struct DefFacade {
    service: Arc<DefService>,
}

impl DefFacade {
    pub fn new() -> Self {
        Self {
            service: Arc::new(DefService::new()),
        }
    }

    pub fn subscribe(
        &self,
        topic: impl Into<String>,
        handler: Arc<dyn EventHandler>,
    ) {
        self.service.subscribe(topic, handler);
    }

    pub fn middleware(&self, middleware: MiddlewareRef) {
        self.service.add_middleware(middleware);
    }

    pub fn publish(&self, event: Event) -> DefResult<PublishResult> {
        self.service.publish(event)
    }

    pub fn statistics(&self) -> Statistics {
        self.service.statistics()
    }

    pub fn health(&self) -> Health {
        self.service.health()
    }
}