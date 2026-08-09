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
