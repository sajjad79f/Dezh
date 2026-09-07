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

#[derive(Serialize)]
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
}

fn default_priority() -> u32 {
    100
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub role: String,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub role: String,
    pub display_name: Option<String>,
    pub enabled: bool,
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

#[derive(Serialize)]
pub struct AccountingSessionDto {
    pub id: String,
    pub identity_id: Option<String>,
    pub protocol: String,
    pub ip_address: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub bytes_in: i64,
    pub bytes_out: i64,
}

#[derive(Deserialize)]
pub struct StartSessionRequest {
    pub identity_id: Option<String>,
    pub protocol: String, // vpn | nac | portal | other
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
    pub agent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct AgentUserInactiveRequest {
    pub username: String,
    pub ip_address: Option<String>,
    #[serde(default)]
    pub bytes_in: i64,
    #[serde(default)]
    pub bytes_out: i64,
}

#[derive(Serialize)]
pub struct InterfaceDto {
    pub name: String,
    pub zone: String,
    pub up: bool,
    pub addresses: Vec<String>,
}

#[derive(Deserialize)]
pub struct SetZoneRequest {
    pub name: String,
    pub zone: String,
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