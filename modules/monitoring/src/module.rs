use std::sync::Arc;
use std::thread;
use std::time::Duration;

use contracts::{Event, Module, ModuleDescriptor};
use dai::{AssetStatus, AssetType, DaiService};
use def::DefService;

use crate::service::MonitoringService;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

pub struct MonitoringModule {
    def: Arc<DefService>,
    dai: Arc<DaiService>,
}

impl MonitoringModule {
    pub fn new(
        def: Arc<DefService>,
        dai: Arc<DaiService>,
    ) -> Self {
        Self { def, dai }
    }
}

impl Module for MonitoringModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "monitoring",
            name: "Monitoring",
            version: "0.1.0",
        }
    }

    fn initialize(&self) {
        println!("[Monitoring] initialized");
    }

    fn start(&self) {
        println!("[Monitoring] started");

        let def = self.def.clone();
        let dai = self.dai.clone();

        thread::spawn(move || loop {
            let info = MonitoringService::system_info();

            let existing = dai
                .list_assets()
                .into_iter()
                .find(|asset| asset.name == info.hostname);

            let asset_id = match existing {
                Some(asset) => asset.id,
                None => dai.register_asset(info.hostname.clone(), AssetType::Server),
            };

            dai.set_status(asset_id, AssetStatus::Active);

            def.publish(Event::new("monitoring.heartbeat", info.hostname));

            thread::sleep(HEARTBEAT_INTERVAL);
        });
    }

    fn stop(&self) {
        println!("[Monitoring] stopped");
    }
}