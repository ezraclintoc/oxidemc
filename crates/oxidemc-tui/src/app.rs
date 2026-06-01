use crossterm::event::{Event, KeyCode, KeyEvent};
use oxidemc_core::downloader::get_platforms;
use crate::theme::Theme;

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
            FieldKind::Ram => {
                if is_valid_ram(&self.edit_buf) {
                    match self.selected {
                        6 => self.max_ram = self.edit_buf.clone(),
                        7 => self.min_ram = self.edit_buf.clone(),
                        _ => {}
                    }
                }
            }
            FieldKind::Text => match self.selected {
                0  => self.motd = self.edit_buf.clone(),
                12 => self.rcon_password = self.edit_buf.clone(),
                _  => {}
            },
            _ => {}
        }
    }
}

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
}

pub enum Screen {
    MainMenu { selected: usize },
    Install(InstallState),
    Configure,
    Manage,
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub theme: Theme,
}

impl App {
    pub fn new() -> Self {
        App {
            screen: Screen::MainMenu { selected: 0 },
            should_quit: false,
            theme: Theme::default(),
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        let Event::Key(KeyEvent { code, .. }) = event else { return };

        // in-place mutations within current screen
        match &mut self.screen {
            Screen::MainMenu { selected } => match code {
                KeyCode::Up   => *selected = selected.saturating_sub(1),
                KeyCode::Down => *selected = (*selected + 1).min(2),
                _ => {}
            },
            Screen::Install(state) => {
                // settings popup open: route all keys to the edit handler
                if state.step == 3 && state.settings.editing {
                    match field_kind(state.settings.selected) {
                        FieldKind::Choice(options) => match code {
                            KeyCode::Up    => state.settings.edit_option_idx = state.settings.edit_option_idx.saturating_sub(1),
                            KeyCode::Down  => state.settings.edit_option_idx = (state.settings.edit_option_idx + 1).min(options.len().saturating_sub(1)),
                            KeyCode::Enter => { state.settings.commit_edit(); state.settings.editing = false; }
                            KeyCode::Esc   => { state.settings.editing = false; }
                            _ => {}
                        },
                        FieldKind::Number { .. } => match code {
                            KeyCode::Enter => { state.settings.commit_edit(); state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Esc   => { state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Char(c) if c.is_ascii_digit() => state.settings.edit_buf.push(c),
                            KeyCode::Backspace => { state.settings.edit_buf.pop(); }
                            _ => {}
                        },
                        FieldKind::Ram => match code {
                            KeyCode::Enter => { state.settings.commit_edit(); state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Esc   => { state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Char(c) if c.is_ascii_digit() || matches!(c, 'G' | 'g' | 'M' | 'm') => state.settings.edit_buf.push(c),
                            KeyCode::Backspace => { state.settings.edit_buf.pop(); }
                            _ => {}
                        },
                        _ => match code { // Text
                            KeyCode::Enter => { state.settings.commit_edit(); state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Esc   => { state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Char(c) => state.settings.edit_buf.push(c),
                            KeyCode::Backspace => { state.settings.edit_buf.pop(); }
                            _ => {}
                        },
                    }
                    return; // skip phase 2 while popup is open
                }

                let max = match state.step {
                    1 => get_platforms().len().saturating_sub(1),
                    2 => state.versions.len().saturating_sub(1),
                    3 => 13,
                    _ => 0,
                };
                match code {
                    KeyCode::Up   => state.cursor = state.cursor.saturating_sub(1),
                    KeyCode::Down => state.cursor = (state.cursor + 1).min(max),
                    KeyCode::Char(c) if state.step == 0 => { state.server_name.push(c); }
                    KeyCode::Backspace if state.step == 0 => { state.server_name.pop(); }
                    KeyCode::Enter if state.step == 3 && state.cursor != 13 => {
                        state.settings.open_edit();
                    }
                    _ => {}
                }
                if state.step == 3 {
                    state.settings.selected = state.cursor;
                }
            }
            _ => {}
        }

        // global keys and screen transitions (mutable borrow above is dropped here)
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.screen = Screen::MainMenu { selected: 0 },
            KeyCode::Enter => {
                let next = match &self.screen {
                    Screen::MainMenu { selected: 0 } => Some(Screen::Install(InstallState::default())),
                    Screen::MainMenu { selected: 1 } => Some(Screen::Configure),
                    Screen::MainMenu { selected: 2 } => Some(Screen::Manage),
                    Screen::Install(state) => match state.step {
                        0 => Some(Screen::Install(InstallState {
                            step: 1,
                            server_name: state.server_name.clone(),
                            ..InstallState::default()
                        })),
                        1 => {
                            let platform = get_platforms().get(state.cursor).copied().unwrap_or("paper").to_string();
                            Some(Screen::Install(InstallState {
                                step: 2,
                                server_name: state.server_name.clone(),
                                server_type: platform,
                                ..InstallState::default()
                            }))
                        }
                        2 => {
                            let version = state.versions.get(state.cursor).cloned().unwrap_or_default();
                            Some(Screen::Install(InstallState {
                                step: 3,
                                server_name: state.server_name.clone(),
                                server_type: state.server_type.clone(),
                                version,
                                settings: SettingsState::with_defaults(),
                                ..InstallState::default()
                            }))
                        }
                        3 if state.cursor == 13 => Some(Screen::Install(InstallState {
                            step: 4,
                            server_name: state.server_name.clone(),
                            server_type: state.server_type.clone(),
                            version: state.version.clone(),
                            settings: state.settings.clone(),
                            ..InstallState::default()
                        })),
                        4 => Some(Screen::Install(InstallState {
                            step: 5,
                            server_name: state.server_name.clone(),
                            server_type: state.server_type.clone(),
                            version: state.version.clone(),
                            settings: state.settings.clone(),
                            ..InstallState::default()
                        })),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(s) = next {
                    self.screen = s;
                }
            }
            _ => {}
        }
    }
}
