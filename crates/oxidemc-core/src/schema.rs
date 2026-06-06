use serde::{Deserialize, Serialize};

fn default_backup_enabled() -> Question<bool> {
    Question { ask: false, default: true, options: None, note: None, requires_rcon: false, condition: None }
}

/// A configurable value. `ask=true` means prompt the user at setup time.
/// `note` is an optional warning or hint shown alongside the prompt.
/// `requires_rcon=true` means this setting is managed at runtime via RCON;
/// if RCON is disabled the feature will be unavailable.
/// `condition` is an optional named predicate the UI evaluates before showing
/// this prompt; if the condition is false the question is skipped entirely.
#[derive(Debug, Deserialize, Serialize)]
pub struct Question<T> {
    pub ask: bool,
    pub default: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub requires_rcon: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

// ── Install ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct InstallConfig {
    pub server_name: Question<String>,
    pub server_type: Question<String>,
    pub minecraft_version: Question<String>,
    /// Skip the EULA prompt and accept automatically
    pub eula_auto_accept: Question<bool>,
    /// Start the server immediately after install completes
    pub auto_start: Question<bool>,
    /// Generate a start.sh / start.bat in the server directory
    pub create_start_script: Question<bool>,
    /// Path to a server-icon.png (64×64) to copy into the server directory
    pub icon: Question<String>,
    /// Name of a preset to apply; skips the wizard when set
    pub load_preset: Question<String>,
    /// Back up the existing directory before overwriting.
    /// Only shown when the target directory already exists and is non-empty.
    pub backup_before_reinstall: Question<bool>,
}

// ── Mods ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ModSource {
    Modrinth,
    CurseForge,
    Direct,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub source: ModSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

// ── Platform-specific ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct PaperConfig {
    pub tnt_duping: Question<bool>,
    pub bedrock_breaking: Question<bool>,
    pub anti_xray: Question<bool>,
    pub anti_xray_engine: Question<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FabricConfig {
    pub loader_version: Question<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForgeConfig {
    pub loader_version: Question<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PurpurConfig {
    pub rideable_phantoms: Question<bool>,
    pub mobs_ignore_rails: Question<bool>,
}

/// Keyed by server type. Only the matching block is shown during setup.
#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformSpecificConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper: Option<PaperConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fabric: Option<FabricConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forge: Option<ForgeConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpur: Option<PurpurConfig>,
}

// ── Configure sections ────────────────────────────────────────────────────────
//
// Each struct here becomes its own submenu in the TUI/webui.
// The UI recurses into struct-valued fields and treats Question<T> as leaf prompts.

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerSection {
    pub motd: Question<String>,
    pub max_players: Question<u32>,
    /// survival | creative | adventure | spectator
    pub gamemode: Question<String>,
    /// peaceful | easy | normal | hard
    pub difficulty: Question<String>,
    pub pvp: Question<bool>,
    pub whitelist: Question<bool>,
    pub online_mode: Question<bool>,
    /// World folder name; changing this starts a fresh world
    pub level_name: Question<String>,
    /// Empty string = random seed
    pub seed: Question<String>,
    /// Block radius around spawn that only ops can build in; 0 = disabled
    pub spawn_protection: Question<u16>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RconSection {
    pub enabled: Question<bool>,
    pub port: Question<u16>,
    pub password: Question<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkSection {
    pub port: Question<u16>,
    pub rcon: RconSection,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PerformanceSection {
    pub max_ram: Question<String>,
    pub min_ram: Question<String>,
    pub jvm_flags: Question<String>,
    pub render_distance: Question<u8>,
    pub simulation_distance: Question<u8>,
}

/// Player names or UUIDs. ask=true prompts the user to add entries interactively.
#[derive(Debug, Deserialize, Serialize)]
pub struct PlayersSection {
    pub ops: Question<Vec<String>>,
    pub banned_players: Question<Vec<String>>,
    pub whitelist_players: Question<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModsSection {
    /// ask=true prompts the user to add mods interactively on top of the defaults
    pub mods: Question<Vec<ModEntry>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigureConfig {
    pub server: ServerSection,
    pub network: NetworkSection,
    pub performance: PerformanceSection,
    pub players: PlayersSection,
    pub mods: ModsSection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_specific: Option<PlatformSpecificConfig>,
}

// ── Resolved state (oxide.json) ───────────────────────────────────────────────
//
// Plain values — no Question<T> wrapper. Written per-server after the wizard runs.

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolvedServerSection {
    pub motd: String,
    pub max_players: u32,
    pub gamemode: String,
    pub difficulty: String,
    pub pvp: bool,
    pub whitelist: bool,
    pub online_mode: bool,
    pub level_name: String,
    pub seed: String,
    pub spawn_protection: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolvedRconSection {
    pub enabled: bool,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolvedNetworkSection {
    pub port: u16,
    pub rcon: ResolvedRconSection,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolvedPerformanceSection {
    pub max_ram: String,
    pub min_ram: String,
    pub jvm_flags: String,
    pub render_distance: u8,
    pub simulation_distance: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolvedPlayersSection {
    pub ops: Vec<String>,
    pub banned_players: Vec<String>,
    pub whitelist_players: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolvedModsSection {
    pub mods: Vec<ModEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerState {
    pub server_name: String,
    pub server_type: String,
    pub minecraft_version: String,
    pub directory: String,
    pub server: ResolvedServerSection,
    pub network: ResolvedNetworkSection,
    pub performance: ResolvedPerformanceSection,
    pub players: ResolvedPlayersSection,
    pub mods: ResolvedModsSection,
    /// Whether RCON TPS polling has been confirmed to work for this server.
    /// None = unknown (probe on next start), Some(false) = not supported (skip probing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tps_capable: Option<bool>,
}

// ── Manage ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct ManageConfig {
    pub data_directory: Question<String>,
    pub presets_directory: Question<String>,
    pub servers_directory: Question<String>,
    /// Path to a Java executable; empty = use java found on $PATH
    pub java_path: Question<String>,
    /// Warn if the Java version is incompatible with the MC version being installed
    pub check_java_version: Question<bool>,
    /// Check for OxideMC updates when the app launches
    pub auto_update_check: Question<bool>,
    /// Whether automatic backups are enabled
    #[serde(default = "default_backup_enabled")]
    pub backup_enabled: Question<bool>,
    /// Directory where server backups are stored
    pub backup_directory: Question<String>,
    /// Number of rolling backups to keep per server; 0 = unlimited
    pub backup_count: Question<u32>,
    /// OxideMC log verbosity
    pub log_level: Question<String>,
    /// Preset to load automatically; skips the preset picker when set
    pub default_preset: Question<String>,
    /// TUI color theme name
    pub theme: Question<String>,
}
