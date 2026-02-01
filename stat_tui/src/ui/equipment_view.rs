//! Equipment tab view - interactive equipment management

use crate::app::{App, EquipFocus, EquipTarget};
use loot_core::types::{AffixScope, DamageType, Rarity, StatType};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use stat_core::types::EquipmentSlot;
use stat_core::Item;
use std::collections::HashMap;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    // Split into three columns: slots, inventory, preview
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area);

    draw_equipment_slots(f, app, chunks[0]);
    draw_inventory(f, app, chunks[1]);
    draw_preview(f, app, chunks[2]);
}

fn draw_equipment_slots(f: &mut Frame, app: &App, area: Rect) {
    let equipment = app.current_equipment();
    let slots = EquipmentSlot::all();
    let selected_slot = app.selected_slot;
    let is_focused = app.equip_focus == EquipFocus::Slots;

    let mut lines: Vec<Line> = vec![];

    // Header with target indicator
    let target_name = match app.equip_target {
        EquipTarget::Player => "PLAYER",
        EquipTarget::Enemy => "ENEMY",
    };
    lines.push(Line::from(Span::styled(
        format!("  [e] Switch to: {}", if app.equip_target == EquipTarget::Player { "Enemy" } else { "Player" }),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    for (i, slot) in slots.iter().enumerate() {
        let slot_name = format_slot_name(slot);
        let is_selected = i == selected_slot && is_focused;

        let (prefix, style) = if is_selected {
            ("> ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            ("  ", Style::default().fg(Color::White))
        };

        if let Some(item) = equipment.get(slot) {
            let rarity_color = rarity_to_color(&item.rarity);
            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{:12}", slot_name), style),
                Span::styled(&item.name, Style::default().fg(rarity_color)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{:12}", slot_name), style),
                Span::styled("(empty)", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    // Help text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Enter] Unequip  [→] Inventory",
        Style::default().fg(Color::DarkGray),
    )));

    let border_color = if is_focused { Color::Yellow } else { Color::White };
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(format!(" {} Equipment ", target_name)),
    );

    f.render_widget(paragraph, area);
}

fn draw_inventory(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_inventory();
    let is_focused = app.equip_focus == EquipFocus::Inventory;
    let selected = app.selected_inventory;

    let mut lines: Vec<Line> = vec![];

    let slot = app.current_slot();
    lines.push(Line::from(Span::styled(
        format!("  Items for: {}", format_slot_name(&slot)),
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No items available for this slot",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, item) in filtered.iter().enumerate() {
            let is_selected = i == selected && is_focused;
            let rarity_color = rarity_to_color(&item.rarity);

            let (prefix, name_style) = if is_selected {
                ("> ", Style::default().fg(rarity_color).add_modifier(Modifier::BOLD | Modifier::REVERSED))
            } else {
                ("  ", Style::default().fg(rarity_color))
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(&item.name, name_style),
            ]));

            // Show brief stats for selected item
            if is_selected {
                // Base type
                lines.push(Line::from(Span::styled(
                    format!("    {}", item.base_name),
                    Style::default().fg(Color::Gray),
                )));

                // Defenses
                if item.defenses.armour.is_some() || item.defenses.evasion.is_some() {
                    let mut parts = vec![];
                    if let Some(a) = item.defenses.armour {
                        parts.push(format!("Arm: {}", a));
                    }
                    if let Some(e) = item.defenses.evasion {
                        parts.push(format!("Eva: {}", e));
                    }
                    lines.push(Line::from(Span::styled(
                        format!("    {}", parts.join(", ")),
                        Style::default().fg(Color::White),
                    )));
                }

                // Weapon damage
                if let Some(ref dmg) = item.damage {
                    for entry in &dmg.damages {
                        lines.push(Line::from(Span::styled(
                            format!("    {:?}: {}-{}", entry.damage_type, entry.min, entry.max),
                            Style::default().fg(Color::White),
                        )));
                    }
                }

                // Modifiers count
                let mod_count = item.prefixes.len() + item.suffixes.len() + if item.implicit.is_some() { 1 } else { 0 };
                if mod_count > 0 {
                    lines.push(Line::from(Span::styled(
                        format!("    {} modifiers", mod_count),
                        Style::default().fg(Color::Magenta),
                    )));
                }
            }
        }
    }

    // Help text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Enter] Equip  [←] Slots",
        Style::default().fg(Color::DarkGray),
    )));

    let border_color = if is_focused { Color::Yellow } else { Color::White };
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Inventory "),
    );

    f.render_widget(paragraph, area);
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = vec![];

    // Show currently equipped item details OR preview of selected inventory item
    if app.equip_focus == EquipFocus::Inventory {
        let filtered = app.filtered_inventory();
        if let Some(&item) = filtered.get(app.selected_inventory) {
            lines.push(Line::from(Span::styled(
                "Stat Changes if Equipped:",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            let diffs = app.preview_equip_diff(item);
            if diffs.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No stat changes",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                for (name, old, new) in diffs {
                    let diff = new - old;
                    let color = if diff > 0.0 { Color::Green } else { Color::Red };
                    let sign = if diff > 0.0 { "+" } else { "" };

                    lines.push(Line::from(vec![
                        Span::styled(format!("{:12}", name), Style::default().fg(Color::White)),
                        Span::styled(format!("{:.0}", old), Style::default().fg(Color::Gray)),
                        Span::styled(" → ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{:.0}", new), Style::default().fg(Color::White)),
                        Span::styled(format!(" ({}{:.0})", sign, diff), Style::default().fg(color)),
                    ]));
                }
            }

            // Show item modifiers
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Item Modifiers:",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));

            if let Some(ref implicit) = item.implicit {
                lines.push(Line::from(Span::styled(
                    format!("  {}", format_modifier(implicit)),
                    Style::default().fg(Color::Magenta),
                )));
            }

            // Collect local and global modifiers
            let all_mods: Vec<&loot_core::item::Modifier> = item
                .prefixes
                .iter()
                .chain(item.suffixes.iter())
                .collect();

            let local_mods: Vec<_> = all_mods
                .iter()
                .filter(|m| m.scope == AffixScope::Local)
                .collect();
            let global_mods: Vec<_> = all_mods
                .iter()
                .filter(|m| m.scope == AffixScope::Global)
                .collect();

            if !local_mods.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Local:",
                    Style::default().fg(Color::Cyan),
                )));
                for modifier in local_mods {
                    let color = if item.prefixes.iter().any(|p| p.affix_id == modifier.affix_id) {
                        Color::Blue
                    } else {
                        Color::Green
                    };
                    lines.push(Line::from(Span::styled(
                        format!("    {}", format_modifier(modifier)),
                        Style::default().fg(color),
                    )));
                }
            }

            if !global_mods.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Global:",
                    Style::default().fg(Color::Yellow),
                )));
                for modifier in global_mods {
                    let color = if item.prefixes.iter().any(|p| p.affix_id == modifier.affix_id) {
                        Color::Blue
                    } else {
                        Color::Green
                    };
                    lines.push(Line::from(Span::styled(
                        format!("    {}", format_modifier(modifier)),
                        Style::default().fg(color),
                    )));
                }
            }
        }
    } else {
        // Show currently equipped item
        let slot = app.current_slot();
        if let Some(item) = app.current_equipment().get(&slot) {
            lines.push(Line::from(Span::styled(
                "Currently Equipped:",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            draw_item_details(&mut lines, item);
        } else {
            lines.push(Line::from(Span::styled(
                "Slot Empty",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Press → to browse inventory",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Preview "),
    );

    f.render_widget(paragraph, area);
}

fn draw_item_details<'a>(lines: &mut Vec<Line<'a>>, item: &Item) {
    let rarity_color = rarity_to_color(&item.rarity);

    lines.push(Line::from(Span::styled(
        item.name.clone(),
        Style::default().fg(rarity_color).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        item.base_name.clone(),
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));

    // Defenses
    if item.defenses.armour.is_some() || item.defenses.evasion.is_some() || item.defenses.energy_shield.is_some() {
        if let Some(armour) = item.defenses.armour {
            lines.push(Line::from(Span::styled(
                format!("Armour: {}", armour),
                Style::default().fg(Color::White),
            )));
        }
        if let Some(evasion) = item.defenses.evasion {
            lines.push(Line::from(Span::styled(
                format!("Evasion: {}", evasion),
                Style::default().fg(Color::White),
            )));
        }
        if let Some(es) = item.defenses.energy_shield {
            lines.push(Line::from(Span::styled(
                format!("Energy Shield: {}", es),
                Style::default().fg(Color::White),
            )));
        }
        lines.push(Line::from(""));
    }

    // Weapon damage with local modifier calculations
    if let Some(ref damage) = item.damage {
        // Collect local modifiers for damage calculation
        let local_mods: Vec<_> = item
            .prefixes
            .iter()
            .chain(item.suffixes.iter())
            .filter(|m| m.scope == AffixScope::Local)
            .collect();

        // Calculate added flat damage and increased % from local mods
        let mut added_damage: HashMap<DamageType, (i32, i32)> = HashMap::new();
        let mut increased_phys_percent: f64 = 0.0;

        for modifier in &local_mods {
            match modifier.stat {
                StatType::AddedPhysicalDamage => {
                    let entry = added_damage.entry(DamageType::Physical).or_insert((0, 0));
                    entry.0 += modifier.value;
                    entry.1 += modifier.value_max.unwrap_or(modifier.value);
                }
                StatType::AddedFireDamage => {
                    let entry = added_damage.entry(DamageType::Fire).or_insert((0, 0));
                    entry.0 += modifier.value;
                    entry.1 += modifier.value_max.unwrap_or(modifier.value);
                }
                StatType::AddedColdDamage => {
                    let entry = added_damage.entry(DamageType::Cold).or_insert((0, 0));
                    entry.0 += modifier.value;
                    entry.1 += modifier.value_max.unwrap_or(modifier.value);
                }
                StatType::AddedLightningDamage => {
                    let entry = added_damage.entry(DamageType::Lightning).or_insert((0, 0));
                    entry.0 += modifier.value;
                    entry.1 += modifier.value_max.unwrap_or(modifier.value);
                }
                StatType::AddedChaosDamage => {
                    let entry = added_damage.entry(DamageType::Chaos).or_insert((0, 0));
                    entry.0 += modifier.value;
                    entry.1 += modifier.value_max.unwrap_or(modifier.value);
                }
                StatType::IncreasedPhysicalDamage => {
                    increased_phys_percent += modifier.value as f64;
                }
                _ => {}
            }
        }

        let has_local_damage_mods = !added_damage.is_empty() || increased_phys_percent > 0.0;

        // Show base damage
        lines.push(Line::from(Span::styled(
            "Base Damage:",
            Style::default().fg(Color::Gray),
        )));
        for entry in &damage.damages {
            lines.push(Line::from(Span::styled(
                format!("  {:?}: {}-{}", entry.damage_type, entry.min, entry.max),
                Style::default().fg(Color::White),
            )));
        }

        // Show calculated damage if there are local modifiers
        if has_local_damage_mods {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "With Local Mods:",
                Style::default().fg(Color::Cyan),
            )));

            // Calculate effective damage for each type
            let mut effective_damages: HashMap<DamageType, (i32, i32)> = HashMap::new();

            // Start with base damages
            for entry in &damage.damages {
                effective_damages.insert(entry.damage_type, (entry.min, entry.max));
            }

            // Add flat damage from local mods
            for (dmg_type, (add_min, add_max)) in &added_damage {
                let entry = effective_damages.entry(*dmg_type).or_insert((0, 0));
                entry.0 += add_min;
                entry.1 += add_max;
            }

            // Apply increased physical damage
            if increased_phys_percent > 0.0 {
                if let Some(phys) = effective_damages.get_mut(&DamageType::Physical) {
                    let multiplier = 1.0 + increased_phys_percent / 100.0;
                    phys.0 = (phys.0 as f64 * multiplier).round() as i32;
                    phys.1 = (phys.1 as f64 * multiplier).round() as i32;
                }
            }

            // Display in consistent order
            let damage_order = [
                DamageType::Physical,
                DamageType::Fire,
                DamageType::Cold,
                DamageType::Lightning,
                DamageType::Chaos,
            ];
            for dmg_type in damage_order {
                if let Some((min, max)) = effective_damages.get(&dmg_type) {
                    if *min > 0 || *max > 0 {
                        let color = match dmg_type {
                            DamageType::Physical => Color::White,
                            DamageType::Fire => Color::Red,
                            DamageType::Cold => Color::Cyan,
                            DamageType::Lightning => Color::Yellow,
                            DamageType::Chaos => Color::Magenta,
                        };
                        lines.push(Line::from(Span::styled(
                            format!("  {:?}: {}-{}", dmg_type, min, max),
                            Style::default().fg(color),
                        )));
                    }
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Attack Speed: {:.2}", damage.attack_speed),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(Span::styled(
            format!("Critical: {:.1}%", damage.critical_chance),
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));
    }

    // Implicit
    if let Some(ref implicit) = item.implicit {
        lines.push(Line::from(Span::styled(
            format_modifier(implicit),
            Style::default().fg(Color::Magenta),
        )));
    }

    // Collect local and global modifiers
    let all_mods: Vec<&loot_core::item::Modifier> = item
        .prefixes
        .iter()
        .chain(item.suffixes.iter())
        .collect();

    let local_mods: Vec<_> = all_mods
        .iter()
        .filter(|m| m.scope == AffixScope::Local)
        .collect();
    let global_mods: Vec<_> = all_mods
        .iter()
        .filter(|m| m.scope == AffixScope::Global)
        .collect();

    // Local modifiers (affect item base stats)
    if !local_mods.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Local:",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        for modifier in local_mods {
            let color = if item.prefixes.iter().any(|p| p.affix_id == modifier.affix_id) {
                Color::Blue
            } else {
                Color::Green
            };
            lines.push(Line::from(Span::styled(
                format!("  {}", format_modifier(modifier)),
                Style::default().fg(color),
            )));
        }
    }

    // Global modifiers (affect character stats)
    if !global_mods.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Global:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        for modifier in global_mods {
            let color = if item.prefixes.iter().any(|p| p.affix_id == modifier.affix_id) {
                Color::Blue
            } else {
                Color::Green
            };
            lines.push(Line::from(Span::styled(
                format!("  {}", format_modifier(modifier)),
                Style::default().fg(color),
            )));
        }
    }
}

fn format_slot_name(slot: &EquipmentSlot) -> &'static str {
    match slot {
        EquipmentSlot::MainHand => "Main Hand",
        EquipmentSlot::OffHand => "Off Hand",
        EquipmentSlot::Helmet => "Helmet",
        EquipmentSlot::BodyArmour => "Body",
        EquipmentSlot::Gloves => "Gloves",
        EquipmentSlot::Boots => "Boots",
        EquipmentSlot::Ring1 => "Ring 1",
        EquipmentSlot::Ring2 => "Ring 2",
        EquipmentSlot::Amulet => "Amulet",
        EquipmentSlot::Belt => "Belt",
    }
}

fn rarity_to_color(rarity: &Rarity) -> Color {
    match rarity {
        Rarity::Normal => Color::White,
        Rarity::Magic => Color::Blue,
        Rarity::Rare => Color::Yellow,
        Rarity::Unique => Color::Rgb(175, 96, 37),
    }
}

fn format_modifier(modifier: &loot_core::item::Modifier) -> String {
    let stat_name = format!("{:?}", modifier.stat);
    let readable: String = stat_name
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                format!(" {}", c)
            } else {
                c.to_string()
            }
        })
        .collect();

    format!("+{} {}", modifier.value, readable)
}
