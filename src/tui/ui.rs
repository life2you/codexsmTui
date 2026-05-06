use chrono::{DateTime, Local};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
};

use crate::{
    codex::{Session, parser::truncate_display_width},
    tui::{
        app::{App, Focus},
        styles,
    },
};

pub fn render(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(2)])
        .split(frame.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(38), Constraint::Min(40)])
        .split(layout[0]);

    render_projects(frame, app, body[0]);
    render_sessions(frame, app, body[1]);
    render_status(frame, app, layout[1]);

    if app.show_help {
        render_help(frame);
    }

    if let Some(detail) = &app.detail {
        render_detail(frame, detail);
    }

    if let Some(confirm) = &app.confirm_delete {
        render_confirm(frame, &confirm.prompt);
    }
}

fn render_projects(frame: &mut Frame, app: &App, area: Rect) {
    let items = app
        .projects
        .iter()
        .map(|project| {
            let path = truncate_display_width(&project.path, 26);
            ListItem::new(Line::from(vec![
                Span::styled(path, styles::title()),
                Span::styled(format!(" ({})", project.session_count), styles::muted()),
            ]))
        })
        .collect::<Vec<_>>();

    let block = Block::default()
        .title("Projects")
        .borders(Borders::ALL)
        .border_style(if app.focus == Focus::Projects {
            styles::focused_border()
        } else {
            styles::normal_border()
        });

    let list = List::new(items)
        .block(block)
        .highlight_style(styles::highlight());
    let mut state = app.project_state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_sessions(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.search_query.is_empty() {
        "Sessions".to_string()
    } else {
        format!("Sessions / {}", app.search_query)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if app.focus == Focus::Sessions {
            styles::focused_border()
        } else {
            styles::normal_border()
        });

    if app.filtered_session_indices.is_empty() {
        let paragraph = Paragraph::new(app.no_sessions_message())
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let items = app
        .filtered_session_indices
        .iter()
        .filter_map(|index| app.sessions.get(*index))
        .map(session_item)
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(block)
        .highlight_style(styles::highlight())
        .highlight_symbol("> ");

    let mut state = app.session_state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}

fn session_item(session: &Session) -> ListItem<'static> {
    let marker = if session.selected { "[x]" } else { "[ ]" };
    let title = truncate_display_width(&session.title, 68);
    let meta = format!(
        "{}  {}  {}",
        session.id,
        format_datetime(session.updated_at.as_ref()),
        human_size(session.size)
    );

    ListItem::new(vec![
        Line::from(vec![
            Span::styled(marker, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(title, styles::title()),
        ]),
        Line::from(vec![
            Span::styled(
                truncate_display_width(&session.project_path, 58),
                styles::muted(),
            ),
            Span::raw(" "),
            Span::styled(meta, styles::muted()),
        ]),
    ])
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let left = if app.search_mode {
        format!("Search: {}", app.search_query)
    } else {
        app.status.clone()
    };
    let warnings = if app.warnings.is_empty() {
        String::new()
    } else {
        format!(" | warnings: {}", app.warnings.len())
    };
    let selected = if app.selected_count() == 0 {
        String::new()
    } else {
        format!(" | selected: {}", app.selected_count())
    };

    let text = Line::from(vec![
        Span::raw(left),
        Span::styled(warnings, Style::default().fg(styles::WARNING)),
        Span::styled(selected, Style::default().fg(styles::ACCENT)),
        Span::styled(
            " | tab focus | / search | enter detail | d delete | D delete selected | ? help | q quit",
            styles::muted(),
        ),
    ]);

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(styles::normal_border()),
    );
    frame.render_widget(paragraph, area);
}

fn render_help(frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let text = Text::from(vec![
        Line::from("q      quit"),
        Line::from("?      toggle help"),
        Line::from("tab    switch focus"),
        Line::from("up/down move"),
        Line::from("enter  open detail"),
        Line::from("/      search"),
        Line::from("esc    close search or popup"),
        Line::from("space  select current session"),
        Line::from("d      delete current session"),
        Line::from("D      delete all selected sessions"),
        Line::from("r      refresh scan"),
        Line::from("g/G    top / bottom"),
        Line::from("y      confirm deletion"),
    ]);

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(styles::focused_border())
                .padding(Padding::uniform(1)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_confirm(frame: &mut Frame, prompt: &str) {
    let area = centered_rect(60, 18, frame.area());
    frame.render_widget(Clear, area);

    let paragraph = Paragraph::new(prompt)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title("Confirm Delete")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(styles::ERROR))
                .padding(Padding::uniform(1)),
        );
    frame.render_widget(paragraph, area);
}

fn render_detail(frame: &mut Frame, detail: &crate::codex::SessionDetail) {
    let area = centered_rect(80, 82, frame.area());
    frame.render_widget(Clear, area);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Session ID: ", styles::title()),
            Span::raw(detail.session.id.clone()),
        ]),
        Line::from(vec![
            Span::styled("Title: ", styles::title()),
            Span::raw(detail.session.title.clone()),
        ]),
        Line::from(vec![
            Span::styled("Project: ", styles::title()),
            Span::raw(detail.session.project_path.clone()),
        ]),
        Line::from(vec![
            Span::styled("Created: ", styles::title()),
            Span::raw(format_datetime(detail.session.created_at.as_ref())),
        ]),
        Line::from(vec![
            Span::styled("Updated: ", styles::title()),
            Span::raw(format_datetime(detail.session.updated_at.as_ref())),
        ]),
        Line::from(vec![
            Span::styled("File: ", styles::title()),
            Span::raw(detail.session.file_path.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled("Size: ", styles::title()),
            Span::raw(human_size(detail.session.size)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Recent Messages", styles::title())),
    ];

    if detail.recent_messages.is_empty() {
        lines.push(Line::from(Span::styled(
            "No recent user/assistant messages parsed.",
            styles::muted(),
        )));
    } else {
        for message in &detail.recent_messages {
            let header = format!(
                "[{}] {}",
                message.role,
                format_datetime(message.timestamp.as_ref())
            );
            lines.push(Line::from(Span::styled(header, styles::muted())));
            lines.push(Line::from(message.text.clone()));
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(Span::styled(
        "esc close | d delete",
        styles::muted(),
    )));

    let paragraph = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .title("Session Detail")
                .borders(Borders::ALL)
                .border_style(styles::focused_border())
                .padding(Padding::uniform(1)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn format_datetime(value: Option<&DateTime<Local>>) -> String {
    value
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn human_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / KB)
    } else {
        format!("{:.1} MB", size as f64 / MB)
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
