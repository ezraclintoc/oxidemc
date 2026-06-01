use crate::app::{App, Screen};
use crate::screens::settings;
use oxidemc_core::downloader::get_platforms;
use ratatui::Frame;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

// Install wizard steps:
// 0 - Server name input (free text)
// 1 - Platform picker   (List from get_platforms)
// 2 - Version picker    (List from state.versions, fetched async in run loop)
// 3 - Settings          (shared settings panel, all fields optional)
// 4 - Summary           (review all selections, Enter to confirm and download)
// 5 - Download          (Gauge progress bar; directory derived as servers_dir/server_name)
// 6 - Done              (success message, Enter returns to main menu)

pub fn draw(frame: &mut Frame, app: &App) {
    let t = &app.theme;
    let istate = match &app.screen {
        Screen::Install(state) => state,
        _ => return,
    };

    match istate.step {
        // step 0: server name input
        0 => {
            let text = Paragraph::new(Line::from(vec![
                Span::raw("> "),
                Span::styled(format!("{}_", istate.server_name), t.value),
            ]))
            .block(
                Block::default()
                    .title(Span::styled("Choose a Server Name", t.border_title))
                    .borders(Borders::ALL)
                    .border_style(t.border),
            );
            frame.render_widget(text, frame.area());
        }
        // step 1: platform picker
        1 => {
            let mut lstate = ListState::default();
            lstate.select(Some(istate.cursor));

            let items: Vec<ListItem> = get_platforms().iter().map(|p| ListItem::new(*p)).collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(Span::styled("Choose a Platform", t.border_title))
                        .borders(Borders::ALL)
                        .border_style(t.border),
                )
                .highlight_style(t.highlight)
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, frame.area(), &mut lstate);
        }
        // step 2: version picker
        2 => {
            let mut lstate = ListState::default();
            lstate.select(Some(istate.cursor));

            let items: Vec<ListItem> = if istate.versions.is_empty() {
                vec![ListItem::new(Span::styled("Loading...", t.hint))]
            } else {
                istate.versions.iter().map(|v| ListItem::new(v.as_str())).collect()
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(Span::styled("Choose a Version", t.border_title))
                        .borders(Borders::ALL)
                        .border_style(t.border),
                )
                .highlight_style(t.highlight)
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, frame.area(), &mut lstate);
        }
        // step 3: settings panel — selected is synced from istate.cursor in app.rs
        3 => {
            settings::draw(frame, frame.area(), &istate.settings, t);
        }
        // step 4: summary — shows all choices before downloading
        4 => {
            let s = &istate.settings;
            let row = |label: &'static str, val: String| -> Line<'static> {
                Line::from(vec![
                    Span::raw(format!("{:<22}", label)),
                    Span::styled(val, t.value),
                ])
            };
            let lines = vec![
                Line::from(Span::styled("Review your server", t.normal.add_modifier(Modifier::BOLD))),
                Line::from(""),
                row("Server Name",  istate.server_name.clone()),
                row("Platform",     istate.server_type.clone()),
                row("Version",      istate.version.clone()),
                Line::from(""),
                row("MOTD",         s.motd.clone()),
                row("Max Players",  s.max_players.to_string()),
                row("Gamemode",     s.gamemode.clone()),
                row("Difficulty",   s.difficulty.clone()),
                row("PVP",          s.pvp.to_string()),
                row("Port",         s.port.to_string()),
                row("Max RAM",      s.max_ram.clone()),
                row("Min RAM",      s.min_ram.clone()),
                row("RCON",         if s.rcon_enabled { "enabled".into() } else { "disabled".into() }),
                Line::from(""),
                Line::from(Span::styled(
                    "Press Enter to install,  Esc to go back",
                    t.success,
                )),
            ];

            let para = Paragraph::new(lines).block(
                Block::default()
                    .title(Span::styled("Summary", t.border_title))
                    .borders(Borders::ALL)
                    .border_style(t.border),
            );
            frame.render_widget(para, frame.area());
        }
        _ => {
            let text = Paragraph::new(Span::styled("Not implemented yet. Press Esc.", t.hint));
            frame.render_widget(text, frame.area());
        }
    }
}
