use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

// --- Method-specific params and results ---

// Method: "init"
#[derive(Serialize, Deserialize, Debug)]
pub struct InitParams {
    pub api_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitResult {
    pub plugin_name: String,
    pub plugin_version: String,
    pub capabilities: Vec<String>,
}

// Method: "on_select"
#[derive(Serialize, Deserialize, Debug)]
pub struct OnSelectParams {
    pub path: PathBuf,
    pub mime_type: Option<String>,
}

// A preview can be simple text for now
#[derive(Serialize, Deserialize, Debug)]
pub enum PreviewResult {
    Text(String),
    Error(String),
}
