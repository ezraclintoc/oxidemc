# oxidemc-webui

An `axum` web server that exposes OxideMC's server-management logic over HTTP +
WebSocket, and serves the React single-page UI. It reuses `oxidemc-core` directly —
no logic is duplicated.

```
┌──────────────┐    REST /api/*      ┌──────────────┐     ┌──────────────┐
│  React SPA   │ ──────────────────▶ │ oxidemc-webui│ ──▶ │ oxidemc-core │
│ (web/dist)   │ ◀───── WS /ws/* ─── │   (axum)     │     │  config/rcon │
└──────────────┘   console+metrics   └──────────────┘     │  downloader  │
                                                           │  server      │
                                                           └──────────────┘
```

## Layout

```
oxidemc-webui/
├── Cargo.toml
├── API.md               # endpoint ↔ screen map (read this first)
├── src/
│   ├── main.rs          # router, static file serving, startup
│   ├── state.rs         # AppState: process registry, launch/stop, metrics poller
│   ├── error.rs         # ApiError → HTTP status + JSON body
│   ├── ws.rs            # Monitor socket: console + metrics + RCON command input
│   └── routes/
│       ├── servers.rs   # list / get / update / start|stop|restart
│       ├── install.rs   # platforms / versions / install
│       └── manage.rs    # global settings get/put
└── web/                 # the React app (build output served from web/dist)
```

## Run

```bash
# from the workspace root
cargo run -p oxidemc-webui
# → OxideMC web UI on http://127.0.0.1:7878
```

Put the built frontend at `oxidemc-webui/web/dist` (Vite/`index.html` + assets).
During UI development, run the SPA dev server separately and let it proxy `/api`
and `/ws` to `:7878` (CORS is permissive in dev).

## Integration notes / TODO

These are the deliberate seams left for you to finish:

1. **Piped stdout for the console.** `core::server::launch` inherits stdio (correct
   for the TUI). `AppState::start` re-implements the launch with piped stdout so the
   console can stream. Consider adding `server::launch_with_stdio(...)` to core so the
   command construction lives in one place.

2. **Install progress WebSocket.** `routes::install` spawns `download_jar` with an
   `mpsc<DownloadProgress>` but currently drains it. Add a jobs registry
   (`Arc<Mutex<HashMap<String, broadcast::Sender<DownloadProgress>>>>` on `AppState`)
   and a `GET /ws/install/:job` handler that forwards frames to the wizard's progress bar.

3. **TPS source.** `metrics.tps` is a placeholder. Vanilla has no TPS readout; parse it
   from Paper/Purpur `tps` RCON output, or estimate from tick timing in the log stream.

4. **Player list polling.** Wire the metrics poller to periodically run RCON `list`
   and parse names into `metrics.players` (kept separate from the 1.5s sysinfo loop so
   a slow RCON call can't stall CPU/RAM sampling).

5. **Validation.** Mirror the field rules (clamped ranges, RAM `G/M` regex,
   RCON-password-required) in `servers::update` — the UI shows inline errors, but the
   API must enforce them too.

6. **AppState cache.** `servers_directory` / `java_path` are read once at startup.
   After `PUT /api/manage`, rebuild `AppState` or hot-reload those fields.

7. **Auth / binding.** Binds to `127.0.0.1` and CORS is permissive — fine for local
   single-user. Add a token + tighten CORS before exposing on a network.
