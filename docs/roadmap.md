# OxideMC Roadmap

Milestones are ordered: **Alpha → Beta → v1.0.0**. Each milestone separates features from bug fixes.

---

## Alpha — v0.1.0

### Alpha Features

#### Core

- [x] Download server JARs — Vanilla, Paper, Fabric, Forge, NeoForge, Purpur
- [x] `manage.json` — controls which questions are asked and their default values
- [x] `server.properties` generation from resolved state
- [x] `eula.txt` written automatically on install and server start
- [x] RCON client (async, tokio)
- [x] Server lifecycle — accept EULA, launch JVM, stop via RCON, restart

#### Web UI

- [x] axum HTTP + WebSocket server on `http://127.0.0.1:7878`
- [x] Server dashboard — card grid with status, platform logo, player count, TPS
- [x] Live monitor tab — real-time console (piped stdout), CPU/RAM gauges, TPS chart, player list
- [x] RCON command input in the console panel
- [x] Platform-aware TPS polling via RCON (Paper/Purpur/Spigot, Forge, NeoForge; Vanilla/Fabric hidden)
- [x] TPS probe timeout + persistence (`OXIDEMC_TPS_TIMEOUT`, `OXIDEMC_TPS_PERSIST` env vars)
- [x] Configure tab — load/save `oxide.json`; restart prompt when saving while running
- [x] Install wizard — platform picker, version picker (live from API), settings, review + EULA notice
- [x] Install progress WebSocket (`/ws/install/:job`) — real-time download progress bar
- [x] Global settings page — paths, backup toggle, TUI theme, WebUI background theme
- [x] Client-side routing via page.js (`/`, `/servers/:name`, `/servers/:name/configure`, `/new`, `/settings`)
- [x] Console log cache — survives full navigation away and back to a server
- [x] RCON warning banner when RCON is disabled

#### TUI

- [x] Full ratatui interface with install wizard and theme system
- [x] Download progress display

### Alpha Bug Fixes

- [x] WebSocket infinite loop after server stop — distinguish `Closed` from `Lagged` in `RecvError`
- [x] Metrics reset to zero every 5s — `summaryToServer` was overwriting live fields; fixed with `??` fallback
- [x] Mutex starvation in `stop()` — drop lock guard immediately before any I/O
- [x] Half-installed server left on disk on failure — `remove_dir_all` rollback when directory didn't exist
- [x] Stale RCON credentials after Configure save — re-read `oxide.json` per poll cycle
- [x] Wrong `directory` saved in `oxide.json` — backend overrides with resolved `server_dir()` before saving
- [x] TOCTOU race in `restart()` — treat `NotRunning` as non-fatal in the stop phase
- [x] Case-sensitive server filter — `.toLowerCase()` on both sides
- [x] 0 TPS rendered as "..." — gated display on `tps_available` boolean instead of value check
- [x] RCON reconnect on every TPS poll — persistent connection, reconnect only on failure or credential change
- [x] TUI creates `./~/servers/server-name` instead of `/home/user/servers/server-name` — tilde not expanded when loading path from `manage.json`

---

## Beta — v0.2.0

### Beta Features

#### Core

- [ ] Preset system — save and load named server configurations from the presets folder
- [ ] Backup engine — manual and scheduled backups, configurable retention count and destination
- [ ] Crash detection and auto-restart — detect unexpected JVM exit, restart with configurable cooldown
- [ ] Java version detection — auto-detect installed JDKs and suggest the correct version per MC release

#### Web UI

- [ ] AppState hot-reload after `PUT /api/manage` — `servers_directory` / `java_path` changes take effect without process restart
- [ ] Server-side API field validation — mirrors client-side rules; returns structured errors
- [ ] Scheduled restarts — cron-style schedule per server (e.g. daily at 4am)
- [ ] Player management — op/deop, ban/unban, whitelist add/remove via RCON
- [ ] Port conflict detection — warn before start if the configured port is already bound

#### Mod Management

- [ ] Modrinth support
- [ ] CurseForge support
- [ ] Install mods/plugins by URL

#### TUI

- [ ] Server management screen — start, stop, restart, status
- [ ] Preset selector menu — pick from saved presets at install time
- [ ] Live console streaming — real-time stdout tail per server
- [ ] Server configuration menu — edit `oxide.json` fields inline
- [ ] Global settings menu — edit `manage.json` (paths, java, backup config)
- [ ] Theme manager — create, edit, preview, and switch themes

#### Distribution

- [ ] GitHub Actions — automated release builds for Linux, macOS, Windows (x86-64 + ARM)
- [ ] Install script

### Beta Bug Fixes

- [ ] *(tracked as discovered)*

---

## v1.0.0

### v1.0.0 Features

#### Web UI

- [ ] Authentication — token-based auth; configurable bind address (currently 127.0.0.1-only with permissive CORS)
- [ ] World management — list, delete, and reset worlds per server
- [ ] Log file persistence — write piped stdout to a rotating log file; viewable after restart

#### Core

- [ ] Update checker — notify when a newer OxideMC release is available
- [ ] Automatic port forwarding / UPnP

### v1.0.0 Bug Fixes

- [ ] *(tracked as discovered)*

---

## Unscheduled / Future

- [ ] Multi-user support — per-user permissions alongside auth
- [ ] Discord / webhook notifications — server start, stop, crash, player join/leave events
- [ ] Server performance profiles — one-click JVM flag presets tuned by player count
- [ ] Mobile-responsive Web UI
