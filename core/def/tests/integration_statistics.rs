use std::sync::Arc;

use contracts::{DezhResult, Event, EventHandler};

use def::DefFacade;

struct Handler;

impl EventHandler for Handler {
    fn name(&self) -> &'static str {
        "stats"
    }

    fn handle(&self, _: &Event) -> DezhResult<()> {
        Ok(())
    }
}

#[test]
fn statistics_after_publish() {
    let def = DefFacade::new();

    def.subscribe("stats", Arc::new(Handler));

    let event = Event::simple("stats", "");

    def.publish(event).unwrap();

    let stats = def.statistics();

    assert_eq!(stats.published_events, 1);

    assert_eq!(stats.dispatched_events, 1);
}
