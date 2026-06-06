use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use oxidemc_core::schema::ServerState;
use serde::Serialize;
use serde_json::Value;

/// Compact summary used by the Servers home grid.
#[derive(Serialize)]
pub struct ServerSummary {
    pub name: String,
    pub server_type: String,
    pub minecraft_version: String,
    pub motd: String,
    pub port: u16,
    pub max_players: u32,
    pub status: &'static str, // running | stopped
}

fn summarize(state: &ServerState, running: bool) -> ServerSummary {
    ServerSummary {
        name: state.server_name.clone(),
        server_type: state.server_type.clone(),
        minecraft_version: state.minecraft_version.clone(),
        motd: state.server.motd.clone(),
        port: state.network.port,
        max_players: state.server.max_players,
        status: if running { "running" } else { "stopped" },
    }
}

/// GET /api/servers  → Servers home (the card grid).
pub async fn list(State(st): State<AppState>) -> ApiResult<Json<Vec<ServerSummary>>> {
    let states = st.list_states()?;
    let mut out = Vec::with_capacity(states.len());
    for s in &states {
        let running = st.is_running(&s.server_name).await;
        out.push(summarize(s, running));
    }
    Ok(Json(out))
}

/// GET /api/servers/:name  → full oxide.json for the Configure tab + header.
pub async fn get(
    State(st): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ServerState>> {
    Ok(Json(st.load_state(&name)?))
}

/// PUT /api/servers/:name  → save edited oxide.json (Configure tab "save").
pub async fn update(
    State(st): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<ServerState>,
) -> ApiResult<Json<ServerState>> {
    // (Add field validation here — clamp ranges, RAM regex, RCON-password-required.)
    let dir = st.server_dir(&name);
    // Override the directory field so oxide.json always reflects the actual path,
    // not whatever the client sent.
    let mut body = body;
    body.directory = dir.to_string_lossy().to_string();
    oxidemc_core::config::save_server_state(&body, &dir)?;
    oxidemc_core::server_properties::write_server_properties(&body, &dir)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(Json(body))
}

/// POST /api/servers/:name/start | /stop | /restart  → the Start/Stop/Restart buttons.
pub async fn action(
    State(st): State<AppState>,
    Path((name, action)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    match action.as_str() {
        "start" => st.start(&name).await?,
        "stop" => st.stop(&name).await?,
        "restart" => st.restart(&name).await?,
        other => return Err(ApiError::Validation(format!("unknown action: {other}"))),
    }
    Ok(Json(serde_json::json!({ "ok": true, "status": if action == "stop" { "stopped" } else { "running" } })))
}
