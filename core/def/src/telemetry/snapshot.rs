use crate::models::{Health, Snapshot, Statistics};

pub struct TelemetrySnapshot;

impl TelemetrySnapshot {

    pub fn build(

        statistics: Statistics,

        health: Health,

    ) -> Snapshot {

        Snapshot::new(

            statistics,

            health,
        )
    }
}