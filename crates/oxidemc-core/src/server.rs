use crate::rcon::RconClient;
use crate::schema::ServerState;
use std::fs;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Java is not found or installed")]
    JavaNotFound,

    #[error("RCON error: {0}")]
    Rcon(#[from] crate::rcon::RconError),
}

pub fn accept_eula(server_dir: &PathBuf) -> Result<(), ServerError> {
    fs::write(
        server_dir.join("eula.txt"),
        "//This file was edited by OxideMC\neula=true\n",
    )?;
    Ok(())
}

pub async fn launch(
    state: &ServerState,
    java_path: &str,
) -> Result<tokio::process::Child, ServerError> {
    let java_bin = if java_path.is_empty() {
        "java"
    } else {
        java_path
    };
    if !java_path.is_empty() && !PathBuf::from(java_path).exists() {
        return Err(ServerError::JavaNotFound);
    }

    let jar_name = format!("{}-{}.jar", state.server_type, state.minecraft_version);

    let child = Command::new(java_bin)
        .current_dir(&state.directory)
        .arg(format!("-Xms{}", state.performance.min_ram))
        .arg(format!("-Xmx{}", state.performance.max_ram))
        .args(state.performance.jvm_flags.split_whitespace())
        .arg("-jar")
        .arg(&jar_name)
        .arg("--nogui")
        .spawn()?;

    Ok(child)
}

pub async fn stop(rcon: &mut RconClient) -> Result<(), ServerError> {
    rcon.send_command("stop").await?;
    Ok(())
}
