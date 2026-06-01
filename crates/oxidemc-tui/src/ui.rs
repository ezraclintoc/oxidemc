use ratatui::Frame;
use crate::app::{App, Screen};
use crate::screens;

pub fn draw(frame: &mut Frame, app: &App) {
    match &app.screen {
        Screen::MainMenu { .. } => screens::menu::draw(frame, app),
        Screen::Install(_)      => screens::install::draw(frame, app),
        Screen::Configure      => {}  // stub
        Screen::Manage         => {}  // stub
    }
}
 