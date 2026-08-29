use dai::DaiService;
use dde::DdeService;
use def::DefService;
use die::DieService;

use crate::handlers;

pub fn register(
    def: &DefService,
    dai: &DaiService,
    die: &DieService,
    dde: &DdeService,
) {
    handlers::register(def, dai, die, dde);
}