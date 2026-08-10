use std::sync::Arc;

use bootstrap::bootstrap;
use shared_kernel::prelude::*;

use storage::{bootstrap_admin, create_pool, DatabaseConfig};
const WEB_CONSOLE_ADDR: &str = "127.0.0.1:7878";

pub struct Application {
    services: ServiceContainer,
    modules: ModuleRegistry,
    commands: CommandRegistry,
}

impl Application {
    pub fn new() -> Self {
        Self {
            services: ServiceContainer::new(),
            modules: ModuleRegistry::new(),
            commands: CommandRegistry::new(),
        }
    }

    fn bootstrap(&mut self) {
        bootstrap(
            &mut self.services,
            &mut self.modules,
            &mut self.commands,
        );
    }

    pub async fn run(mut self) {
        self.bootstrap();

        // Modules, commands, and services are only mutated during
        // bootstrap (above). From here on they are read-only, so it is
        // safe to share them between the blocking CLI shell thread and
        // the async Web Console through a plain Arc.
        let modules = Arc::new(self.modules);
        let commands = Arc::new(self.commands);
        let services = Arc::new(self.services);

        let shell_modules = modules.clone();
        let shell_commands = commands.clone();
        let shell_services = services.clone();

        let shell_handle = tokio::task::spawn_blocking(move || {
            crate::shell::run(&shell_modules, &shell_commands, &shell_services);
        });

        let state = api::AppState {
            modules: modules.clone(),
            commands: commands.clone(),
            services: services.clone(),
        };

        let db_config = DatabaseConfig::from_env();
        match create_pool(&db_config).await {
            Ok(pool) => {
                if let Err(e) = bootstrap_admin(&pool).await {
                    eprintln!("[dcm] bootstrap admin failed: {e}");
                }
                // فعلاً pool را نگه دار — فاز بعد در ServiceContainer
                // services.register(pool);  // اگر TypeId/Arc لازم شد بعداً
                let _pool = pool;
            }
            Err(e) => {
                eprintln!("[dcm] database unavailable: {e} — continuing without persistence");
            }
        }

        let web_handle = tokio::spawn(async move {
            if let Err(err) = api::serve(state, WEB_CONSOLE_ADDR).await {
                eprintln!("Web Console error: {err}");
            }
        });

        // The process exits once the interactive shell exits (e.g. "exit").
        let _ = shell_handle.await;
        modules.stop_all();
        web_handle.abort();
    }
}
