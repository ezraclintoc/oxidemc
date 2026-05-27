# Configuration Reference

OxideMC uses three JSON config files. `manage.json` is global; `install.json` and `configure.json` are per-server.

---

## manage.json

Controls the question flow and default values for all server setups. Lives at `~/.config/oxidemc/manage.json`.

| Field | Type | Description |
|-------|------|-------------|
| `default_server_type` | `string` | Default server type (`"paper"`, `"vanilla"`, `"fabric"`, `"forge"`, `"neoforge"`, `"purpur"`). If set, skips the type question. |
| `default_version` | `string \| "latest"` | Default MC version. `"latest"` always picks the newest stable release. |
| `ask_version` | `bool` | Whether to prompt for version selection. Default `true`. |
| `ask_directory` | `bool` | Whether to prompt for install directory. Default `true`. |
| `default_directory` | `string` | Default install path if `ask_directory` is `false`. |
| `ask_port` | `bool` | Whether to prompt for server port. Default `true`. |
| `default_port` | `number` | Default port if `ask_port` is `false`. Default `25565`. |

---

## install.json

Generated in the server directory after installation. Records how the server was set up.

| Field | Type | Description |
|-------|------|-------------|
| `server_type` | `string` | Server software used. |
| `minecraft_version` | `string` | Minecraft version installed. |
| `directory` | `string` | Absolute path to the server directory. |
| `jar_name` | `string` | Filename of the downloaded JAR. |

---

## configure.json

Generated in the server directory. Stores runtime configuration.

| Field | Type | Description |
|-------|------|-------------|
| `port` | `number` | Server port. Default `25565`. |
| `max_ram` | `string` | Max JVM heap size (e.g. `"4G"`). |
| `min_ram` | `string` | Min JVM heap size (e.g. `"1G"`). |
| `jvm_flags` | `string[]` | Additional JVM flags. |

---

## Preset files

Presets live in `~/.config/oxidemc/presets/` as individual `.json` files. Each preset can override any field from `manage.json`. Select a preset at startup to apply its values as defaults for that session.

```json
{
  "name": "Paper Latest",
  "default_server_type": "paper",
  "default_version": "latest",
  "ask_version": false,
  "default_port": 25565,
  "max_ram": "4G"
}
```

> **Note:** Field names above are provisional. This document will be updated as the schema stabilizes.
