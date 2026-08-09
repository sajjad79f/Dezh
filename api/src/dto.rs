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