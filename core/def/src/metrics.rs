use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

pub struct EventMetrics {

    published: AtomicU64,

    dispatched: AtomicU64,
}

impl EventMetrics {

    pub const fn new() -> Self {

        Self {

            published: AtomicU64::new(0),

            dispatched: AtomicU64::new(0),
        }
    }

    pub fn published(
        &self,
    ) {

        self.published.fetch_add(
            1,
            Ordering::Relaxed,
        );
    }

    pub fn dispatched(
        &self,
    ) {

        self.dispatched.fetch_add(
            1,
            Ordering::Relaxed,
        );
    }
}