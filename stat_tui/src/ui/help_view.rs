//! Help tab view

use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "═══ Navigation ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        key_line("1-6", "Jump to tab (Stats/Equip/Calc/Combat/Skills/Help)"),
        key_line("Tab / Shift+Tab", "Next/previous tab"),
        key_line("↑/k  ↓/j", "Navigate lists / scroll"),
        key_line("q / Ctrl+C", "Quit"),
        key_line("?", "Toggle help"),
        Line::from(""),
        Line::from(Span::styled(
            "═══ Equipment ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        key_line("←/→", "Switch between slots and inventory"),
        key_line("e", "Toggle player/enemy equipment"),
        key_line("u", "Unequip from selected slot"),
        key_line("Enter", "Equip selected item"),
        Line::from(""),
        Line::from(Span::styled(
            "═══ Combat ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        key_line("a / Enter / Space", "Attack with selected skill"),
        key_line("t", "Advance time by 1 second (for DoT ticks)"),
        key_line("r", "Reset combat (restore enemy HP)"),
        Line::from(""),
        Line::from(Span::styled(
            "═══ Game Mechanics ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Stat Calculation (POE-style):",
            Style::default().fg(Color::Yellow),
        )),
        Line::from("  Final = (Base + Flat) × (1 + Σ Increased%) × Π(1 + More%)"),
        Line::from(""),
        Line::from(Span::styled(
            "Armour (Physical Reduction):",
            Style::default().fg(Color::Yellow),
        )),
        Line::from("  Reduction = Armour / (Armour + 5 × Damage)"),
        Line::from("  More effective vs small hits, less vs big hits"),
        Line::from(""),
        Line::from(Span::styled(
            "Evasion (One-Shot Protection):",
            Style::default().fg(Color::Yellow),
        )),
        Line::from("  Max damage per hit = Evasion rating"),
        Line::from("  Excess damage is completely negated"),
        Line::from(""),
        Line::from(Span::styled(
            "Resistance (100% Cap):",
            Style::default().fg(Color::Yellow),
        )),
        Line::from("  100% = immunity (achievable with investment)"),
        Line::from("  Penetration vs capped: 50% effectiveness"),
        Line::from(""),
        Line::from(Span::styled(
            "Energy Shield:",
            Style::default().fg(Color::Yellow),
        )),
        Line::from("  Does NOT passively regenerate"),
        Line::from("  Must be applied via warding spells"),
        Line::from("  Absorbs damage before HP"),
        Line::from(""),
        Line::from(Span::styled(
            "DoT Stacking:",
            Style::default().fg(Color::Yellow),
        )),
        Line::from("  Ignite: Strongest only"),
        Line::from("  Poison: Unlimited stacking"),
        Line::from("  Bleed: 8 max stacks, 50% effectiveness"),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Help & Mechanics "));

    f.render_widget(paragraph, area);
}

fn key_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:20}", key),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(desc.to_string(), Style::default().fg(Color::White)),
    ])
}
