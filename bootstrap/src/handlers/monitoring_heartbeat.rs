use std::sync::Arc;

use contracts::{Event, EventHandler, DezhResult};
use def::DefService;

pub struct MonitoringHeartbeatHandler;

impl EventHandler for MonitoringHeartbeatHandler {
    fn name(&self) -> &'static str {
        "monitoring-heartbeat"
    }

    fn handle(&self, _: &Event) -> DezhResult<()> {
        println!("Heartbeat received.");
        Ok(())
    }
}

pub fn register(def: &DefService) {
    def.subscribe(
        "monitoring.heartbeat",
        Arc::new(MonitoringHeartbeatHandler),
    );
}