use def::DefService;

use crate::handlers;

pub fn register(def: &DefService) {
    handlers::register(def);
}