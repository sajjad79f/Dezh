use shared_kernel::prelude::*;

use def::DefService;
use dai::DaiService;
use dkg::DkgService;
use die::DieService;
use dde::DdeService;
use firewall::FirewallService;
use routing::RoutingService;
use accounting::AccountingService;

pub fn register(services: &mut ServiceContainer) {
    services.register(DefService::new());
    services.register(DaiService::new());
    services.register(DkgService::new());
    services.register(DieService::new());
    services.register(DdeService::new());
    services.register(FirewallService::new());
    services.register(RoutingService::new());
    services.register(AccountingService::new());
}