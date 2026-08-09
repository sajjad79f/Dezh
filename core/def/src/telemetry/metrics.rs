use std::sync::atomic::{AtomicU64, Ordering};

use crate::models::Statistics;

#[derive(Default)]
pub struct TelemetryMetrics {

    published: AtomicU64,

    dispatched: AtomicU64,

    failed: AtomicU64,
}

impl TelemetryMetrics {

    pub fn new() -> Self {

        Self::default()
    }

    pub fn record_publish(&self) {

        self.published.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dispatch(&self) {

        self.dispatched.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {

        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn statistics(

        &self,

        handlers: usize,

        topics: usize,

        middleware: usize,

    ) -> Statistics {

        Statistics {

            published_events: self.published.load(Ordering::Relaxed),

            dispatched_events: self.dispatched.load(Ordering::Relaxed),

            failed_events: self.failed.load(Ordering::Relaxed),

            registered_topics: topics,

            registered_handlers: handlers,

            middleware_count: middleware,
        }
    }
}