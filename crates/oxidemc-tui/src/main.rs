use std::io::{self, Stdout};
use std::path::PathBuf;
use app::Screen;
use state::{InstallState, ManageState};
use oxidemc_core::downloader::{download_jar, get_versions, jar_name, DownloadProgress};
use oxidemc_core::config::load_manage;
use tokio::sync::mpsc;

use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::time::Duration;
use crossterm::event;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub mod ui;
pub mod app;
pub mod screens;
pub mod theme;
pub mod state;

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<()> {
    let mut app = app::App::new();

    // populate ManageState from manage.json, fields fall back to defaults individually
    if let Ok(c) = load_manage() {
        let d = ManageState::default();
        app.manage = ManageState {
            servers_directory: non_empty_path(c.servers_directory.default)
                .unwrap_or(d.servers_directory),
            java_path:          c.java_path.default,
            theme_name:         non_empty(c.theme.default).unwrap_or(d.theme_name),
            backup_directory:   non_empty_path(c.backup_directory.default)
                .unwrap_or(d.backup_directory),
            backup_count:       c.backup_count.default,
            auto_update_check:  c.auto_update_check.default,
            check_java_version: c.check_java_version.default,
        };
    }

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // fetch versions when entering step 2 and list is empty
        if let Screen::Install(InstallState { step: 2, versions, server_type, .. }) = &mut app.screen
            && versions.is_empty()
        {
            *versions = get_versions(server_type).await.unwrap_or_default();
        }

        // spawn download task on first entry to step 5
        if let Screen::Install(state) = &mut app.screen
            && state.step == 5 && !state.download_started
        {
            state.download_started = true;
            let (tx, rx) = mpsc::channel(64);
            app.download_rx = Some(rx);
            let dest = app.manage.servers_directory
                .join(&state.server_name)
                .join(jar_name(&state.server_type, &state.version));
            let (platform, version) = (state.server_type.clone(), state.version.clone());
            tokio::spawn(async move {
                let _ = download_jar(&platform, &version, &dest, tx).await;
            });
        }

        // drain download progress updates
        let updates: Vec<DownloadProgress> = app.download_rx
            .as_mut()
            .map(|rx| std::iter::from_fn(|| rx.try_recv().ok()).collect())
            .unwrap_or_default();

        for update in updates {
            if let Screen::Install(state) = &mut app.screen {
                match update {
                    DownloadProgress::Chunk { downloaded, total } if total > 0 => {
                        state.download_progress = downloaded as f64 / total as f64;
                    }
                    DownloadProgress::Finished => {
                        state.download_progress = 1.0;
                        state.step = 6;
                        app.download_rx = None;
                    }
                    DownloadProgress::Started { .. } => {}
                    _ => {}
                }
            }
        }

        if event::poll(Duration::from_millis(16))? {
            let event = event::read()?;
            app.handle_event(event);
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}


fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn non_empty_path(s: String) -> Option<PathBuf> {
    if s.is_empty() { None } else { Some(PathBuf::from(s)) }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result?;
    Ok(())
}
