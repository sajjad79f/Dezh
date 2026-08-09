use contracts::{Event, EventHandler};

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use std::collections::HashMap;
use std::sync::Arc;

pub struct EventBus {
    handlers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn subscribe<H>(
        &mut self,
        topic: &str,
        handler: H,
    )
    where
        H: EventHandler + 'static,
    {
        self.handlers
            .entry(topic.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    pub fn publish(&self, event: Event) {
        if let Some(list) = self.handlers.get(&event.topic) {
            for handler in list {
                handler.handle(&event);
            }
        }
    }
}

pub static GLOBAL_EVENT_BUS: Lazy<Arc<RwLock<EventBus>>> =
    Lazy::new(|| Arc::new(RwLock::new(EventBus::new())));