#[derive(Debug, Clone)]
pub struct ZoneInfo {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub accounting: bool,
    pub description: String,
}

/// خلاصهٔ داشبورد شبکه (N1)
#[derive(Debug, Clone)]
pub struct NetworkSummary {
    pub total_interfaces: usize,
    pub up_interfaces: usize,
    pub down_interfaces: usize,
    pub zones: usize,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub generated_at: u64,
}

#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub name: String,
    pub zone: String,
    pub up: bool,
    pub addresses: Vec<String>,
    pub enabled: bool,
    pub ipv4_mode: String,
    pub address_cidr: Option<String>,
    pub gateway: Option<String>,
    pub description: String,
}


// ─── [N2] PPPoE ────────────────────────────────────────────────────

/// پروفایل PPPoE ذخیره‌شده در DB + نشست زندهٔ متناظر (اگر dial شده باشد)
#[derive(Debug, Clone)]
pub struct PppoeProfile {
    pub name: String,
    pub parent_iface: String,
    pub username: String,
    pub mtu: u32,
    pub default_route: bool,
    pub keepalive: u32,
    pub enabled: bool,
    pub description: String,
    pub session: Option<PppoeSession>,
}

/// یک نشست ppp زندهٔ سیستم (pppX)
#[derive(Debug, Clone)]
pub struct PppoeSession {
    pub iface: String,
    /// نام پروفایل دژ اگر از cmdline (linkname dezh-*) تشخیص داده شود
    pub profile: Option<String>,
    pub local_ip: Option<String>,
    pub remote_ip: Option<String>,
    pub up: bool,
}

/// وضعیت کامل PPPoE برای API (N2)
#[derive(Debug, Clone)]
pub struct PppoeState {
    pub profiles: Vec<PppoeProfile>,
    pub sessions: Vec<PppoeSession>,
}


// ─── [N3] VPN WAN (PPTP / L2TP) ────────────────────────────────────

/// یک اتصال VPN WAN ذخیره‌شده + نشست زندهٔ متناظر (اگر برقرار باشد)
#[derive(Debug, Clone)]
pub struct VpnConnection {
    pub name: String,
    /// "pptp" | "l2tp"
    pub kind: String,
    pub server: String,
    pub username: String,
    pub mtu: u32,
    pub default_route: bool,
    pub keepalive: u32,
    pub enabled: bool,
    pub description: String,
    /// از PppoeSession استفاده مجدد میشود — شکل نشست ppp یکی است
    pub session: Option<PppoeSession>,
}

/// وضعیت کامل VPN WAN برای API (N3)
#[derive(Debug, Clone)]
pub struct VpnWanState {
    pub connections: Vec<VpnConnection>,
    pub sessions: Vec<PppoeSession>,
}
