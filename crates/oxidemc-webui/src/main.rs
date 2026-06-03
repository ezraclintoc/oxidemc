mod error;
mod routes;
mod state;
mod ws;

use axum::routing::{get, post};
use axum::Router;
use state::AppState;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oxidemc_webui=debug,tower_http=info".into()),
        )
        .init();

    let state = AppState::from_manage()?;

    let api = Router::new()
        // ── Servers home + detail ────────────────────────────────────────────
        .route("/servers", get(routes::servers::list))
        .route("/servers/:name", get(routes::servers::get).put(routes::servers::update))
        .route("/servers/:name/:action", post(routes::servers::action)) // start|stop|restart
        // ── Install wizard ───────────────────────────────────────────────────
        .route("/platforms", get(routes::install::platforms))
        .route("/platforms/:platform/versions", get(routes::install::versions))
        .route("/install", post(routes::install::install))
        // ── Global settings ──────────────────────────────────────────────────
        .route("/manage", get(routes::manage::get).put(routes::manage::update));

    let app = Router::new()
        .nest("/api", api)
        // ── live Monitor socket (console + metrics + command input) ──────────
        .route("/ws/servers/:name", get(ws::monitor))
        // ── serve the built React SPA; unknown paths fall back to index.html ─
        .fallback_service(
            ServeDir::new("oxidemc-webui/web/dist")
                .not_found_service(ServeDir::new("oxidemc-webui/web/dist")),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive()) // tighten for production
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    tracing::info!("OxideMC web UI on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
