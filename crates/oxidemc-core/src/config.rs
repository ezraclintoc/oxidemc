use crate::schema::{ConfigureConfig, InstallConfig, ManageConfig, ServerState};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn load_manage() -> Result<ManageConfig, ConfigError> {
    let raw = read_to_string("assets/manage.json")?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn load_install() -> Result<InstallConfig, ConfigError> {
    let raw = read_to_string("assets/install.json")?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn load_configure() -> Result<ConfigureConfig, ConfigError> {
    let raw = read_to_string("assets/configure.json")?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_manage(config: &ManageConfig) -> Result<(), ConfigError> {
    let json = serde_json::to_string_pretty(config)?;
    let mut file = File::create("assets/manage.json")?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

// ── Per-server state (oxide.json) ─────────────────────────────────────────────

pub fn save_server_state(state: &ServerState, server_dir: &PathBuf) -> Result<(), ConfigError> {
    std::fs::create_dir_all(server_dir)?;
    let json = serde_json::to_string_pretty(state)?;
    let mut file = File::create(server_dir.join("oxide.json"))?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn load_server_state(server_dir: &PathBuf) -> Result<ServerState, ConfigError> {
    let raw = read_to_string(server_dir.join("oxide.json"))?;
    Ok(serde_json::from_str(&raw)?)
}
