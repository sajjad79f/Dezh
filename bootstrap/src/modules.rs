use std::sync::Arc;

use shared_kernel::prelude::*;
use def::DefService;
use dai::DaiService;

use firewall::FirewallModule;
use monitoring::MonitoringModule;

pub fn register(
    services: &ServiceContainer,
    modules: &mut ModuleRegistry,
) {
    let def = services
        .resolve::<DefService>()
        .expect("DEF service not registered")
        .clone();

    let dai = services
        .resolve::<DaiService>()
        .expect("DAI service not registered")
        .clone();

    modules.register(FirewallModule::new());

    modules.register(MonitoringModule::new(
        Arc::new(def),
        Arc::new(dai),
    ));
}