mod monitoring_heartbeat;

pub use monitoring_heartbeat::*;

use dai::DaiService;
use dde::DdeService;
use def::DefService;
use die::DieService;

pub fn register(
    def: &DefService,
    dai: &DaiService,
    die: &DieService,
    dde: &DdeService,
) {
    monitoring_heartbeat::register(def, dai, die, dde);
}