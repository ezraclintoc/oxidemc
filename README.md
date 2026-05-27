<div align="center">

<img src="logo.png" width="150" style="margin: 20px; border-radius: 50%;" alt="OxideMC Logo">

# OxideMC

**A Rust-powered Minecraft server setup and management tool**

[![GitHub license](https://img.shields.io/github/license/ezraclintoc/OxideMC?style=for-the-badge)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)

</div>

---

## Overview

OxideMC is a fast, configurable tool for setting up and managing Minecraft servers. It handles downloading server JARs, configuring properties, managing mods, and starting servers — all from a terminal UI or a web interface.

The core idea: save a **preset** (a `.json` file in your presets folder), and every new server you create can load it — skipping questions you've already answered and applying your preferred defaults automatically.

## Features

- **Interactive TUI** — full terminal interface built with [ratatui](https://github.com/ratatui-org/ratatui)
- **Web UI** — browser-based alternative to the TUI
- **Preset system** — save named presets to reuse across servers (e.g. always use Paper, always pick latest version)
- **Multiple server types** — Vanilla, Paper, Fabric, Forge, NeoForge, Purpur
- **Mod management** — install mods from Modrinth and CurseForge
- **Server management** — start, stop, and restart servers from within OxideMC
- **Automatic downloads** — fetches server JARs with progress tracking, powered by [msjm](https://crates.io/crates/msjm)

## How the config system works

OxideMC uses three config files to drive setup:

| File | Purpose |
|------|---------|
| `manage.json` | Global settings — which questions to ask, default values, server type preferences |
| `install.json` | Per-server installation record (JAR, version, directory) |
| `configure.json` | Per-server configuration (port, RAM, JVM flags, properties) |

`manage.json` controls the question flow that generates `install.json` and `configure.json`. **Presets** are separate named `.json` files you can select at startup to override `manage.json` defaults for a specific setup — e.g. a "Paper Latest" preset that skips the version question entirely.

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
./target/release/oxidemc
```

Requires Rust 1.85+ (edition 2024).

## Project Structure

OxideMC is a Cargo workspace with three crates:

```
oxidemc/
├── crates/
│   ├── oxidemc-core/    # Server management logic, config, downloads
│   ├── oxidemc-tui/     # ratatui terminal interface
│   └── oxidemc-webui/   # Web interface
└── docs/
```

## License

[GNU General Public License v3.0](LICENSE)

---

<div align="center">

Made by [ezraclintoc](https://github.com/ezraclintoc)

</div>
