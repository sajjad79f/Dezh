use shared_kernel::prelude::*;

use def::DefService;
use dai::DaiService;
use dkg::DkgService;
use die::DieService;
use dde::DdeService;
use firewall::FirewallService;

pub fn register(services: &mut ServiceContainer) {
    services.register(DefService::new());
    services.register(DaiService::new());
    services.register(DkgService::new());
    services.register(DieService::new());
    services.register(DdeService::new());
    services.register(FirewallService::new());
}