use crate::error::{ApiError, ApiResult};
use crate::state::{AppState, InstallMsg};
use axum::extract::{Path, State};
use axum::Json;
use oxidemc_core::downloader::DownloadProgress;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::{broadcast, mpsc};

pub async fn platforms() -> Json<Value> {
    Json(json!({ "platforms": oxidemc_core::downloader::get_platforms() }))
}

pub async fn versions(Path(platform): Path<String>) -> ApiResult<Json<Value>> {
    let versions = oxidemc_core::downloader::get_versions(&platform).await?;
    Ok(Json(json!({ "platform": platform, "versions": versions })))
}

#[derive(Deserialize)]
pub struct InstallRequest {
    pub server_name: String,
    pub server_type: String,
    pub minecraft_version: String,
    pub state: oxidemc_core::schema::ServerState,
}

/// POST /api/install — write config files, start download, register progress channel.
pub async fn install(
    State(st): State<AppState>,
    Json(req): Json<InstallRequest>,
) -> ApiResult<Json<Value>> {
    let job = req.server_name.clone();
    let dir = st.server_dir(&req.server_name);
    let dest = dir.join(oxidemc_core::downloader::jar_name(&req.server_type, &req.minecraft_version));

    // Ensure the directory field in oxide.json reflects the actual path on disk,
    // not the hardcoded client-side guess.
    let mut state = req.state;
    state.directory = dir.to_string_lossy().to_string();

    // Write config files. On failure, roll back the directory if we created it.
    let dir_existed = dir.exists();
    if let Err(e) = write_install_files(&state, &dir) {
        if !dir_existed {
            let _ = std::fs::remove_dir_all(&dir);
        }
        return Err(e);
    }

    // Register a broadcast channel so the install WS can stream progress.
    let (bcast_tx, _) = broadcast::channel::<InstallMsg>(128);
    st.install_jobs.lock().await.insert(job.clone(), bcast_tx.clone());

    let platform = req.server_type.clone();
    let version = req.minecraft_version.clone();
    let jobs = st.install_jobs.clone();
    let job_key = job.clone();

    tokio::spawn(async move {
        let (mpsc_tx, mut mpsc_rx) = mpsc::channel::<DownloadProgress>(32);

        // Forward mpsc DownloadProgress → broadcast InstallMsg concurrently.
        let fwd_tx = bcast_tx.clone();
        let forward = tokio::spawn(async move {
            while let Some(p) = mpsc_rx.recv().await {
                if let DownloadProgress::Chunk { downloaded, total } = p {
                    let _ = fwd_tx.send(InstallMsg::Progress { downloaded, total });
                }
            }
        });

        let result = oxidemc_core::downloader::download_jar(&platform, &version, &dest, mpsc_tx).await;
        let _ = forward.await;

        match result {
            Ok(()) => { let _ = bcast_tx.send(InstallMsg::Done); }
            Err(e) => { let _ = bcast_tx.send(InstallMsg::Error { msg: e.to_string() }); }
        }

        jobs.lock().await.remove(&job_key);
    });

    Ok(Json(json!({ "job": job, "status": "downloading" })))
}

fn write_install_files(
    state: &oxidemc_core::schema::ServerState,
    dir: &std::path::PathBuf,
) -> ApiResult<()> {
    oxidemc_core::config::save_server_state(state, dir)?;
    oxidemc_core::server::accept_eula(dir)?;
    oxidemc_core::server_properties::write_server_properties(state, dir)
        .map_err(|e| ApiError::Io(std::io::Error::other(e.to_string())))?;
    Ok(())
}
