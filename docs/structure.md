# OxideMC Structure

## Workspace Crates

```text
oxidemc/
├── crates/
│   ├── oxidemc-core/    # server logic, config parsing, JAR downloads, RCON, server.properties
│   ├── oxidemc-tui/     # ratatui terminal UI (binary: oxidemc)
│   └── oxidemc-webui/   # axum web server + React SPA (binary: oxidemc-webui)
├── assets/              # question schema files shipped with the binary
└── docs/
```

`oxidemc-tui` and `oxidemc-webui` both depend on `oxidemc-core`. No UI code lives in core.

---

## Assets (question schema files)

Shipped with OxideMC. Define what questions are shown and their default values.

```text
assets/
├── manage.json          # global OxideMC settings (Java path, servers dir, backup, theme)
├── install.json         # questions shown when setting up a new server
├── configure.json       # questions shown when configuring a server
└── presets/             # named presets that override question defaults
    └── example.json
```

---

## oxidemc-core modules

```text
crates/oxidemc-core/src/
├── schema.rs            # Question<T>, all config structs, ServerState (resolved)
├── config.rs            # load/save JSON files (oxide.json, manage/install/configure)
├── server_properties.rs # generate and write server.properties from ServerState
├── rcon.rs              # async RCON client (connect, send_command, disconnect)
├── downloader.rs        # download server JARs with streaming DownloadProgress
└── server.rs            # server lifecycle (accept_eula, launch, stop)
```

---

## oxidemc-webui

```text
crates/oxidemc-webui/
├── Cargo.toml
├── API.md               # endpoint <-> screen map (REST + WebSocket reference)
├── README.md            # crate-level docs and run instructions
└── src/
    ├── main.rs          # router, static file serving, startup
    ├── state.rs         # AppState: process registry, piped launch, console pump,
    │                    #   sysinfo metrics poller, platform-aware RCON TPS polling,
    │                    #   player join/leave tracking from stdout
    ├── error.rs         # ApiError -> HTTP status + JSON body
    ├── ws.rs            # Monitor WebSocket: console + metrics (cpu/ram/tps/players/uptime)
    │                    #   + RCON command input
    └── routes/
        ├── servers.rs   # list / get / update (writes oxide.json + server.properties)
        │                #   / start | stop | restart
        ├── install.rs   # platforms / versions / install (writes oxide.json + eula + props)
        └── manage.rs    # global settings get / put
```

The React SPA lives at `crates/oxidemc-webui/src/web/` and is served directly (no build step — uses CDN React + Babel standalone):

```text
src/web/
├── index.html           # entry point; all asset refs use absolute paths (/)
├── styles.css           # design tokens, dark-mode palette, component styles
├── assets/              # platform logo PNGs (paper, fabric, purpur, neoforge, vanilla)
└── js/
    ├── page.js          # client-side router (page.js 1.11.6, local copy)
    ├── data.jsx         # icons, platform catalogue, API helpers (apiFetch, wsUrl),
    │                    #   ServerState <-> flat form mapping helpers
    ├── components.jsx   # shared primitives: Gauge, BarMeter, AreaChart, Toggle,
    │                    #   PlatformLogo, StatusPill, Footer, PlayerHead
    ├── settings.jsx     # SettingsForm — drives both Configure tab and install wizard
    ├── monitor.jsx      # MonitorView: TPS chart, CPU/RAM gauges, console, players
    │                    #   useLiveServer hook: WebSocket + module-level log cache
    ├── wizard.jsx       # NewServerWizard: 5-step install flow
    └── app.jsx          # App shell: sidebar, routing, server actions, toasts,
                         #   GlobalSettings, RestartPrompt, ServerDetail
```

---

## Per-server files

Each server OxideMC manages gets these files in its directory:

```text
<server-directory>/
├── oxide.json           # saved state — answers to all install/configure questions
├── server.properties    # generated from ServerState; regenerated on install and Configure save
├── eula.txt             # written by accept_eula() on install and on server start
└── <type>-<version>.jar # e.g. paper-1.21.4.jar
```

`oxide.json` captures the full state of a server setup. Loading it as a preset lets you duplicate that server exactly. The Web UI reads and writes it directly via the REST API.

---

## Configuration Flow

```mermaid
graph TD
    M[assets/manage.json] -->|configures question schema of| I[assets/install.json]
    M -->|configures question schema of| C[assets/configure.json]
    M -->|configures OxideMC settings| OX[OxideMC]
    P[assets/presets/*.json] -->|overrides defaults in| I
    P -->|overrides defaults in| C
    O[oxide.json] -->|load as preset to| P
    I --> S{server}
    C --> S
```

---

## Routing (Web UI)

The SPA uses `page.js` for client-side routing. The axum server returns `index.html` for all unmatched paths (SPA fallback), so deep links and browser refresh work correctly.

| URL | Screen |
| --- | --- |
| `/` | Servers home (card grid) |
| `/servers/:name` | Server detail — Monitor tab |
| `/servers/:name/configure` | Server detail — Configure tab |
| `/new` | Install wizard |
| `/settings` | Global settings |
