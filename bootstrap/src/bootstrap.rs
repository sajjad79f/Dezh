use shared_kernel::prelude::*;

use crate::{
    commands,
    events,
    modules,
    services,
};

pub fn bootstrap(
    services_container: &mut ServiceContainer,
    module_registry: &mut ModuleRegistry,
    command_registry: &mut CommandRegistry,
) {
    let handles = services::register(services_container);

    modules::register(module_registry, services_container, &handles);
    commands::register(command_registry);

    events::register(&handles);

    // این دو خط قبلاً کلاً جا افتاده بود — ماژول‌ها ثبت می‌شدن ولی
    // initialize/start هیچ‌وقت واقعاً صدا زده نمی‌شد.
    module_registry.initialize_all();
    module_registry.start_all();
}