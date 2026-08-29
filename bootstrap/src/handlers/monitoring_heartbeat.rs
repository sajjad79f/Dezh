use std::sync::Arc;

use contracts::{DezhResult, Event, EventHandler};
use dai::DaiService;
use dde::{AutonomyLevel, DdeService};
use def::DefService;
use die::DieService;

pub struct MonitoringHeartbeatHandler {
    dai: DaiService,
    die: DieService,
    dde: DdeService,
}

impl EventHandler for MonitoringHeartbeatHandler {
    fn name(&self) -> &'static str {
        "monitoring-heartbeat"
    }

    fn handle(&self, _event: &Event) -> DezhResult<()> {
        let findings = self.die.analyze_assets(&self.dai.list_assets());

        for finding in findings {
            if let Err(e) = self.dde.submit_decision(
                finding.subject,
                finding.message,
                AutonomyLevel::High,
            ) {
                eprintln!("[monitoring-heartbeat] failed to submit decision: {e}");
            }
        }

        Ok(())
    }
}

pub fn register(
    def: &DefService,
    dai: &DaiService,
    die: &DieService,
    dde: &DdeService,
) {
    def.subscribe(
        "monitoring.heartbeat",
        Arc::new(MonitoringHeartbeatHandler {
            dai: dai.clone(),
            die: die.clone(),
            dde: dde.clone(),
        }),
    );
}