use std::sync::Arc;

use shared_kernel::prelude::*;
use def::DefService;
use dai::DaiService;

use firewall::FirewallModule;
use monitoring::MonitoringModule;
use routing::RoutingModule;
use accounting::{AccountingModule, AccountingService};

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
    modules.register(RoutingModule::new());

    // اکانتینگ به سرویس خودش وصل می‌شود تا interim/GC task بتواند راه بیفتد
    let accounting = services
        .resolve::<AccountingService>()
        .expect("AccountingService not registered")
        .clone();
    modules.register(AccountingModule::new(Arc::new(accounting)));

    modules.register(MonitoringModule::new(
        Arc::new(def),
        Arc::new(dai),
    ));
}