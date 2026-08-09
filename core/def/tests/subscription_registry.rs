use std::sync::Arc;

use contracts::{Event, EventHandler, Result};

use def::registry::SubscriptionRegistry;

struct TestHandler;

impl EventHandler for TestHandler {

    fn name(&self) -> &'static str {
        "test"
    }

    fn handle(&self, _: &Event) -> Result<()> {
        Ok(())
    }
}

#[test]
fn register_handler() {

    let registry = SubscriptionRegistry::new();

    registry.register(

        "system.start",

        Arc::new(TestHandler),
    );

    assert_eq!(

        registry.topic_count(),

        1,
    );

    assert_eq!(

        registry.handler_count(),

        1,
    );
}