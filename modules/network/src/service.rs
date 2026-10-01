use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use storage::{DbPool, InterfaceRepo, ZoneRepo};
use uuid::Uuid;

use crate::error::{NetworkError, NetworkResult};
use crate::models::{InterfaceInfo, ZoneInfo};
use crate::system::{detect_system_interfaces, run_ip};

pub struct NetworkService {
    pool: Arc<RwLock<Option<DbPool>>>,
    /// iface name → zone name
    zones: Arc<RwLock<HashMap<String, String>>>,
}

impl NetworkService {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            zones: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn attach_pool(&self, pool: DbPool) {
        if let Err(e) = self.load_zones_from_db(&pool) {
            eprintln!("[network] load iface zones failed: {e}");
        }
        *self.pool.write().expect("network pool lock") = Some(pool);
        eprintln!("[network] persistence attached");
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

    fn pool(&self) -> NetworkResult<DbPool> {
        self.pool
            .read()
            .expect("network pool lock")
            .clone()
            .ok_or_else(|| NetworkError::Internal("database not attached".into()))
    }

    fn load_zones_from_db(&self, pool: &DbPool) -> Result<(), String> {
        let repo = InterfaceRepo::new(pool.inner());
        let rows = Self::block_on(repo.list()).map_err(|e| e.to_string())?;
        let mut map = self.zones.write().expect("zones lock");
        map.clear();
        for r in rows {
            map.insert(r.name, r.zone);
        }
        Ok(())
    }

    // ─── Zones ─────────────────────────────────────────

    pub fn list_zones(&self) -> NetworkResult<Vec<ZoneInfo>> {
        let pool = self.pool()?;
        let rows = Self::block_on(ZoneRepo::new(pool.inner()).list())
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| ZoneInfo {
                id: r.id.to_string(),
                name: r.name,
                display_name: r.display_name,
                accounting: r.accounting,
                description: r.description,
            })
            .collect())
    }

    pub fn create_zone(
        &self,
        name: &str,
        display_name: &str,
        accounting: bool,
        description: &str,
    ) -> NetworkResult<Uuid> {
        let pool = self.pool()?;
        Self::block_on(
            ZoneRepo::new(pool.inner()).create(name, display_name, accounting, description),
        )
        .map_err(|e| NetworkError::Internal(e.to_string()))
    }

    pub fn update_zone(
        &self,
        name: &str,
        display_name: &str,
        accounting: bool,
        description: &str,
    ) -> NetworkResult<()> {
        let pool = self.pool()?;
        Self::block_on(
            ZoneRepo::new(pool.inner()).update(name, display_name, accounting, description),
        )
        .map_err(|e| NetworkError::Internal(e.to_string()))
    }

    pub fn delete_zone(&self, name: &str) -> NetworkResult<()> {
        let pool = self.pool()?;
        // اگر ifaceای روی این زون است، حذف نکن
        let ifaces = Self::block_on(InterfaceRepo::new(pool.inner()).list())
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
        if ifaces.iter().any(|i| i.zone == name) {
            return Err(NetworkError::Internal(format!(
                "zone '{name}' is assigned to an interface"
            )));
        }
        Self::block_on(ZoneRepo::new(pool.inner()).delete(name)).map_err(|e| {
            if matches!(e, storage::StorageError::NotFound(_)) {
                NetworkError::NotFound(name.into())
            } else {
                NetworkError::Internal(e.to_string())
            }
        })
    }

    /// اینترفیس‌هایی که زون‌شان accounting=true دارد
    pub fn accounting_interfaces(&self) -> NetworkResult<Vec<String>> {
        let pool = self.pool()?;
        let acc_zones = Self::block_on(ZoneRepo::new(pool.inner()).accounting_zone_names())
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
        if acc_zones.is_empty() {
            return Ok(vec![]);
        }
        let map = self.zones.read().expect("zones lock");
        Ok(map
            .iter()
            .filter(|(_, z)| acc_zones.iter().any(|a| a == *z))
            .map(|(iface, _)| iface.clone())
            .collect())
    }

    /// اولین اینترفیس accounting (برای nft counter — سازگار با start_accounting فعلی)
    pub fn primary_accounting_interface(&self) -> Option<String> {
        self.accounting_interfaces().ok().and_then(|v| v.into_iter().next())
    }

    // ─── Interfaces ────────────────────────────────────

    pub fn list_interfaces(&self) -> NetworkResult<Vec<InterfaceInfo>> {
        let system = detect_system_interfaces()?;
        let pool = self.pool().ok();
        let mut cfg: HashMap<String, storage::InterfaceRow> = HashMap::new();

        if let Some(ref pool) = pool {
            let repo = InterfaceRepo::new(pool.inner());
            if let Ok(rows) = Self::block_on(repo.list()) {
                for r in rows {
                    cfg.insert(r.name.clone(), r);
                }
            }
            for (name, _, _) in &system {
                let _ = Self::block_on(repo.ensure_known(name));
            }
        }

        let zone_map = self.zones.read().expect("zones lock");
        let mut out = Vec::new();
        for (name, up, addresses) in system {
            let row = cfg.get(&name);
            let zone = zone_map
                .get(&name)
                .cloned()
                .or_else(|| row.map(|r| r.zone.clone()))
                .unwrap_or_else(|| "lan".into());
            out.push(InterfaceInfo {
                name,
                zone,
                up,
                addresses,
                enabled: row.map(|r| r.enabled).unwrap_or(true),
                ipv4_mode: row
                    .map(|r| r.ipv4_mode.clone())
                    .unwrap_or_else(|| "none".into()),
                address_cidr: row.and_then(|r| r.address_cidr.clone()),
                gateway: row.and_then(|r| r.gateway.clone()),
                description: row
                    .map(|r| r.description.clone())
                    .unwrap_or_default(),
            });
        }
        Ok(out)
    }

    pub fn set_zone(&self, iface: &str, zone: &str) -> NetworkResult<()> {
        let zone = zone.trim().to_lowercase();
        let pool = self.pool()?;
        if Self::block_on(ZoneRepo::new(pool.inner()).find_by_name(&zone))
            .map_err(|e| NetworkError::Internal(e.to_string()))?
            .is_none()
        {
            return Err(NetworkError::Internal(format!(
                "unknown zone '{zone}' — create it under Network → Zones"
            )));
        }
        Self::block_on(InterfaceRepo::new(pool.inner()).set_zone(iface, &zone))
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
        self.zones
            .write()
            .expect("zones lock")
            .insert(iface.to_string(), zone);
        Ok(())
    }

    pub fn apply_interface_config(
        &self,
        name: &str,
        zone: &str,
        enabled: bool,
        ipv4_mode: &str,
        address_cidr: Option<&str>,
        gateway: Option<&str>,
        description: &str,
    ) -> NetworkResult<()> {
        let zone = zone.trim().to_lowercase();
        let mode = ipv4_mode.trim().to_lowercase();
        if !matches!(mode.as_str(), "none" | "static" | "dhcp") {
            return Err(NetworkError::Internal("ipv4_mode: none|static|dhcp".into()));
        }
        if mode == "static" && address_cidr.map(|s| s.trim().is_empty()).unwrap_or(true) {
            return Err(NetworkError::Internal("static needs address_cidr".into()));
        }

        let pool = self.pool()?;
        if Self::block_on(ZoneRepo::new(pool.inner()).find_by_name(&zone))
            .map_err(|e| NetworkError::Internal(e.to_string()))?
            .is_none()
        {
            return Err(NetworkError::Internal(format!("unknown zone '{zone}'")));
        }

        Self::block_on(InterfaceRepo::new(pool.inner()).update_config(
            name,
            &zone,
            enabled,
            &mode,
            address_cidr,
            gateway,
            description,
        ))
        .map_err(|e| NetworkError::Internal(e.to_string()))?;

        self.zones
            .write()
            .expect("zones lock")
            .insert(name.to_string(), zone);

        // live apply
        run_ip(&["link", "set", "dev", name, if enabled { "up" } else { "down" }])?;

        if mode == "static" {
            let _ = run_ip(&["addr", "flush", "dev", name, "scope", "global"]);
            if let Some(cidr) = address_cidr {
                run_ip(&["addr", "add", cidr.trim(), "dev", name])?;
            }
            let _ = gateway; // default route → ماژول routing
        } else if mode == "dhcp" {
            let _ = std::process::Command::new("dhclient")
                .args(["-v", name])
                .spawn();
        }

        Ok(())
    }
}

impl Default for NetworkService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for NetworkService {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            zones: Arc::clone(&self.zones),
        }
    }
}