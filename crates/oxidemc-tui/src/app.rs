use crossterm::event::{Event, KeyCode, KeyEvent};
use oxidemc_core::downloader::{get_platforms, DownloadProgress};
use tokio::sync::mpsc::Receiver;
use crate::theme::Theme;
use crate::state::{FieldKind, InstallState, ManageState, SettingsState, field_kind};

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
    pub manage: ManageState,
    pub download_rx: Option<Receiver<DownloadProgress>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            screen: Screen::MainMenu { selected: 0 },
            should_quit: false,
            theme: Theme::default(),
            manage: ManageState::default(),
            download_rx: None,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        let Event::Key(KeyEvent { code, .. }) = event else { return };

        match &mut self.screen {
            Screen::MainMenu { selected } => match code {
                KeyCode::Up   => *selected = selected.saturating_sub(1),
                KeyCode::Down => *selected = (*selected + 1).min(2),
                _ => {}
            },
            Screen::Install(state) => {
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
                            KeyCode::Enter     => { state.settings.commit_edit(); state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Esc       => { state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Char(c) if c.is_ascii_digit() => state.settings.edit_buf.push(c),
                            KeyCode::Backspace => { state.settings.edit_buf.pop(); }
                            _ => {}
                        },
                        FieldKind::Ram => match code {
                            KeyCode::Enter     => { state.settings.commit_edit(); state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Esc       => { state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Char(c) if c.is_ascii_digit() || matches!(c, 'G' | 'g' | 'M' | 'm') => state.settings.edit_buf.push(c),
                            KeyCode::Backspace => { state.settings.edit_buf.pop(); }
                            _ => {}
                        },
                        _ => match code {
                            KeyCode::Enter     => { state.settings.commit_edit(); state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Esc       => { state.settings.editing = false; state.settings.edit_buf.clear(); }
                            KeyCode::Char(c)   => state.settings.edit_buf.push(c),
                            KeyCode::Backspace => { state.settings.edit_buf.pop(); }
                            _ => {}
                        },
                    }
                    return;
                }

                let max = match state.step {
                    1 => get_platforms().len().saturating_sub(1),
                    2 => state.versions.len().saturating_sub(1),
                    3 => 13,
                    _ => 0,
                };
                match code {
                    KeyCode::Up        => state.cursor = state.cursor.saturating_sub(1),
                    KeyCode::Down      => state.cursor = (state.cursor + 1).min(max),
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
                        6 => Some(Screen::MainMenu { selected: 0 }),
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
