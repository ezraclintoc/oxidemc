use crate::app::{App, Screen};
use crate::screens::settings;
use oxidemc_core::downloader::get_platforms;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::layout::{Alignment, Layout};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, RatatuiLogo};
use tui_big_text::{BigText, PixelSize};

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
            frame.render_widget(text, frame.area().centered(Constraint::Percentage(50), Constraint::Length(3)));
        }
        // step 1: platform picker
        1 => {
            let mut lstate = ListState::default();
            lstate.select(Some(istate.cursor));

            let items: Vec<ListItem> = get_platforms().iter().map(|p| ListItem::new(*p)).collect();

            let list = List::new(items.clone())
                .block(
                    Block::default()
                        .title(Span::styled("Choose a Platform", t.border_title))
                        .borders(Borders::ALL)
                        .border_style(t.border),
                )
                .highlight_style(t.highlight)
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, frame.area().centered(Constraint::Length(20), Constraint::Length(items.len() as u16 + 2)), &mut lstate);
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

            let list = List::new(items.clone())
                .block(
                    Block::default()
                        .title(Span::styled("Choose a Version", t.border_title))
                        .borders(Borders::ALL)
                        .border_style(t.border),
                )
                .highlight_style(t.highlight)
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, frame.area().centered(Constraint::Length(20), Constraint::Length(items.len() as u16 + 2)), &mut lstate);
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

            let para = Paragraph::new(lines.clone()).block(
                Block::default()
                    .title(Span::styled("Summary", t.border_title))
                    .borders(Borders::ALL)
                    .border_style(t.border),
            );
            frame.render_widget(para, frame.area().centered(Constraint::Length(50), Constraint::Length(lines.len() as u16 + 2)));
        }
        5 => {
            let outer = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Fill(1),
            ])
            .split(frame.area());

            let inner = Layout::vertical([
                Constraint::Length(1), // filename label
                Constraint::Length(1), // gap
                Constraint::Length(3), // gauge
                Constraint::Length(1), // error (if any)
            ])
            .split(outer[1].centered(Constraint::Percentage(60), Constraint::Length(6)));

            let jar = format!("{}-{}.jar", istate.server_type, istate.version);
            let label = Paragraph::new(Span::styled(
                format!("Downloading {jar}"),
                t.border_title,
            ))
            .alignment(Alignment::Center);
            frame.render_widget(label, inner[0]);

            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).border_style(t.border))
                .gauge_style(t.gauge)
                .ratio(istate.download_progress)
                .label(format!("{:.0}%", istate.download_progress * 100.0));
            frame.render_widget(gauge, inner[2]);

            if let Some(err) = &istate.download_error {
                let msg = Paragraph::new(Span::styled(err.clone(), t.error))
                    .alignment(Alignment::Center);
                frame.render_widget(msg, inner[3]);
            }
        }
        6 => {
            // split full screen: content area + 3-row bottom bar
            let [main, bottom] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(3), // "Powered by" + logo
            ])
            .areas(frame.area());

            // centered content: 4 + 1 + 1 + 1 + 1 = 8 rows
            let vert = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Fill(1),
            ])
            .split(main);

            let col = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(58),
                Constraint::Fill(1),
            ])
            .split(vert[1])[1];

            let sections = Layout::vertical([
                Constraint::Length(4), // OxideMC big text
                Constraint::Length(1), // gap
                Constraint::Length(1), // success message
                Constraint::Length(1), // gap
                Constraint::Length(1), // footer hints
            ])
            .split(col);

            // OxideMC title
            let title = BigText::builder()
                .pixel_size(PixelSize::HalfHeight)
                .lines(vec![Line::from("OxideMC")])
                .style(t.title)
                .build();
            frame.render_widget(title, sections[0]);

            // success message
            let msg = Paragraph::new(Line::from(vec![
                Span::styled(format!("\"{}\"", istate.server_name), t.value),
                Span::styled(" installed successfully!", t.success),
            ]))
            .alignment(Alignment::Center);
            frame.render_widget(msg, sections[2]);

            // footer hints
            let footer = Paragraph::new(Line::from(vec![
                Span::styled(" Enter ", t.footer_key),
                Span::styled("menu  ", t.footer_desc),
                Span::styled(" q ", t.footer_key),
                Span::styled("quit", t.footer_desc),
            ]))
            .alignment(Alignment::Center);
            frame.render_widget(footer, sections[4]);

            // "Powered by ratatui" pinned to the bottom
            let [label_row, logo_row] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .areas(bottom);

            let powered_by = Paragraph::new(Span::styled("Powered by", t.hint))
                .alignment(Alignment::Center);
            frame.render_widget(powered_by, label_row);

            let logo_area = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(27),
                Constraint::Fill(1),
            ])
            .split(logo_row)[1];
            frame.render_widget(RatatuiLogo::small(), logo_area);
        }
        _ => {
            let text = Paragraph::new(Span::styled("Not implemented yet. Press Esc.", t.hint));
            frame.render_widget(text, frame.area());
        }
    }
}
