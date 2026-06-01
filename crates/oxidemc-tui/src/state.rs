use std::path::PathBuf;

// ── Field types for the settings panel ───────────────────────────────────────

#[derive(Clone, Copy)]
pub enum FieldKind {
    Text,
    Number { min: u32, max: u32 },
    Ram,
    Choice(&'static [&'static str]),
    Toggle,
    Confirm,
}

pub fn field_kind(row: usize) -> FieldKind {
    match row {
        0  => FieldKind::Text,
        1  => FieldKind::Number { min: 1, max: 10000 },
        2  => { const O: &[&str; 4] = &["survival", "creative", "adventure", "spectator"]; FieldKind::Choice(O) }
        3  => { const O: &[&str; 4] = &["peaceful", "easy", "normal", "hard"]; FieldKind::Choice(O) }
        4  => FieldKind::Toggle,
        5  => FieldKind::Number { min: 1, max: 65535 },
        6  => FieldKind::Ram,
        7  => FieldKind::Ram,
        8  => FieldKind::Number { min: 2, max: 32 },
        9  => FieldKind::Number { min: 2, max: 32 },
        10 => FieldKind::Toggle,
        11 => FieldKind::Number { min: 1, max: 65535 },
        12 => FieldKind::Text,
        _  => FieldKind::Confirm,
    }
}

fn is_valid_ram(s: &str) -> bool {
    if s.len() < 2 { return false; }
    let (num, suffix) = s.split_at(s.len() - 1);
    matches!(suffix, "G" | "g" | "M" | "m") && !num.is_empty() && num.chars().all(|c| c.is_ascii_digit())
}

// ── Settings panel state ──────────────────────────────────────────────────────

#[derive(Default, Clone)]
pub struct SettingsState {
    pub selected: usize,
    pub editing: bool,
    pub edit_buf: String,
    pub edit_option_idx: usize,
    // server
    pub motd: String,
    pub max_players: u32,
    pub gamemode: String,
    pub difficulty: String,
    pub pvp: bool,
    pub port: u16,
    // performance
    pub max_ram: String,
    pub min_ram: String,
    pub render_distance: u8,
    pub simulation_distance: u8,
    // rcon
    pub rcon_enabled: bool,
    pub rcon_port: u16,
    pub rcon_password: String,
}

impl SettingsState {
    pub fn with_defaults() -> Self {
        Self {
            motd: "A Minecraft Server".into(),
            max_players: 20,
            gamemode: "survival".into(),
            difficulty: "normal".into(),
            pvp: true,
            port: 25565,
            max_ram: "2G".into(),
            min_ram: "1G".into(),
            render_distance: 10,
            simulation_distance: 10,
            rcon_enabled: true,
            rcon_port: 25575,
            rcon_password: String::new(),
            ..Self::default()
        }
    }

    fn current_value_str(&self) -> String {
        match self.selected {
            0  => self.motd.clone(),
            1  => self.max_players.to_string(),
            2  => self.gamemode.clone(),
            3  => self.difficulty.clone(),
            5  => self.port.to_string(),
            6  => self.max_ram.clone(),
            7  => self.min_ram.clone(),
            8  => self.render_distance.to_string(),
            9  => self.simulation_distance.to_string(),
            11 => self.rcon_port.to_string(),
            12 => self.rcon_password.clone(),
            _  => String::new(),
        }
    }

    pub fn open_edit(&mut self) {
        match field_kind(self.selected) {
            FieldKind::Toggle => match self.selected {
                4  => self.pvp = !self.pvp,
                10 => self.rcon_enabled = !self.rcon_enabled,
                _  => {}
            },
            FieldKind::Confirm => {}
            FieldKind::Choice(options) => {
                let current = self.current_value_str();
                self.edit_option_idx = options.iter().position(|&o| o == current).unwrap_or(0);
                self.editing = true;
            }
            _ => {
                self.edit_buf = self.current_value_str();
                self.editing = true;
            }
        }
    }

    pub fn commit_edit(&mut self) {
        match field_kind(self.selected) {
            FieldKind::Choice(options) => {
                if let Some(&opt) = options.get(self.edit_option_idx) {
                    match self.selected {
                        2 => self.gamemode = opt.to_string(),
                        3 => self.difficulty = opt.to_string(),
                        _ => {}
                    }
                }
            }
            FieldKind::Number { min, max } => match self.selected {
                1  => { if let Ok(v) = self.edit_buf.parse::<u32>() { self.max_players = v.clamp(min, max) } }
                5  => { if let Ok(v) = self.edit_buf.parse::<u16>() { self.port = v.clamp(min as u16, max as u16) } }
                8  => { if let Ok(v) = self.edit_buf.parse::<u8>()  { self.render_distance = v.clamp(min as u8, max as u8) } }
                9  => { if let Ok(v) = self.edit_buf.parse::<u8>()  { self.simulation_distance = v.clamp(min as u8, max as u8) } }
                11 => { if let Ok(v) = self.edit_buf.parse::<u16>() { self.rcon_port = v.clamp(min as u16, max as u16) } }
                _  => {}
            },
            FieldKind::Ram if is_valid_ram(&self.edit_buf) => match self.selected {
                6 => self.max_ram = self.edit_buf.clone(),
                7 => self.min_ram = self.edit_buf.clone(),
                _ => {}
            },
            FieldKind::Text => match self.selected {
                0  => self.motd = self.edit_buf.clone(),
                12 => self.rcon_password = self.edit_buf.clone(),
                _  => {}
            },
            _ => {}
        }
    }
}

// ── Install wizard state ──────────────────────────────────────────────────────

#[derive(Default, Clone)]
pub struct InstallState {
    pub step: usize,
    pub cursor: usize,
    pub server_name: String,
    pub server_type: String,
    pub versions: Vec<String>,
    pub version: String,
    pub settings: SettingsState,
    pub accept_eula: bool,
    pub autostart: bool,
    // download step
    pub download_started: bool,
    pub download_progress: f64,
    pub download_error: Option<String>,
}

// ── Global OxideMC settings (resolved from manage.json) ──────────────────────
// Mirrors ManageConfig from oxidemc-core but as plain values, no Question<T>.

#[derive(Clone)]
pub struct ManageState {
    pub servers_directory: PathBuf,
    pub java_path: String,
    pub theme_name: String,
    pub backup_directory: PathBuf,
    pub backup_count: u32,
    pub auto_update_check: bool,
    pub check_java_version: bool,
}

impl Default for ManageState {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            servers_directory:  home.join("servers"),
            java_path:          String::new(),
            theme_name:         "rust".into(),
            backup_directory:   home.join(".config/oxidemc/backups"),
            backup_count:       5,
            auto_update_check:  true,
            check_java_version: true,
        }
    }
}
