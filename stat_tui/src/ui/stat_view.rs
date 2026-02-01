//! Stats tab view

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    draw_player_stats(f, app, chunks[0]);
    draw_enemy_stats(f, app, chunks[1]);
}

fn draw_player_stats(f: &mut Frame, app: &App, area: Rect) {
    let player = &app.player;

    let mut lines = vec![
        Line::from(Span::styled(
            "═══ Resources ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        format_resource("Life", player.current_life, player.computed_max_life(), Color::Red),
        format_resource("Mana", player.current_mana, player.computed_max_mana(), Color::Blue),
        format_resource("ES", player.current_energy_shield, player.max_energy_shield, Color::Cyan),
        Line::from(""),
        Line::from(Span::styled(
            "═══ Attributes ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        format_stat("Strength", player.strength.compute()),
        format_stat("Dexterity", player.dexterity.compute()),
        format_stat("Intelligence", player.intelligence.compute()),
        format_stat("Constitution", player.constitution.compute()),
        format_stat("Wisdom", player.wisdom.compute()),
        format_stat("Charisma", player.charisma.compute()),
        Line::from(""),
        Line::from(Span::styled(
            "═══ Defenses ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        format_stat("Armour", player.armour.compute()),
        format_stat("Evasion", player.evasion.compute()),
        format_resistance("Fire Res", player.fire_resistance.compute()),
        format_resistance("Cold Res", player.cold_resistance.compute()),
        format_resistance("Lightning Res", player.lightning_resistance.compute()),
        format_resistance("Chaos Res", player.chaos_resistance.compute()),
        Line::from(""),
        Line::from(Span::styled(
            "═══ Offense ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        format_stat("Attack Speed", player.computed_attack_speed()),
        format_stat("Crit Chance", player.computed_attack_crit_chance()),
        format_stat("Crit Multiplier", player.computed_crit_multiplier() * 100.0),
        Line::from(""),
        Line::from(Span::styled(
            "═══ Weapon ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        format_damage_range("Physical", player.weapon_physical_min, player.weapon_physical_max),
        format_stat("Attack Speed", player.weapon_attack_speed),
        format_stat("Crit Chance", player.weapon_crit_chance),
        format_stat("DPS", player.weapon_dps()),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Player Stats "),
        )
        .scroll((app.stats_scroll as u16, 0));

    f.render_widget(paragraph, area);
}

fn draw_enemy_stats(f: &mut Frame, app: &App, area: Rect) {
    let enemy = &app.enemy;

    let mut lines = vec![
        Line::from(Span::styled(
            "═══ Resources ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        format_resource("Life", enemy.current_life, enemy.computed_max_life(), Color::Red),
        Line::from(""),
        Line::from(Span::styled(
            "═══ Defenses ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        format_stat("Armour", enemy.armour.compute()),
        format_stat("Evasion", enemy.evasion.compute()),
        format_resistance("Fire Res", enemy.fire_resistance.compute()),
        format_resistance("Cold Res", enemy.cold_resistance.compute()),
        format_resistance("Lightning Res", enemy.lightning_resistance.compute()),
        format_resistance("Chaos Res", enemy.chaos_resistance.compute()),
    ];

    // Show active DoTs
    if !enemy.active_dots.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "═══ Active DoTs ═══",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )));

        for dot in &enemy.active_dots {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:12}", dot.dot_type),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("{:.0} DPS, {:.1}s", dot.dps(), dot.duration_remaining),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Enemy Stats "),
        );

    f.render_widget(paragraph, area);
}

fn format_resource(name: &str, current: f64, max: f64, color: Color) -> Line<'static> {
    let percent = if max > 0.0 { current / max * 100.0 } else { 0.0 };
    Line::from(vec![
        Span::styled(
            format!("{:12}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.0}/{:.0}", current, max),
            Style::default().fg(color),
        ),
        Span::styled(
            format!(" ({:.0}%)", percent),
            Style::default().fg(Color::DarkGray),
        ),
    ])
}

fn format_stat(name: &str, value: f64) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:16}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.1}", value),
            Style::default().fg(Color::White),
        ),
    ])
}

fn format_resistance(name: &str, value: f64) -> Line<'static> {
    let color = if value >= 75.0 {
        Color::Green
    } else if value >= 50.0 {
        Color::Yellow
    } else if value >= 0.0 {
        Color::White
    } else {
        Color::Red
    };

    Line::from(vec![
        Span::styled(
            format!("{:16}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.0}%", value),
            Style::default().fg(color),
        ),
    ])
}

fn format_damage_range(name: &str, min: f64, max: f64) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:16}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.0}-{:.0}", min, max),
            Style::default().fg(Color::White),
        ),
    ])
}
