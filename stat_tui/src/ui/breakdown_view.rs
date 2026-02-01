//! Stat breakdown tab - shows how each stat is calculated

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use stat_core::stat_block::StatValue;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    draw_player_breakdown(f, app, chunks[0]);
    draw_formulas(f, app, chunks[1]);
}

fn draw_player_breakdown(f: &mut Frame, app: &App, area: Rect) {
    let player = &app.player;
    let mut lines: Vec<Line> = vec![];

    // Resources
    lines.push(section_header("Resources"));
    lines.extend(stat_breakdown("Max Life", &player.max_life));
    lines.extend(stat_breakdown("Max Mana", &player.max_mana));
    lines.push(Line::from(""));

    // Attributes
    lines.push(section_header("Attributes"));
    lines.extend(stat_breakdown("Strength", &player.strength));
    lines.extend(stat_breakdown("Dexterity", &player.dexterity));
    lines.extend(stat_breakdown("Intelligence", &player.intelligence));
    lines.push(Line::from(""));

    // Offense
    lines.push(section_header("Offense"));
    lines.extend(stat_breakdown("Accuracy", &player.accuracy));
    lines.extend(stat_breakdown("Attack Speed", &player.attack_speed));
    lines.extend(stat_breakdown("Cast Speed", &player.cast_speed));
    lines.extend(stat_breakdown("Crit Chance", &player.critical_chance));
    lines.extend(stat_breakdown("Crit Multi", &player.critical_multiplier));
    lines.push(Line::from(""));

    // Damage scaling
    lines.push(section_header("Damage Scaling"));
    lines.extend(stat_breakdown("Physical", &player.global_physical_damage));
    lines.extend(stat_breakdown("Fire", &player.global_fire_damage));
    lines.extend(stat_breakdown("Cold", &player.global_cold_damage));
    lines.extend(stat_breakdown("Lightning", &player.global_lightning_damage));
    lines.extend(stat_breakdown("Chaos", &player.global_chaos_damage));
    lines.push(Line::from(""));

    // Defenses
    lines.push(section_header("Defenses"));
    lines.extend(stat_breakdown("Armour", &player.armour));
    lines.extend(stat_breakdown("Evasion", &player.evasion));
    lines.extend(stat_breakdown("Fire Res", &player.fire_resistance));
    lines.extend(stat_breakdown("Cold Res", &player.cold_resistance));
    lines.extend(stat_breakdown("Light Res", &player.lightning_resistance));
    lines.extend(stat_breakdown("Chaos Res", &player.chaos_resistance));
    lines.push(Line::from(""));

    // Penetration
    lines.push(section_header("Penetration"));
    lines.extend(stat_breakdown("Fire Pen", &player.fire_penetration));
    lines.extend(stat_breakdown("Cold Pen", &player.cold_penetration));
    lines.extend(stat_breakdown("Light Pen", &player.lightning_penetration));
    lines.extend(stat_breakdown("Chaos Pen", &player.chaos_penetration));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Player Stat Breakdown "),
        )
        .scroll((app.breakdown_scroll as u16, 0));

    f.render_widget(paragraph, area);
}

fn draw_formulas(f: &mut Frame, _app: &App, area: Rect) {
    let lines = vec![
        section_header("Stat Calculation Formula"),
        Line::from(""),
        Line::from(Span::styled(
            "Final = (Base + Flat) × (1 + Inc%) × Π(1 + More%)",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled("Where:", Style::default().fg(Color::Gray))),
        Line::from(vec![
            Span::styled("  Base  ", Style::default().fg(Color::Cyan)),
            Span::styled("= Character/skill base value", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Flat  ", Style::default().fg(Color::Blue)),
            Span::styled("= Sum of +X flat bonuses", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Inc%  ", Style::default().fg(Color::Green)),
            Span::styled("= Sum of X% increased (additive)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  More% ", Style::default().fg(Color::Magenta)),
            Span::styled("= Each X% more (multiplicative)", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        section_header("Defense Formulas"),
        Line::from(""),
        Line::from(Span::styled("Armour (Physical Reduction):", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled(
            "  Reduction = Armour / (Armour + 5 × Damage)",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled("Evasion (Damage Cap):", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled(
            "  Cap = Accuracy / (1 + Evasion / 1000)",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled("Resistance:", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled(
            "  Damage = Raw × (1 - EffectiveRes / 100)",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  EffectiveRes = Resist - Penetration",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "  (Pen is 50% effective vs capped res)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        section_header("Critical Strikes"),
        Line::from(""),
        Line::from(Span::styled(
            "Base Crit = Skill Crit + Weapon Crit",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "Final Crit = Base × (1 + Inc%) × Π(1 + More%)",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "Crit Damage = Damage × Crit Multiplier",
            Style::default().fg(Color::White),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Formulas & Reference "),
        );

    f.render_widget(paragraph, area);
}

fn section_header(name: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("═══ {} ═══", name),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ))
}

fn stat_breakdown(name: &str, stat: &StatValue) -> Vec<Line<'static>> {
    let mut lines = vec![];
    let final_value = stat.compute();

    // Stat name and final value
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:14}", name),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("= {:.1}", final_value),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    // Show the breakdown if there are any modifiers
    let has_modifiers = stat.flat != 0.0 || stat.increased != 0.0 || !stat.more.is_empty();

    if has_modifiers || stat.base != 0.0 {
        let mut parts = vec![];

        // Base
        if stat.base != 0.0 {
            parts.push(Span::styled(
                format!("{:.0}", stat.base),
                Style::default().fg(Color::Cyan),
            ));
        }

        // Flat
        if stat.flat != 0.0 {
            parts.push(Span::styled(
                format!(" +{:.0}", stat.flat),
                Style::default().fg(Color::Blue),
            ));
        }

        // Increased
        if stat.increased != 0.0 {
            parts.push(Span::styled(
                format!(" ×{:.2}", 1.0 + stat.increased),
                Style::default().fg(Color::Green),
            ));
            parts.push(Span::styled(
                format!("({:+.0}%)", stat.increased * 100.0),
                Style::default().fg(Color::DarkGray),
            ));
        }

        // More multipliers
        for (i, more) in stat.more.iter().enumerate() {
            parts.push(Span::styled(
                format!(" ×{:.2}", 1.0 + more),
                Style::default().fg(Color::Magenta),
            ));
        }

        if !parts.is_empty() {
            let mut full_parts = vec![Span::styled("  ", Style::default())];
            full_parts.extend(parts);
            lines.push(Line::from(full_parts));
        }
    }

    lines
}
