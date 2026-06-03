# Claude Code prompt — build `oxidemc-webui`

Copy everything below the line into your local Claude Code session, run from the
**root of the `oxidemc` workspace**. The scaffold files referenced are in this
handoff folder (`oxidemc-webui/`) — copy them into the repo first, or let Claude
Code recreate them from this prompt.

---

You are working in the `oxidemc` Rust workspace (a Minecraft server manager). The
workspace already lists `crates/oxidemc-webui` as a member in the root `Cargo.toml`,
but the crate is empty. Build it out.

## Goal
A new crate `oxidemc-webui`: an `axum` HTTP + WebSocket server that exposes the
existing `oxidemc-core` logic to a React single-page UI, and serves that UI's static
build. **Reuse `oxidemc-core` — do not reimplement config parsing, RCON, downloading,
or process launching.**

## Use the real core APIs (already implemented in `crates/oxidemc-core`)
- `config::{load_manage, save_manage, load_server_state, save_server_state}` and
  `ConfigError`.
- `downloader::{get_platforms, get_versions, download_jar, jar_name, DownloadProgress,
  DownloadError}` — `download_jar` reports progress over a `tokio::sync::mpsc::Sender`.
- `server::{accept_eula, launch, stop, ServerError}` — note `launch` **inherits stdio**.
- `rcon::RconClient::{connect, send_command, disconnect}` and `RconError`.
- `schema::{ServerState, ManageConfig, ...}` — all `serde`-derived; `ServerState` is the
  per-server `oxide.json`, `ManageConfig` is the global `manage.json`.

## Endpoints (see API.md for full request/response shapes)
REST under `/api`:
- `GET  /api/servers`                      → list summaries (scan `servers_directory`
                                              for sub-folders containing `oxide.json`)
- `GET  /api/servers/:name`                → full `ServerState`
- `PUT  /api/servers/:name`                → save `ServerState` (validate first)
- `POST /api/servers/:name/:action`        → action ∈ {start, stop, restart}
- `GET  /api/platforms`                    → `downloader::get_platforms()`
- `GET  /api/platforms/:platform/versions` → `downloader::get_versions()` (async)
- `POST /api/install`                      → write oxide.json + spawn `download_jar`
- `GET  /api/manage` / `PUT /api/manage`   → global settings

WebSocket:
- `GET /ws/servers/:name` → one socket for the Monitor screen. Server pushes
  `{type:"console",...}` lines and `{type:"metrics",cpu,ram,tps,players}` samples;
  client sends `{type:"command",cmd}` which is run over RCON and echoed back.
- `GET /ws/install/:job` → stream `DownloadProgress` for the wizard progress bar.

Static: serve the React build from `oxidemc-webui/web/dist` with a SPA fallback to
`index.html` (use `tower_http::services::ServeDir`).

## State + behavior
- `AppState { servers_dir, java_path, running: Arc<Mutex<HashMap<String, RunningServer>>> }`,
  built from `manage.json` at startup.
- A server is "running" iff it's in the `running` map.
- **To stream the console you must pipe the child's stdout** (core's `launch` inherits
  stdio). Either add `server::launch_with_stdio(...)` to core, or build the `Command` in
  the web crate mirroring `core::server::launch` but with `Stdio::piped()`. Pump stdout
  lines into a `tokio::sync::broadcast` channel that every Monitor socket subscribes to.
- Metrics: poll the child PID every 1.5s with the `sysinfo` crate for CPU% and RAM (MB and
  % of `-Xmx`). Poll RCON `list` on a slower cadence for the player list. Parse/estimate TPS.
- Stop = RCON `stop` (graceful) then `child.kill()` fallback with a timeout.

## Errors
Single `ApiError` enum with `IntoResponse`: `404` unknown server, `422` validation,
`409` start-already-running / stop-not-running / RCON-disabled, `500` for core errors.
Body is always `{ "error": "..." }` so the UI can show it inline (no toasts).

## Dependencies
`axum` (ws, macros), `tokio` (full), `tower`, `tower-http` (fs, cors, trace), `serde`,
`serde_json`, `sysinfo`, `thiserror`, `anyhow`, `tracing`, `tracing-subscriber`, `futures`,
and `oxidemc-core = { path = "../oxidemc-core" }`.

## Acceptance
1. `cargo run -p oxidemc-webui` serves on `http://127.0.0.1:7878` and logs the address.
2. `GET /api/servers` returns the servers found on disk.
3. `POST /api/servers/:name/start` launches via core and the server then shows as running.
4. Connecting to `/ws/servers/:name` streams console lines; sending a `command` frame runs
   it over RCON and echoes the reply.
5. `GET/PUT /api/manage` round-trips `manage.json`.
6. `cargo clippy -p oxidemc-webui` is clean.

Build it incrementally: `Cargo.toml` → `error.rs` → `state.rs` → `routes/*` → `ws.rs` →
`main.rs`. Then write a short `web/README` describing where the React build goes. Match the
module layout in the provided scaffold.
