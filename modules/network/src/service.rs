use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use storage::{DbPool, InterfaceRepo, ZoneRepo};
use shared_kernel::runtime::block_on;
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


    fn pool(&self) -> NetworkResult<DbPool> {
        self.pool
            .read()
            .expect("network pool lock")
            .clone()
            .ok_or_else(|| NetworkError::Internal("database not attached".into()))
    }

    fn load_zones_from_db(&self, pool: &DbPool) -> Result<(), String> {
        let repo = InterfaceRepo::new(pool.inner());
        let rows = block_on(repo.list()).map_err(|e| e.to_string())?;
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
        let rows = block_on(ZoneRepo::new(pool.inner()).list())
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
        block_on(
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
        block_on(
            ZoneRepo::new(pool.inner()).update(name, display_name, accounting, description),
        )
        .map_err(|e| NetworkError::Internal(e.to_string()))
    }

    pub fn delete_zone(&self, name: &str) -> NetworkResult<()> {
        let pool = self.pool()?;
        // اگر ifaceای روی این زون است، حذف نکن
        let ifaces = block_on(InterfaceRepo::new(pool.inner()).list())
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
        if ifaces.iter().any(|i| i.zone == name) {
            return Err(NetworkError::Internal(format!(
                "zone '{name}' is assigned to an interface"
            )));
        }
        block_on(ZoneRepo::new(pool.inner()).delete(name)).map_err(|e| {
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
        let acc_zones = block_on(ZoneRepo::new(pool.inner()).accounting_zone_names())
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
            if let Ok(rows) = block_on(repo.list()) {
                for r in rows {
                    cfg.insert(r.name.clone(), r);
                }
            }
            for (name, _, _) in &system {
                let _ = block_on(repo.ensure_known(name));
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
        if block_on(ZoneRepo::new(pool.inner()).find_by_name(&zone))
            .map_err(|e| NetworkError::Internal(e.to_string()))?
            .is_none()
        {
            return Err(NetworkError::Internal(format!(
                "unknown zone '{zone}' — create it under Network → Zones"
            )));
        }
        block_on(InterfaceRepo::new(pool.inner()).set_zone(iface, &zone))
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
        if block_on(ZoneRepo::new(pool.inner()).find_by_name(&zone))
            .map_err(|e| NetworkError::Internal(e.to_string()))?
            .is_none()
        {
            return Err(NetworkError::Internal(format!("unknown zone '{zone}'")));
        }

        block_on(InterfaceRepo::new(pool.inner()).update_config(
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

// ─── Summary (N1) ──────────────────────────────────
impl NetworkService {
    /// خلاصهٔ داشبورد: شمارشها + مجموع ترافیک sysfs (N1)
    pub fn summary(&self) -> NetworkResult<crate::models::NetworkSummary> {
        let ifaces = self.list_interfaces()?;
        let zones = self.list_zones().unwrap_or_default();

        let up = ifaces.iter().filter(|i| i.up).count();
        let down = ifaces.len() - up;

        let mut rx = 0u64;
        let mut tx = 0u64;
        for i in &ifaces {
            if let Some((r, t)) = crate::system::read_iface_stats(&i.name) {
                rx = rx.saturating_add(r);
                tx = tx.saturating_add(t);
            }
        }

        let generated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(crate::models::NetworkSummary {
            total_interfaces: ifaces.len(),
            up_interfaces: up,
            down_interfaces: down,
            zones: zones.len(),
            rx_bytes: rx,
            tx_bytes: tx,
            generated_at,
        })
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


// ─── [N1b] rename interface — کرنل + DB + کش زون‌ها ────────────────

impl NetworkService {
    /// تغییر نام اینترفیس. کسکید:
    ///   ۱) کرنل: down → name → بازگردانی وضعیت قبلی
    ///   ۲) DB:   upsert رکورد جدید با همان config، حذف رکورد قدیم
    ///   ۳) کش:   کلید zones map قدیم → جدید
    /// اگر بعد از موفقیت کرنل، DB شکست بخورد، اجرای دوباره خودش را ترمیم
    /// می‌کند (kernel rename دیگر لازم نیست؛ فقط upsert/delete).
    pub fn rename_interface(&self, old: &str, new: &str) -> NetworkResult<()> {
        let old = old.trim();
        let new = new.trim();
        if old.is_empty() || new.is_empty() {
            return Err(NetworkError::Internal("interface name is required".into()));
        }
        if old == new {
            return Ok(());
        }

        // مقصد نباید وجود داشته باشد — نه در سیستم، نه در DB
        let system = detect_system_interfaces()?;
        if system.iter().any(|(n, _, _)| n == new) {
            return Err(NetworkError::Internal(format!(
                "interface '{new}' already exists"
            )));
        }
        let old_in_system = system.iter().any(|(n, _, _)| n == old);

        let pool = self.pool()?;
        let repo = InterfaceRepo::new(pool.inner());
        let existing = block_on(repo.find_by_name(old))
            .map_err(|e| NetworkError::Internal(e.to_string()))?;

        if !old_in_system && existing.is_none() {
            return Err(NetworkError::NotFound(format!("interface '{old}' not found")));
        }

        // ۱) کرنل
        if old_in_system {
            crate::system::rename_interface_link(old, new)?;

            // [N3] پایدارسازی: نام در /etc/network/interfaces هم عوض شود.
            // best-effort — شکستش rename را باطل نمیکند، فقط هشدار.
            match crate::system::update_interfaces_file(old, new) {
                Ok(n) if n > 0 => {
                    eprintln!(
                        "[network] rename: /etc/network/interfaces updated in {n} file(s)"
                    );
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "[network] rename: could not update /etc/network/interfaces: {e}"
                    );
                }
            }
        }

        // ۲) DB — کپی config روی نام جدید، حذف نام قدیم
        if let Some(ref row) = existing {
            block_on(repo.update_config(
                new,
                &row.zone,
                row.enabled,
                &row.ipv4_mode,
                row.address_cidr.as_deref(),
                row.gateway.as_deref(),
                &row.description,
            ))
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
            if let Err(e) = block_on(repo.delete_iface(old)) {
                // رکورد قدیم یتیم می‌شود ولی UI از سیستم می‌خواند؛ فقط لاگ
                eprintln!("[network] rename: delete old db row failed: {e}");
            }
        }

        // ۳) کش زون‌ها
        {
            let mut map = self.zones.write().expect("zones lock");
            if let Some(z) = map.remove(old) {
                map.insert(new.to_string(), z);
            }
        }

        eprintln!("[network] interface renamed: {old} -> {new}");
        Ok(())
    }
}


// ─── [N2] PPPoE — پروفایل‌ها و نشست‌های زنده ───────────────────────

impl NetworkService {
    /// وضعیت PPPoE: پروفایل‌ها (+ نشست زندهٔ هرکدام) و همهٔ نشست‌های ppp
    pub fn pppoe_state(&self) -> NetworkResult<crate::models::PppoeState> {
        let pool = self.pool()?;
        let rows = block_on(storage::PppoeRepo::new(pool.inner()).list())
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
        let sessions = crate::system::list_ppp_sessions();

        let profiles = rows
            .into_iter()
            .map(|r| {
                let session = sessions
                    .iter()
                    .find(|s| s.profile.as_deref() == Some(r.name.as_str()))
                    .cloned();
                crate::models::PppoeProfile {
                    name: r.name,
                    parent_iface: r.parent_iface,
                    username: r.username,
                    mtu: r.mtu.max(0) as u32,
                    default_route: r.default_route,
                    keepalive: r.keepalive.max(0) as u32,
                    enabled: r.enabled,
                    description: r.description,
                    session,
                }
            })
            .collect();

        Ok(crate::models::PppoeState { profiles, sessions })
    }

    /// ساخت/ویرایش پروفایل. اگر password خالی باشد، پسورد قبلی حفظ میشود.
    #[allow(clippy::too_many_arguments)]
    pub fn pppoe_save(
        &self,
        name: &str,
        parent_iface: &str,
        username: &str,
        password: &str,
        mtu: u32,
        default_route: bool,
        keepalive: u32,
        enabled: bool,
        description: &str,
    ) -> NetworkResult<()> {
        let name = name.trim().to_lowercase();
        let parent = parent_iface.trim().to_string();
        let username = username.trim().to_string();

        if name.is_empty()
            || name.len() > 32
            || !name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(NetworkError::Internal(
                "pppoe profile name: 1-32 chars, a-z 0-9 -".into(),
            ));
        }
        if parent.is_empty() || parent.len() > 15 {
            return Err(NetworkError::Internal("parent interface name is invalid".into()));
        }
        if username.is_empty() || username.len() > 128 {
            return Err(NetworkError::Internal("username is required (max 128)".into()));
        }
        if !(576..=1500).contains(&mtu) {
            return Err(NetworkError::Internal("mtu must be 576..1500".into()));
        }
        if !(1..=600).contains(&keepalive) {
            return Err(NetworkError::Internal("keepalive must be 1..600".into()));
        }
        if description.len() > 256 {
            return Err(NetworkError::Internal("description too long (max 256)".into()));
        }

        // اینترفیس مادر باید روی همین نوده
        if !detect_system_interfaces()?.iter().any(|(n, _, _)| *n == parent) {
            return Err(NetworkError::Internal(format!(
                "parent interface '{parent}' not found on this node"
            )));
        }

        let pool = self.pool()?;
        let repo = storage::PppoeRepo::new(pool.inner());

        let password: String = if password.is_empty() {
            // ویرایش بدون پسورد جدید → پسورد قبلی
            let existing = block_on(repo.find_by_name(&name))
                .map_err(|e| NetworkError::Internal(e.to_string()))?;
            match existing {
                Some(row) => row.password,
                None => {
                    return Err(NetworkError::Internal(
                        "password is required for a new profile".into(),
                    ))
                }
            }
        } else {
            if password.len() > 128 {
                return Err(NetworkError::Internal("password too long (max 128)".into()));
            }
            password.to_string()
        };

        block_on(repo.upsert(
            &name,
            &parent,
            &username,
            &password,
            mtu as i32,
            default_route,
            keepalive as i32,
            enabled,
            description,
        ))
        .map_err(|e| NetworkError::Internal(e.to_string()))?;

        eprintln!("[network] pppoe profile saved: {name} (parent={parent})");
        Ok(())
    }

    /// حذف پروفایل — نشست زنده اول بسته میشود
    pub fn pppoe_delete(&self, name: &str) -> NetworkResult<()> {
        let name = name.trim().to_lowercase();
        let pool = self.pool()?;

        if crate::system::pppd_running(&name) {
            let _ = crate::system::stop_pppoe(&name);
        }

        block_on(storage::PppoeRepo::new(pool.inner()).delete(&name)).map_err(|e| match e {
            storage::StorageError::NotFound(_) => {
                NetworkError::NotFound(format!("pppoe profile '{name}'"))
            }
            other => NetworkError::Internal(other.to_string()),
        })?;
        eprintln!("[network] pppoe profile deleted: {name}");
        Ok(())
    }

    /// dial کردن پروفایل (اجرای pppd)
    pub fn pppoe_start(&self, name: &str) -> NetworkResult<()> {
        let name = name.trim().to_lowercase();
        let pool = self.pool()?;
        let row = block_on(storage::PppoeRepo::new(pool.inner()).find_by_name(&name))
            .map_err(|e| NetworkError::Internal(e.to_string()))?
            .ok_or_else(|| NetworkError::NotFound(format!("pppoe profile '{name}'")))?;

        crate::system::start_pppoe(
            &row.name,
            &row.parent_iface,
            &row.username,
            &row.password,
            row.mtu.max(0) as u32,
            row.default_route,
            row.keepalive.max(0) as u32,
        )?;
        eprintln!("[network] pppoe session started: {name}");
        Ok(())
    }

    /// قطع نشست پروفایل
    pub fn pppoe_stop(&self, name: &str) -> NetworkResult<()> {
        let name = name.trim().to_lowercase();
        crate::system::stop_pppoe(&name)?;
        eprintln!("[network] pppoe session stopped: {name}");
        Ok(())
    }
}


// ─── [N3] VPN WAN — PPTP / L2TP ────────────────────────────────────

impl NetworkService {
    /// وضعیت اتصال‌های VPN WAN + نشست‌های زندهٔ ppp
    pub fn vpn_state(&self) -> NetworkResult<crate::models::VpnWanState> {
        let pool = self.pool()?;
        let rows = block_on(storage::VpnWanRepo::new(pool.inner()).list())
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
        let sessions = crate::system::list_ppp_sessions();

        let connections = rows
            .into_iter()
            .map(|r| {
                let session = sessions
                    .iter()
                    .find(|s| s.profile.as_deref() == Some(r.name.as_str()))
                    .cloned();
                crate::models::VpnConnection {
                    name: r.name,
                    kind: r.kind,
                    server: r.server,
                    username: r.username,
                    mtu: r.mtu.max(0) as u32,
                    default_route: r.default_route,
                    keepalive: r.keepalive.max(0) as u32,
                    enabled: r.enabled,
                    description: r.description,
                    session,
                }
            })
            .collect();

        Ok(crate::models::VpnWanState { connections, sessions })
    }

    /// ساخت/ویرایش اتصال VPN. اگر password خالی باشد، پسورد قبلی حفظ میشود.
    #[allow(clippy::too_many_arguments)]
    pub fn vpn_save(
        &self,
        name: &str,
        kind: &str,
        server: &str,
        username: &str,
        password: &str,
        mtu: u32,
        default_route: bool,
        keepalive: u32,
        enabled: bool,
        description: &str,
    ) -> NetworkResult<()> {
        let name = name.trim().to_lowercase();
        let kind = kind.trim().to_lowercase();
        let server = server.trim().to_string();
        let username = username.trim().to_string();

        if name.is_empty()
            || name.len() > 32
            || !name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(NetworkError::Internal(
                "connection name: 1-32 chars, a-z 0-9 -".into(),
            ));
        }
        if kind != "pptp" && kind != "l2tp" {
            return Err(NetworkError::Internal("kind must be 'pptp' or 'l2tp'".into()));
        }
        // server سختگیرانه — داخل رشتهٔ pty / فایل کانفیگ می‌نشیند
        if server.is_empty()
            || server.len() > 253
            || !server
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':'))
        {
            return Err(NetworkError::Internal(
                "server: host or ip, chars a-z 0-9 . - _ : (max 253)".into(),
            ));
        }
        if username.is_empty()
            || username.len() > 128
            || username
                .chars()
                .any(|c| c.is_whitespace() || matches!(c, '"' | '\''))
        {
            return Err(NetworkError::Internal(
                "username: 1-128 chars, no spaces or quotes".into(),
            ));
        }
        if !(576..=1500).contains(&mtu) {
            return Err(NetworkError::Internal("mtu must be 576..1500".into()));
        }
        if !(1..=600).contains(&keepalive) {
            return Err(NetworkError::Internal("keepalive must be 1..600".into()));
        }
        if description.len() > 256 {
            return Err(NetworkError::Internal("description too long (max 256)".into()));
        }

        let pool = self.pool()?;
        let repo = storage::VpnWanRepo::new(pool.inner());

        let password: String = if password.is_empty() {
            // ویرایش بدون پسورد جدید → پسورد قبلی
            let existing = block_on(repo.find_by_name(&name))
                .map_err(|e| NetworkError::Internal(e.to_string()))?;
            match existing {
                Some(row) => row.password,
                None => {
                    return Err(NetworkError::Internal(
                        "password is required for a new connection".into(),
                    ))
                }
            }
        } else {
            if password.len() > 128
                || password
                    .chars()
                    .any(|c| c.is_whitespace() || matches!(c, '"' | '\''))
            {
                return Err(NetworkError::Internal(
                    "password: max 128 chars, no spaces or quotes".into(),
                ));
            }
            password.to_string()
        };

        block_on(repo.upsert(
            &name,
            &kind,
            &server,
            &username,
            &password,
            mtu as i32,
            default_route,
            keepalive as i32,
            enabled,
            description,
        ))
        .map_err(|e| NetworkError::Internal(e.to_string()))?;

        // فایل‌های سیستمی L2TP را همگام کن (کانفیگ هنگام استارت xl2tpd خوانده میشود)
        if kind == "l2tp" {
            crate::system::write_l2tp_options(
                &name,
                &username,
                &password,
                mtu,
                default_route,
                keepalive,
            )?;
            let all = block_on(repo.list_by_kind("l2tp"))
                .map_err(|e| NetworkError::Internal(e.to_string()))?;
            let pairs: Vec<(String, String)> = all
                .iter()
                .map(|r| (r.name.clone(), r.server.clone()))
                .collect();
            crate::system::regenerate_l2tp_conf(&pairs)?;
            crate::system::reload_xl2tpd();
        }

        eprintln!("[network] vpn connection saved: {name} ({kind} -> {server})");
        Ok(())
    }

    /// حذف اتصال — نشست زنده اول بسته میشود، فایل‌های سیستمی پاک میشوند
    pub fn vpn_delete(&self, name: &str) -> NetworkResult<()> {
        let name = name.trim().to_lowercase();
        let pool = self.pool()?;
        let repo = storage::VpnWanRepo::new(pool.inner());

        let row = block_on(repo.find_by_name(&name))
            .map_err(|e| NetworkError::Internal(e.to_string()))?;
        let kind = row.map(|r| r.kind).unwrap_or_default();

        let _ = crate::system::stop_vpn(&name, &kind);
        let _ = std::fs::remove_file(format!("/etc/ppp/options.dezh-{name}"));

        block_on(repo.delete(&name)).map_err(|e| match e {
            storage::StorageError::NotFound(_) => {
                NetworkError::NotFound(format!("vpn connection '{name}'"))
            }
            other => NetworkError::Internal(other.to_string()),
        })?;

        if kind == "l2tp" {
            let all = block_on(repo.list_by_kind("l2tp"))
                .map_err(|e| NetworkError::Internal(e.to_string()))?;
            let pairs: Vec<(String, String)> = all
                .iter()
                .map(|r| (r.name.clone(), r.server.clone()))
                .collect();
            crate::system::regenerate_l2tp_conf(&pairs)?;
            crate::system::reload_xl2tpd();
        }

        eprintln!("[network] vpn connection deleted: {name}");
        Ok(())
    }

    /// برقراری اتصال
    pub fn vpn_start(&self, name: &str) -> NetworkResult<()> {
        let name = name.trim().to_lowercase();
        let pool = self.pool()?;
        let row = block_on(storage::VpnWanRepo::new(pool.inner()).find_by_name(&name))
            .map_err(|e| NetworkError::Internal(e.to_string()))?
            .ok_or_else(|| NetworkError::NotFound(format!("vpn connection '{name}'")))?;

        match row.kind.as_str() {
            "pptp" => crate::system::start_pptp(
                &row.name,
                &row.server,
                &row.username,
                &row.password,
                row.mtu.max(0) as u32,
                row.default_route,
                row.keepalive.max(0) as u32,
            )?,
            "l2tp" => crate::system::start_l2tp(&row.name)?,
            other => {
                return Err(NetworkError::Internal(format!(
                    "unknown vpn kind '{other}'"
                )))
            }
        }
        eprintln!("[network] vpn connection started: {name} ({})", row.kind);
        Ok(())
    }

    /// قطع اتصال
    pub fn vpn_stop(&self, name: &str) -> NetworkResult<()> {
        let name = name.trim().to_lowercase();
        let pool = self.pool()?;
        let kind = block_on(storage::VpnWanRepo::new(pool.inner()).find_by_name(&name))
            .map_err(|e| NetworkError::Internal(e.to_string()))?
            .map(|r| r.kind)
            .unwrap_or_default();

        crate::system::stop_vpn(&name, &kind)?;
        eprintln!("[network] vpn connection stopped: {name}");
        Ok(())
    }
}
