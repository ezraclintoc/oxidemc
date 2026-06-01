use crate::state::{field_kind, FieldKind, SettingsState};
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};

pub fn draw(frame: &mut Frame, area: Rect, state: &SettingsState, t: &Theme) {
    let rows: Vec<(&str, String)> = vec![
        ("MOTD",                state.motd.clone()),
        ("Max Players",         state.max_players.to_string()),
        ("Gamemode",            state.gamemode.clone()),
        ("Difficulty",          state.difficulty.clone()),
        ("PVP",                 state.pvp.to_string()),
        ("Port",                state.port.to_string()),
        ("Max RAM",             state.max_ram.clone()),
        ("Min RAM",             state.min_ram.clone()),
        ("Render Distance",     state.render_distance.to_string()),
        ("Simulation Distance", state.simulation_distance.to_string()),
        ("RCON",                if state.rcon_enabled { "enabled".into() } else { "disabled".into() }),
        ("RCON Port",           state.rcon_port.to_string()),
        ("RCON Password",       if state.rcon_password.is_empty() { "(not set)".into() } else { "••••••••".into() }),
        ("[ Confirm ]",         String::new()),
    ];

    let items: Vec<ListItem> = rows
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            if *label == "[ Confirm ]" {
                ListItem::new(Line::from(Span::styled(
                    "[ Confirm ]",
                    t.success.add_modifier(Modifier::BOLD),
                )))
            } else {
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{:<22}", label)),
                    Span::styled(value.clone(), t.value),
                    if matches!(field_kind(i), FieldKind::Toggle) {
                        Span::raw("")
                    } else {
                        Span::styled("  [Enter]", t.hint)
                    },
                ]))
            }
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled("Settings (optional)", t.border_title))
                .borders(Borders::ALL)
                .border_style(t.border),
        )
        .highlight_style(t.highlight)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area.centered(Constraint::Length(50), Constraint::Length(rows.len() as u16 + 2)), &mut list_state);

    if state.editing {
        let field_name = rows.get(state.selected).map(|(l, _)| *l).unwrap_or("Value");
        match field_kind(state.selected) {
            FieldKind::Choice(options) => {
                let popup_area = centered_rect_fixed_height(40, options.len() as u16 + 2, area);
                frame.render_widget(Clear, popup_area);
                let items: Vec<ListItem> = options.iter().map(|&opt| ListItem::new(opt)).collect();
                let mut lstate = ListState::default();
                lstate.select(Some(state.edit_option_idx));
                let list = List::new(items)
                    .block(
                        Block::default()
                            .title(Span::styled(field_name, t.border_title))
                            .borders(Borders::ALL)
                            .border_style(t.border),
                    )
                    .highlight_style(t.highlight)
                    .highlight_symbol("> ");
                frame.render_stateful_widget(list, popup_area, &mut lstate);
            }
            kind => {
                let hint = match kind {
                    FieldKind::Number { min, max } => format!(" ({min}-{max})"),
                    FieldKind::Ram => " (e.g. 2G, 512M)".to_string(),
                    _ => String::new(),
                };
                let popup_area = centered_rect_fixed_height(50, 3, area);
                frame.render_widget(Clear, popup_area);
                let popup = Paragraph::new(Line::from(vec![
                    Span::raw("> "),
                    Span::styled(format!("{}_", state.edit_buf), t.value),
                ]))
                .block(
                    Block::default()
                        .title(Span::styled(format!("{field_name}{hint}"), t.border_title))
                        .borders(Borders::ALL)
                        .border_style(t.border),
                );
                frame.render_widget(popup, popup_area);
            }
        }
    }
}

fn centered_rect_fixed_height(percent_x: u16, height: u16, r: Rect) -> Rect {
    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vert[1])[1]
}
