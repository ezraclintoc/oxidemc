use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub title:        Style,  // big text logo
    pub border:       Style,  // box borders
    pub border_title: Style,  // text inside block titles
    pub highlight:    Style,  // selected list row
    pub normal:       Style,  // unselected list row
    pub value:        Style,  // settings values
    pub hint:         Style,  // dim hints ([Enter], descriptions)
    pub footer_key:   Style,  // key labels in footer bar
    pub footer_desc:  Style,  // descriptions in footer bar
    pub success:      Style,
    pub error:        Style,
    pub gauge:        Style,  // download progress bar
}

impl Default for Theme {
    fn default() -> Self {
        Self::rust()
    }
}

impl Theme {

    // Warm rust-metal orange — matches the Rust language brand colour (~#E8561A)
    pub fn rust() -> Self {
        Self {
            title:        Style::default().fg(Color::Rgb(232, 86, 26)).add_modifier(Modifier::BOLD),
            border:       Style::default().fg(Color::Rgb(110, 48, 14)),
            border_title: Style::default().fg(Color::Rgb(255, 168, 100)),
            highlight:    Style::default().fg(Color::Rgb(255, 130, 30)).add_modifier(Modifier::BOLD),
            normal:       Style::default().fg(Color::White),
            value:        Style::default().fg(Color::Rgb(255, 196, 80)),  // amber
            hint:         Style::default().fg(Color::DarkGray),
            footer_key:   Style::default().fg(Color::Rgb(255, 130, 30)).add_modifier(Modifier::BOLD),
            footer_desc:  Style::default().fg(Color::Gray),
            success:      Style::default().fg(Color::Green),
            error:        Style::default().fg(Color::Red),
            gauge:        Style::default().fg(Color::Rgb(232, 86, 26)),
        }
    }

    pub fn green() -> Self {
        Self {
            title:        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            border:       Style::default().fg(Color::DarkGray),
            border_title: Style::default().fg(Color::White),
            highlight:    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            normal:       Style::default().fg(Color::White),
            value:        Style::default().fg(Color::Yellow),
            hint:         Style::default().fg(Color::DarkGray),
            footer_key:   Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            footer_desc:  Style::default().fg(Color::Gray),
            success:      Style::default().fg(Color::Green),
            error:        Style::default().fg(Color::Red),
            gauge:        Style::default().fg(Color::Cyan),
        }
    }

    pub fn ocean() -> Self {
        Self {
            title:        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            border:       Style::default().fg(Color::Blue),
            border_title: Style::default().fg(Color::Cyan),
            highlight:    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            normal:       Style::default().fg(Color::White),
            value:        Style::default().fg(Color::LightBlue),
            hint:         Style::default().fg(Color::DarkGray),
            footer_key:   Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            footer_desc:  Style::default().fg(Color::Gray),
            success:      Style::default().fg(Color::Green),
            error:        Style::default().fg(Color::Red),
            gauge:        Style::default().fg(Color::LightBlue),
        }
    }

    pub fn lava() -> Self {
        Self {
            title:        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            border:       Style::default().fg(Color::DarkGray),
            border_title: Style::default().fg(Color::LightRed),
            highlight:    Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
            normal:       Style::default().fg(Color::White),
            value:        Style::default().fg(Color::LightYellow),
            hint:         Style::default().fg(Color::DarkGray),
            footer_key:   Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
            footer_desc:  Style::default().fg(Color::Gray),
            success:      Style::default().fg(Color::Green),
            error:        Style::default().fg(Color::Red),
            gauge:        Style::default().fg(Color::LightRed),
        }
    }
}
