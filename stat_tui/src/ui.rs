//! UI rendering

mod breakdown_view;
mod combat_view;
mod equipment_view;
mod help_view;
mod skills_view;
mod stat_view;

use crate::app::{App, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Keybindings footer
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);

    match app.current_tab {
        Tab::Stats => stat_view::draw(f, app, chunks[1]),
        Tab::Equipment => equipment_view::draw(f, app, chunks[1]),
        Tab::Breakdown => breakdown_view::draw(f, app, chunks[1]),
        Tab::Combat => combat_view::draw(f, app, chunks[1]),
        Tab::Skills => skills_view::draw(f, app, chunks[1]),
        Tab::Help => help_view::draw(f, app, chunks[1]),
    }

    draw_keybindings(f, app, chunks[2]);
}

fn draw_keybindings(f: &mut Frame, app: &App, area: Rect) {
    let common_keys = vec![
        ("Tab", "Next tab"),
        ("q", "Quit"),
    ];

    let tab_keys: Vec<(&str, &str)> = match app.current_tab {
        Tab::Stats => vec![
            ("↑/↓", "Scroll"),
        ],
        Tab::Equipment => vec![
            ("↑/↓", "Scroll"),
        ],
        Tab::Breakdown => vec![
            ("↑/↓", "Scroll"),
        ],
        Tab::Combat => vec![
            ("a/Space", "Attack"),
            ("t", "+1 sec"),
            ("r", "Reset"),
            ("↑/↓", "Scroll log"),
        ],
        Tab::Skills => vec![
            ("↑/↓", "Select skill"),
            ("Enter/Space", "Use skill"),
            ("a", "Attack"),
        ],
        Tab::Help => vec![],
    };

    let mut spans: Vec<Span> = Vec::new();

    // Add tab-specific keys first
    for (i, (key, desc)) in tab_keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  │  ", Style::default().fg(Color::DarkGray)));
        }
        spans.push(Span::styled(
            format!("[{}]", key),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::White),
        ));
    }

    // Add separator if we have tab-specific keys
    if !tab_keys.is_empty() {
        spans.push(Span::styled("  │  ", Style::default().fg(Color::DarkGray)));
    }

    // Add common keys
    for (i, (key, desc)) in common_keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  │  ", Style::default().fg(Color::DarkGray)));
        }
        spans.push(Span::styled(
            format!("[{}]", key),
            Style::default().fg(Color::Cyan),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::Gray),
        ));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .block(Block::default().borders(Borders::ALL).title(" Keys "))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == app.current_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::from(Span::styled(t.name(), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Stat Manager "),
        )
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider("|");

    f.render_widget(tabs, area);
}

pub fn progress_bar(current: f64, max: f64, width: u16, filled_color: Color) -> Paragraph<'static> {
    let percent = if max > 0.0 { current / max } else { 0.0 };
    let filled = (percent * width as f64) as usize;
    let empty = width as usize - filled;

    let bar = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(empty)
    );

    Paragraph::new(bar).style(Style::default().fg(filled_color))
}

pub fn stat_line(name: &str, value: f64) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:20}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.1}", value),
            Style::default().fg(Color::White),
        ),
    ])
}

pub fn stat_line_with_computed(name: &str, base: f64, computed: f64) -> Line<'static> {
    if (base - computed).abs() < 0.1 {
        stat_line(name, computed)
    } else {
        Line::from(vec![
            Span::styled(
                format!("{:20}", name),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!("{:.1}", computed),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!(" (base: {:.1})", base),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    }
}
