use shared_kernel::prelude::*;

use def::DefService;

pub fn register(

    services: &ServiceContainer,

) {

    let def =

        services

            .resolve::<DefService>()

            .expect("DEF service not registered");

    crate::handles::register(def);
}