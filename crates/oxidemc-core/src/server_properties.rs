use crate::schema::ServerState;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

#[derive(Debug, thiserror::Error)]
pub enum PropertiesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

fn build_properties(state: &ServerState) -> String {
    let s = &state.server;
    let n = &state.network;
    let p = &state.performance;
    let rcon = &n.rcon;

    let mut lines = vec![
        format!("difficulty={}", s.difficulty),
        format!("gamemode={}", s.gamemode),
        format!("level-name={}", s.level_name),
        format!("level-seed={}", s.seed),
        format!("max-players={}", s.max_players),
        format!("motd={}", s.motd),
        format!("online-mode={}", s.online_mode),
        format!("pvp={}", s.pvp),
        format!("server-port={}", n.port),
        format!("simulation-distance={}", p.simulation_distance),
        format!("spawn-protection={}", s.spawn_protection),
        format!("view-distance={}", p.render_distance),
        format!("white-list={}", s.whitelist),
        // hardcoded defaults
        "allow-flight=false".into(),
        "broadcast-console-to-ops=true".into(),
        "broadcast-rcon-to-ops=true".into(),
        "enable-query=false".into(),
        "enable-status=true".into(),
        "enforce-secure-profile=true".into(),
        "enforce-whitelist=false".into(),
        "force-gamemode=false".into(),
        "generate-structures=true".into(),
        "generator-settings={}".into(),
        "hardcore=false".into(),
        "level-type=minecraft\\:normal".into(),
        "log-ips=true".into(),
        "max-chained-neighbor-updates=1000000".into(),
        "max-tick-time=60000".into(),
        "max-world-size=29999984".into(),
        "network-compression-threshold=256".into(),
        "op-permission-level=4".into(),
        "player-idle-timeout=0".into(),
        "prevent-proxy-connections=false".into(),
        "rate-limit=0".into(),
        "region-file-compression=deflate".into(),
        "sync-chunk-writes=true".into(),
        "use-native-transport=true".into(),
    ];

    // only write rcon keys if enabled
    lines.push(format!("enable-rcon={}", rcon.enabled));
    if rcon.enabled {
        lines.push(format!("rcon.port={}", rcon.port));
        lines.push(format!("rcon.password={}", rcon.password));
    }

    lines.sort();
    lines.join("\n")
}

fn write_properties(contents: &str, server_dir: &PathBuf) -> Result<(), PropertiesError> {
    std::fs::create_dir_all(server_dir)?;
    let mut file = File::create(server_dir.join("server.properties"))?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn write_server_properties(state: &ServerState, server_dir: &PathBuf) -> Result<(), PropertiesError> {
    let contents = build_properties(state);
    write_properties(&contents, server_dir)
}