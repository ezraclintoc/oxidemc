# OxideMC Web API — endpoint ↔ screen map

Every screen in the web UI maps to a small, predictable set of endpoints.
Base URL: `http://127.0.0.1:7878`. REST under `/api`, live data over `/ws`.

---

## Servers (home)

| UI element | Method | Path | Returns |
| --- | --- | --- | --- |
| Card grid | `GET` | `/api/servers` | `ServerSummary[]` — name, type, version, motd, port, max_players, status |
| Start button | `POST` | `/api/servers/:name/start` | `{ ok, status:"running" }` |
| Stop button | `POST` | `/api/servers/:name/stop` | `{ ok, status:"stopped" }` |
| Restart button | `POST` | `/api/servers/:name/restart` | `{ ok, status:"running" }` |

`status` is derived: a server is `running` iff it has a live child process in the registry.

---

## Server detail → Monitor tab

One WebSocket carries everything live.

```
GET /ws/servers/:name        (WebSocket upgrade)
```

**server → client**
```jsonc
{ "type": "console", "t": "12:04:56", "level": "INFO",  "source": "Server", "msg": "Done (4.1s)!" }
{ "type": "metrics", "cpu": 34.2, "ram": 61.8, "ram_mb": 3705, "tps": 19.9, "players": ["Notch"] }
{ "type": "status",  "status": "stopped" }   // sent if the server isn't running
```

**client → server** (the RCON command input box)
```jsonc
{ "type": "command", "cmd": "list" }
```
The server echoes the command as a `CMD` console line, runs it over RCON, and pushes the
reply as an `INFO` line — exactly what the mocked console does in the prototype.

| Monitor widget | Source frame |
| --- | --- |
| Console stream | `console` (from piped child stdout) |
| CPU / RAM gauges | `metrics.cpu` / `metrics.ram` (sysinfo, polled 1.5s) |
| TPS chart | `metrics.tps` |
| Player list | `metrics.players` (RCON `list`, polled) |
| Uptime / version / port chips | `GET /api/servers/:name` once on open |

---

## Server detail → Configure tab

| UI element | Method | Path | Body |
| --- | --- | --- | --- |
| Load form | `GET` | `/api/servers/:name` | — → full `ServerState` (oxide.json) |
| Save | `PUT` | `/api/servers/:name` | `ServerState` |

Field validation (clamped numbers, RAM `G/M` regex, RCON-password-required) should run
both client-side (inline errors, no toasts) and in `servers::update` server-side.

---

## New Server wizard

| Step | Method | Path | Notes |
| --- | --- | --- | --- |
| Platform picker | `GET` | `/api/platforms` | `{ platforms: [...] }` from `downloader::get_platforms()` |
| Version picker | `GET` | `/api/platforms/:platform/versions` | async; `{ versions: [...] }` |
| Install (Review → Download) | `POST` | `/api/install` | body: name, type, version, full `ServerState` |
| Download progress | `GET` | `/ws/install/:job` | streams `DownloadProgress` (Started/Chunk/Finished) — **TODO**, see README |

The `/api/install` handler writes `oxide.json` immediately (so the server appears in the list),
then streams `downloader::DownloadProgress` for the progress bar.

---

## Global settings (Settings screen)

| UI element | Method | Path | Body |
| --- | --- | --- | --- |
| Load | `GET` | `/api/manage` | — → `ManageConfig` (manage.json) |
| Save | `PUT` | `/api/manage` | `ManageConfig` |

---

## Status codes

| Situation | Code |
| --- | --- |
| OK | `200` |
| Unknown server | `404` |
| Field validation failed | `422` |
| Start a running server / stop a stopped one / RCON disabled | `409` |
| Core IO / download / launch error | `500` |

All error bodies are `{ "error": "message" }` for inline rendering.
