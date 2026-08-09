use std::sync::Arc;

use contracts::CoreService;
use shared_kernel::prelude::*;

use dai::DaiService;
use dde::DdeService;
use def::DefService;
use die::DieService;
use dkg::DkgService;

use crate::handles::CoreServiceHandles;

pub fn register(
    services: &mut ServiceContainer,
) -> CoreServiceHandles {

    let def_service = Arc::new(DefService::new());
    def_service.initialize();
    services.register(def_service.clone());

    let dai_service = Arc::new(DaiService::new());
    dai_service.initialize();
    services.register(dai_service.clone());

    let dkg_service = Arc::new(DkgService::new());
    dkg_service.initialize();
    services.register(dkg_service.clone());

    let die_service = Arc::new(DieService::new());
    die_service.initialize();
    services.register(die_service.clone());

    let dde_service = Arc::new(DdeService::new());
    dde_service.initialize();
    services.register(dde_service.clone());

    CoreServiceHandles {
        def: def_service,
        dai: dai_service,
        dkg: dkg_service,
        die: die_service,
        dde: dde_service,
    }
}