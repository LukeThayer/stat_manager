//! Combat log view

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use stat_core::damage::calculate_skill_dps;
use stat_core::calculate_damage_cap;
use stat_core::StatusEffect;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(50),       // Main combat area
            Constraint::Length(35),    // Damage preview panel
        ])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Status bar
            Constraint::Min(0),    // Combat log
        ])
        .split(chunks[0]);

    draw_status_bar(f, app, main_chunks[0]);
    draw_combat_log(f, app, main_chunks[1]);
    draw_damage_preview(f, app, chunks[1]);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let enemy = &app.enemy;
    let life_percent = enemy.life_percent();

    let life_color = if life_percent > 50.0 {
        Color::Green
    } else if life_percent > 25.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    // Create life bar
    let bar_width = area.width.saturating_sub(4) as usize;
    let filled = ((life_percent / 100.0) * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);

    let life_bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    let lines = vec![
        Line::from(vec![
            Span::styled("Enemy: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}/{:.0}", enemy.current_life, enemy.computed_max_life()),
                Style::default().fg(life_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({:.0}%)", life_percent),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(Span::styled(life_bar, Style::default().fg(life_color))),
        Line::from(vec![
            Span::styled("Time: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}s", app.time_elapsed),
                Style::default().fg(Color::White),
            ),
            Span::raw("  "),
            Span::styled("Skill: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &app.skills[app.selected_skill].name,
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Combat Status "));

    f.render_widget(paragraph, area);
}

fn draw_combat_log(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .combat_log
        .iter()
        .enumerate()
        .skip(app.log_scroll)
        .take(area.height.saturating_sub(2) as usize)
        .map(|(_, line)| {
            let style = if line.starts_with("━━━") {
                // Header line for new attack
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if line.contains("CRIT!") {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if line.contains("DEFEATED") {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if line.contains("▶ Damage dealt:") {
                // Outgoing damage
                Style::default().fg(Color::Green)
            } else if line.contains("▶ Enemy takes") {
                // Final damage result
                Style::default().fg(Color::Red)
            } else if line.contains("── Defense") {
                // Section header
                Style::default().fg(Color::Blue)
            } else if line.contains("☠️") || line.contains("Poison") {
                // Poison status
                Style::default().fg(Color::Green)
            } else if line.contains("🩸") || line.contains("Bleed") {
                // Bleed status
                Style::default().fg(Color::Red)
            } else if line.contains("🔥") || line.contains("Burn") || line.contains("Applied") {
                // Burn/DoT application
                Style::default().fg(Color::Magenta)
            } else if line.contains("❄️") || line.contains("Freeze") {
                // Freeze status
                Style::default().fg(Color::Cyan)
            } else if line.contains("🥶") || line.contains("Chill") {
                // Chill status
                Style::default().fg(Color::LightCyan)
            } else if line.contains("⚡") {
                // Static or Evasion triggered
                Style::default().fg(Color::Yellow)
            } else if line.contains("😱") || line.contains("Fear") {
                // Fear status
                Style::default().fg(Color::Magenta)
            } else if line.contains("🐌") || line.contains("Slow") {
                // Slow status
                Style::default().fg(Color::Gray)
            } else if line.contains("⚪") && line.contains("failed") {
                // Status effect failed to apply
                Style::default().fg(Color::DarkGray)
            } else if line.contains("🛡") {
                // ES absorption
                Style::default().fg(Color::Cyan)
            } else if line.contains("Total mitigated") {
                Style::default().fg(Color::DarkGray)
            } else if line.starts_with("  Skill:") || line.starts_with("  Weapon:") || line.starts_with("  Scaling:") {
                // Skill/weapon info
                Style::default().fg(Color::White)
            } else if line.starts_with("  Physical:") || line.starts_with("  Fire:") ||
                      line.starts_with("  Cold:") || line.starts_with("  Lightning:") ||
                      line.starts_with("  Chaos:") {
                // Per-type damage breakdown
                Style::default().fg(Color::Gray)
            } else if line.starts_with("[") && line.contains("DoT deals") {
                // DoT tick damage
                Style::default().fg(Color::Magenta)
            } else if line.starts_with("[") {
                // Timestamp lines
                Style::default().fg(Color::White)
            } else if line.starts_with("  ") {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(line.clone(), style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Combat Log (↑/↓ to scroll) "),
        );

    f.render_widget(list, area);
}

fn draw_damage_preview(f: &mut Frame, app: &App, area: Rect) {
    let skill = &app.skills[app.selected_skill];
    let player = &app.player;
    let enemy = &app.enemy;

    let dps = calculate_skill_dps(player, skill, &app.dot_registry);

    let mut lines = vec![
        Line::from(Span::styled(
            &skill.name,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "── Offense ──",
            Style::default().fg(Color::Cyan),
        )),
    ];

    // Show base damage sources
    if skill.is_attack() && skill.weapon_effectiveness > 0.0 {
        let wpn_avg = (player.weapon_physical_min + player.weapon_physical_max) / 2.0
            * skill.weapon_effectiveness;
        lines.push(Line::from(vec![
            Span::styled("Weapon avg: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}", wpn_avg), Style::default().fg(Color::White)),
        ]));
    }

    for base in &skill.base_damages {
        let avg = (base.min + base.max) / 2.0;
        lines.push(Line::from(vec![
            Span::styled(format!("{:?} avg: ", base.damage_type), Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}", avg), Style::default().fg(Color::White)),
        ]));
    }

    // Show scaling
    let phys_mult = player.global_physical_damage.total_increased_multiplier();
    let fire_mult = player.global_fire_damage.total_increased_multiplier();
    if phys_mult > 1.0 {
        lines.push(Line::from(vec![
            Span::styled("Phys scale: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("×{:.2}", phys_mult), Style::default().fg(Color::Green)),
        ]));
    }
    if fire_mult > 1.0 {
        lines.push(Line::from(vec![
            Span::styled("Fire scale: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("×{:.2}", fire_mult), Style::default().fg(Color::Green)),
        ]));
    }

    // Crit info
    let crit_chance = if skill.is_attack() {
        skill.base_crit_chance + player.weapon_crit_chance
    } else {
        skill.base_crit_chance
    };
    let crit_mult = player.computed_crit_multiplier();
    lines.push(Line::from(vec![
        Span::styled("Crit: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.0}% @ {:.0}%", crit_chance, crit_mult * 100.0),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    // Attack speed
    let speed = if skill.is_attack() {
        player.computed_attack_speed() * skill.attack_speed_modifier
    } else {
        player.computed_cast_speed() * skill.attack_speed_modifier
    };
    lines.push(Line::from(vec![
        Span::styled("Speed: ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{:.2}/s", speed), Style::default().fg(Color::White)),
    ]));

    // Estimated DPS
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Est. DPS: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.0}", dps),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ]));

    // Enemy defenses section
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "── Enemy Defenses ──",
        Style::default().fg(Color::Blue),
    )));

    lines.push(Line::from(vec![
        Span::styled("Armour: ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{:.0}", enemy.armour.compute()), Style::default().fg(Color::White)),
    ]));

    let evasion = enemy.evasion.compute();
    let accuracy = player.accuracy.compute();
    let damage_cap = calculate_damage_cap(accuracy, evasion);
    lines.push(Line::from(vec![
        Span::styled("Evasion: ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{:.0}", evasion), Style::default().fg(Color::White)),
        Span::styled(format!(" (cap: {:.0} vs {:.0} acc)", damage_cap, accuracy), Style::default().fg(Color::DarkGray)),
    ]));

    let fire_res = enemy.fire_resistance.compute();
    let cold_res = enemy.cold_resistance.compute();
    let light_res = enemy.lightning_resistance.compute();
    let chaos_res = enemy.chaos_resistance.compute();

    if fire_res != 0.0 {
        lines.push(Line::from(vec![
            Span::styled("Fire Res: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}%", fire_res), Style::default().fg(res_color(fire_res))),
        ]));
    }
    if cold_res != 0.0 {
        lines.push(Line::from(vec![
            Span::styled("Cold Res: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}%", cold_res), Style::default().fg(res_color(cold_res))),
        ]));
    }
    if light_res != 0.0 {
        lines.push(Line::from(vec![
            Span::styled("Light Res: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}%", light_res), Style::default().fg(res_color(light_res))),
        ]));
    }
    if chaos_res != 0.0 {
        lines.push(Line::from(vec![
            Span::styled("Chaos Res: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}%", chaos_res), Style::default().fg(res_color(chaos_res))),
        ]));
    }

    // Status effect chances
    let status_effects = [
        (StatusEffect::Poison, "☠️", Color::Green),
        (StatusEffect::Bleed, "🩸", Color::Red),
        (StatusEffect::Burn, "🔥", Color::Yellow),
        (StatusEffect::Freeze, "❄️", Color::Cyan),
        (StatusEffect::Chill, "🥶", Color::LightCyan),
        (StatusEffect::Static, "⚡", Color::LightYellow),
        (StatusEffect::Fear, "😱", Color::Magenta),
        (StatusEffect::Slow, "🐌", Color::Gray),
    ];

    // Calculate average damage for status calculations
    let avg_damages: Vec<(stat_core::DamageType, f64)> = skill.base_damages.iter()
        .map(|d| (d.damage_type, (d.min + d.max) / 2.0))
        .collect();

    let mut has_status = false;
    for (effect, _, _) in &status_effects {
        let conv = player.status_effect_stats.get_conversions(*effect);
        if conv.total() > 0.0 {
            has_status = true;
            break;
        }
    }

    if has_status {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "── Status Chance ──",
            Style::default().fg(Color::Magenta),
        )));

        let target_health = enemy.computed_max_life();

        for (effect, icon, color) in &status_effects {
            let status_dmg = player.status_effect_stats.calculate_status_damage(*effect, &avg_damages);
            if status_dmg > 0.0 {
                let chance = (status_dmg / target_health * 100.0).min(100.0);
                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(*color)),
                    Span::styled(format!("{:?}: ", effect), Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{:.0}%", chance),
                        Style::default().fg(if chance >= 50.0 { Color::Green } else if chance >= 25.0 { Color::Yellow } else { Color::White }),
                    ),
                ]));
            }
        }
    }

    // Active effects on enemy (DoTs and Status Effects)
    let has_active_effects = !enemy.active_dots.is_empty() || !enemy.active_status_effects.is_empty();

    if has_active_effects {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "── Active Effects ──",
            Style::default().fg(Color::Red),
        )));

        // Show active DoTs
        for dot in &enemy.active_dots {
            let icon = match dot.dot_type.as_str() {
                "ignite" => "🔥",
                "poison" => "☠️",
                "bleed" => "🩸",
                _ => "💀",
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(Color::Magenta)),
                Span::styled(
                    format!("{} ({:.0} DPS, {:.1}s)", dot.dot_type, dot.dps(), dot.duration_remaining),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        // Show active status effects
        for status in &enemy.active_status_effects {
            let icon = match status.effect_type {
                StatusEffect::Poison => "☠️",
                StatusEffect::Bleed => "🩸",
                StatusEffect::Burn => "🔥",
                StatusEffect::Freeze => "❄️",
                StatusEffect::Chill => "🥶",
                StatusEffect::Static => "⚡",
                StatusEffect::Fear => "😱",
                StatusEffect::Slow => "🐌",
            };
            // Show DoT DPS for damaging statuses
            let info = if status.is_damaging() && status.dot_dps > 0.0 {
                format!(
                    "{:?} ×{} ({:.0} DPS, {:.1}s)",
                    status.effect_type, status.stacks, status.dot_dps * status.stacks as f64, status.duration_remaining
                )
            } else {
                format!("{:?} ×{} ({:.1}s)", status.effect_type, status.stacks, status.duration_remaining)
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(Color::Red)),
                Span::styled(info, Style::default().fg(Color::White)),
            ]));
        }
    }

    // Kill time estimate
    if dps > 0.0 && enemy.is_alive() {
        let kill_time = enemy.current_life / dps;
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Kill in: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("~{:.1}s", kill_time),
                Style::default().fg(Color::Red),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Damage Preview "));

    f.render_widget(paragraph, area);
}

fn res_color(res: f64) -> Color {
    if res >= 75.0 {
        Color::Green
    } else if res >= 50.0 {
        Color::Yellow
    } else if res > 0.0 {
        Color::White
    } else {
        Color::Red
    }
}
