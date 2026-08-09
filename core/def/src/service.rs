use std::sync::Mutex;

use contracts::{CoreService, Event, EventHandler};

use crate::{
    context::DefContext,
    models::EventContext,
    pipeline::EventPipeline,
    stages::{DispatchStage, MetricsStage, RoutingStage, TraceStage, ValidationStage},
};

/// Event Fabric — routes events through a fixed processing pipeline
/// (validate -> trace -> count -> route -> dispatch) to subscribed
/// handlers.
///
/// `context` (registry + metrics) is wrapped in a `Mutex` because the
/// pipeline needs `&mut DefContext` to run a stage, while `DefService`
/// itself is shared read-only (`&self`) through the `ServiceContainer`
/// across the CLI shell thread and the async Web Console (see ADR-0009).
/// The pipeline itself never changes after construction, so it needs no
/// locking.
pub struct DefService {
    context: Mutex<DefContext>,
    pipeline: EventPipeline,
}

impl DefService {
    pub fn new() -> Self {
        let mut pipeline = EventPipeline::new();

        pipeline.add(ValidationStage);
        pipeline.add(TraceStage);
        pipeline.add(MetricsStage);
        pipeline.add(RoutingStage);
        pipeline.add(DispatchStage);

        Self {
            context: Mutex::new(DefContext::new()),
            pipeline,
        }
    }

    pub fn subscribe<H>(
        &self,
        topic: impl Into<String>,
        handler: H,
    )
    where
        H: EventHandler + 'static,
    {
        self.context
            .lock()
            .expect("DEF context lock poisoned")
            .registry_mut()
            .subscribe(topic, handler);
    }

    /// Runs an event through the full pipeline and returns the resulting
    /// context (useful to check e.g. whether it was cancelled by
    /// validation).
    pub fn publish(&self, event: Event) -> EventContext {
        let mut event_context = EventContext::new(event);

        let mut def_context = self
            .context
            .lock()
            .expect("DEF context lock poisoned");

        self.pipeline.execute(&mut def_context, &mut event_context);

        event_context
    }
}

impl CoreService for DefService {
    fn name(&self) -> &'static str {
        "DEF"
    }

    fn initialize(&self) {
        println!("[DEF] initialized");
    }

    fn shutdown(&self) {
        println!("[DEF] shutdown");
    }
}
