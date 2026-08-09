use std::sync::Arc;

use contracts::{Event, EventHandler};

use crate::{
    dispatcher::Dispatcher,
    error::*,
    middleware::MiddlewarePipeline,
    models::{Health, PublishResult},
    registry::{MiddlewareRegistry, SubscriptionRegistry},
    router::Router,
    telemetry::TelemetryMetrics,
};

pub struct DefService {
    subscriptions: Arc<SubscriptionRegistry>,
    middleware: Arc<MiddlewareRegistry>,
    router: Router,
    dispatcher: Dispatcher,
    telemetry: Arc<TelemetryMetrics>,
}

impl Clone for DefService {
    fn clone(&self) -> Self {
        Self {
            subscriptions: Arc::clone(&self.subscriptions),
            middleware: Arc::clone(&self.middleware),
            router: Router::new(Arc::clone(&self.subscriptions)),
            dispatcher: Dispatcher::new(),
            telemetry: Arc::clone(&self.telemetry),
        }
    }
}

impl DefService {
    pub fn new() -> Self {
        let subscriptions = Arc::new(SubscriptionRegistry::new());
        let middleware = Arc::new(MiddlewareRegistry::new());

        Self {
            router: Router::new(subscriptions.clone()),
            dispatcher: Dispatcher::new(),
            telemetry: Arc::new(TelemetryMetrics::new()),
            subscriptions,
            middleware,
        }
    }

    pub fn subscribe(
        &self,
        topic: impl Into<String>,
        handler: Arc<dyn EventHandler>,
    ) {
        self.subscriptions.register(topic, handler);
    }

    pub fn add_middleware(&self, middleware: crate::registry::MiddlewareRef) {
        self.middleware.register(middleware);
    }

    pub fn publish(&self, mut event: Event) -> DefResult<PublishResult> {
        self.telemetry.record_publish();

        MiddlewarePipeline::new(&self.middleware).execute(&mut event)?;

        let handlers = self.router.resolve(&event);

        if handlers.is_empty() {
            return Err(DefError::NoSubscriber);
        }

        let result = self.dispatcher.dispatch(&event, handlers)?;

        for item in &result {
            if item.success {
                self.telemetry.record_dispatch();
            } else {
                self.telemetry.record_failure();
            }
        }

        Ok(PublishResult {
            delivered: result.iter().filter(|x| x.success).count(),
            failed: result.iter().filter(|x| !x.success).count(),
            duration: std::time::Duration::ZERO,
        })
    }

    pub fn health(&self) -> Health {
        Health::healthy()
    }

    pub fn statistics(&self) -> crate::models::Statistics {
        self.telemetry.statistics(
            self.subscriptions.handler_count(),
            self.subscriptions.topic_count(),
            self.middleware.count(),
        )
    }
}