use std::sync::Arc;

use firewall::{FirewallModule, FirewallService};
use monitoring::MonitoringModule;
use vpn::VpnModule;

use shared_kernel::prelude::*;

use crate::handles::CoreServiceHandles;

pub fn register(
    registry: &mut ModuleRegistry,
    services: &mut ServiceContainer,
    handles: &CoreServiceHandles,
) {
    registry.register(MonitoringModule::new(
        handles.def.clone(),
        handles.dai.clone(),
    ));

    let firewall_service = Arc::new(FirewallService::new());
    services.register(firewall_service);

    registry.register(FirewallModule);

    registry.register(VpnModule);
}