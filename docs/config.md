# Configuration Reference

OxideMC uses JSON files in two roles:

- **Schema files** (`assets/*.json`) — shipped with OxideMC. Define which questions are shown during setup and their default values.
- **State files** (`oxide.json`) — generated per-server after setup. Record every answered value. Can be loaded as a preset to duplicate a server.

---

## Question shape

Every leaf value in a schema file is a `Question` object:

```json
{
  "ask": true,
  "default": "survival",
  "options": ["survival", "creative", "adventure", "spectator"],
  "note": "Optional hint shown alongside the prompt.",
  "requires_rcon": false
}
```

| Field | Type | Description |
| --- | --- | --- |
| `ask` | `bool` | `true` = prompt the user. `false` = silently use `default`. |
| `default` | any | Value used when `ask` is `false`, or when the user accepts without typing. |
| `options` | `T[]` \| omit | Constrained list shown as a picker. Omit for free-form input. |
| `note` | `string` \| omit | Warning or hint rendered below the prompt. |
| `requires_rcon` | `bool` \| omit | `true` = this feature is applied at runtime via RCON. Omit or `false` otherwise. |
| `condition` | `string` \| omit | Named predicate the UI evaluates at runtime. Prompt is skipped entirely if false. |

---

## manage.json

Global OxideMC settings. All fields default to `ask: false` — edit the file directly to change them.

| Field | Default | Description |
| --- | --- | --- |
| `data_directory` | `~/.config/oxidemc` | Where OxideMC stores its own data. |
| `presets_directory` | `~/.config/oxidemc/presets` | Where preset files are loaded from. |
| `servers_directory` | `~/servers` | Default parent directory for new servers. |
| `java_path` | `""` | Path to a Java executable. Empty = use `java` found on `$PATH`. |
| `check_java_version` | `true` | Warn if Java version is incompatible with the MC version being installed. |
| `auto_update_check` | `true` | Check for OxideMC updates on launch. |
| `backup_directory` | `~/.config/oxidemc/backups` | Where server backups are stored. |
| `backup_count` | `5` | Rolling backups kept per server. `0` = unlimited. |
| `log_level` | `"info"` | OxideMC log verbosity. Options: `error`, `warn`, `info`, `debug`. |
| `default_preset` | `""` | Preset name to load automatically; skips the preset picker when set. |
| `theme` | `"default"` | TUI color theme name. |

---

## install.json

Drives the install wizard.

| Field | Default | Description |
| --- | --- | --- |
| `server_name` | `"my-server"` | Human-readable name for this server. |
| `server_type` | `"paper"` | Server software. Options: `vanilla`, `paper`, `fabric`, `forge`, `neoforge`, `purpur`. |
| `minecraft_version` | `"latest"` | MC version to install. `"latest"` always picks newest stable. |
| `directory` | `"./servers"` | Parent directory for the server folder. |
| `eula_auto_accept` | `true` | Accept the Minecraft EULA automatically without prompting. |
| `auto_start` | `false` | Start the server immediately after install completes. |
| `create_start_script` | `true` | Generate a `start.sh` / `start.bat` in the server directory. |
| `icon` | `""` | Path to a `server-icon.png` (64×64) to copy into the server directory. |
| `load_preset` | `""` | Preset name to apply; skips the wizard when set. |
| `backup_before_reinstall` | `true` | Back up the existing directory before overwriting. Only shown when the target directory already exists and is non-empty (`condition: install_dir_exists_and_nonempty`). |

---

## configure.json

Drives the configuration wizard. Settings are grouped into submenus — each top-level key is a separate screen.

### `server` — Server

| Field | Default | Description |
| --- | --- | --- |
| `motd` | `"A Minecraft Server"` | Message shown in the server list. |
| `max_players` | `20` | Maximum concurrent players. |
| `gamemode` | `"survival"` | Default gamemode. Options: `survival`, `creative`, `adventure`, `spectator`. |
| `difficulty` | `"normal"` | World difficulty. Options: `peaceful`, `easy`, `normal`, `hard`. |
| `pvp` | `true` | Allow player-vs-player combat. |
| `whitelist` | `false` | Only allow players on the whitelist to join. |
| `online_mode` | `true` | Verify player accounts with Mojang/Microsoft. Disable only for offline/LAN setups. |
| `level_name` | `"world"` | World folder name. Changing this starts a fresh world. |
| `seed` | `""` | World generation seed. Empty string = random. |
| `spawn_protection` | `16` | Block radius around spawn only ops can build in. `0` = disabled. |

### `network` — Network

| Field | Default | Description |
| --- | --- | --- |
| `port` | `25565` | Port the server listens on. |
| `rcon.enabled` | `true` | Enable RCON. **Required for OxideMC live management** (console, player commands, restart hooks). |
| `rcon.port` | `25575` | RCON port. |
| `rcon.password` | `""` | RCON password. Required when RCON is enabled. |

### `performance` — Performance

| Field | Default | Description |
| --- | --- | --- |
| `max_ram` | `"2G"` | JVM max heap (`-Xmx`). Use `G`/`M` suffix. |
| `min_ram` | `"1G"` | JVM initial heap (`-Xms`). |
| `jvm_flags` | `""` | Extra JVM arguments appended to the launch command. |
| `render_distance` | `10` | Chunk view distance sent to clients (chunks). |
| `simulation_distance` | `10` | Chunk simulation distance for mobs/redstone (chunks). |

### `players` — Players

All three fields are applied at runtime via RCON (`requires_rcon: true`). They will have no effect if RCON is disabled.

| Field | Default | Description |
| --- | --- | --- |
| `ops` | `[]` | Player names or UUIDs to op on first start. |
| `banned_players` | `[]` | Player names or UUIDs to ban on first start. |
| `whitelist_players` | `[]` | Player names or UUIDs to add to the whitelist. |

### `mods` — Mods

| Field | Default | Description |
| --- | --- | --- |
| `mods` | `[]` | Pre-configured mods to install. `ask: true` prompts the user to add more interactively. |

Each entry in the `mods` array:

```json
{
  "name": "Lithium",
  "id": "lithium",
  "version": "0.13.0",
  "source": "modrinth"
}
```

| Field | Description |
| --- | --- |
| `name` | Display name. |
| `id` | Modrinth/CurseForge project ID or slug. |
| `version` | Version string. Omit to use latest compatible. |
| `source` | `modrinth`, `curse_forge`, or `direct`. |
| `url` | Download URL. Required when `source` is `direct`. |

### `platform_specific` — Platform-specific

Only the block matching the installed `server_type` is shown. Absent blocks are silently skipped.

#### `paper`

| Field | Default | Description |
| --- | --- | --- |
| `tnt_duping` | `false` | Allow TNT duplication via quasi-connectivity. |
| `bedrock_breaking` | `false` | Allow breaking bedrock blocks. |
| `anti_xray` | `true` | Enable Paper's anti-xray obfuscation. |
| `anti_xray_engine` | `1` | Engine mode: `1` = hide-ore, `2` = obfuscate. |

#### `fabric`

| Field | Default | Description |
| --- | --- | --- |
| `loader_version` | `"latest"` | Fabric Loader version. |

#### `forge`

| Field | Default | Description |
| --- | --- | --- |
| `loader_version` | `"latest"` | Forge version. |

#### `purpur`

| Field | Default | Description |
| --- | --- | --- |
| `rideable_phantoms` | `false` | Allow players to ride phantoms. |
| `mobs_ignore_rails` | `false` | Mobs path-find across rails without avoiding them. |

---

## Preset files (`assets/presets/*.json`)

Named files that override defaults in `install.json` and `configure.json`. Select a preset at startup to apply it. A preset can fully reproduce a server setup by capturing every default.

```json
{
  "name": "Paper SMP",
  "server": {
    "server_type": { "ask": false, "default": "paper" },
    "minecraft_version": { "ask": false, "default": "latest" }
  },
  "performance": {
    "max_ram": { "ask": false, "default": "4G" }
  }
}
```

---

## oxide.json (per-server state)

Generated in each server's directory after setup. Records every answered value. Load it as a preset to duplicate that server.

```json
{
  "server": {
    "motd": "My SMP",
    "max_players": 10,
    "gamemode": "survival",
    "difficulty": "hard",
    "pvp": true,
    "whitelist": true,
    "online_mode": true,
    "level_name": "world",
    "seed": "",
    "spawn_protection": 0
  },
  "network": {
    "port": 25565,
    "rcon": {
      "enabled": true,
      "port": 25575,
      "password": "secret"
    }
  },
  "performance": {
    "max_ram": "4G",
    "min_ram": "2G",
    "jvm_flags": "",
    "render_distance": 12,
    "simulation_distance": 8
  },
  "players": {
    "ops": ["Notch"],
    "banned_players": [],
    "whitelist_players": ["Notch", "jeb_"]
  },
  "mods": {
    "mods": []
  }
}
```
