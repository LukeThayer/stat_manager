//! Skills tab view

use crate::app::App;
use loot_core::types::DamageType;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use stat_core::damage::calculate_skill_dps;
use stat_core::StatusEffect;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(area);

    draw_skill_list(f, app, chunks[0]);
    draw_skill_details(f, app, chunks[1]);
}

fn draw_skill_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .skills
        .iter()
        .enumerate()
        .map(|(i, skill)| {
            let style = if i == app.selected_skill {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if i == app.selected_skill { "► " } else { "  " };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, skill.name),
                style,
            )))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Skills (↑/↓ to select, Enter/Space to use) "),
        );

    f.render_widget(list, area);
}

fn draw_skill_details(f: &mut Frame, app: &App, area: Rect) {
    let skill = &app.skills[app.selected_skill];
    let dps = calculate_skill_dps(&app.player, skill, &app.dot_registry);

    let mut lines = vec![
        Line::from(Span::styled(
            &skill.name,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Tags
    let tags: Vec<String> = skill.tags.iter().map(|t| format!("{:?}", t)).collect();
    lines.push(Line::from(vec![
        Span::styled("Tags: ", Style::default().fg(Color::Gray)),
        Span::styled(tags.join(", "), Style::default().fg(Color::Cyan)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "═══ Scaling ═══",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // Weapon Effectiveness
    lines.push(Line::from(vec![
        Span::styled("Weapon Effectiveness:   ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.0}%", skill.weapon_effectiveness * 100.0),
            Style::default().fg(if skill.weapon_effectiveness >= 1.0 { Color::Green }
                               else if skill.weapon_effectiveness > 0.0 { Color::Yellow }
                               else { Color::DarkGray }),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        "  How much of weapon damage the skill uses",
        Style::default().fg(Color::DarkGray),
    )));

    // Damage Effectiveness
    lines.push(Line::from(vec![
        Span::styled("Damage Effectiveness:   ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.0}%", skill.damage_effectiveness * 100.0),
            Style::default().fg(if skill.damage_effectiveness >= 1.0 { Color::Green }
                               else { Color::Yellow }),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        "  Multiplier for added damage & base damage",
        Style::default().fg(Color::DarkGray),
    )));

    // Attack Speed Modifier
    if (skill.attack_speed_modifier - 1.0).abs() > f64::EPSILON {
        lines.push(Line::from(vec![
            Span::styled("Speed Modifier:         ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0}%", skill.attack_speed_modifier * 100.0),
                Style::default().fg(if skill.attack_speed_modifier >= 1.0 { Color::Green }
                                   else { Color::Yellow }),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            "  Modifies attack/cast speed for this skill",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Base damages
    if !skill.base_damages.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "═══ Base Damage ═══",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        for dmg in &skill.base_damages {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:?}: ", dmg.damage_type),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!("{:.0}-{:.0}", dmg.min, dmg.max),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

    // Damage conversions
    if skill.damage_conversions.has_conversions() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "═══ Damage Conversions ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));

        let conv = &skill.damage_conversions;
        if conv.physical_to_fire > 0.0 {
            lines.push(format_conversion_dmg("Physical → Fire", conv.physical_to_fire));
        }
        if conv.physical_to_cold > 0.0 {
            lines.push(format_conversion_dmg("Physical → Cold", conv.physical_to_cold));
        }
        if conv.physical_to_lightning > 0.0 {
            lines.push(format_conversion_dmg("Physical → Lightning", conv.physical_to_lightning));
        }
        if conv.physical_to_chaos > 0.0 {
            lines.push(format_conversion_dmg("Physical → Chaos", conv.physical_to_chaos));
        }
        if conv.lightning_to_fire > 0.0 {
            lines.push(format_conversion_dmg("Lightning → Fire", conv.lightning_to_fire));
        }
        if conv.lightning_to_cold > 0.0 {
            lines.push(format_conversion_dmg("Lightning → Cold", conv.lightning_to_cold));
        }
        if conv.cold_to_fire > 0.0 {
            lines.push(format_conversion_dmg("Cold → Fire", conv.cold_to_fire));
        }
        if conv.fire_to_chaos > 0.0 {
            lines.push(format_conversion_dmg("Fire → Chaos", conv.fire_to_chaos));
        }
    }

    // Type effectiveness (only show if not default)
    if !skill.type_effectiveness.is_default() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "═══ Type Effectiveness ═══",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));

        let eff = &skill.type_effectiveness;
        if (eff.physical - 1.0).abs() > f64::EPSILON {
            lines.push(format_type_effectiveness("Physical", eff.physical, DamageType::Physical));
        }
        if (eff.fire - 1.0).abs() > f64::EPSILON {
            lines.push(format_type_effectiveness("Fire", eff.fire, DamageType::Fire));
        }
        if (eff.cold - 1.0).abs() > f64::EPSILON {
            lines.push(format_type_effectiveness("Cold", eff.cold, DamageType::Cold));
        }
        if (eff.lightning - 1.0).abs() > f64::EPSILON {
            lines.push(format_type_effectiveness("Lightning", eff.lightning, DamageType::Lightning));
        }
        if (eff.chaos - 1.0).abs() > f64::EPSILON {
            lines.push(format_type_effectiveness("Chaos", eff.chaos, DamageType::Chaos));
        }
    }

    // Crit
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "═══ Critical Strike ═══",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    lines.push(format_value("Base Crit Chance", skill.base_crit_chance));
    lines.push(format_value("Crit Multi Bonus", skill.crit_multiplier_bonus));

    // Skill status conversions
    let conv = &skill.status_conversions;
    let has_skill_conversions = conv.physical_to_poison > 0.0
        || conv.chaos_to_poison > 0.0
        || conv.physical_to_bleed > 0.0
        || conv.fire_to_burn > 0.0
        || conv.cold_to_freeze > 0.0
        || conv.cold_to_chill > 0.0
        || conv.lightning_to_static > 0.0
        || conv.chaos_to_fear > 0.0
        || conv.physical_to_slow > 0.0
        || conv.cold_to_slow > 0.0;

    if has_skill_conversions {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "═══ Skill Status Conversions ═══",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )));

        if conv.physical_to_poison > 0.0 {
            lines.push(format_conversion("Phys→Poison", conv.physical_to_poison));
        }
        if conv.chaos_to_poison > 0.0 {
            lines.push(format_conversion("Chaos→Poison", conv.chaos_to_poison));
        }
        if conv.physical_to_bleed > 0.0 {
            lines.push(format_conversion("Phys→Bleed", conv.physical_to_bleed));
        }
        if conv.fire_to_burn > 0.0 {
            lines.push(format_conversion("Fire→Burn", conv.fire_to_burn));
        }
        if conv.cold_to_freeze > 0.0 {
            lines.push(format_conversion("Cold→Freeze", conv.cold_to_freeze));
        }
        if conv.cold_to_chill > 0.0 {
            lines.push(format_conversion("Cold→Chill", conv.cold_to_chill));
        }
        if conv.lightning_to_static > 0.0 {
            lines.push(format_conversion("Light→Static", conv.lightning_to_static));
        }
        if conv.chaos_to_fear > 0.0 {
            lines.push(format_conversion("Chaos→Fear", conv.chaos_to_fear));
        }
        if conv.physical_to_slow > 0.0 {
            lines.push(format_conversion("Phys→Slow", conv.physical_to_slow));
        }
        if conv.cold_to_slow > 0.0 {
            lines.push(format_conversion("Cold→Slow", conv.cold_to_slow));
        }
    }

    // Status effect conversions from player stats
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

    let mut has_status_conversions = false;
    for (effect, _, _) in &status_effects {
        let conversions = app.player.status_effect_stats.get_conversions(*effect);
        if conversions.total() > 0.0 {
            has_status_conversions = true;
            break;
        }
    }

    if has_status_conversions {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "═══ Status Effect Conversions ═══",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )));

        for (effect, icon, color) in &status_effects {
            let conversions = app.player.status_effect_stats.get_conversions(*effect);
            let stats = app.player.status_effect_stats.get_stats(*effect);
            let total_conv = conversions.total();

            if total_conv > 0.0 {
                // Show conversion sources
                let mut conv_parts = Vec::new();
                if conversions.from_physical > 0.0 {
                    conv_parts.push(format!("Phys {:.0}%", conversions.from_physical * 100.0));
                }
                if conversions.from_fire > 0.0 {
                    conv_parts.push(format!("Fire {:.0}%", conversions.from_fire * 100.0));
                }
                if conversions.from_cold > 0.0 {
                    conv_parts.push(format!("Cold {:.0}%", conversions.from_cold * 100.0));
                }
                if conversions.from_lightning > 0.0 {
                    conv_parts.push(format!("Light {:.0}%", conversions.from_lightning * 100.0));
                }
                if conversions.from_chaos > 0.0 {
                    conv_parts.push(format!("Chaos {:.0}%", conversions.from_chaos * 100.0));
                }

                lines.push(Line::from(vec![
                    Span::styled(format!("{} {:?}: ", icon, effect), Style::default().fg(*color)),
                    Span::styled(conv_parts.join(", "), Style::default().fg(Color::White)),
                ]));

                // Show modifiers if any
                let mut mods = Vec::new();
                if stats.duration_increased > 0.0 {
                    mods.push(format!("+{:.0}% dur", stats.duration_increased * 100.0));
                }
                if stats.magnitude > 0.0 {
                    mods.push(format!("+{:.0}% mag", stats.magnitude * 100.0));
                }
                if stats.dot_increased > 0.0 {
                    mods.push(format!("+{:.0}% DoT", stats.dot_increased * 100.0));
                }
                if stats.max_stacks > 0 {
                    mods.push(format!("+{} stacks", stats.max_stacks));
                }

                if !mods.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("    ", Style::default()),
                        Span::styled(mods.join(", "), Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }
        }
    }

    // Damage Calculation Breakdown
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "═══ Damage Calculation ═══",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    )));

    // Show weapon damage if it's an attack skill
    if skill.is_attack() && skill.weapon_effectiveness > 0.0 {
        lines.push(Line::from(Span::styled(
            "Weapon Damage:",
            Style::default().fg(Color::White),
        )));

        let mut has_weapon_damage = false;
        for damage_type in [DamageType::Physical, DamageType::Fire, DamageType::Cold, DamageType::Lightning, DamageType::Chaos] {
            let (min, max) = app.player.weapon_damage(damage_type);
            if max > 0.0 {
                has_weapon_damage = true;
                let color = damage_type_color(damage_type);
                let scaled_min = min * skill.weapon_effectiveness;
                let scaled_max = max * skill.weapon_effectiveness;
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:?}: ", damage_type), Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:.0}-{:.0}", min, max), Style::default().fg(color)),
                    Span::styled(
                        format!(" × {:.0}% = {:.0}-{:.0}", skill.weapon_effectiveness * 100.0, scaled_min, scaled_max),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }
        if !has_weapon_damage {
            lines.push(Line::from(Span::styled(
                "  (no weapon equipped)",
                Style::default().fg(Color::DarkGray),
            )));
        }
        lines.push(Line::from(""));
    }

    // Calculate average damage by type for display
    let mut total_min = 0.0;
    let mut total_max = 0.0;
    let mut damage_breakdown: Vec<(DamageType, f64, f64, f64)> = Vec::new(); // (type, min, max, multiplier)

    // Skill base damages
    for base_dmg in &skill.base_damages {
        let damage_stat = match base_dmg.damage_type {
            DamageType::Physical => &app.player.global_physical_damage,
            DamageType::Fire => &app.player.global_fire_damage,
            DamageType::Cold => &app.player.global_cold_damage,
            DamageType::Lightning => &app.player.global_lightning_damage,
            DamageType::Chaos => &app.player.global_chaos_damage,
        };
        let mult = damage_stat.total_increased_multiplier() * damage_stat.total_more_multiplier() * skill.damage_effectiveness;
        let min = base_dmg.min * mult;
        let max = base_dmg.max * mult;
        damage_breakdown.push((base_dmg.damage_type, min, max, mult));
    }

    // Weapon damages for attacks
    if skill.is_attack() && skill.weapon_effectiveness > 0.0 {
        for damage_type in [DamageType::Physical, DamageType::Fire, DamageType::Cold, DamageType::Lightning, DamageType::Chaos] {
            let (wep_min, wep_max) = app.player.weapon_damage(damage_type);
            if wep_max > 0.0 {
                let damage_stat = match damage_type {
                    DamageType::Physical => &app.player.global_physical_damage,
                    DamageType::Fire => &app.player.global_fire_damage,
                    DamageType::Cold => &app.player.global_cold_damage,
                    DamageType::Lightning => &app.player.global_lightning_damage,
                    DamageType::Chaos => &app.player.global_chaos_damage,
                };
                let mult = damage_stat.total_increased_multiplier() * damage_stat.total_more_multiplier() * skill.damage_effectiveness;
                let min = wep_min * skill.weapon_effectiveness * mult;
                let max = wep_max * skill.weapon_effectiveness * mult;

                // Add to existing or create new
                if let Some(entry) = damage_breakdown.iter_mut().find(|(dt, _, _, _)| *dt == damage_type) {
                    entry.1 += min;
                    entry.2 += max;
                } else {
                    damage_breakdown.push((damage_type, min, max, mult));
                }
            }
        }
    }

    // Show damage by type
    lines.push(Line::from(Span::styled(
        "Damage per Hit:",
        Style::default().fg(Color::White),
    )));

    for (damage_type, min, max, mult) in &damage_breakdown {
        let color = damage_type_color(*damage_type);
        let avg = (min + max) / 2.0;
        total_min += min;
        total_max += max;
        lines.push(Line::from(vec![
            Span::styled(format!("  {:?}: ", damage_type), Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}-{:.0}", min, max), Style::default().fg(color)),
            Span::styled(format!(" (avg {:.0})", avg), Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" [×{:.2}]", mult), Style::default().fg(Color::DarkGray)),
        ]));
    }

    let total_avg = (total_min + total_max) / 2.0;
    lines.push(Line::from(vec![
        Span::styled("  Total: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.0}-{:.0}", total_min, total_max),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" (avg {:.0})", total_avg), Style::default().fg(Color::DarkGray)),
    ]));

    // Crit calculation
    lines.push(Line::from(""));
    let base_crit = if skill.is_attack() {
        skill.base_crit_chance + app.player.weapon_crit_chance
    } else {
        skill.base_crit_chance
    };
    let flat_crit = base_crit + app.player.critical_chance.flat;
    let crit_inc_mult = app.player.critical_chance.total_increased_multiplier();
    let crit_more_mult = app.player.critical_chance.total_more_multiplier();
    let final_crit = (flat_crit * crit_inc_mult * crit_more_mult).clamp(0.0, 100.0);
    let crit_mult = app.player.computed_crit_multiplier() + skill.crit_multiplier_bonus;

    lines.push(Line::from(vec![
        Span::styled("Crit Chance: ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{:.1}%", final_crit), Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" (base {:.0} × {:.2} inc × {:.2} more)", flat_crit, crit_inc_mult, crit_more_mult),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Crit Multi: ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{:.0}%", crit_mult * 100.0), Style::default().fg(Color::Yellow)),
    ]));

    // Attack/Cast speed
    lines.push(Line::from(""));
    let speed = if skill.is_attack() {
        app.player.computed_attack_speed() * skill.attack_speed_modifier
    } else {
        app.player.computed_cast_speed() * skill.attack_speed_modifier
    };
    let speed_label = if skill.is_attack() { "Attack Speed" } else { "Cast Speed" };
    lines.push(Line::from(vec![
        Span::styled(format!("{}: ", speed_label), Style::default().fg(Color::Gray)),
        Span::styled(format!("{:.2}/s", speed), Style::default().fg(Color::White)),
    ]));

    if skill.hits_per_attack > 1 {
        lines.push(Line::from(vec![
            Span::styled("Hits per Attack: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}", skill.hits_per_attack), Style::default().fg(Color::White)),
        ]));
    }

    // DPS calculation breakdown
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "═══ DPS Breakdown ═══",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    )));

    // DPS = Avg Damage × (1 + (CritMult-1) × CritChance) × Speed × HitsPerAttack
    let crit_chance_decimal = final_crit / 100.0;
    let crit_dps_mult = 1.0 + (crit_mult - 1.0) * crit_chance_decimal;
    let hit_dps = total_avg * crit_dps_mult * speed * skill.hits_per_attack as f64;

    lines.push(Line::from(Span::styled(
        "Formula: AvgDmg × CritFactor × Speed × Hits",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:.0} × {:.2} × {:.2} × {} = ", total_avg, crit_dps_mult, speed, skill.hits_per_attack),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.1}", hit_dps),
            Style::default().fg(Color::Green),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Hit DPS: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.1}", hit_dps),
            Style::default().fg(Color::Green),
        ),
    ]));

    // Check if skill has any status conversions that can apply DoTs
    let skill_conv = &skill.status_conversions;
    let has_dot_conversions = skill_conv.physical_to_poison > 0.0
        || skill_conv.chaos_to_poison > 0.0
        || skill_conv.physical_to_bleed > 0.0
        || skill_conv.fire_to_burn > 0.0;

    // Calculate and display DoT DPS breakdown (only for skills with DoT conversions)
    let dot_dps = dps - hit_dps;
    if has_dot_conversions {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "═══ DoT Breakdown ═══",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )));

        // Calculate DoT for each damaging status
        // (status, name, status_color, damage_type, damage_type_name)
        let damaging_statuses = [
            (StatusEffect::Poison, "Poison", Color::Green, DamageType::Chaos, "Chaos"),
            (StatusEffect::Bleed, "Bleed", Color::Red, DamageType::Physical, "Physical"),
            (StatusEffect::Burn, "Burn", Color::Yellow, DamageType::Fire, "Fire"),
        ];

        for (status, name, color, dot_damage_type, dot_type_name) in damaging_statuses {
            // Get conversions from skill and player
            let player_conv = app.player.status_effect_stats.get_conversions(status);

            // Calculate total conversion from all damage types
            let mut total_status_damage = 0.0;
            for (dt, min, max, _mult) in &damage_breakdown {
                let from_skill = match (*dt, status) {
                    (DamageType::Physical, StatusEffect::Poison) => skill_conv.physical_to_poison,
                    (DamageType::Chaos, StatusEffect::Poison) => skill_conv.chaos_to_poison,
                    (DamageType::Physical, StatusEffect::Bleed) => skill_conv.physical_to_bleed,
                    (DamageType::Fire, StatusEffect::Burn) => skill_conv.fire_to_burn,
                    _ => 0.0,
                };
                let from_player = player_conv.from_damage_type(*dt);
                let conv_rate = from_skill + from_player;
                if conv_rate > 0.0 {
                    let avg = (min + max) / 2.0;
                    total_status_damage += avg * conv_rate;
                }
            }

            if total_status_damage > 0.0 {
                let base_percent = app.dot_registry.get_base_damage_percent(status);
                let stats = app.player.status_effect_stats.get_stats(status);
                let dot_increased = 1.0 + stats.dot_increased;
                let status_dot_dps = base_percent * total_status_damage * dot_increased * speed;
                let duration = app.dot_registry.get_base_duration(status) * (1.0 + stats.duration_increased);
                let dot_type_color = damage_type_color(dot_damage_type);

                lines.push(Line::from(vec![
                    Span::styled(format!("  {}", name), Style::default().fg(color)),
                    Span::styled(" (", Style::default().fg(Color::DarkGray)),
                    Span::styled(dot_type_name, Style::default().fg(dot_type_color)),
                    Span::styled("): ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:.1} DPS", status_dot_dps), Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(
                        format!("{:.0}% base × {:.0} dmg × {:.2} inc × {:.2}/s",
                            base_percent * 100.0, total_status_damage, dot_increased, speed),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(format!("Duration: {:.1}s", duration), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        if dot_dps > 0.1 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("DoT DPS: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}", dot_dps),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Total DPS: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.1}", dps),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Skill Details "));

    f.render_widget(paragraph, area);
}

fn damage_type_color(dt: DamageType) -> Color {
    match dt {
        DamageType::Physical => Color::White,
        DamageType::Fire => Color::Red,
        DamageType::Cold => Color::Cyan,
        DamageType::Lightning => Color::Yellow,
        DamageType::Chaos => Color::Magenta,
    }
}

fn format_value(name: &str, value: f64) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:24}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.1}", value),
            Style::default().fg(Color::White),
        ),
    ])
}

fn format_percent(name: &str, value: f64) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:24}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.0}%", value),
            Style::default().fg(Color::White),
        ),
    ])
}

fn format_conversion(name: &str, value: f64) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:22}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.0}%", value * 100.0),
            Style::default().fg(Color::Magenta),
        ),
    ])
}

fn format_conversion_dmg(name: &str, value: f64) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:24}", name),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(
            format!("{:.0}%", value * 100.0),
            Style::default().fg(Color::Yellow),
        ),
    ])
}

fn format_type_effectiveness(name: &str, value: f64, dt: DamageType) -> Line<'static> {
    let color = damage_type_color(dt);
    Line::from(vec![
        Span::styled(
            format!("  {:20}", name),
            Style::default().fg(color),
        ),
        Span::styled(
            format!("{:.0}%", value * 100.0),
            Style::default().fg(Color::White),
        ),
    ])
}
