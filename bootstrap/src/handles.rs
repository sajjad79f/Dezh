use std::sync::Arc;

use dai::DaiService;
use dde::DdeService;
use def::DefService;
use die::DieService;
use dkg::DkgService;

#[derive(Clone)]
pub struct CoreServiceHandles {
    pub def: Arc<DefService>,
    pub dai: Arc<DaiService>,
    pub dkg: Arc<DkgService>,
    pub die: Arc<DieService>,
    pub dde: Arc<DdeService>,
}