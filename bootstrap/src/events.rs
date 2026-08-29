use shared_kernel::prelude::*;

use dai::DaiService;
use dde::DdeService;
use def::DefService;
use die::DieService;

pub fn register(services: &ServiceContainer) {
    let def = services
        .resolve::<DefService>()
        .expect("DEF service not registered");

    let dai = services
        .resolve::<DaiService>()
        .expect("DAI service not registered");

    let die = services
        .resolve::<DieService>()
        .expect("DIE service not registered");

    let dde = services
        .resolve::<DdeService>()
        .expect("DDE service not registered");

    crate::handles::register(def, dai, die, dde);
}