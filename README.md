<div align="center">

<img src="logo.png" width="150" style="margin: 20px; border-radius: 50%;" alt="OxideMC Logo">

# OxideMC

**A Rust-powered Minecraft server setup and management tool**

[![GitHub license](https://img.shields.io/github/license/ezraclintoc/OxideMC?style=for-the-badge)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)

</div>

---

## Overview

OxideMC is a fast, configurable tool for setting up and managing Minecraft servers. It handles downloading server JARs, configuring properties, managing mods, and running servers — all from a terminal UI or a browser-based web interface.

The core idea: save a **preset** (a `.json` file in your presets folder), and every new server you create can load it — skipping questions you've already answered and applying your preferred defaults automatically.

## Features

- **Interactive TUI** — full terminal interface built with [ratatui](https://github.com/ratatui-org/ratatui)
- **Web UI** — browser-based server manager at `http://127.0.0.1:7878`
  - Server dashboard with live CPU/RAM gauges, TPS chart, player list
  - Real-time console streaming via WebSocket
  - RCON command input
  - Install wizard for new servers
  - Configure tab with hot-reload (save + optional restart prompt)
  - Global settings page
  - Client-side routing (bookmarkable URLs, browser back/forward)
- **Preset system** — save named presets to reuse across servers
- **Multiple server types** — Vanilla, Paper, Fabric, Forge, NeoForge, Purpur
- **Mod management** — install mods from Modrinth and CurseForge
- **Server lifecycle** — start, stop, and restart servers; graceful RCON stop with SIGKILL fallback
- **Automatic downloads** — fetches server JARs with progress tracking, powered by [msjm](https://crates.io/crates/msjm)
- **TPS monitoring** — platform-aware RCON polling (Paper/Purpur/Spigot use `tps`; Forge/NeoForge use `forge tps` / `neoforge tps`; Vanilla/Fabric show nothing rather than a broken value)

## How the config system works

OxideMC uses JSON files to drive setup:

| File | Purpose |
|------|---------|
| `manage.json` | Global settings — Java path, servers directory, backup config, TUI theme |
| `install.json` | Per-server installation schema (server type, MC version, directory) |
| `configure.json` | Per-server configuration schema (port, RAM, JVM flags, RCON, properties) |
| `oxide.json` | Per-server resolved state — generated after setup, lives in the server directory |

`manage.json` controls the question flow that generates `install.json` and `configure.json`. **Presets** are named `.json` files that override defaults in `install.json` / `configure.json` — e.g. a "Paper SMP" preset that always picks Paper latest and sets RAM to 4G.

The Web UI reads and writes `oxide.json` directly via the `/api/servers/:name` endpoint, and `manage.json` via `/api/manage`.

## Installation

### Binary (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/ezraclintoc/OxideMC/releases/latest).

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `oxidemc-linux-x86_64` |
| macOS ARM64 | `oxidemc-macos-arm64` |
| Windows x86_64 | `oxidemc-windows-x86_64.exe` |

### Build from Source

```bash
git clone https://github.com/ezraclintoc/OxideMC.git
cd OxideMC
cargo build --release

# TUI
./target/release/oxidemc

# Web UI (serves on http://127.0.0.1:7878)
./target/release/oxidemc-webui
```

Requires Rust 1.85+ (edition 2024).

## Running the Web UI

```bash
cargo run -p oxidemc-webui
# OxideMC web UI on http://127.0.0.1:7878
```

The server must be started from the **workspace root** so that `assets/manage.json` and the `crates/oxidemc-webui/src/web/` static files resolve correctly.

Set `RUST_LOG=oxidemc_webui=debug` for verbose output including RCON diagnostics.

## Project Structure

OxideMC is a Cargo workspace with three crates:

```
oxidemc/
├── crates/
│   ├── oxidemc-core/    # Server management logic, config, downloads, RCON
│   ├── oxidemc-tui/     # ratatui terminal interface (binary: oxidemc)
│   └── oxidemc-webui/   # axum HTTP + WebSocket server + React SPA (binary: oxidemc-webui)
├── assets/              # Question schema files (manage.json, install.json, configure.json)
└── docs/                # Config reference, roadmap, structure guide
```

See [`docs/structure.md`](docs/structure.md) for a detailed breakdown and [`crates/oxidemc-webui/API.md`](crates/oxidemc-webui/API.md) for the full REST + WebSocket API reference.

## License

[GNU General Public License v3.0](LICENSE)

---

<div align="center">

Made by [ezraclintoc](https://github.com/ezraclintoc)

*WebUI coded and designed by AI, with human guidance.*

</div>
