use crate::error::ApiResult;
use axum::Json;
use oxidemc_core::schema::ManageConfig;

/// GET /api/manage  → global OxideMC settings (the Settings screen).
pub async fn get() -> ApiResult<Json<ManageConfig>> {
    Ok(Json(oxidemc_core::config::load_manage()?))
}

/// PUT /api/manage  → save global settings.
///
/// NOTE: restarting the process (or rebuilding AppState) is required for changes
/// to `servers_directory` / `java_path` to take effect, since AppState caches them.
pub async fn update(Json(body): Json<ManageConfig>) -> ApiResult<Json<ManageConfig>> {
    oxidemc_core::config::save_manage(&body)?;
    Ok(Json(oxidemc_core::config::load_manage()?))
}
