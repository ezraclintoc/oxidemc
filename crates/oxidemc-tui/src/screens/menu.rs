use crate::app::{App, Screen};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui_big_text::{BigText, PixelSize};

pub fn draw(frame: &mut Frame, app: &App) {
    let t = &app.theme;
    let selected = match app.screen {
        Screen::MainMenu { selected } => selected,
        _ => 0,
    };

    // Centre a 12-row band vertically, then a 58-col column horizontally
    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(12),
        Constraint::Fill(1),
    ])
    .split(frame.area());

    let horiz = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(58),
        Constraint::Fill(1),
    ])
    .split(vert[1])[1];

    // title(4) + gap(1) + list(5) + gap(1) + footer(1)
    let sections = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(horiz);

    // BigText title — HalfHeight = 4 terminal rows per char row
    let title = BigText::builder()
        .pixel_size(PixelSize::HalfHeight)
        .lines(vec![Line::from("OxideMC")])
        .style(t.title)
        .build();
    frame.render_widget(title, sections[0]);

    // Menu list with item descriptions
    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("Install    ", t.normal.add_modifier(Modifier::BOLD)),
            Span::styled("Download and set up a new server", t.hint),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Configure  ", t.normal.add_modifier(Modifier::BOLD)),
            Span::styled("Edit an existing server's settings", t.hint),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Manage     ", t.normal.add_modifier(Modifier::BOLD)),
            Span::styled("OxideMC global settings", t.hint),
        ])),
    ];

    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(t.border))
        .highlight_style(t.highlight)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, sections[2], &mut list_state);

    // Footer keybinding hints
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓ ", t.footer_key),
        Span::styled("navigate  ", t.footer_desc),
        Span::styled(" Enter ", t.footer_key),
        Span::styled("select  ", t.footer_desc),
        Span::styled(" q ", t.footer_key),
        Span::styled("quit", t.footer_desc),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, sections[4]);
}
