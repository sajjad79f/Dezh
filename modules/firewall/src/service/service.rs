use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use storage::{DbPool, FirewallRepo, FirewallRuleRow, InterfaceRepo};

use crate::error::{FirewallError, FirewallResult};
use crate::models::{FirewallRule, InterfaceInfo, Protocol, RuleAction, RuleDirection, Zone};
use crate::registry::RuleRegistry;
use crate::system::detect_system_interfaces;

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
        if let Err(e) = self.apply_rules() {
            eprintln!("[firewall] apply_rules on attach failed: {e}");
        }
        *self.pool.write().expect("firewall pool lock") = Some(pool);
        eprintln!("[firewall] persistence attached");
    }

    fn block_on<F, T>(f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(f)),
            Err(_) => {
                // اگر runtime نبود (نادر): runtime موقت
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("temp runtime");
                rt.block_on(f)
            }
        }
    }

    fn load_from_db(&self, pool: &DbPool) -> Result<(), String> {
        let repo = FirewallRepo::new(pool.inner());
        let rows = Self::block_on(repo.list()).map_err(|e| e.to_string())?;

        self.registry.clear();
        for row in rows {
            let rule = row_to_rule(&row)?;
            self.registry.add(rule);
        }
        Ok(())
    }

    fn load_zones_from_db(&self, pool: &DbPool) -> Result<(), String> {
        let repo = InterfaceRepo::new(pool.inner());
        let rows = Self::block_on(repo.list()).map_err(|e| e.to_string())?;

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
                    if let Err(e) = Self::block_on(repo.ensure_known(&name)) {
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
        Self::block_on(repo.set_zone(name, zone.as_str()))
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

    fn run_nft_shell(script: &str) -> FirewallResult<String> {
        let output = std::process::Command::new("sh")
            .args(["-c", script])
            .output()
            .map_err(|e| FirewallError::Internal(format!("nft shell failed: {e}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(FirewallError::Internal(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ))
        }
    }

    fn ensure_accounting_base(&self) -> FirewallResult<()> {
        // خطا را نادیده نگیر برای table؛ اگر از قبل باشد nft معمولاً error می‌دهد — آن را ignore کن
        let _ = Self::run_nft_shell("nft add table inet accounting 2>/dev/null");
        let _ = Self::run_nft_shell(
            "nft 'add chain inet accounting forward { type filter hook forward priority 0; policy accept; }' 2>/dev/null",
        );
        Ok(())
    }

    fn counter_name(session_id: Uuid, direction: &str) -> String {
        format!("sess_{}_{}", session_id.simple(), direction)
    }

    /// Starts counting bytes for `ip`, but only for traffic that
    /// actually crosses the WAN interface -- LAN-to-LAN traffic never
    /// matches these rules, on purpose (that's the whole point: usage
    /// should count internet traffic, not internal traffic). Returns
    /// an error if no interface is currently assigned the `Wan` zone.
    pub fn start_accounting(&self, session_id: Uuid, ip: &str) -> FirewallResult<()> {
        let Some(wan) = self.wan_interface() else {
            return Err(FirewallError::Internal(
                "no interface is assigned the 'wan' zone yet".into(),
            ));
        };

        self.ensure_accounting_base()?;

        let in_name = Self::counter_name(session_id, "in");
        let out_name = Self::counter_name(session_id, "out");

        Self::run_nft(&["add", "counter", "inet", "accounting", &in_name])?;
        Self::run_nft(&["add", "counter", "inet", "accounting", &out_name])?;

        // Inbound: comes IN through the WAN interface, destined for this IP.
        Self::run_nft(&[
            "add", "rule", "inet", "accounting", "forward",
            "iifname", &wan, "ip", "daddr", ip, "counter", "name", &in_name,
        ])?;

        // Outbound: leaves OUT through the WAN interface, sourced from this IP.
        Self::run_nft(&[
            "add", "rule", "inet", "accounting", "forward",
            "oifname", &wan, "ip", "saddr", ip, "counter", "name", &out_name,
        ])?;

        Self::run_nft_shell(&format!(
            "nft add counter inet accounting {in_name}"
        ))?;
        Self::run_nft_shell(&format!(
            "nft add counter inet accounting {out_name}"
        ))?;
        Self::run_nft_shell(&format!(
            "nft add rule inet accounting forward iifname \"{wan}\" ip daddr {ip} counter name {in_name}"
        ))?;
        Self::run_nft_shell(&format!(
            "nft add rule inet accounting forward oifname \"{wan}\" ip saddr {ip} counter name {out_name}"
        ))?;

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
        if let Err(e) = Self::block_on(repo.insert(&row)) {
            eprintln!("[firewall] persist insert failed: {e}");
        }
    }

    fn persist_delete(&self, id: Uuid) {
        let guard = self.pool.read().expect("firewall pool lock");
        let Some(pool) = guard.as_ref() else {
            return;
        };

        let repo = FirewallRepo::new(pool.inner());
        if let Err(e) = Self::block_on(repo.delete(id)) {
            eprintln!("[firewall] persist delete failed: {e}");
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

    pub fn remove_rule(&self, id: Uuid) -> FirewallResult<()> {
        self.registry.remove(id)?;
        self.persist_delete(id);
        self.registry.remove(id)?;
        self.persist_delete(id);
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

    /// بازسازی کامل ruleset فیلتر از روی registry (منبع حقیقت = DB/RAM)
    pub fn apply_rules(&self) -> FirewallResult<()> {
        self.ensure_filter_base()?;
        self.flush_filter_chains()?;

        let mut rules = self.registry.list();
        rules.sort_by_key(|r| r.priority);

        for rule in rules.iter().filter(|r| r.enabled) {
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

                if let Some(proto) = Self::protocol_to_nft(&rule.protocol) {
                    args.push("ip".into());
                    args.push("protocol".into());
                    args.push(proto.into());
                }

                if rule.source != "any" {
                    args.push("ip".into());
                    args.push("saddr".into());
                    args.push(rule.source.clone());
                }
                if rule.destination != "any" {
                    args.push("ip".into());
                    args.push("daddr".into());
                    args.push(rule.destination.clone());
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

                args.push(verdict.into());

                let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                Self::run_nft(&args_ref)?;
            }
        }

        eprintln!(
            "[firewall] applied {} enabled rule(s) to nftables",
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
    })
}