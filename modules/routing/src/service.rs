use std::process::Command;
use std::sync::{Arc, RwLock};

use storage::{DbPool, StaticRouteRepo, StaticRouteRow};
use uuid::Uuid;

use crate::error::{RoutingError, RoutingResult};
use crate::models::RouteEntry;

pub struct RoutingService {
    pool: Arc<RwLock<Option<DbPool>>>,
}

impl RoutingService {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
        }
    }

    pub fn attach_pool(&self, pool: DbPool) {
        *self.pool.write().expect("routing pool lock") = Some(pool);
        if let Err(e) = self.apply_persisted_routes() {
            eprintln!("[routing] apply persisted routes failed: {e}");
        } else {
            eprintln!("[routing] persisted routes applied");
        }
        eprintln!("[routing] persistence attached");
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

    fn pool(&self) -> Option<DbPool> {
        self.pool.read().expect("routing pool lock").clone()
    }

    fn run_ip(args: &[&str]) -> RoutingResult<String> {
        let sudo = Command::new("sudo")
            .arg("-n")
            .arg("ip")
            .args(args)
            .output();

        if let Ok(output) = sudo {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
            }
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if err.contains("RTNETLINK")
                || err.contains("File exists")
                || err.contains("No such")
            {
                return Err(RoutingError::Internal(err));
            }
        }

        let output = Command::new("ip")
            .args(args)
            .output()
            .map_err(|e| RoutingError::Internal(format!("ip failed: {e}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if err.contains("Operation not permitted") {
                return Err(RoutingError::Internal(
                    "Operation not permitted — run Dezh as root or CAP_NET_ADMIN".into(),
                ));
            }
            Err(RoutingError::Internal(err))
        }
    }

    pub fn list_routes(&self) -> RoutingResult<Vec<RouteEntry>> {
        let out = Self::run_ip(&["-4", "route", "show"])?;
        let mut routes = Vec::new();

        for line in out.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let destination = parts[0].to_string();
            let mut gateway = None;
            let mut device = None;
            let mut proto = None;
            let mut metric = None;

            let mut i = 1;
            while i < parts.len() {
                match parts[i] {
                    "via" if i + 1 < parts.len() => {
                        gateway = Some(parts[i + 1].to_string());
                        i += 2;
                    }
                    "dev" if i + 1 < parts.len() => {
                        device = Some(parts[i + 1].to_string());
                        i += 2;
                    }
                    "proto" if i + 1 < parts.len() => {
                        proto = Some(parts[i + 1].to_string());
                        i += 2;
                    }
                    "metric" if i + 1 < parts.len() => {
                        metric = parts[i + 1].parse().ok();
                        i += 2;
                    }
                    _ => i += 1,
                }
            }

            routes.push(RouteEntry {
                destination,
                gateway,
                device,
                proto,
                metric,
            });
        }

        Ok(routes)
    }

    /// اضافه کردن route زنده + ذخیره در DB
    pub fn add_route(
        &self,
        destination: &str,
        gateway: Option<&str>,
        device: Option<&str>,
    ) -> RoutingResult<()> {
        if destination.trim().is_empty() {
            return Err(RoutingError::Invalid("destination required".into()));
        }
        if gateway.is_none() && device.is_none() {
            return Err(RoutingError::Invalid(
                "at least gateway or device is required".into(),
            ));
        }

        Self::ip_route_add(destination, gateway, device)?;

        if let Some(pool) = self.pool() {
            let row = StaticRouteRow {
                id: Uuid::new_v4(),
                destination: destination.trim().to_string(),
                gateway: gateway.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                device: device.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                description: String::new(),
                enabled: true,
            };
            if let Err(e) = Self::block_on(StaticRouteRepo::new(pool.inner()).insert(&row)) {
                eprintln!("[routing] persist add_route failed: {e}");
            }
        }

        Ok(())
    }

    pub fn delete_route(
        &self,
        destination: &str,
        gateway: Option<&str>,
        device: Option<&str>,
    ) -> RoutingResult<()> {
        Self::ip_route_del(destination, gateway, device)?;

        if let Some(pool) = self.pool() {
            if let Ok(rows) = Self::block_on(StaticRouteRepo::new(pool.inner()).list()) {
                for r in rows {
                    let same_dest = r.destination == destination;
                    let same_gw = r.gateway.as_deref() == gateway.filter(|s| !s.is_empty());
                    let same_dev = r.device.as_deref() == device.filter(|s| !s.is_empty());
                    // تطبیق نرم: اگر gateway/device در request خالی بود، فقط dest
                    let gw_ok = gateway.map(|s| s.is_empty()).unwrap_or(true)
                        || same_gw
                        || (r.gateway.is_none() && gateway.is_none());
                    let dev_ok = device.map(|s| s.is_empty()).unwrap_or(true)
                        || same_dev
                        || (r.device.is_none() && device.is_none());
                    if same_dest
                        && (gateway.is_none() || gateway == Some("") || r.gateway.as_deref() == gateway)
                        && (device.is_none() || device == Some("") || r.device.as_deref() == device)
                    {
                        let _ = Self::block_on(StaticRouteRepo::new(pool.inner()).delete(r.id));
                        let _ = (gw_ok, dev_ok);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn set_default_gateway(&self, gateway: &str, device: Option<&str>) -> RoutingResult<()> {
        let _ = Self::run_ip(&["route", "del", "default"]);

        let mut args = vec!["route", "add", "default", "via", gateway];
        if let Some(dev) = device {
            if !dev.is_empty() {
                args.push("dev");
                args.push(dev);
            }
        }
        Self::run_ip(&args)?;

        // default را هم در DB نگه می‌داریم (destination = default)
        if let Some(pool) = self.pool() {
            // قبلی‌های default را پاک کن
            if let Ok(rows) = Self::block_on(StaticRouteRepo::new(pool.inner()).list()) {
                for r in rows.into_iter().filter(|r| r.destination == "default") {
                    let _ = Self::block_on(StaticRouteRepo::new(pool.inner()).delete(r.id));
                }
            }
            let row = StaticRouteRow {
                id: Uuid::new_v4(),
                destination: "default".into(),
                gateway: Some(gateway.trim().to_string()),
                device: device.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                description: "default gateway".into(),
                enabled: true,
            };
            if let Err(e) = Self::block_on(StaticRouteRepo::new(pool.inner()).insert(&row)) {
                eprintln!("[routing] persist default gateway failed: {e}");
            }
        }

        Ok(())
    }

    fn ip_route_add(
        destination: &str,
        gateway: Option<&str>,
        device: Option<&str>,
    ) -> RoutingResult<()> {
        let mut args = vec!["route", "add", destination];
        if let Some(gw) = gateway {
            if !gw.is_empty() {
                args.push("via");
                args.push(gw);
            }
        }
        if let Some(dev) = device {
            if !dev.is_empty() {
                args.push("dev");
                args.push(dev);
            }
        }
        Self::run_ip(&args)?;
        Ok(())
    }

    fn ip_route_del(
        destination: &str,
        gateway: Option<&str>,
        device: Option<&str>,
    ) -> RoutingResult<()> {
        let mut args = vec!["route", "del", destination];
        if let Some(gw) = gateway {
            if !gw.is_empty() {
                args.push("via");
                args.push(gw);
            }
        }
        if let Some(dev) = device {
            if !dev.is_empty() {
                args.push("dev");
                args.push(dev);
            }
        }
        Self::run_ip(&args)?;
        Ok(())
    }

    /// بعد از boot: routeهای enabled از DB
    fn apply_persisted_routes(&self) -> RoutingResult<()> {
        let Some(pool) = self.pool() else {
            return Ok(());
        };
        let rows = Self::block_on(StaticRouteRepo::new(pool.inner()).list())
            .map_err(|e| RoutingError::Internal(e.to_string()))?;

        for r in rows.into_iter().filter(|r| r.enabled) {
            if let Err(e) = Self::ip_route_add(
                &r.destination,
                r.gateway.as_deref(),
                r.device.as_deref(),
            ) {
                // File exists = از قبل هست — نادیده بگیر
                let msg = e.to_string();
                if msg.contains("File exists") {
                    continue;
                }
                eprintln!(
                    "[routing] apply {} failed: {e}",
                    r.destination
                );
            }
        }
        Ok(())
    }
}

impl Default for RoutingService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RoutingService {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
        }
    }
}