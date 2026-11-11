use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ModuleDto {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct CommandDto {
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct CoreServiceDto {
    pub name: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct ExecuteCommandRequest {
    pub name: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Serialize)]
pub struct ExecuteCommandResponse {
    pub output: String,
}

#[derive(Serialize)]
pub struct ErrorDto {
    pub error: String,
}

// ─── Firewall Rules ────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct FirewallRuleDto {
    pub id: String,
    pub name: String,
    pub action: String,
    pub direction: String,
    pub protocol: String,
    pub source: String,
    pub destination: String,
    pub port: Option<u16>,
    pub enabled: bool,
    pub priority: u32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub log: bool,
    #[serde(default)]
    pub source_set: Option<String>,
    #[serde(default)]
    pub dest_set: Option<String>,
    // ── فاز ۵: ارجاع به شیء دامنه (اولویت تطبیق: fqdn > set > متن) ──
    #[serde(default)]
    pub source_fqdn: Option<String>,
    #[serde(default)]
    pub dest_fqdn: Option<String>,
    #[serde(default)]
    pub rate_limit: Option<u32>,
    // ── فاز ۴: زمان‌بندی قانون ──
    #[serde(default)]
    pub schedule_enabled: bool,
    #[serde(default)]
    pub schedule_start: String,
    #[serde(default)]
    pub schedule_end: String,
    #[serde(default)]
    pub schedule_days: Vec<String>,
    /// آیا الان داخل پنجره‌ی زمانی است؟ — با ساعت سرور محاسبه می‌شود
    #[serde(default)]
    pub schedule_active: bool,
}

#[derive(Deserialize)]
pub struct CreateFirewallRuleRequest {
    pub name: String,
    pub action: String,
    pub direction: String,
    pub protocol: String,
    pub source: String,
    pub destination: String,
    pub port: Option<u16>,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub log: bool,
    #[serde(default)]
    pub source_set: Option<String>,
    #[serde(default)]
    pub dest_set: Option<String>,
    /// رشته‌ی خالی = حذف ارجاع به شیء دامنه
    pub source_fqdn: Option<String>,
    pub dest_fqdn: Option<String>,
    #[serde(default)]
    pub rate_limit: Option<u32>,
    // ── فاز ۴: زمان‌بندی قانون (اختیاری) ──
    #[serde(default)]
    pub schedule_enabled: bool,
    #[serde(default)]
    pub schedule_start: String,
    #[serde(default)]
    pub schedule_end: String,
    #[serde(default)]
    pub schedule_days: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateFirewallRuleRequest {
    pub enabled: Option<bool>,
    pub priority: Option<u32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub log: Option<bool>,
    /// 0 = حذف سقف
    pub rate_limit: Option<u32>,
    // ── فاز ۴: ویرایش کامل فیلدهای تطبیق ──
    pub action: Option<String>,
    pub direction: Option<String>,
    pub protocol: Option<String>,
    /// IP یا CIDR یا "any"
    pub source: Option<String>,
    /// IP یا CIDR یا "any"
    pub destination: Option<String>,
    /// 0 = هر پورت (any)
    pub port: Option<u16>,
    /// رشته‌ی خالی = حذف ارجاع به لیست
    pub source_set: Option<String>,
    /// رشته‌ی خالی = حذف ارجاع به لیست
    pub dest_set: Option<String>,
    /// رشته‌ی خالی = حذف ارجاع به شیء دامنه
    pub source_fqdn: Option<String>,
    pub dest_fqdn: Option<String>,
    // ── فاز ۴: زمان‌بندی — قرارداد: هر ۴ فیلد باید با هم فرستاده شوند ──
    /// false = حذف کامل زمان‌بندی
    pub schedule_enabled: Option<bool>,
    /// "HH:MM"
    pub schedule_start: Option<String>,
    /// "HH:MM"
    pub schedule_end: Option<String>,
    /// ["sun",..,"sat"] — لیست خالی یعنی همه‌ی روزها
    pub schedule_days: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct DeleteFirewallRuleRequest {
    pub id: String,
}

fn default_priority() -> u32 {
    100
}

/// آمار per-rule (تعداد hit و بایت) — از nftables counter
#[derive(Serialize)]
pub struct RuleStatDto {
    pub rule_id: String,
    pub packets: u64,
    pub bytes: u64,
}

// ─── NGFW: Address Lists ───────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct IpSetDto {
    pub id: String,
    pub name: String,
    pub kind: String, // blacklist | whitelist
    pub description: String,
    pub enabled: bool,
    pub entries_count: usize,
    #[serde(default)]
    pub entries: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateIpSetRequest {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub struct AddSetEntriesRequest {
    pub entries: Vec<String>,
}

#[derive(Deserialize)]
pub struct RemoveSetEntryRequest {
    pub cidr: String,
}

#[derive(Deserialize)]
pub struct UpdateIpSetRequest {
    pub enabled: bool,
}

// ─── NGFW: اتصال‌های زنده ──────────────────────────────

#[derive(Serialize)]
pub struct ConnectionDto {
    pub protocol: String,
    pub state: String,
    pub src: String,
    pub dst: String,
    pub sport: Option<u16>,
    pub dport: Option<u16>,
    pub timeout_secs: Option<u64>,
}

// ─── NGFW: تنظیمات ─────────────────────────────────────

#[derive(Serialize)]
pub struct FirewallSettingsDto {
    pub dos_protection: bool,
    pub syn_rate: u32,
}

#[derive(Deserialize)]
pub struct UpdateFirewallSettingsRequest {
    pub dos_protection: Option<bool>,
    pub syn_rate: Option<u32>,
}

// ─── NGFW: Import / Export ─────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct FirewallExport {
    #[serde(default = "default_export_version")]
    pub version: u32,
    #[serde(default)]
    pub exported_at: String,
    #[serde(default)]
    pub rules: Vec<FirewallRuleDto>,
    #[serde(default)]
    pub nat_rules: Vec<NatRuleDto>,
    #[serde(default)]
    pub ip_sets: Vec<IpSetDto>,
}

fn default_export_version() -> u32 {
    1
}

#[derive(Serialize)]
pub struct ImportResultDto {
    pub rules_added: usize,
    pub rules_skipped: usize,
    pub nat_added: usize,
    pub nat_skipped: usize,
    pub sets_added: usize,
    pub sets_skipped: usize,
}

// ─── فاز ۳: Security Events ─────────────────────────────

#[derive(Serialize)]
pub struct SecurityEventDto {
    pub id: String,
    pub created_at: String,
    pub kind: String,
    pub severity: String,
    pub source_ip: Option<String>,
    pub dest_port: Option<u16>,
    pub rule_name: Option<String>,
    pub hits: u64,
    pub message: String,
}

#[derive(Serialize)]
pub struct EventKindCount {
    pub kind: String,
    pub count: u64,
}

#[derive(Serialize)]
pub struct EventSourceCount {
    pub ip: String,
    pub count: u64,
}

#[derive(Serialize)]
pub struct EventPortCount {
    pub port: u16,
    pub count: u64,
}

#[derive(Serialize)]
pub struct EventTimePoint {
    pub hour: String,
    pub count: u64,
}

#[derive(Serialize)]
pub struct EventSummaryDto {
    pub since_hours: u32,
    pub total_hits: u64,
    pub by_kind: Vec<EventKindCount>,
    pub top_sources: Vec<EventSourceCount>,
    pub top_ports: Vec<EventPortCount>,
    pub timeline: Vec<EventTimePoint>,
}

// ─── Auth / Users ──────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub role: String,
    pub display_name: Option<String>,
    pub must_change_password: bool,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub display_name: Option<String>,
    pub must_change_password: bool,
}

#[derive(Serialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub role: String,
    pub display_name: Option<String>,
    pub enabled: bool,
    pub must_change_password: bool,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
pub struct IdentityDto {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub source: String,
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct CreateIdentityRequest {
    pub username: String,
    pub display_name: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "local".into()
}

#[derive(Deserialize)]
pub struct StartSessionRequest {
    pub identity_id: Option<String>,
    pub protocol: String, // vpn | nac | portal | other
    #[allow(dead_code)]
    pub ip_address: Option<String>,
}

#[derive(Deserialize)]
pub struct EndSessionRequest {
    pub session_id: String,
    #[serde(default)]
    pub bytes_in: i64,
    #[serde(default)]
    pub bytes_out: i64,
    pub terminate_cause: Option<String>,
}

#[derive(Deserialize)]
pub struct AgentUserActiveRequest {
    pub username: String,
    pub hostname: Option<String>,
    pub ip_address: Option<String>,
    pub os: Option<String>,
    #[allow(dead_code)]
    pub agent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct AgentUserInactiveRequest {
    pub username: String,
    #[allow(dead_code)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub bytes_in: i64,
    #[serde(default)]
    pub bytes_out: i64,
}

// ─── Interfaces / Routing ──────────────────────────────

#[derive(Serialize)]
pub struct InterfaceDto {
    pub name: String,
    pub zone: String,
    pub up: bool,
    pub addresses: Vec<String>,
}

#[derive(Serialize)]
pub struct RouteDto {
    pub destination: String,
    pub gateway: Option<String>,
    pub device: Option<String>,
    pub proto: Option<String>,
    pub metric: Option<u32>,
}

#[derive(Deserialize)]
pub struct AddRouteRequest {
    pub destination: String,
    pub gateway: Option<String>,
    pub device: Option<String>,
}

#[derive(Deserialize)]
pub struct SetDefaultGatewayRequest {
    pub gateway: String,
    pub device: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct HistoryQuery {
    pub identity_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct UsageQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateIdentityRequest {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct DeleteRouteRequest {
    pub destination: String,
    pub gateway: Option<String>,
    pub device: Option<String>,
}

// ─── NAT ───────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct NatRuleDto {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub kind: String,
    pub interface: String,
    pub source: String,
    pub destination: String,
    pub protocol: String,
    pub dest_port: Option<u16>,
    pub target: Option<String>,
    pub description: String,
}

#[derive(Deserialize)]
pub struct CreateNatRuleRequest {
    pub name: String,
    pub kind: String, // masquerade | dnat
    pub interface: String,
    #[serde(default = "default_any")]
    pub source: String,
    #[serde(default = "default_any")]
    pub destination: String,
    #[serde(default = "default_any")]
    pub protocol: String,
    pub dest_port: Option<u16>,
    pub target: Option<String>,
    #[serde(default)]
    pub description: String,
}

fn default_any() -> String {
    "any".into()
}

// ─── Network Zones ─────────────────────────────────────

#[derive(Serialize)]
pub struct ZoneDto {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub accounting: bool,
    pub description: String,
}

#[derive(Deserialize)]
pub struct CreateZoneRequest {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub accounting: bool,
    #[serde(default)]
    pub description: String,
}

#[derive(Deserialize)]
pub struct UpdateZoneRequest {
    pub display_name: String,
    pub accounting: bool,
    #[serde(default)]
    pub description: String,
}

/// خلاصهٔ داشبورد شبکه (N1)
#[derive(Serialize)]
pub struct NetworkSummaryDto {
    pub total_interfaces: usize,
    pub up_interfaces: usize,
    pub down_interfaces: usize,
    pub zones: usize,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub generated_at: u64,
}

#[derive(Serialize)]
pub struct NetworkInterfaceDto {
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

#[derive(Deserialize)]
pub struct SetNetworkZoneRequest {
    pub name: String,
    pub zone: String,
}

#[derive(Deserialize)]
pub struct InterfaceConfigRequest {
    pub name: String,
    pub zone: String,
    pub enabled: bool,
    pub ipv4_mode: String, // none | static | dhcp
    pub address_cidr: Option<String>,
    pub gateway: Option<String>,
    #[serde(default)]
    pub description: String,
}

// ─── فاز ۵: اشیاء FQDN ─────────────────────────────────────

#[derive(Serialize)]
pub struct FqdnObjectDto {
    pub id: String,
    pub name: String,
    pub fqdn: String,
    pub description: String,
    pub enabled: bool,
    /// کش رزولوشن — v4/v6 مخلوط (تفکیک با وجود ':')
    pub resolved: Vec<String>,
    pub last_resolved_at: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateFqdnRequest {
    pub name: String,
    pub fqdn: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateFqdnRequest {
    pub enabled: bool,
}


// ─── [N1b] rename interface ────────────────────────────────────────

#[derive(Deserialize)]
pub struct RenameInterfaceRequest {
    pub name: String,
    pub new_name: String,
}


// ─── N2: PPPoE ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PppoeSessionDto {
    pub iface: String,
    pub profile: Option<String>,
    pub local_ip: Option<String>,
    pub remote_ip: Option<String>,
    pub up: bool,
}

#[derive(Serialize)]
pub struct PppoeProfileDto {
    pub name: String,
    pub parent_iface: String,
    pub username: String,
    pub mtu: u32,
    pub default_route: bool,
    pub keepalive: u32,
    pub enabled: bool,
    pub description: String,
    pub session: Option<PppoeSessionDto>,
}

#[derive(Serialize)]
pub struct PppoeStateDto {
    pub profiles: Vec<PppoeProfileDto>,
    pub sessions: Vec<PppoeSessionDto>,
}

fn dezh_default_mtu() -> u32 {
    1492
}
fn dezh_default_keepalive() -> u32 {
    10
}
fn dezh_default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub struct PppoeSaveRequest {
    pub name: String,
    pub parent_iface: String,
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "dezh_default_mtu")]
    pub mtu: u32,
    #[serde(default = "dezh_default_true")]
    pub default_route: bool,
    #[serde(default = "dezh_default_keepalive")]
    pub keepalive: u32,
    #[serde(default = "dezh_default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: String,
}


// ─── N3: VPN WAN (PPTP / L2TP) ─────────────────────────────────────

#[derive(Serialize)]
pub struct VpnConnectionDto {
    pub name: String,
    pub kind: String,
    pub server: String,
    pub username: String,
    pub mtu: u32,
    pub default_route: bool,
    pub keepalive: u32,
    pub enabled: bool,
    pub description: String,
    pub session: Option<PppoeSessionDto>,
}

#[derive(Serialize)]
pub struct VpnWanStateDto {
    pub connections: Vec<VpnConnectionDto>,
    pub sessions: Vec<PppoeSessionDto>,
}

fn dezh_default_mtu_vpn() -> u32 {
    1400
}

#[derive(Deserialize)]
pub struct VpnSaveRequest {
    pub name: String,
    pub kind: String,
    pub server: String,
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "dezh_default_mtu_vpn")]
    pub mtu: u32,
    #[serde(default)]
    pub default_route: bool,
    #[serde(default = "dezh_default_keepalive")]
    pub keepalive: u32,
    #[serde(default = "dezh_default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: String,
}
