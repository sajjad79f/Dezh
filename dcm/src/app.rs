use std::sync::Arc;

use bootstrap::bootstrap;
use shared_kernel::prelude::*;
use storage::{bootstrap_admin, create_pool, DatabaseConfig};

const WEB_CONSOLE_ADDR: &str = "0.0.0.0:7878";

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

    fn bootstrap_app(&mut self) {
        bootstrap(
            &mut self.services,
            &mut self.modules,
            &mut self.commands,
        );
    }

    pub async fn run(mut self) {
        if let Err(e) = storage::run_migrations(pool.inner()).await {
            eprintln!("[dcm] migration failed: {e}");
            // اگر تابع Result برمی‌گرداند، بهتر است اینجا return Err کنی —
            // سیستم بدون اسکیما نباید بالا بیاید
        }
        let db_config = DatabaseConfig::from_env();
        match create_pool(&db_config).await {
            Ok(pool) => {
                eprintln!("[dcm] database connected");
                if let Err(e) = bootstrap_admin(&pool).await {
                    eprintln!("[dcm] bootstrap admin failed: {e}");
                }
                self.services.register(pool);
            }
            Err(e) => {
                eprintln!("[dcm] database unavailable: {e}");
            }
        }

        self.bootstrap_app();

        if let Some(pool) = self.services.resolve::<storage::DbPool>() {
            if let Some(fw) = self.services.resolve::<firewall::FirewallService>() {
                fw.attach_pool((*pool).clone());
            }
            if let Some(dai) = self.services.resolve::<dai::DaiService>() {
                dai.attach_pool((*pool).clone());
            }
            if let Some(acc) = self.services.resolve::<accounting::AccountingService>() {
                acc.attach_pool((*pool).clone());
                if let Some(fw) = self.services.resolve::<firewall::FirewallService>() {
                    acc.attach_firewall((*fw).clone());
                }
            }
        }

        let modules = Arc::new(self.modules);
        let commands = Arc::new(self.commands);
        let services = Arc::new(self.services);

        let state = api::AppState {
            modules: modules.clone(),
            commands: commands.clone(),
            services: services.clone(),
        };

        let web_handle = tokio::spawn(async move {
            if let Err(err) = api::serve(state, WEB_CONSOLE_ADDR).await {
                eprintln!("Web Console error: {err}");
            }
        });

        let stdin_is_tty = std::io::IsTerminal::is_terminal(&std::io::stdin());

        if stdin_is_tty {
            let shell_modules = modules.clone();
            let shell_commands = commands.clone();
            let shell_services = services.clone();

            let shell_handle = tokio::task::spawn_blocking(move || {
                crate::shell::run(&shell_modules, &shell_commands, &shell_services);
            });

            let _ = shell_handle.await;
            modules.stop_all();
            web_handle.abort();
        } else {
            eprintln!("[dcm] no TTY — interactive shell disabled (service mode)");
            let _ = web_handle.await;
            modules.stop_all();
        }
    }
}