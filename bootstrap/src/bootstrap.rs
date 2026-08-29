use shared_kernel::prelude::*;

pub fn bootstrap(
    services: &mut ServiceContainer,
    modules: &mut ModuleRegistry,
    commands: &mut CommandRegistry,
) {
    crate::services::register(services);

    crate::modules::register(services, modules);

    crate::events::register(services);

    crate::commands::register(commands);   // ← فقط این آرگومان

    modules.initialize_all();
    modules.start_all();
}