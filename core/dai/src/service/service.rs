use std::sync::{Arc, RwLock};
use uuid::Uuid;

use contracts::CoreService;
use storage::{AssetRepo, DbPool};

use crate::error::DaiResult;
use crate::models::{Asset, AssetStatus, AssetType};
use crate::registry::AssetRegistry;

pub struct DaiService {
    registry: Arc<AssetRegistry>,
    pool: Arc<RwLock<Option<DbPool>>>,
}

impl DaiService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(AssetRegistry::new()),
            pool: Arc::new(RwLock::new(None)),
        }
    }

    pub fn attach_pool(&self, pool: DbPool) {
        if let Err(e) = self.load_from_db(&pool) {
            eprintln!("[dai] load from db failed: {e}");
        }
        *self.pool.write().expect("dai pool lock") = Some(pool);
        eprintln!("[dai] persistence attached");
    }

    fn block_on<F, T>(f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(f)),
            Err(_) => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("temp runtime");
                rt.block_on(f)
            }
        }
    }

    fn load_from_db(&self, pool: &DbPool) -> Result<(), String> {
        let repo = AssetRepo::new(pool.inner());
        let rows = Self::block_on(repo.list()).map_err(|e| e.to_string())?;

        self.registry.clear();
        for row in rows {
            let asset_type = parse_asset_type(&row.asset_type);
            let status = parse_status(&row.status);
            let asset = Asset {
                id: row.id,
                name: row.name,
                asset_type,
                status,
                metadata: Default::default(),
            };
            self.registry.insert_existing(asset);
        }
        Ok(())
    }

    /// این همان جایی است که بعد از ثبت در RAM، روی DB هم می‌نویسد
    fn persist_insert(&self, id: Uuid, name: &str, asset_type: &str, status: &str) {
        let guard = self.pool.read().expect("dai pool lock");
        let Some(pool) = guard.as_ref() else {
            return; // هنوز pool وصل نشده — فقط RAM
        };
        let repo = AssetRepo::new(pool.inner());
        if let Err(e) = Self::block_on(repo.insert(id, name, asset_type, status)) {
            eprintln!("[dai] persist insert failed: {e}");
        }
    }

    fn persist_status(&self, id: Uuid, status: &str) {
        let guard = self.pool.read().expect("dai pool lock");
        let Some(pool) = guard.as_ref() else {
            return;
        };
        let repo = AssetRepo::new(pool.inner());
        if let Err(e) = Self::block_on(repo.set_status(id, status)) {
            eprintln!("[dai] persist status failed: {e}");
        }
    }

    pub fn register_asset(
        &self,
        name: impl Into<String>,
        asset_type: AssetType,
    ) -> DaiResult<Uuid> {
        let name = name.into();

        // 1) ثبت در حافظه
        let id = self.registry.register(name.clone(), asset_type.clone())?;

        // 2) ثبت در دیتابیس — دقیقاً اینجا
        self.persist_insert(
            id,
            &name,
            &format_asset_type(&asset_type),
            "unknown",
        );

        Ok(id)
    }

    pub fn get_asset(&self, id: Uuid) -> Option<Asset> {
        self.registry.get(id)
    }

    pub fn list_assets(&self) -> Vec<Asset> {
        self.registry.list()
    }

    pub fn set_status(&self, id: Uuid, status: AssetStatus) -> DaiResult<()> {
        self.registry.set_status(id, status.clone())?;
        self.persist_status(id, &format_status(&status));
        Ok(())
    }

    pub fn remove_asset(&self, id: Uuid) -> DaiResult<()> {
        self.registry.remove(id)
        // بعداً می‌توانی DELETE از DB هم اضافه کنی
    }

    pub fn find_by_name(&self, name: &str) -> Option<Asset> {
        self.registry.find_by_name(name)
    }

    pub fn count(&self) -> usize {
        self.registry.count()
    }
}

impl Clone for DaiService {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            pool: Arc::clone(&self.pool),
        }
    }
}

impl CoreService for DaiService {
    fn name(&self) -> &'static str {
        "DAI"
    }

    fn initialize(&self) {
        println!("[DAI] initialized");
    }

    fn shutdown(&self) {
        println!("[DAI] shutdown");
    }
}

fn format_asset_type(t: &AssetType) -> String {
    match t {
        AssetType::Endpoint => "endpoint".into(),
        AssetType::Server => "server".into(),
        AssetType::VirtualMachine => "vm".into(),
        AssetType::Firewall => "firewall".into(),
        AssetType::VpnGateway => "vpn_gateway".into(),
        AssetType::User => "user".into(),
        AssetType::Other(s) => s.clone(),
    }
}

fn parse_asset_type(s: &str) -> AssetType {
    match s {
        "endpoint" => AssetType::Endpoint,
        "server" => AssetType::Server,
        "vm" => AssetType::VirtualMachine,
        "firewall" => AssetType::Firewall,
        "vpn_gateway" => AssetType::VpnGateway,
        "user" => AssetType::User,
        other => AssetType::Other(other.to_string()),
    }
}

fn format_status(s: &AssetStatus) -> String {
    match s {
        AssetStatus::Active => "active".into(),
        AssetStatus::Inactive => "inactive".into(),
        AssetStatus::Unknown => "unknown".into(),
    }
}

fn parse_status(s: &str) -> AssetStatus {
    match s {
        "active" => AssetStatus::Active,
        "inactive" => AssetStatus::Inactive,
        _ => AssetStatus::Unknown,
    }
}