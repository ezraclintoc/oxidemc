# oxidemc-core

The library crate powering OxideMC. Contains all server management logic — no UI code lives here.

Both `oxidemc-tui` and `oxidemc-webui` depend on this crate.

---

## Modules

| Module | Purpose |
| --- | --- |
| `schema` | All data types — `Question<T>`, config structs, resolved state (`ServerState`) |
| `config` | Load/save JSON files (`oxide.json`, `manage.json`, `install.json`, `configure.json`) |
| `server_properties` | Generate and write `server.properties` from a `ServerState` |
| `rcon` | Async RCON client — connect, send commands, disconnect |
| `downloader` | Download server JARs with streaming progress reporting |
| `server` | Server lifecycle — accept EULA, launch JVM process, stop via RCON |

---

## Creating a Server

The full pipeline for setting up a new server from scratch:

```rust
use std::path::PathBuf;
use tokio::sync::mpsc;
use oxidemc_core::{config, server, server_properties, downloader};

// 1. Load wizard answers (populated by TUI/WebUI from JSON schemas)
let install  = config::load_install()?;
let configure = config::load_configure()?;
let manage   = config::load_manage()?;

// 2. Build resolved ServerState from wizard answers
let state = ServerState {
    server_name:       install.server_name.default.clone(),
    server_type:       install.server_type.default.clone(),
    minecraft_version: install.minecraft_version.default.clone(),
    directory:         install.directory.default.clone(),
    server:      ResolvedServerSection      { /* from configure.server */      },
    network:     ResolvedNetworkSection     { /* from configure.network */     },
    performance: ResolvedPerformanceSection { /* from configure.performance */ },
    players:     ResolvedPlayersSection     { /* from configure.players */     },
    mods:        ResolvedModsSection        { /* from configure.mods */        },
};

let dir = PathBuf::from(&state.directory);

// 3. Download the JAR (TUI/WebUI listens on rx to show progress)
let (tx, rx) = mpsc::channel(32);
let jar = downloader::jar_name(&state.server_type, &state.minecraft_version);
downloader::download_jar(
    &state.server_type,
    &state.minecraft_version,
    &dir.join(&jar),
    tx,
).await?;

// 4. Write server.properties
server_properties::write_server_properties(&state, &dir)?;

// 5. Accept EULA
server::accept_eula(&dir)?;

// 6. Persist state
config::save_server_state(&state, &dir)?;

// 7. Launch (if auto_start is enabled)
if install.auto_start.default {
    let child = server::launch(&state, &manage.java_path.default).await?;
}
```

---

## Download Progress

`download_jar` reports progress via an `mpsc::Sender<DownloadProgress>`. The UI holds the receiver and renders updates as they arrive.

```rust
pub enum DownloadProgress {
    Started { total_bytes: u64 },
    Chunk   { downloaded: u64, total: u64 },
    Finished,
}
```

---

## RCON

`RconClient` is async, built on tokio. Use it to send commands to a running server.

```rust
let mut rcon = RconClient::connect("127.0.0.1", 25575, "password").await?;
let response = rcon.send_command("list").await?;
rcon.disconnect().await?;
```

`server::stop` wraps this — it sends `/stop` and returns.

---

## Schema System

Config files use `Question<T>` — a wrapper that carries a default value and whether the UI should prompt the user.

```rust
pub struct Question<T> {
    pub ask: bool,             // prompt the user at setup?
    pub default: T,            // value used when ask=false or user skips
    pub options: Option<Vec<T>>, // constrain to a list
    pub note: Option<String>,  // hint shown alongside the prompt
    pub requires_rcon: bool,   // warn if RCON is disabled
    pub condition: Option<String>, // skip prompt unless condition is met
}
```

After the wizard runs, answers are flattened into `ServerState` (plain types, no `Question<T>`) and written to `oxide.json`.

---

## Error Types

Each module has its own error enum using `thiserror`:

| Type | Location |
| --- | --- |
| `ConfigError` | `config` |
| `PropertiesError` | `server_properties` |
| `RconError` | `rcon` |
| `DownloadError` | `downloader` |
| `ServerError` | `server` |

All implement `std::error::Error` and have `#[from]` conversions where relevant so `?` works naturally.
