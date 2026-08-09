mod monitoring_heartbeat;

pub use monitoring_heartbeat::*;

use def::DefService;

pub fn register(def: &DefService) {
    monitoring_heartbeat::register(def);
}