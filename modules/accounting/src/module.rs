use std::sync::Arc;
use std::thread;
use std::time::Duration;

use contracts::{Module, ModuleDescriptor};

use crate::AccountingService;

pub struct AccountingModule {
    service: Arc<AccountingService>,
}

impl AccountingModule {
    pub fn new(service: Arc<AccountingService>) -> Self {
        Self { service }
    }
}

impl Module for AccountingModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "accounting",
            name: "Accounting",
            version: "0.2.0",
        }
    }

    fn initialize(&self) {
        println!("[Accounting] initialized");
    }

    fn start(&self) {
        println!("[Accounting] started");

        let service = self.service.clone();
        let interval = env_secs("DEZH_INTERIM_SECS", 60);
        let max_idle = env_secs("DEZH_STALE_SECS", 1800);

        thread::spawn(move || {
            // ران‌تایم کوچک مخصوص این حلقه؛ تا pool وصل نشده، هر تیک بی‌صدا skip می‌شود
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("accounting interim runtime");

            loop {
                thread::sleep(Duration::from_secs(interval));

                let updated = rt.block_on(service.interim_once());
                let stale = rt.block_on(service.close_stale_sessions(max_idle));

                if updated > 0 || stale > 0 {
                    eprintln!("[accounting] interim updated={updated} stale_closed={stale}");
                }
            }
        });
    }

    fn stop(&self) {
        println!("[Accounting] stopped");
    }
}

fn env_secs(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}