# oxidemc-webui

An `axum` web server that exposes OxideMC's server-management logic over HTTP +
WebSocket, and serves the React single-page UI. It reuses `oxidemc-core` directly —
no logic is duplicated.

```
+----------------+    REST /api/*      +----------------+     +--------------+
|   React SPA    | ------------------> | oxidemc-webui  | --> | oxidemc-core |
|  (src/web/)    | <--- WS /ws/*  ---- |    (axum)      |     |  config/rcon |
+----------------+  console+metrics    +----------------+     |  downloader  |
                                                               |  server      |
                                                               +--------------+
```

## Layout

```
oxidemc-webui/
├── Cargo.toml
├── API.md               # endpoint <-> screen map (read this first)
├── README.md            # this file
└── src/
    ├── main.rs          # router, static file serving, startup
    ├── state.rs         # AppState: process registry, piped launch/stop,
    │                    #   console stdout pump + player join/leave tracking,
    │                    #   sysinfo metrics poller (1.5s),
    │                    #   platform-aware RCON TPS polling (~10s)
    ├── error.rs         # ApiError -> HTTP status + JSON body
    ├── ws.rs            # Monitor WebSocket: console + metrics + RCON command input
    └── routes/
        ├── servers.rs   # list / get / update / start | stop | restart
        ├── install.rs   # platforms / versions / install
        └── manage.rs    # global settings get / put
    └── web/             # React SPA (no build step required)
        ├── index.html   # entry; all asset refs absolute (/)
        ├── styles.css
        ├── assets/      # platform logo PNGs
        └── js/
            ├── page.js          # page.js router (local copy)
            ├── data.jsx         # icons, API helpers, data mapping
            ├── components.jsx   # shared UI primitives
            ├── settings.jsx     # settings form (Configure + wizard)
            ├── monitor.jsx      # Monitor tab: console, gauges, TPS, players
            ├── wizard.jsx       # install wizard (5 steps)
            └── app.jsx          # app shell, routing, toasts, dialogs
```

## Run

```bash
# from the workspace root
cargo run -p oxidemc-webui
# -> OxideMC web UI on http://127.0.0.1:7878
```

The server must be started from the **workspace root** so that:
- `assets/manage.json` resolves correctly for startup config
- `crates/oxidemc-webui/src/web/` resolves for static file serving

For debug output including RCON TPS diagnostics:

```bash
RUST_LOG=oxidemc_webui=debug cargo run -p oxidemc-webui
```

## How the console works

When a server starts (`AppState::start`), the JVM is spawned with `stdout: Stdio::piped()`.
A background task reads stdout line by line, parses each into a `ConsoleLine` (timestamp,
level, source, message), and broadcasts it over a `tokio::sync::broadcast::Sender`.
Player join/leave events are also detected from these lines to maintain the live player list.

When a browser connects to `GET /ws/servers/:name`, the WebSocket handler subscribes to that
broadcast channel and forwards every new line as a `{type:"console"}` frame. Metrics
(`{type:"metrics"}`) are pushed every 1.5s from the sysinfo poller, including TPS when RCON
is available and the platform supports it.

## TPS polling

TPS is polled via RCON every ~10 seconds. The command used depends on the server platform:

| Platform | Command | Notes |
| --- | --- | --- |
| Paper / Purpur / Spigot | `tps` | Returns 1m/5m/15m values |
| Forge | `forge tps` | Returns per-dimension + Overall |
| NeoForge | `neoforge tps` | Returns per-dimension + Overall |
| Vanilla / Fabric | — | No built-in TPS command; panel is hidden |

If RCON is disabled, or the server type has no TPS command, the TPS chart and stat are
hidden rather than showing a misleading `--` value.

## API

See [`API.md`](API.md) for the full REST + WebSocket endpoint reference.

## Known limitations / future work

1. **Install progress WebSocket** — `POST /api/install` spawns the download and currently
   polls until the server appears rather than streaming `DownloadProgress` frames over
   `/ws/install/:job`. A jobs registry keyed by server name is the missing piece.

2. **AppState hot-reload** — `servers_directory` and `java_path` are read once at startup.
   Changes via `PUT /api/manage` take effect on the next run.

3. **Server-side field validation** — the Configure tab enforces rules client-side (RAM
   format, port range, RCON password required). `servers::update` should mirror these.

4. **Auth** — binds to `127.0.0.1` and CORS is permissive. Add a token and tighten CORS
   before exposing on a network.
