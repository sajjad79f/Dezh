use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use storage::{
    DbPool, FirewallRepo, FirewallRuleRow, InterfaceRepo, IpSetRepo, IpSetRow, NatRepo, NatRuleRow,
    SettingsRepo,
};
use storage::fqdn::{FqdnObjectRow, FqdnRepo};
use shared_kernel::runtime::block_on;

use super::{fqdn, kmsg, schedule};
use crate::error::{FirewallError, FirewallResult};
use crate::models::{
    FirewallRule, InterfaceInfo, Protocol, RuleAction, RuleDirection, RuleSchedule, Zone,
};
use crate::registry::RuleRegistry;
use crate::system::detect_system_interfaces;

/// تنظیمات فایروال (جدول firewall_settings)
#[derive(Debug, Clone)]
pub struct FirewallSettings {
    /// حفاظت در برابر SYN Flood روی زنجیره input
    pub dos_protection: bool,
    /// حداکثر اتصال SYN جدید مجاز در ثانیه
    pub syn_rate: u32,
}

impl Default for FirewallSettings {
    fn default() -> Self {
        Self {
            dos_protection: false,
            syn_rate: 200,
        }
    }
}

/// یک اتصال زنده از conntrack
#[derive(Debug, Clone)]
pub struct ConnectionRow {
    pub protocol: String,
    pub state: String,
    pub src: String,
    pub dst: String,
    pub sport: Option<u16>,
    pub dport: Option<u16>,
    pub timeout_secs: Option<u64>,
}

pub struct FirewallService {
    registry: Arc<RuleRegistry>,
    pool: Arc<RwLock<Option<DbPool>>>,
    zones: Arc<RwLock<HashMap<String, Zone>>>,
}

impl FirewallService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RuleRegistry::new()),
            pool: Arc::new(RwLock::new(None)),
            zones: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// بعد از ثبت DbPool در dcm صدا زده می‌شود
    pub fn attach_pool(&self, pool: DbPool) {
        // بارگذاری رول‌های ذخیره‌شده
        if let Err(e) = self.load_from_db(&pool) {
            eprintln!("[firewall] load from db failed: {e}");
        }
        if let Err(e) = self.load_zones_from_db(&pool) {
            eprintln!("[firewall] load zones from db failed: {e}");
        }
        kmsg::spawn_collector(pool.clone());
 
        // فاز ۵: ترد resolver اشیای دامنه — کش FQDN را هر بازه تازه می‌کند
        fqdn::spawn(self.clone(), pool.clone());

        *self.pool.write().expect("firewall pool lock") = Some(pool);
        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules on attach failed: {e}");
        }
        if let Err(e) = self.apply_nat() {
            eprintln!("[firewall] apply_nat on attach failed: {e}");
        }

        // فاز ۴: thread زمان‌بند قوانین — هر ۲۰ ثانیه وضعیت پنجره‌های
        // زمانی را چک می‌کند و در گذر مرز، apply_rules را دوباره می‌زند
        schedule::spawn_scheduler(self.clone());

        eprintln!("[firewall] persistence attached");
    }

    fn load_from_db(&self, pool: &DbPool) -> Result<(), String> {
        let repo = FirewallRepo::new(pool.inner());
        let rows = block_on(repo.list()).map_err(|e| e.to_string())?;

        self.registry.clear();
        for row in rows {
            match row_to_rule(&row) {
                Ok(rule) => {
                    self.registry.add(rule);
                }
                Err(e) => {
                    eprintln!(
                        "[firewall] skip rule id={} name={}: {e}",
                        row.id, row.name
                    );
                }
            }
        }
        Ok(())
    }

    fn load_zones_from_db(&self, pool: &DbPool) -> Result<(), String> {
        let repo = InterfaceRepo::new(pool.inner());
        let rows = block_on(repo.list()).map_err(|e| e.to_string())?;

        let mut zones = self.zones.write().expect("firewall zones lock");
        zones.clear();
        for row in rows {
            if let Some(zone) = Zone::parse(&row.zone) {
                zones.insert(row.name, zone);
            }
        }
        Ok(())
    }

    fn run_nft(args: &[&str]) -> FirewallResult<String> {
        let output = std::process::Command::new("nft")
            .args(args)
            .output()
            .map_err(|_| {
                FirewallError::Internal(
                    "'nft' was not found -- this targets nftables on Debian".into(),
                )
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(FirewallError::Internal(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ))
        }
    }

    /// Real system interfaces (via `ip`), each paired with its assigned
    /// zone (defaulting to `Lan` for anything not yet configured). Any
    /// interface seen for the first time is persisted with a default
    /// zone so it shows up for an admin to (re)assign later.
    pub fn list_interfaces(&self) -> FirewallResult<Vec<InterfaceInfo>> {
        let system = detect_system_interfaces()?;

        let zones = self.zones.read().expect("firewall zones lock");
        let guard = self.pool.read().expect("firewall pool lock");

        let mut result = Vec::with_capacity(system.len());

        for (name, up, addresses) in system {
            let zone = zones.get(&name).copied().unwrap_or(Zone::Lan);

            if !zones.contains_key(&name) {
                if let Some(pool) = guard.as_ref() {
                    let repo = InterfaceRepo::new(pool.inner());
                    if let Err(e) = block_on(repo.ensure_known(&name)) {
                        eprintln!("[firewall] failed to record interface '{name}': {e}");
                    }
                }
            }

            result.push(InterfaceInfo {
                name,
                zone,
                up,
                addresses,
            });
        }

        Ok(result)
    }

    /// Assigns a zone to an interface. Requires persistence to be
    /// attached (an interface's zone is a durable admin decision, not
    /// something to lose on restart).
    pub fn set_zone(&self, name: &str, zone: Zone) -> FirewallResult<()> {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else {
            return Err(FirewallError::Internal(
                "cannot assign a zone: database is not connected".into(),
            ));
        };

        let repo = InterfaceRepo::new(pool.inner());
        block_on(repo.set_zone(name, zone.as_str()))
            .map_err(|e| FirewallError::Internal(e.to_string()))?;

        self.zones
            .write()
            .expect("firewall zones lock")
            .insert(name.to_string(), zone);

        Ok(())
    }

    /// The first interface currently assigned the `Wan` zone, if any.
    pub fn wan_interface(&self) -> Option<String> {
        self.zones
            .read()
            .expect("firewall zones lock")
            .iter()
            .find(|(_, zone)| **zone == Zone::Wan)
            .map(|(name, _)| name.clone())
    }

    fn ensure_accounting_base(&self) -> FirewallResult<()> {
        // اگر table/chain از قبل موجود باشد nft خطا می‌دهد؛ عمداً نادیده گرفته می‌شود
        let _ = Self::run_nft(&["add", "table", "inet", "accounting"]);
        let _ = Self::run_nft(&[
            "add", "chain", "inet", "accounting", "forward",
            "{", "type", "filter", "hook", "forward", "priority", "0;",
            "policy", "accept;", "}",
        ]);
        Ok(())
    }

    fn counter_name(session_id: Uuid, direction: &str) -> String {
        format!("sess_{}_{}", session_id.simple(), direction)
    }

    /// فقط IP واقعی (IPv4/IPv6) می‌پذیرد و شکل استاندارد آن را برمی‌گرداند.
    /// هر رشته دیگری → خطا؛ یعنی هیچ ورودی دستکاری‌شده‌ای به nft نمی‌رسد.
    fn validate_ip(ip: &str) -> FirewallResult<String> {
        let parsed: std::net::IpAddr = ip
            .trim()
            .parse()
            .map_err(|_| FirewallError::InvalidRule(format!("invalid IP address: '{ip}'")))?;
        Ok(parsed.to_string())
    }

    /// مثل validate_ip، اما CIDR (مثل 192.168.1.0/24) را هم قبول می‌کند.
    fn validate_ip_or_cidr(value: &str) -> FirewallResult<String> {
        let v = value.trim();
        let (ip_part, prefix) = match v.split_once('/') {
            Some((ip, p)) => (ip, Some(p)),
            None => (v, None),
        };

        let parsed: std::net::IpAddr = ip_part
            .parse()
            .map_err(|_| FirewallError::InvalidRule(format!("invalid IP/CIDR: '{value}'")))?;

        if let Some(p) = prefix {
            let max = match parsed {
                std::net::IpAddr::V4(_) => 32u8,
                std::net::IpAddr::V6(_) => 128u8,
            };
            let p: u8 = p
                .parse()
                .map_err(|_| FirewallError::InvalidRule(format!("invalid prefix in '{value}'")))?;
            if p > max {
                return Err(FirewallError::InvalidRule(format!(
                    "prefix /{p} is too large for '{ip_part}'"
                )));
            }
        }

        Ok(v.to_string())
    }

    /// نام معتبر برای Address List — چون مستقیم در شناسه nft استفاده
    /// می‌شود، باید سخت‌گیرانه باشد: ۲ تا ۳۲ کاراکتر از [a-z0-9_-]
    fn validate_set_name(name: &str) -> FirewallResult<String> {
        let n = name.trim().to_lowercase();
        if !(2..=32).contains(&n.len()) {
            return Err(FirewallError::InvalidRule(
                "address list name must be 2-32 characters".into(),
            ));
        }
        let ok = n
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
        // شناسه باید با حرف/رقم شروع شود، نه - یا _
        if !ok || n.starts_with('-') || n.starts_with('_') {
            return Err(FirewallError::InvalidRule(
                "address list name may only contain a-z, 0-9, '-' and '_' (must start with a letter or digit)".into(),
            ));
        }
        Ok(n)
    }

    /// Starts counting bytes for `ip`, but only for traffic that
    /// actually crosses the WAN interface -- LAN-to-LAN traffic never
    /// matches these rules, on purpose (that's the whole point: usage
    /// should count internet traffic, not internal traffic). Returns
    /// an error if no interface is currently assigned the `Wan` zone.
    pub fn start_accounting(&self, session_id: Uuid, ip: &str) -> FirewallResult<()> {
        let Some(wan) = self.wan_interface() else {
            return Err(FirewallError::Internal(
                "no accounting interface (set a zone with accounting=true and assign an interface)"
                    .into(),
            ));
        };
        self.start_accounting_on(session_id, ip, &wan)
    }

    /// شمارش ترافیک روی اینترفیس مشخص (از network / zone.accounting)
    pub fn start_accounting_on(
        &self,
        session_id: Uuid,
        ip: &str,
        iface: &str,
    ) -> FirewallResult<()> {
        Self::validate_ip(ip)?;
        let iface = iface.trim();
        if iface.is_empty() {
            return Err(FirewallError::Internal("interface name required".into()));
        }

        self.ensure_accounting_base()?;

        let in_name = Self::counter_name(session_id, "in");
        let out_name = Self::counter_name(session_id, "out");

        Self::run_nft(&["add", "counter", "inet", "accounting", &in_name])?;
        Self::run_nft(&["add", "counter", "inet", "accounting", &out_name])?;

        // Inbound: از این اینترفیس می‌آید به سمت IP کلاینت
        Self::run_nft(&[
            "add",
            "rule",
            "inet",
            "accounting",
            "forward",
            "iifname",
            iface,
            "ip",
            "daddr",
            ip,
            "counter",
            "name",
            &in_name,
        ])?;

        // Outbound: از IP کلاینت از این اینترفیس خارج می‌شود
        Self::run_nft(&[
            "add",
            "rule",
            "inet",
            "accounting",
            "forward",
            "oifname",
            iface,
            "ip",
            "saddr",
            ip,
            "counter",
            "name",
            &out_name,
        ])?;

        Ok(())
    }

    /// Current cumulative (bytes_in, bytes_out) for a session, without
    /// stopping it.
    pub fn read_accounting(&self, session_id: Uuid) -> FirewallResult<(i64, i64)> {
        let bytes_in = Self::read_counter_bytes(&Self::counter_name(session_id, "in"))?;
        let bytes_out = Self::read_counter_bytes(&Self::counter_name(session_id, "out"))?;
        Ok((bytes_in, bytes_out))
    }

    fn read_counter_bytes(name: &str) -> FirewallResult<i64> {
        let output = Self::run_nft(&["list", "counter", "inet", "accounting", name])?;

        // Output looks like: counter sess_xxx_in { packets 12 bytes 3456 }
        output
            .split("bytes")
            .nth(1)
            .and_then(|tail| tail.trim().split_whitespace().next())
            .and_then(|n| n.trim_end_matches('}').trim().parse::<i64>().ok())
            .ok_or_else(|| FirewallError::Internal(format!("could not parse counter '{name}'")))
    }

    /// Reads the final totals, then removes the counters/rules for this
    /// session. Rules must be deleted before their counters (nftables
    /// refuses to delete a counter still referenced by a rule), so this
    /// looks up each rule's handle first via `-a list`.
    pub fn stop_accounting(&self, session_id: Uuid) -> FirewallResult<(i64, i64)> {
        let totals = self.read_accounting(session_id)?;

        let in_name = Self::counter_name(session_id, "in");
        let out_name = Self::counter_name(session_id, "out");

        let _ = self.delete_rule_matching(&in_name);
        let _ = self.delete_rule_matching(&out_name);

        let _ = Self::run_nft(&["delete", "counter", "inet", "accounting", &in_name]);
        let _ = Self::run_nft(&["delete", "counter", "inet", "accounting", &out_name]);

        Ok(totals)
    }

    fn delete_rule_matching(&self, counter_name: &str) -> FirewallResult<()> {
        let listing = Self::run_nft(&["-a", "list", "chain", "inet", "accounting", "forward"])?;

        let handle = listing
            .lines()
            .find(|line| line.contains(counter_name))
            .and_then(|line| line.rsplit("handle").next())
            .and_then(|tail| tail.trim().split_whitespace().next())
            .ok_or_else(|| {
                FirewallError::Internal(format!("no rule found for counter '{counter_name}'"))
            })?;

        Self::run_nft(&[
            "delete", "rule", "inet", "accounting", "forward", "handle", handle,
        ])?;

        Ok(())
    }

    fn persist_insert(&self, rule: &FirewallRule) {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else {
            return;
        };

        let row = rule_to_row(rule);
        let repo = FirewallRepo::new(pool.inner());
        if let Err(e) = block_on(repo.insert(&row)) {
            eprintln!("[firewall] persist insert failed: {e}");
        }
    }

    fn persist_update(&self, rule: &FirewallRule) {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else {
            return;
        };

        let row = rule_to_row(rule);
        let repo = FirewallRepo::new(pool.inner());
        if let Err(e) = block_on(repo.update(&row)) {
            eprintln!("[firewall] persist update failed: {e}");
        }
    }

    pub fn add_rule(
        &self,
        name: impl Into<String>,
        action: RuleAction,
        direction: RuleDirection,
        protocol: Protocol,
        source: impl Into<String>,
        destination: impl Into<String>,
        port: Option<u16>,
        priority: u32,
    ) -> FirewallResult<Uuid> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(FirewallError::InvalidRule("name cannot be empty".into()));
        }

        let rule = FirewallRule::new(
            name, action, direction, protocol, source, destination, port, priority,
        );
        let id = rule.id;
        self.registry.add(rule.clone());
        self.persist_insert(&rule);
        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after add failed: {e}");
        }
        Ok(id)
    }

    /// NGFW: ساخت قانون کامل (با description / log / Address List)
    pub fn add_rule_full(&self, rule: FirewallRule) -> FirewallResult<Uuid> {
        if rule.name.trim().is_empty() {
            return Err(FirewallError::InvalidRule("name cannot be empty".into()));
        }
        let id = rule.id;
        self.registry.add(rule.clone());
        self.persist_insert(&rule);
        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after add failed: {e}");
        }
        Ok(id)
    }

    pub fn list_rules(&self) -> Vec<FirewallRule> {
        self.registry.list()
    }

    pub fn get_rule(&self, id: Uuid) -> Option<FirewallRule> {
        self.registry.get(id)
    }

    /// NGFW: ویرایش قانون (enabled / priority / description / log / name)
    /// حافظه + DB + بازسازی ruleset
    pub fn update_rule(
        &self,
        id: Uuid,
        f: impl FnOnce(&mut FirewallRule),
    ) -> FirewallResult<()> {
        self.registry.update(id, f)?;
        let updated = self
            .registry
            .get(id)
            .ok_or(FirewallError::RuleNotFound(id))?;
        self.persist_update(&updated);

        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after update failed: {e}");
        }
        Ok(())
    }

    pub fn remove_rule(&self, id: Uuid) -> FirewallResult<()> {
        // 1) حافظه
        let in_memory = self.registry.remove(id).is_ok();

        // 2) دیتابیس — حتی اگر در حافظه نبود
        let mut in_db = false;
        if let Some(pool) = self.pool.read().expect("firewall pool lock").clone() {
            match block_on(FirewallRepo::new(pool.inner()).delete(id)) {
                Ok(()) => in_db = true,
                Err(e) => {
                    // NotFound از DB را نادیده بگیر اگر در حافظه بود
                    eprintln!("[firewall] persist delete: {e}");
                }
            }
        }

        if !in_memory && !in_db {
            return Err(FirewallError::RuleNotFound(id));
        }

        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after remove failed: {e}");
        }
        Ok(())
    }

    pub fn enable_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.set_enabled(id, true)?;
        Ok(())
    }

    pub fn disable_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.set_enabled(id, false)?;
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.registry.count()
    }

    pub fn block_ip(&self, ip: &str) -> FirewallResult<Uuid> {
        self.add_rule(
            format!("block-{ip}"),
            RuleAction::Drop,
            RuleDirection::Inbound,
            Protocol::Any,
            ip,
            "any",
            None,
            10,
        )
    }

    pub fn unblock_ip(&self, ip: &str) -> FirewallResult<usize> {
        let matching: Vec<Uuid> = self
            .list_rules()
            .into_iter()
            .filter(|r| r.source == ip)
            .map(|r| r.id)
            .collect();

        let n = matching.len();
        for id in matching {
            self.remove_rule(id)?;
        }
        Ok(n)
    }

    // ─── NGFW: آمار per-rule از nftables counter ───────────

    /// تعداد hit و بایتِ هر قانون از خروجی `nft -a list chain` خوانده
    /// می‌شود. هر قانونِ کاربر هنگام apply با `counter` و
    /// `comment "dezh:<uuid>"` ساخته می‌شود، پس از روی comment قابل
    /// ردیابی است. قوانینِ both روی دو زنجیره جمع زده می‌شوند.
    pub fn rule_stats(&self) -> FirewallResult<HashMap<Uuid, (u64, u64)>> {
        let mut stats: HashMap<Uuid, (u64, u64)> = HashMap::new();

        for chain in ["input", "forward", "output"] {
            let listing = Self::run_nft(&["-a", "list", "chain", "inet", "dezh_filter", chain])?;

            for line in listing.lines() {
                // فقط قوانین خودمان که comment دارند
                let Some(after_marker) = line.split("comment \"dezh:").nth(1) else {
                    continue;
                };
                let Some(end) = after_marker.find('"') else {
                    continue;
                };
                let Ok(id) = Uuid::parse_str(&after_marker[..end]) else {
                    continue;
                };

                let packets = line
                    .find("packets ")
                    .and_then(|p| {
                        line[p + 8..]
                            .split_whitespace()
                            .next()
                            .and_then(|t| t.parse::<u64>().ok())
                    })
                    .unwrap_or(0);

                let bytes = line
                    .find("bytes ")
                    .and_then(|b| {
                        line[b + 6..]
                            .split_whitespace()
                            .next()
                            .and_then(|t| t.trim_end_matches('}').parse::<u64>().ok())
                    })
                    .unwrap_or(0);

                let entry = stats.entry(id).or_insert((0, 0));
                entry.0 += packets;
                entry.1 += bytes;
            }
        }

        Ok(stats)
    }

    // ─── NGFW: اتصال‌های زنده (conntrack) ──────────────────

    /// فهرست اتصال‌های زنده از /proc/net/nf_conntrack یا دستور
    /// `conntrack -L`. تا `limit` ردیف برمی‌گرداند.
    pub fn list_connections(&self, limit: usize) -> FirewallResult<Vec<ConnectionRow>> {
        let text = match std::fs::read_to_string("/proc/net/nf_conntrack") {
            Ok(t) => t,
            Err(_) => {
                // fallback: ابزار conntrack
                let output = std::process::Command::new("conntrack")
                    .arg("-L")
                    .output()
                    .map_err(|_| {
                        FirewallError::Internal(
                            "conntrack unavailable (neither /proc/net/nf_conntrack nor the 'conntrack' tool could be read)".into(),
                        )
                    })?;
                if !output.status.success() {
                    return Err(FirewallError::Internal(
                        String::from_utf8_lossy(&output.stderr).into_owned(),
                    ));
                }
                String::from_utf8_lossy(&output.stdout).into_owned()
            }
        };

        let mut out = Vec::new();
        for line in text.lines() {
            if out.len() >= limit {
                break;
            }
            if let Some(conn) = parse_conntrack_line(line) {
                out.push(conn);
            }
        }
        Ok(out)
    }

    // ─── NGFW: Address Lists (nftables sets) ───────────────

    fn db_or_err(&self) -> FirewallResult<DbPool> {
        self.pool
            .read()
            .expect("firewall pool lock")
            .clone()
            .ok_or_else(|| FirewallError::Internal("database not attached".into()))
    }

    pub fn list_sets(&self) -> FirewallResult<Vec<IpSetRow>> {
        let pool = self.db_or_err()?;
        block_on(IpSetRepo::new(pool.inner()).list())
            .map_err(|e| FirewallError::Internal(e.to_string()))
    }

    pub fn create_set(
        &self,
        name: &str,
        kind: &str,
        description: &str,
        enabled: bool,
    ) -> FirewallResult<Uuid> {
        let name = Self::validate_set_name(name)?;
        let kind = kind.trim().to_lowercase();
        if kind != "blacklist" && kind != "whitelist" {
            return Err(FirewallError::InvalidRule(
                "kind must be blacklist or whitelist".into(),
            ));
        }

        let row = IpSetRow {
            id: Uuid::new_v4(),
            name,
            kind,
            description: description.trim().to_string(),
            enabled,
            entries: Vec::new(),
            created_at: chrono::Utc::now(),
        };
        let id = row.id;

        let pool = self.db_or_err()?;
        block_on(IpSetRepo::new(pool.inner()).insert(&row))
            .map_err(|e| FirewallError::Internal(e.to_string()))?;

        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after set create failed: {e}");
        }
        Ok(id)
    }

    pub fn delete_set(&self, name: &str) -> FirewallResult<()> {
        let pool = self.db_or_err()?;
        block_on(IpSetRepo::new(pool.inner()).delete(name))
            .map_err(|e| FirewallError::Internal(e.to_string()))?;

        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after set delete failed: {e}");
        }
        Ok(())
    }

    pub fn set_set_enabled(&self, name: &str, enabled: bool) -> FirewallResult<()> {
        let pool = self.db_or_err()?;
        block_on(IpSetRepo::new(pool.inner()).set_enabled(name, enabled))
            .map_err(|e| FirewallError::Internal(e.to_string()))?;

        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after set toggle failed: {e}");
        }
        Ok(())
    }

    /// افزودن گروهی ورودی (IP یا CIDR) به یک لیست. خروجی: تعداد
    /// ورودیِ واقعاً جدید (دوباره‌ها نادیده گرفته می‌شوند).
    pub fn add_set_entries(&self, name: &str, entries: Vec<String>) -> FirewallResult<usize> {
        let pool = self.db_or_err()?;
        let repo = IpSetRepo::new(pool.inner());

        let set = block_on(repo.find_by_name(name))
            .map_err(|e| FirewallError::Internal(e.to_string()))?
            .ok_or_else(|| FirewallError::SetNotFound(name.to_string()))?;

        let mut current = set.entries;
        let mut added = 0usize;
        for e in entries {
            if e.trim().is_empty() {
                continue;
            }
            let norm = Self::validate_ip_or_cidr(&e)?;
            if !current.iter().any(|x| *x == norm) {
                current.push(norm);
                added += 1;
            }
        }

        if added > 0 {
            block_on(repo.update_entries(name, &current))
                .map_err(|e| FirewallError::Internal(e.to_string()))?;
            if let Err(e) = self.apply_rules() {
                eprintln!("[firewall] apply_rules after set update failed: {e}");
            }
        }
        Ok(added)
    }

    pub fn remove_set_entry(&self, name: &str, cidr: &str) -> FirewallResult<()> {
        let pool = self.db_or_err()?;
        let repo = IpSetRepo::new(pool.inner());

        let set = block_on(repo.find_by_name(name))
            .map_err(|e| FirewallError::Internal(e.to_string()))?
            .ok_or_else(|| FirewallError::SetNotFound(name.to_string()))?;

        let norm = Self::validate_ip_or_cidr(cidr)?;
        let before = set.entries.len();
        let current: Vec<String> = set.entries.into_iter().filter(|e| *e != norm).collect();

        if current.len() == before {
            return Err(FirewallError::InvalidRule(format!(
                "entry '{norm}' not found in list '{name}'"
            )));
        }

        block_on(repo.update_entries(name, &current))
            .map_err(|e| FirewallError::Internal(e.to_string()))?;
        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after set entry remove failed: {e}");
        }
        Ok(())
    }

    // ─── NGFW: تنظیمات ─────────────────────────────────────

    fn load_settings(&self) -> FirewallResult<FirewallSettings> {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else {
            return Ok(FirewallSettings::default());
        };
        let repo = SettingsRepo::new(pool.inner());

        let dos = block_on(repo.get("dos_protection"))
            .map_err(|e| FirewallError::Internal(e.to_string()))?
            .map(|v| v == "true")
            .unwrap_or(false);
        let syn_rate = block_on(repo.get("dos_syn_rate"))
            .map_err(|e| FirewallError::Internal(e.to_string()))?
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(200);

        Ok(FirewallSettings {
            dos_protection: dos,
            syn_rate: syn_rate.clamp(1, 100_000),
        })
    }

    /// تنظیمات فعلی — اگر DB نبود، پیش‌فرض
    pub fn get_settings(&self) -> FirewallSettings {
        self.load_settings().unwrap_or_default()
    }

    pub fn set_settings(&self, dos_protection: bool, syn_rate: u32) -> FirewallResult<()> {
        let pool = self.db_or_err()?;
        let repo = SettingsRepo::new(pool.inner());
        let syn_rate = syn_rate.clamp(1, 100_000);

        block_on(async {
            repo.set("dos_protection", if dos_protection { "true" } else { "false" })
                .await?;
            repo.set("dos_syn_rate", &syn_rate.to_string()).await?;
            Ok::<(), storage::StorageError>(())
        })
        .map_err(|e| FirewallError::Internal(e.to_string()))?;

        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules after settings change failed: {e}");
        }
        Ok(())
    }

    // ─── ruleset reconstruction ────────────────────────────

    fn ensure_filter_base(&self) -> FirewallResult<()> {
        let _ = Self::run_nft(&["add", "table", "inet", "dezh_filter"]);
        let _ = Self::run_nft(&[
            "add", "chain", "inet", "dezh_filter", "input", "{", "type", "filter",
            "hook", "input", "priority", "0", ";", "policy", "accept", ";", "}",
        ]);
        let _ = Self::run_nft(&[
            "add", "chain", "inet", "dezh_filter", "forward", "{", "type", "filter",
            "hook", "forward", "priority", "0", ";", "policy", "accept", ";", "}",
        ]);
        let _ = Self::run_nft(&[
            "add", "chain", "inet", "dezh_filter", "output", "{", "type", "filter",
            "hook", "output", "priority", "0", ";", "policy", "accept", ";", "}",
        ]);
        Ok(())
    }

    fn flush_filter_chains(&self) -> FirewallResult<()> {
        let _ = Self::run_nft(&["flush", "chain", "inet", "dezh_filter", "input"]);
        let _ = Self::run_nft(&["flush", "chain", "inet", "dezh_filter", "forward"]);
        let _ = Self::run_nft(&["flush", "chain", "inet", "dezh_filter", "output"]);
        Ok(())
    }

    fn action_to_nft(action: &RuleAction) -> &'static str {
        match action {
            RuleAction::Allow => "accept",
            RuleAction::Deny | RuleAction::Drop => "drop",
            RuleAction::Reject => "reject",
        }
    }

    fn protocol_to_nft(p: &Protocol) -> Option<&'static str> {
        match p {
            Protocol::Tcp => Some("tcp"),
            Protocol::Udp => Some("udp"),
            Protocol::Icmp => Some("icmp"),
            Protocol::Any => None,
        }
    }

    fn chain_for_direction(dir: &RuleDirection) -> &'static [&'static str] {
        match dir {
            RuleDirection::Inbound => &["input"],
            RuleDirection::Outbound => &["output"],
            RuleDirection::Both => &["input", "output"],
        }
    }

    /// یک قانون set-based (blacklist/whitelist) روی یک زنجیره
    /// هر Address List دو set دارد: _4 (IPv4) و _6 (IPv6)
    fn add_set_rule(
        chain: &str,
        field: &str, // saddr | daddr
        set_name: &str,
        verdict: &str,
    ) -> FirewallResult<()> {
        Self::run_nft(&[
            "add", "rule", "inet", "dezh_filter", chain,
            "ip", field, &format!("@dezh_set_{set_name}_4"), verdict,
        ])?;
        Self::run_nft(&[
            "add", "rule", "inet", "dezh_filter", chain,
            "ip6", field, &format!("@dezh_set_{set_name}_6"), verdict,
        ])?;
        Ok(())
    }

    /// ساخت/خالی کردن/پر کردن nft sets از روی DB
    /// خروجی: فهرست setهای موجود (حتی غیرفعال‌ها — برای اعتبارسنجی ارجاع قوانین)
    fn apply_sets(&self) -> FirewallResult<Vec<IpSetRow>> {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else {
            return Ok(Vec::new());
        };
        let rows = block_on(IpSetRepo::new(pool.inner()).list())
            .map_err(|e| FirewallError::Internal(e.to_string()))?;

        for set in &rows {
            // ۱) ساخت (یا استفاده از) set و خالی کردنش — منبع حقیقت = DB
            for (suffix, set_type) in [("_4", "ipv4_addr"), ("_6", "ipv6_addr")] {
                let nft_name = format!("dezh_set_{}{}", set.name, suffix);
                let _ = Self::run_nft(&[
                    "add", "set", "inet", "dezh_filter", &nft_name,
                    "{", "type", set_type, ";", "flags", "interval", ";", "}",
                ]);
                let _ = Self::run_nft(&["flush", "set", "inet", "dezh_filter", &nft_name]);
            }

            if !set.enabled {
                continue;
            }

            // ۲) جدا کردن ورودی‌ها بر اساس خانواده و افزودن به‌صورت دسته‌ای
            let mut v4: Vec<&String> = Vec::new();
            let mut v6: Vec<&String> = Vec::new();
            for entry in &set.entries {
                match Self::validate_ip_or_cidr(entry) {
                    Ok(_) => {
                        if entry.contains(':') {
                            v6.push(entry);
                        } else {
                            v4.push(entry);
                        }
                    }
                    Err(e) => eprintln!(
                        "[firewall] set '{}': skipping invalid entry '{entry}': {e}",
                        set.name
                    ),
                }
            }

            for (suffix, list) in [("_4", &v4), ("_6", &v6)] {
                for chunk in list.chunks(256) {
                    let mut args: Vec<String> = vec![
                        "add".into(),
                        "element".into(),
                        "inet".into(),
                        "dezh_filter".into(),
                        format!("dezh_set_{}{}", set.name, suffix),
                        "{".into(),
                    ];
                    for (i, e) in chunk.iter().enumerate() {
                        if i > 0 {
                            args.push(",".into());
                        }
                        args.push((*e).clone());
                    }
                    args.push("}".into());

                    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                    if let Err(e) = Self::run_nft(&refs) {
                        eprintln!(
                            "[firewall] add element to '{}{suffix}' failed: {e}",
                            set.name
                        );
                    }
                }
            }
        }

        Ok(rows)
    }

    /// فاز ۵: ساخت/خالی کردن/پر کردن nft sets اشیای دامنه (FQDN)
    /// هر شیء دو set دارد: dezh_fqdn_{name}_4 و dezh_fqdn_{name}_6
    /// منبع IPها = ستون resolved که ترد dezh-fqdn نگه می‌دارد
    /// خروجی: همه اشیا (حتی غیرفعال) برای اعتبارسنجی ارجاع قوانین
    fn apply_fqdn_sets(&self) -> FirewallResult<Vec<FqdnObjectRow>> {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else {
            return Ok(Vec::new());
        };
        let mut rows = block_on(FqdnRepo::new(pool.inner()).list())
            .map_err(|e| FirewallError::Internal(e.to_string()))?;

        // فقط نام‌های نرمال (همان قاعده Address List) به nft می‌روند؛
        // نام دستکاری‌شده مثل «شیء گمشده» رفتار می‌شود تا قانونش skip شود
        rows.retain(|o| match Self::validate_set_name(&o.name) {
            Ok(n) if n == o.name => true,
            _ => {
                eprintln!(
                    "[firewall] fqdn object '{}': invalid name — skipped (allowed: a-z 0-9 '-' '_', 2-32 chars)",
                    o.name
                );
                false
            }
        });

        for obj in &rows {
            // ۱) ساخت (یا استفاده از) set و خالی کردنش — منبع حقیقت = DB
            for (suffix, set_type) in [("_4", "ipv4_addr"), ("_6", "ipv6_addr")] {
                let nft_name = format!("dezh_fqdn_{}{}", obj.name, suffix);
                let _ = Self::run_nft(&[
                    "add", "set", "inet", "dezh_filter", &nft_name,
                    "{", "type", set_type, ";", "flags", "interval", ";", "}",
                ]);
                let _ = Self::run_nft(&["flush", "set", "inet", "dezh_filter", &nft_name]);
            }

            if !obj.enabled {
                continue;
            }

            // ۲) جداسازی IPهای resolve‌شده بر اساس خانواده و افزودن دسته‌ای
            let mut v4: Vec<&String> = Vec::new();
            let mut v6: Vec<&String> = Vec::new();
            for ip in &obj.resolved {
                match Self::validate_ip_or_cidr(ip) {
                    Ok(_) => {
                        if ip.contains(':') {
                            v6.push(ip);
                        } else {
                            v4.push(ip);
                        }
                    }
                    Err(e) => eprintln!(
                        "[firewall] fqdn '{}': skipping invalid resolved IP '{ip}': {e}",
                        obj.name
                    ),
                }
            }

            for (suffix, list) in [("_4", &v4), ("_6", &v6)] {
                for chunk in list.chunks(256) {
                    let mut args: Vec<String> = vec![
                        "add".into(),
                        "element".into(),
                        "inet".into(),
                        "dezh_filter".into(),
                        format!("dezh_fqdn_{}{}", obj.name, suffix),
                        "{".into(),
                    ];
                    for (i, e) in chunk.iter().enumerate() {
                        if i > 0 {
                            args.push(",".into());
                        }
                        args.push((*e).clone());
                    }
                    args.push("}".into());

                    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                    if let Err(e) = Self::run_nft(&refs) {
                        eprintln!(
                            "[firewall] add element to 'dezh_fqdn_{}{suffix}' failed: {e}",
                            obj.name
                        );
                    }
                }
            }
        }

        Ok(rows)
    }

    /// بازسازی کامل ruleset فیلتر از روی registry (منبع حقیقت = DB/RAM)
    ///
    /// ترتیب نهایی هر زنجیره:
    ///   ۱. blacklist drop      — لیست سیاه، فوراً حتی اتصال زنده
    ///   ۲. whitelist accept    — لیست سفید، همیشه مجاز
    ///   ۳. ct state invalid drop
    ///   ۴. ct state established,related accept
    ///   ۵. قوانین کاربر (به ترتیب priority، هرکدام با counter+comment)
    ///   ۶. حفاظت SYN-flood (فقط input، اگر فعال باشد)
    pub fn apply_rules(&self) -> FirewallResult<()> {
        self.ensure_filter_base()?;
        self.flush_filter_chains()?;

        // ۱) Address List ها — قبل از قوانین تا ارجاع‌ها معتبر باشد
        let sets = match self.apply_sets() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[firewall] apply_sets failed: {e}");
                Vec::new()
            }
        };
        let set_exists =
            |name: &str| sets.iter().any(|s| s.name == name);

        // ۱.۵) اشیای دامنه (FQDN) — قبل از قوانین تا ارجاع‌ها معتبر باشد
        let fqdns = match self.apply_fqdn_sets() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[firewall] apply_fqdn_sets failed: {e}");
                Vec::new()
            }
        };
        let fqdn_exists =
            |name: &str| fqdns.iter().any(|f| f.name == name);

        // ۲) تنظیمات
        let settings = self.load_settings().unwrap_or_default();

        // ۳) blacklist
        for set in sets.iter().filter(|s| s.enabled && s.kind == "blacklist") {
            for chain in ["input", "forward"] {
                let _ = Self::add_set_rule(chain, "saddr", &set.name, "drop");
            }
            let _ = Self::add_set_rule("output", "daddr", &set.name, "drop");
        }

        // ۴) whitelist
        for set in sets.iter().filter(|s| s.enabled && s.kind == "whitelist") {
            for chain in ["input", "forward"] {
                let _ = Self::add_set_rule(chain, "saddr", &set.name, "accept");
            }
            let _ = Self::add_set_rule("output", "daddr", &set.name, "accept");
        }

        // ۵) بسته‌های نامعتبر
        for chain in ["input", "forward", "output"] {
            Self::run_nft(&[
                "add", "rule", "inet", "dezh_filter", chain,
                "ct", "state", "invalid", "drop",
            ])?;
        }

        // ۶) اتصال‌های برقرار — بعد از لیست‌ها تا blacklist بتواند
        //    اتصال زنده را همان لحظه قطع کند
        for chain in ["input", "forward", "output"] {
            Self::run_nft(&[
                "add", "rule", "inet", "dezh_filter", chain,
                "ct", "state", "established,related", "accept",
            ])?;
        }

        // ۷) قوانین کاربر
        let mut rules = self.registry.list();
        rules.sort_by_key(|r| r.priority);
        let enabled_count = rules.iter().filter(|r| r.enabled).count();

        // فاز ۴: قوانین زمان‌دار فقط داخل پنجره‌ی زمانی خودشان اعمال
        // می‌شوند (قانون بدون Schedule = همیشه فعال). عبور از مرز پنجره
        // را thread زمان‌بند (schedule.rs) با apply_rules مجدد مدیریت می‌کند.
        let window_open = |r: &FirewallRule| {
            r.schedule.as_ref().map_or(true, |s| s.is_active_now())
        };

        for rule in rules.iter().filter(|r| r.enabled && window_open(r)) {
            // اگر قانون به لیستی اشاره می‌کند که دیگر وجود ندارد → نادیده
            if let Some(s) = &rule.source_set {
                if !set_exists(s) {
                    eprintln!(
                        "[firewall] rule '{}' references missing address list '{s}' — skipped",
                        rule.name
                    );
                    continue;
                }
            }
            if let Some(s) = &rule.dest_set {
                if !set_exists(s) {
                    eprintln!(
                        "[firewall] rule '{}' references missing address list '{s}' — skipped",
                        rule.name
                    );
                    continue;
                }
            }
            if let Some(f) = &rule.source_fqdn {
                if !fqdn_exists(f) {
                    eprintln!(
                        "[firewall] rule '{}' references missing domain object '{f}' — skipped",
                        rule.name
                    );
                    continue;
                }
            }
            if let Some(f) = &rule.dest_fqdn {
                if !fqdn_exists(f) {
                    eprintln!(
                        "[firewall] rule '{}' references missing domain object '{f}' — skipped",
                        rule.name
                    );
                    continue;
                }
            }

            let verdict = Self::action_to_nft(&rule.action);
            let chains = Self::chain_for_direction(&rule.direction);

            for chain in chains {
                let mut args: Vec<String> = vec![
                    "add".into(),
                    "rule".into(),
                    "inet".into(),
                    "dezh_filter".into(),
                    (*chain).into(),
                ];

                // منبع: شیء دامنه (FQDN) یا Address List یا IP/CIDR متنی
                // اولویت: fqdn > set > متن
                if let Some(f) = &rule.source_fqdn {
                    args.push("ip".into());
                    args.push("saddr".into());
                    args.push(format!("@dezh_fqdn_{f}_4"));
                    args.push("ip6".into());
                    args.push("saddr".into());
                    args.push(format!("@dezh_fqdn_{f}_6"));
                } else if let Some(s) = &rule.source_set {
                    args.push("ip".into());
                    args.push("saddr".into());
                    args.push(format!("@dezh_set_{s}_4"));
                    args.push("ip6".into());
                    args.push("saddr".into());
                    args.push(format!("@dezh_set_{s}_6"));
                } else if rule.source != "any" {
                    args.push("ip".into());
                    args.push("saddr".into());
                    args.push(rule.source.clone());
                }

                // مقصد: شیء دامنه (FQDN) یا Address List یا IP/CIDR متنی
                // اولویت: fqdn > set > متن
                if let Some(f) = &rule.dest_fqdn {
                    args.push("ip".into());
                    args.push("daddr".into());
                    args.push(format!("@dezh_fqdn_{f}_4"));
                    args.push("ip6".into());
                    args.push("daddr".into());
                    args.push(format!("@dezh_fqdn_{f}_6"));
                } else if let Some(s) = &rule.dest_set {
                    args.push("ip".into());
                    args.push("daddr".into());
                    args.push(format!("@dezh_set_{s}_4"));
                    args.push("ip6".into());
                    args.push("daddr".into());
                    args.push(format!("@dezh_set_{s}_6"));
                } else if rule.destination != "any" {
                    args.push("ip".into());
                    args.push("daddr".into());
                    args.push(rule.destination.clone());
                }

                if let Some(proto) = Self::protocol_to_nft(&rule.protocol) {
                    args.push("ip".into());
                    args.push("protocol".into());
                    args.push(proto.into());
                }

                if let Some(port) = rule.port {
                    if let Some(proto) = Self::protocol_to_nft(&rule.protocol) {
                        if proto == "tcp" || proto == "udp" {
                            args.push(proto.into());
                            args.push("dport".into());
                            args.push(port.to_string());
                        }
                    }
                }

                // فاز ۳: rate limit اختیاری — بسته‌های مازاد این قانون را
                // تطبیق نمی‌گیرند (به قانون بعدی / سیاست پیش‌فرض می‌رسند)
                if let Some(rl) = rule.rate_limit {
                    args.push("limit".into());
                    args.push("rate".into());
                    args.push(format!("{rl}/second"));
                }

                // آمار per-rule + لاگ اختیاری + شناسه برای ردیابی                args.push("counter".into());
                if rule.log {
                    args.push("log".into());
                    args.push("prefix".into());
                    // فاز ۳: uuid داخل prefix تا کالکتور kmsg بتواند
                    // hit را به قانون نگاشت کند (security_events)
                    args.push(format!("\"dezh:{}: \"", rule.id));
                }
                args.push("comment".into());
                args.push(format!("\"dezh:{}\"", rule.id));
                args.push(verdict.into());

                let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                Self::run_nft(&args_ref)?;
            }
        }

        // ۸) حفاظت SYN-flood (فقط زنجیره input)
        if settings.dos_protection {
            let rate = settings.syn_rate.max(1);
            Self::run_nft(&[
                "add", "rule", "inet", "dezh_filter", "input",
                "tcp", "flags", "syn", "ct", "state", "new",
                "limit", "rate", &format!("{rate}/second"), "counter", "accept",
            ])?;
            Self::run_nft(&[
                "add", "rule", "inet", "dezh_filter", "input",
                "tcp", "flags", "syn", "ct", "state", "new", "counter", "drop",
            ])?;
        }

        eprintln!(
            "[firewall] applied {enabled_count} rule(s), {} address list(s), {} domain object(s), dos={} (syn_rate={})",
            sets.len(),
            fqdns.len(),
            settings.dos_protection,
            settings.syn_rate
        );
        Ok(())
    }

    // ─── NAT ───────────────────────────────────────────

    pub fn list_nat_rules(&self) -> FirewallResult<Vec<NatRuleRow>> {
        let pool = self.db_or_err()?;
        let repo = NatRepo::new(pool.inner());
        block_on(repo.list()).map_err(|e| FirewallError::Internal(e.to_string()))
    }

    pub fn add_nat_rule(
        &self,
        name: &str,
        kind: &str,
        interface: &str,
        source: &str,
        destination: &str,
        protocol: &str,
        dest_port: Option<u16>,
        target: Option<&str>,
        description: &str,
    ) -> FirewallResult<Uuid> {
        let kind = kind.trim().to_lowercase();
        if kind != "masquerade" && kind != "dnat" {
            return Err(FirewallError::Internal("kind must be masquerade or dnat".into()));
        }
        if interface.trim().is_empty() {
            return Err(FirewallError::Internal("interface required".into()));
        }
        if kind == "dnat" && target.map(|t| t.trim().is_empty()).unwrap_or(true) {
            return Err(FirewallError::Internal("dnat requires target (IP or IP:port)".into()));
        }

        let id = Uuid::new_v4();
        let row = NatRuleRow {
            id,
            name: name.trim().to_string(),
            enabled: true,
            kind,
            interface: interface.trim().to_string(),
            source: if source.trim().is_empty() { "any".into() } else { source.trim().into() },
            destination: if destination.trim().is_empty() { "any".into() } else { destination.trim().into() },
            protocol: protocol.trim().to_lowercase(),
            dest_port: dest_port.map(|p| p as i32),
            target: target.map(|t| t.trim().to_string()).filter(|s| !s.is_empty()),
            description: description.to_string(),
        };

        let pool = self.db_or_err()?;
        let repo = NatRepo::new(pool.inner());
        block_on(repo.insert(&row)).map_err(|e| FirewallError::Internal(e.to_string()))?;

        if let Err(e) = self.apply_nat() {
            eprintln!("[firewall] apply_nat after add failed: {e}");
        }
        Ok(id)
    }

    pub fn remove_nat_rule(&self, id: Uuid) -> FirewallResult<()> {
        let pool = self.db_or_err()?;
        let repo = NatRepo::new(pool.inner());
        block_on(repo.delete(id)).map_err(|e| FirewallError::Internal(e.to_string()))?;
        if let Err(e) = self.apply_nat() {
            eprintln!("[firewall] apply_nat after remove failed: {e}");
        }
        Ok(())
    }

    fn ensure_nat_base(&self) -> FirewallResult<()> {
        let _ = Self::run_nft(&["add", "table", "ip", "dezh_nat"]);
        let _ = Self::run_nft(&[
            "add", "chain", "ip", "dezh_nat", "prerouting",
            "{", "type", "nat", "hook", "prerouting", "priority", "dstnat", ";", "}",
        ]);
        let _ = Self::run_nft(&[
            "add", "chain", "ip", "dezh_nat", "postrouting",
            "{", "type", "nat", "hook", "postrouting", "priority", "srcnat", ";", "}",
        ]);
        Ok(())
    }

    fn flush_nat_chains(&self) -> FirewallResult<()> {
        let _ = Self::run_nft(&["flush", "chain", "ip", "dezh_nat", "prerouting"]);
        let _ = Self::run_nft(&["flush", "chain", "ip", "dezh_nat", "postrouting"]);
        Ok(())
    }

    /// IP forwarding لازم برای NAT
    fn enable_ip_forward() {
        let _ = std::fs::write("/proc/sys/net/ipv4/ip_forward", "1");
    }

    pub fn apply_nat(&self) -> FirewallResult<()> {
        Self::enable_ip_forward();
        self.ensure_nat_base()?;
        self.flush_nat_chains()?;

        let rules = match self.list_nat_rules() {
            Ok(r) => r,
            Err(_) => return Ok(()), // هنوز DB attach نشده
        };

        for rule in rules.iter().filter(|r| r.enabled) {
            match rule.kind.as_str() {
                "masquerade" => {
                    let mut args: Vec<String> = vec![
                        "add".into(), "rule".into(), "ip".into(), "dezh_nat".into(),
                        "postrouting".into(),
                        "oifname".into(), rule.interface.clone(),
                    ];
                    if rule.source != "any" {
                        args.push("ip".into());
                        args.push("saddr".into());
                        args.push(rule.source.clone());
                    }
                    args.push("masquerade".into());
                    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    Self::run_nft(&refs)?;
                }
                "dnat" => {
                    let target = rule.target.as_deref().unwrap_or("");
                    if target.is_empty() {
                        continue;
                    }
                    let mut args: Vec<String> = vec![
                        "add".into(), "rule".into(), "ip".into(), "dezh_nat".into(),
                        "prerouting".into(),
                        "iifname".into(), rule.interface.clone(),
                    ];
                    if rule.destination != "any" {
                        args.push("ip".into());
                        args.push("daddr".into());
                        args.push(rule.destination.clone());
                    }
                    let proto = rule.protocol.as_str();
                    if (proto == "tcp" || proto == "udp") && rule.dest_port.is_some() {
                        args.push(proto.into());
                        args.push("dport".into());
                        args.push(rule.dest_port.unwrap().to_string());
                    }
                    args.push("dnat".into());
                    args.push("to".into());
                    args.push(target.to_string());
                    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                    Self::run_nft(&refs)?;
                }
                _ => {}
            }
        }

        eprintln!(
            "[firewall] applied {} enabled NAT rule(s)",
            rules.iter().filter(|r| r.enabled).count()
        );
        Ok(())
    }
}

impl Clone for FirewallService {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            pool: Arc::clone(&self.pool),
            zones: Arc::clone(&self.zones),
        }
    }
}

fn rule_to_row(r: &FirewallRule) -> FirewallRuleRow {
    FirewallRuleRow {
        id: r.id,
        name: r.name.clone(),
        action: format!("{:?}", r.action).to_lowercase(),
        direction: format!("{:?}", r.direction).to_lowercase(),
        protocol: format!("{:?}", r.protocol).to_lowercase(),
        source: r.source.clone(),
        destination: r.destination.clone(),
        port: r.port.map(|p| p as i32),
        enabled: r.enabled,
        priority: r.priority as i32,
        description: r.description.clone(),
        log: r.log,
        source_set: r.source_set.clone(),
        dest_set: r.dest_set.clone(),
        source_fqdn: r.source_fqdn.clone(),
        dest_fqdn: r.dest_fqdn.clone(),
        rate_limit: r.rate_limit.map(|v| v as i32),
        // فاز ۴: زمان‌بندی — days با ویرگول ذخیره می‌شود
        schedule_enabled: r.schedule.is_some(),
        schedule_start: r.schedule.as_ref().map(|s| s.start.clone()).unwrap_or_default(),
        schedule_end: r.schedule.as_ref().map(|s| s.end.clone()).unwrap_or_default(),
        schedule_days: r
            .schedule
            .as_ref()
            .map(|s| s.days.join(","))
            .unwrap_or_default(),
    }
}

fn row_to_rule(row: &FirewallRuleRow) -> Result<FirewallRule, String> {
    let action = match row.action.as_str() {
        "allow" => RuleAction::Allow,
        "deny" => RuleAction::Deny,
        "drop" => RuleAction::Drop,
        "reject" => RuleAction::Reject,
        other => return Err(format!("unknown action: {other}")),
    };
    let direction = match row.direction.as_str() {
        "inbound" | "in" => RuleDirection::Inbound,
        "outbound" | "out" => RuleDirection::Outbound,
        "both" => RuleDirection::Both,
        other => return Err(format!("unknown direction: {other}")),
    };
    let protocol = match row.protocol.as_str() {
        "tcp" => Protocol::Tcp,
        "udp" => Protocol::Udp,
        "icmp" => Protocol::Icmp,
        "any" => Protocol::Any,
        other => return Err(format!("unknown protocol: {other}")),
    };

    // فاز ۴: بازسازی Schedule از ردیف DB (days با ویرگول جدا شده‌اند)
    let schedule = if row.schedule_enabled && !row.schedule_start.is_empty() {
        Some(RuleSchedule {
            start: row.schedule_start.clone(),
            end: row.schedule_end.clone(),
            days: row
                .schedule_days
                .split(',')
                .map(str::trim)
                .filter(|d| !d.is_empty())
                .map(String::from)
                .collect(),
        })
    } else {
        None
    };

    Ok(FirewallRule {
        id: row.id,
        name: row.name.clone(),
        action,
        direction,
        protocol,
        source: row.source.clone(),
        destination: row.destination.clone(),
        port: row.port.and_then(|p| u16::try_from(p).ok()),
        enabled: row.enabled,
        priority: row.priority as u32,
        description: row.description.clone(),
        log: row.log,
        source_set: row.source_set.clone(),
        dest_set: row.dest_set.clone(),
        source_fqdn: row.source_fqdn.clone(),
        dest_fqdn: row.dest_fqdn.clone(),
        rate_limit: row.rate_limit.and_then(|v| u32::try_from(v).ok()),
        schedule,
    })
}

/// پارس یک خط conntrack؛ با هر دو قالب کار می‌کند:
///   proc:      ipv4 2 tcp 6 120 ESTABLISHED src=1.2.3.4 dst=... sport=.. dport=..
///   conntrack: tcp 6 120 ESTABLISHED src=1.2.3.4 dst=... sport=.. dport=..
fn parse_conntrack_line(line: &str) -> Option<ConnectionRow> {
    let mut tokens = line.split_whitespace();

    let mut first = tokens.next()?;
    if first == "ipv4" || first == "ipv6" {
        tokens.next()?; // l3 protocol number
        first = tokens.next()?;
    }

    let protocol = first.to_string();
    if !matches!(protocol.as_str(), "tcp" | "udp" | "icmp" | "sctp" | "udplite") {
        return None;
    }

    let _protonum = tokens.next()?;
    let timeout_secs: Option<u64> = tokens.next()?.parse().ok();

    let mut state = String::from("—");
    let mut src = None;
    let mut dst = None;
    let mut sport = None;
    let mut dport = None;
    let mut state_seen = false;

    for tok in tokens {
        if let Some(v) = tok.strip_prefix("src=") {
            if src.is_none() {
                src = Some(v.to_string());
            }
        } else if let Some(v) = tok.strip_prefix("dst=") {
            if dst.is_none() {
                dst = Some(v.to_string());
            }
        } else if let Some(v) = tok.strip_prefix("sport=") {
            if sport.is_none() {
                sport = v.parse().ok();
            }
        } else if let Some(v) = tok.strip_prefix("dport=") {
            if dport.is_none() {
                dport = v.parse().ok();
            }
        } else if !state_seen
            && !tok.starts_with('[')
            && tok.len() >= 2
            && tok.chars().all(|c| c.is_ascii_uppercase() || c == '_')
        {
            state = tok.to_string();
            state_seen = true;
        }
    }

    Some(ConnectionRow {
        protocol,
        state,
        src: src?,
        dst: dst?,
        sport,
        dport,
        timeout_secs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_proc_conntrack_line() {
        let line = "ipv4 2 tcp 6 431999 ESTABLISHED src=192.168.1.50 dst=1.2.3.4 \
                    sport=54321 dport=443 src=1.2.3.4 dst=192.168.1.50 sport=443 \
                    dport=54321 [ASSURED] mark=0 use=1";
        let c = parse_conntrack_line(line).expect("must parse");
        assert_eq!(c.protocol, "tcp");
        assert_eq!(c.state, "ESTABLISHED");
        assert_eq!(c.src, "192.168.1.50");
        assert_eq!(c.dst, "1.2.3.4");
        assert_eq!(c.sport, Some(54321));
        assert_eq!(c.dport, Some(443));
        assert_eq!(c.timeout_secs, Some(431999));
    }

    #[test]
    fn parses_conntrack_tool_line() {
        let line = "udp 17 28 src=10.0.0.9 dst=8.8.8.8 sport=5353 dport=53 \
                    src=8.8.8.8 dst=10.0.0.9 sport=53 dport=5353";
        let c = parse_conntrack_line(line).expect("must parse");
        assert_eq!(c.protocol, "udp");
        assert_eq!(c.state, "—");
        assert_eq!(c.src, "10.0.0.9");
        assert_eq!(c.dport, Some(53));
        assert_eq!(c.timeout_secs, Some(28));
    }

    #[test]
    fn rejects_non_connection_lines() {
        assert!(parse_conntrack_line("").is_none());
        assert!(parse_conntrack_line("garbage line here").is_none());
        // بدون src=/dst=
        assert!(parse_conntrack_line("tcp 6 120 ESTABLISHED").is_none());
    }

    #[test]
    fn set_name_validation() {
        assert!(FirewallService::validate_set_name("botnet-c2").is_ok());
        assert!(FirewallService::validate_set_name("My_List2").is_ok());
        assert!(FirewallService::validate_set_name("-bad").is_err());
        assert!(FirewallService::validate_set_name("bad name with space").is_err());
        assert!(FirewallService::validate_set_name("x").is_err()); // خیلی کوتاه
        assert!(FirewallService::validate_set_name("nft; drop").is_err()); // تزریق
    }
}