use ratatui::style::{Color, Modifier, Style};

pub const ACCENT: Color = Color::Cyan;
pub const MUTED: Color = Color::DarkGray;
pub const WARNING: Color = Color::Yellow;
pub const ERROR: Color = Color::Red;
pub fn focused_border() -> Style {
    Style::default().fg(ACCENT)
}

pub fn normal_border() -> Style {
    Style::default().fg(Color::Gray)
}

pub fn highlight() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn title() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED)
}
