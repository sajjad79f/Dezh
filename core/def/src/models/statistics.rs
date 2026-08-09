#[derive(Debug, Clone, Default)]

pub struct Statistics {

    pub published_events: u64,

    pub dispatched_events: u64,

    pub failed_events: u64,

    pub registered_topics: usize,

    pub registered_handlers: usize,

    pub middleware_count: usize,
}

impl Statistics {

    pub fn reset(&mut self) {

        *self = Self::default();
    }
}