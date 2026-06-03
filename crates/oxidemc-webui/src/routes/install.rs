use crate::error::ApiResult;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

/// GET /api/platforms  → wizard step 2 (platform picker cards).
pub async fn platforms() -> Json<Value> {
    Json(json!({ "platforms": oxidemc_core::downloader::get_platforms() }))
}

/// GET /api/platforms/:platform/versions  → wizard step 3 (async version list).
pub async fn versions(Path(platform): Path<String>) -> ApiResult<Json<Value>> {
    let versions = oxidemc_core::downloader::get_versions(&platform).await?;
    Ok(Json(json!({ "platform": platform, "versions": versions })))
}

#[derive(Deserialize)]
pub struct InstallRequest {
    pub server_name: String,
    pub server_type: String,
    pub minecraft_version: String,
    /// Full resolved oxide.json the wizard built on the Settings + Review steps.
    pub state: oxidemc_core::schema::ServerState,
}

/// POST /api/install  → kick off an install job; returns a job id the client
/// then subscribes to over `GET /ws/install/:job` for live download progress
/// (wizard "Download" step progress bar).
pub async fn install(
    State(st): State<AppState>,
    Json(req): Json<InstallRequest>,
) -> ApiResult<Json<Value>> {
    let job = req.server_name.clone();
    let dir = st.server_dir(&req.server_name);
    let dest = dir.join(oxidemc_core::downloader::jar_name(&req.server_type, &req.minecraft_version));

    // persist oxide.json up front so the server shows in the list immediately
    oxidemc_core::config::save_server_state(&req.state, &dir)?;

    // spawn the download; stream DownloadProgress over the install WS (see ws.rs).
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let platform = req.server_type.clone();
    let version = req.minecraft_version.clone();
    tokio::spawn(async move {
        let _ = oxidemc_core::downloader::download_jar(&platform, &version, &dest, tx).await;
    });
    // TODO: register `rx` in a jobs registry keyed by `job` so the WS handler
    // can forward DownloadProgress frames to the browser. Drain here for now.
    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    Ok(Json(json!({ "job": job, "status": "downloading" })))
}
