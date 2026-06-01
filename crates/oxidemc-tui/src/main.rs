use std::io::{self, Stdout};
use app::{Screen, InstallState};
use oxidemc_core::downloader::get_versions;

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

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<()> {
    let mut app = app::App::new();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // fetch versions when entering step 1 and list is empty
        if let Screen::Install(InstallState { step: 2, versions, server_type, .. }) = &mut app.screen {
            if versions.is_empty() {
                *versions = get_versions(server_type).await.unwrap_or_default();
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
