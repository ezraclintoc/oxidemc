# OxideMC Roadmap

Features are grouped by area. Items without a milestone are planned but unscheduled.

---

## Core

### v1

- [x] Download server JARs — Vanilla, Paper, Fabric, Forge, NeoForge, Purpur
- [x] `manage.json` — controls which questions are asked and their default values
- [x] `install.json` — drives installation (version, directory, server type)
- [x] `configure.json` — drives server configuration (port, RAM, JVM flags, properties)
- [x] `server.properties` generation from resolved state
- [x] RCON client (async, tokio)
- [x] Server lifecycle — accept EULA, launch JVM, stop via RCON
- [ ] Preset system — load/save presets from the presets folder

### Future

- [ ] Update checker
- [ ] Automatic port forwarding / proxying

---

## TUI

### v1

- [ ] Full ratatui interface (replaces cliclack wizard from v1/v2)
- [ ] Preset selector menu
- [ ] Server management screen (start / stop / restart)
- [ ] Download progress display

---

## WebUI

### Future

- [ ] Browser-based alternative to TUI
- [ ] Parity with TUI feature set

---

## Mod Management

### v1

- [ ] Modrinth support
- [ ] CurseForge support
- [ ] Install mods by URL

---

## Distribution

### v1

- [ ] GitHub Actions — automated binary builds (Linux, macOS, Windows)
- [ ] Install script

---
