# Configuration Reference

OxideMC uses JSON files for two purposes:

- **Schema files** (`assets/`) — define which questions are shown and their defaults. Shipped with OxideMC.
- **State files** (`oxide.json`) — saved state for a specific server. Generated per-server.

---

## manage.json

Controls the question flow for OxideMC itself and determines how `install.json` and `configure.json` behave.

| Field | Type | Description |
| ----- | ---- | ----------- |
| `ask_server_type` | `bool` | Whether to prompt for server type. Default `true`. |
| `default_server_type` | `string` | Default if `ask_server_type` is `false`. One of `vanilla`, `paper`, `fabric`, `forge`, `neoforge`, `purpur`. |
| `ask_version` | `bool` | Whether to prompt for MC version. Default `true`. |
| `default_version` | `string \| "latest"` | Default version. `"latest"` always picks newest stable. |
| `ask_directory` | `bool` | Whether to prompt for install directory. Default `true`. |
| `default_directory` | `string` | Default path if `ask_directory` is `false`. |

---

## install.json

Question schema for server installation. Defines what the install wizard asks.

| Field | Type | Description |
| ----- | ---- | ----------- |
| `ask_server_name` | `bool` | Whether to prompt for a server name. Default `true`. |
| `ask_port` | `bool` | Whether to prompt for port. Default `true`. |
| `default_port` | `number` | Default port if `ask_port` is `false`. Default `25565`. |

---

## configure.json

Question schema for server configuration. Defines what the configure wizard asks.

| Field | Type | Description |
| ----- | ---- | ----------- |
| `ask_ram` | `bool` | Whether to prompt for RAM allocation. Default `true`. |
| `default_max_ram` | `string` | Default max heap size (e.g. `"4G"`). |
| `default_min_ram` | `string` | Default min heap size (e.g. `"1G"`). |
| `ask_jvm_flags` | `bool` | Whether to prompt for extra JVM flags. Default `false`. |
| `default_jvm_flags` | `string[]` | Default JVM flags. |

---

## Preset files (`assets/presets/*.json`)

Named files that override defaults in `install.json` and `configure.json`. Select a preset at startup to apply it. Because they capture question defaults, a preset can fully reproduce a server setup.

```json
{
  "name": "Paper Latest",
  "default_server_type": "paper",
  "ask_version": false,
  "default_version": "latest",
  "default_port": 25565,
  "default_max_ram": "4G"
}
```

---

## oxide.json (per-server)

Saved state file generated in each server's directory after setup. Records the answers to all install and configure questions. Can be loaded as a preset to duplicate that server.

```json
{
  "server_name": "my-server",
  "server_type": "paper",
  "minecraft_version": "1.21.4",
  "directory": "/home/user/servers/my-server",
  "port": 25565,
  "max_ram": "4G",
  "min_ram": "1G",
  "jvm_flags": []
}
```

> **Note:** Field names are provisional and will be updated as the schema stabilizes.
