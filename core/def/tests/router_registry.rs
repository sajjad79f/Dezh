use std::sync::Arc;

use contracts::{DezhResult, Event, EventHandler};

use def::registry::SubscriptionRegistry;

struct Handler;

impl EventHandler for Handler {
    fn name(&self) -> &'static str {
        "handler"
    }

    fn handle(&self, _: &Event) -> DezhResult<()> {
        Ok(())
    }
}

#[test]
fn resolve_handler() {
    let registry = SubscriptionRegistry::new();

    registry.register("firewall.alert", Arc::new(Handler));

    let event = Event::simple("firewall.alert", "");

    let handlers = registry.handlers(&event);

    assert_eq!(handlers.len(), 1);
}
