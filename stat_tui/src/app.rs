//! Application state

use stat_core::{
    calculate_damage_cap,
    combat::resolve_damage,
    damage::{calculate_damage, DamagePacketGenerator},
    default_skills,
    dot::DotRegistry,
    source::{BaseStatsSource, GearSource, StatSource},
    stat_block::StatBlock,
    types::EquipmentSlot,
    DamageType, Item, StatusEffect,
};
use loot_core::item::{Defenses, WeaponDamage, DamageValue, Modifier};
use loot_core::types::{ItemClass, Rarity, StatType};
use rand::SeedableRng;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// Equipment target (player or enemy)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipTarget {
    Player,
    Enemy,
}

/// Equipment panel focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipFocus {
    Slots,
    Inventory,
}

/// Container for loading items from JSON
#[derive(Debug, Deserialize)]
struct ItemsFile {
    items: Vec<Item>,
}

/// Helper to create a simple global modifier
fn simple_mod(stat: StatType, value: i32) -> Modifier {
    simple_mod_with_scope(stat, value, loot_core::types::AffixScope::Global)
}

/// Helper to create a modifier with a specific scope
fn simple_mod_with_scope(stat: StatType, value: i32, scope: loot_core::types::AffixScope) -> Modifier {
    Modifier {
        affix_id: format!("{:?}", stat).to_lowercase(),
        name: format!("{:?}", stat),
        stat,
        scope,
        tier: 1,
        value,
        value_max: None,
        tier_min: value,
        tier_max: value,
        tier_max_value: None,
    }
}

/// Helper to create a local flat damage modifier with min-max range
fn local_flat_damage(stat: StatType, min: i32, max: i32) -> Modifier {
    Modifier {
        affix_id: format!("{:?}", stat).to_lowercase(),
        name: format!("{:?}", stat),
        stat,
        scope: loot_core::types::AffixScope::Local,
        tier: 1,
        value: min,
        value_max: Some(max),
        tier_min: min,
        tier_max: max,
        tier_max_value: Some((min, max)),
    }
}

/// Helper to create a local increased physical damage modifier
fn local_increased_phys(value: i32) -> Modifier {
    simple_mod_with_scope(StatType::IncreasedPhysicalDamage, value, loot_core::types::AffixScope::Local)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Stats,
    Equipment,
    Breakdown,
    Combat,
    Skills,
    Help,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Stats, Tab::Equipment, Tab::Breakdown, Tab::Combat, Tab::Skills, Tab::Help]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tab::Stats => "Stats",
            Tab::Equipment => "Equip",
            Tab::Breakdown => "Calc",
            Tab::Combat => "Combat",
            Tab::Skills => "Skills",
            Tab::Help => "Help",
        }
    }
}

pub struct App {
    pub current_tab: Tab,
    pub player: StatBlock,
    pub enemy: StatBlock,
    pub player_equipment: HashMap<EquipmentSlot, Item>,
    pub enemy_equipment: HashMap<EquipmentSlot, Item>,
    pub inventory: Vec<Item>,
    pub skills: Vec<DamagePacketGenerator>,
    pub selected_skill: usize,
    pub combat_log: Vec<String>,
    pub dot_registry: DotRegistry,
    pub rng: rand::rngs::StdRng,
    pub show_help: bool,
    pub stats_scroll: usize,
    pub log_scroll: usize,
    pub time_elapsed: f64,
    pub breakdown_scroll: usize,
    // Equipment UI state
    pub equip_target: EquipTarget,
    pub equip_focus: EquipFocus,
    pub selected_slot: usize,
    pub selected_inventory: usize,
}

impl App {
    pub fn new() -> Self {
        // Create player equipment
        let mut player_equipment = HashMap::new();

        // Weapon: Iron Sword
        // Local mods affect weapon base damage, global mods affect character stats
        let weapon = Item {
            seed: 1,
            operations: vec![],
            base_type_id: "iron_sword".to_string(),
            name: "Warrior's Iron Sword".to_string(),
            base_name: "Iron Sword".to_string(),
            class: ItemClass::OneHandSword,
            rarity: Rarity::Rare,
            tags: vec![],
            requirements: Default::default(),
            implicit: Some(simple_mod(StatType::IncreasedAccuracy, 15)),
            prefixes: vec![
                // Local: adds flat physical damage to weapon (5-12)
                local_flat_damage(StatType::AddedPhysicalDamage, 5, 12),
                // Local: increases weapon's physical damage by 75%
                local_increased_phys(75),
            ],
            suffixes: vec![
                // Global: increases character attack speed
                simple_mod(StatType::IncreasedAttackSpeed, 8),
                // Local: adds flat fire damage to weapon (8-15)
                local_flat_damage(StatType::AddedFireDamage, 8, 15),
            ],
            defenses: Defenses::default(),
            damage: Some(WeaponDamage {
                damages: vec![DamageValue {
                    damage_type: DamageType::Physical,
                    min: 30,
                    max: 60,
                }],
                attack_speed: 1.2,
                critical_chance: 5.0,
                spell_efficiency: 0.0,
            }),
        };
        player_equipment.insert(EquipmentSlot::MainHand, weapon);

        // Body Armour: Iron Plate
        let body = Item {
            seed: 2,
            operations: vec![],
            base_type_id: "iron_plate".to_string(),
            name: "Sturdy Iron Plate".to_string(),
            base_name: "Iron Plate".to_string(),
            class: ItemClass::BodyArmour,
            rarity: Rarity::Magic,
            tags: vec![],
            requirements: Default::default(),
            implicit: None,
            prefixes: vec![
                simple_mod(StatType::AddedLife, 40),
            ],
            suffixes: vec![
                simple_mod(StatType::FireResistance, 20),
            ],
            defenses: Defenses {
                armour: Some(150),
                evasion: None,
                energy_shield: None,
            },
            damage: None,
        };
        player_equipment.insert(EquipmentSlot::BodyArmour, body);

        // Helmet: Iron Helm
        let helmet = Item {
            seed: 3,
            operations: vec![],
            base_type_id: "iron_helm".to_string(),
            name: "Reinforced Iron Helm".to_string(),
            base_name: "Iron Helm".to_string(),
            class: ItemClass::Helmet,
            rarity: Rarity::Magic,
            tags: vec![],
            requirements: Default::default(),
            implicit: None,
            prefixes: vec![
                simple_mod(StatType::AddedLife, 25),
            ],
            suffixes: vec![
                simple_mod(StatType::ColdResistance, 18),
            ],
            defenses: Defenses {
                armour: Some(80),
                evasion: None,
                energy_shield: None,
            },
            damage: None,
        };
        player_equipment.insert(EquipmentSlot::Helmet, helmet);

        // Ring: Gold Ring
        let ring = Item {
            seed: 4,
            operations: vec![],
            base_type_id: "gold_ring".to_string(),
            name: "Ruby Gold Ring".to_string(),
            base_name: "Gold Ring".to_string(),
            class: ItemClass::Ring,
            rarity: Rarity::Magic,
            tags: vec![],
            requirements: Default::default(),
            implicit: Some(simple_mod(StatType::IncreasedItemRarity, 10)),
            prefixes: vec![
                simple_mod(StatType::AddedLife, 30),
            ],
            suffixes: vec![
                simple_mod(StatType::FireResistance, 25),
                simple_mod(StatType::LightningResistance, 15),
            ],
            defenses: Defenses::default(),
            damage: None,
        };
        player_equipment.insert(EquipmentSlot::Ring1, ring);

        // Build player stats from sources
        let mut player = StatBlock::new();
        let base_stats = BaseStatsSource::new(10);

        let mut sources: Vec<Box<dyn StatSource>> = vec![Box::new(base_stats)];
        for (slot, item) in &player_equipment {
            sources.push(Box::new(GearSource::new(*slot, item.clone())));
        }
        player.rebuild_from_sources(&sources);

        // Create enemy equipment (simple)
        let mut enemy_equipment = HashMap::new();

        let enemy_armor = Item {
            seed: 5,
            operations: vec![],
            base_type_id: "bone_armor".to_string(),
            name: "Bone Armor".to_string(),
            base_name: "Bone Armor".to_string(),
            class: ItemClass::BodyArmour,
            rarity: Rarity::Normal,
            tags: vec![],
            requirements: Default::default(),
            implicit: None,
            prefixes: vec![],
            suffixes: vec![],
            defenses: Defenses {
                armour: Some(200),
                evasion: Some(5000),
                energy_shield: None,
            },
            damage: None,
        };
        enemy_equipment.insert(EquipmentSlot::BodyArmour, enemy_armor);

        // Set up enemy stats
        let mut enemy = StatBlock::new();
        enemy.max_life.base = 500.0;
        enemy.current_life = 500.0;
        enemy.armour.base = 200.0;
        enemy.evasion.base = 5000.0;
        enemy.fire_resistance.base = 25.0;
        enemy.cold_resistance.base = 40.0;
        enemy.lightning_resistance.base = 15.0;
        enemy.chaos_resistance.base = -10.0;

        // Load skills from config
        let skill_map = default_skills();
        let mut skills: Vec<DamagePacketGenerator> = skill_map.into_values().collect();
        // Sort skills by name for consistent ordering
        skills.sort_by(|a, b| a.name.cmp(&b.name));

        // Load inventory from file
        let inventory = Self::load_inventory();

        App {
            current_tab: Tab::Stats,
            player,
            enemy,
            player_equipment,
            enemy_equipment,
            inventory,
            skills,
            selected_skill: 0,
            combat_log: vec!["Combat simulation ready.".to_string()],
            dot_registry: DotRegistry::with_defaults(),
            rng: rand::rngs::StdRng::seed_from_u64(42),
            show_help: false,
            stats_scroll: 0,
            log_scroll: 0,
            time_elapsed: 0.0,
            breakdown_scroll: 0,
            equip_target: EquipTarget::Player,
            equip_focus: EquipFocus::Slots,
            selected_slot: 0,
            selected_inventory: 0,
        }
    }

    fn load_inventory() -> Vec<Item> {
        // Try to load from data/items.json
        let paths = [
            "stat_tui/data/items.json",
            "data/items.json",
            "../stat_tui/data/items.json",
        ];

        for path in paths {
            if let Ok(content) = fs::read_to_string(path) {
                match serde_json::from_str::<ItemsFile>(&content) {
                    Ok(items_file) => {
                        eprintln!("Loaded {} items from {}", items_file.items.len(), path);
                        return items_file.items;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", path, e);
                    }
                }
            }
        }

        eprintln!("No items.json found, using empty inventory");
        // Return empty inventory if file not found
        Vec::new()
    }

    pub fn next_tab(&mut self) {
        let tabs = Tab::all();
        let current_idx = tabs.iter().position(|t| *t == self.current_tab).unwrap_or(0);
        let next_idx = (current_idx + 1) % tabs.len();
        self.current_tab = tabs[next_idx];
    }

    pub fn prev_tab(&mut self) {
        let tabs = Tab::all();
        let current_idx = tabs.iter().position(|t| *t == self.current_tab).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            tabs.len() - 1
        } else {
            current_idx - 1
        };
        self.current_tab = tabs[prev_idx];
    }

    pub fn set_tab(&mut self, index: usize) {
        let tabs = Tab::all();
        if index < tabs.len() {
            self.current_tab = tabs[index];
        }
    }

    pub fn on_up(&mut self) {
        match self.current_tab {
            Tab::Skills => {
                if self.selected_skill > 0 {
                    self.selected_skill -= 1;
                }
            }
            Tab::Equipment => {
                match self.equip_focus {
                    EquipFocus::Slots => {
                        if self.selected_slot > 0 {
                            self.selected_slot -= 1;
                        }
                    }
                    EquipFocus::Inventory => {
                        if self.selected_inventory > 0 {
                            self.selected_inventory -= 1;
                        }
                    }
                }
            }
            Tab::Stats => {
                if self.stats_scroll > 0 {
                    self.stats_scroll -= 1;
                }
            }
            Tab::Breakdown => {
                if self.breakdown_scroll > 0 {
                    self.breakdown_scroll -= 1;
                }
            }
            Tab::Combat => {
                if self.log_scroll > 0 {
                    self.log_scroll -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn on_down(&mut self) {
        match self.current_tab {
            Tab::Skills => {
                if self.selected_skill < self.skills.len().saturating_sub(1) {
                    self.selected_skill += 1;
                }
            }
            Tab::Equipment => {
                match self.equip_focus {
                    EquipFocus::Slots => {
                        let max_slots = EquipmentSlot::all().len();
                        if self.selected_slot < max_slots - 1 {
                            self.selected_slot += 1;
                        }
                    }
                    EquipFocus::Inventory => {
                        let filtered = self.filtered_inventory();
                        if self.selected_inventory < filtered.len().saturating_sub(1) {
                            self.selected_inventory += 1;
                        }
                    }
                }
            }
            Tab::Stats => {
                self.stats_scroll += 1;
            }
            Tab::Breakdown => {
                self.breakdown_scroll += 1;
            }
            Tab::Combat => {
                self.log_scroll += 1;
            }
            _ => {}
        }
    }

    pub fn on_left(&mut self) {
        if self.current_tab == Tab::Equipment {
            self.equip_focus = EquipFocus::Slots;
        }
    }

    pub fn on_right(&mut self) {
        if self.current_tab == Tab::Equipment {
            self.equip_focus = EquipFocus::Inventory;
            // Reset inventory selection when switching
            self.selected_inventory = 0;
        }
    }

    pub fn on_enter(&mut self) {
        match self.current_tab {
            Tab::Skills => self.attack(),
            Tab::Equipment => self.equip_selected(),
            _ => {}
        }
    }

    pub fn on_space(&mut self) {
        self.attack();
    }

    pub fn attack(&mut self) {
        if !self.enemy.is_alive() {
            self.combat_log.push("Enemy is already dead!".to_string());
            return;
        }

        let skill = &self.skills[self.selected_skill];
        let packet = calculate_damage(
            &self.player,
            skill,
            "player".to_string(),
            &self.dot_registry,
            &mut self.rng,
        );

        // === DAMAGE PACKET GENERATION ===
        self.combat_log.push(format!(
            "━━━ [{:.1}s] {} uses {} ━━━",
            self.time_elapsed, "Player", skill.name
        ));

        // Show skill info
        let skill_type = if skill.is_attack() { "Attack" } else { "Spell" };
        self.combat_log.push(format!(
            "  Skill: {} ({}% weapon, {}% effectiveness)",
            skill_type,
            (skill.weapon_effectiveness * 100.0) as i32,
            (skill.damage_effectiveness * 100.0) as i32
        ));

        // Show base damages from skill
        if !skill.base_damages.is_empty() {
            let base_str: Vec<String> = skill.base_damages
                .iter()
                .map(|d| format!("{:?} {:.0}-{:.0}", d.damage_type, d.min, d.max))
                .collect();
            self.combat_log.push(format!("  Skill base: {}", base_str.join(", ")));
        }

        // Show weapon contribution if attack
        if skill.is_attack() && skill.weapon_effectiveness > 0.0 {
            let wpn_phys = format!(
                "{:.0}-{:.0}",
                self.player.weapon_physical_min * skill.weapon_effectiveness,
                self.player.weapon_physical_max * skill.weapon_effectiveness
            );
            self.combat_log.push(format!(
                "  Weapon: Physical {} (base {:.0}-{:.0} × {:.0}%)",
                wpn_phys,
                self.player.weapon_physical_min,
                self.player.weapon_physical_max,
                skill.weapon_effectiveness * 100.0
            ));
        }

        // Show damage scaling
        let phys_inc = self.player.global_physical_damage.increased * 100.0;
        let fire_inc = self.player.global_fire_damage.increased * 100.0;
        if phys_inc > 0.0 || fire_inc > 0.0 {
            let mut scaling = Vec::new();
            if phys_inc > 0.0 {
                scaling.push(format!("Physical +{:.0}%", phys_inc));
            }
            if fire_inc > 0.0 {
                scaling.push(format!("Fire +{:.0}%", fire_inc));
            }
            self.combat_log.push(format!("  Scaling: {}", scaling.join(", ")));
        }

        // Show final damage packet
        let damage_breakdown: Vec<String> = packet.damages
            .iter()
            .map(|d| format!("{:?}: {:.0}", d.damage_type, d.amount))
            .collect();

        let crit_info = if packet.is_critical {
            format!(" × {:.0}% CRIT!", packet.crit_multiplier * 100.0)
        } else {
            String::new()
        };

        self.combat_log.push(format!(
            "  ▶ Damage dealt: {} = {:.0} total{}",
            damage_breakdown.join(" + "),
            packet.total_damage(),
            crit_info
        ));

        // Show accuracy
        self.combat_log.push(format!(
            "  Accuracy: {:.0}",
            packet.accuracy
        ));

        // === DAMAGE RESOLUTION ===
        let result = resolve_damage(&mut self.enemy, &packet, &self.dot_registry);

        self.combat_log.push("  ── Defense Calculation ──".to_string());

        // Show evasion cap calculation
        let evasion = self.enemy.evasion.compute();
        let damage_cap = calculate_damage_cap(packet.accuracy, evasion);
        self.combat_log.push(format!(
            "  Evasion cap: {:.0} (acc {:.0} vs eva {:.0})",
            damage_cap, packet.accuracy, evasion
        ));

        // Show per-type breakdown
        for taken in &result.damage_taken {
            let resist = self.enemy.resistance(taken.damage_type);
            let pen = packet.penetration(taken.damage_type);

            if taken.damage_type == DamageType::Physical {
                let armour = self.enemy.armour.compute();
                self.combat_log.push(format!(
                    "  {:?}: {:.0} raw → {:.0} after armour ({:.0} armour, {:.0}% reduction)",
                    taken.damage_type,
                    taken.raw_amount,
                    taken.final_amount,
                    armour,
                    if taken.raw_amount > 0.0 { (taken.mitigated_amount / taken.raw_amount) * 100.0 } else { 0.0 }
                ));
            } else if taken.mitigated_amount > 0.0 || resist != 0.0 {
                let effective_resist = resist - pen;
                self.combat_log.push(format!(
                    "  {:?}: {:.0} raw → {:.0} after resist ({:.0}% res{}, {:.0}% reduction)",
                    taken.damage_type,
                    taken.raw_amount,
                    taken.final_amount,
                    resist,
                    if pen > 0.0 { format!(" - {:.0}% pen", pen) } else { String::new() },
                    if taken.raw_amount > 0.0 { (taken.mitigated_amount / taken.raw_amount) * 100.0 } else { 0.0 }
                ));
            }
        }

        // Show evasion cap if triggered
        if result.triggered_evasion_cap {
            self.combat_log.push(format!(
                "  ⚡ Evasion cap triggered! {:.0} damage prevented",
                result.damage_prevented_by_evasion
            ));
        }

        // Show ES absorption if any
        if result.damage_blocked_by_es > 0.0 {
            self.combat_log.push(format!(
                "  🛡 Energy Shield absorbed {:.0} damage",
                result.damage_blocked_by_es
            ));
        }

        // Final result
        self.combat_log.push(format!(
            "  ▶ Enemy takes {:.0} damage → {:.0}/{:.0} HP",
            result.total_damage,
            self.enemy.current_life,
            self.enemy.computed_max_life()
        ));

        // Show mitigation summary
        let total_mitigated = result.damage_reduced_by_armour
            + result.damage_reduced_by_resists
            + result.damage_prevented_by_evasion
            + result.damage_blocked_by_es;
        if total_mitigated > 0.0 {
            self.combat_log.push(format!(
                "  (Total mitigated: {:.0} = {:.0}%)",
                total_mitigated,
                (total_mitigated / (result.total_damage + total_mitigated)) * 100.0
            ));
        }

        // DoT applications
        if !result.dots_applied.is_empty() {
            for dot in &result.dots_applied {
                self.combat_log.push(format!(
                    "  🔥 Applied {} ({:.0} DPS × {:.1}s = {:.0} total)",
                    dot.dot_type,
                    dot.dps(),
                    dot.duration_remaining,
                    dot.total_remaining_damage()
                ));
            }
        }

        // Status effect applications
        if !result.status_effects_applied.is_empty() {
            for status in &result.status_effects_applied {
                let icon = match status.effect_type {
                    stat_core::StatusEffect::Poison => "☠️",
                    stat_core::StatusEffect::Bleed => "🩸",
                    stat_core::StatusEffect::Burn => "🔥",
                    stat_core::StatusEffect::Freeze => "❄️",
                    stat_core::StatusEffect::Chill => "🥶",
                    stat_core::StatusEffect::Static => "⚡",
                    stat_core::StatusEffect::Fear => "😱",
                    stat_core::StatusEffect::Slow => "🐌",
                };
                // Show DoT DPS for damaging statuses
                let info = if status.is_damaging() && status.dot_dps > 0.0 {
                    format!(
                        "  {} Applied {:?} ({:.0} DPS, {:.1}s)",
                        icon, status.effect_type, status.dot_dps, status.duration_remaining
                    )
                } else {
                    format!(
                        "  {} Applied {:?} ({:.1}s, {:.0}% magnitude)",
                        icon, status.effect_type, status.duration_remaining, status.magnitude * 100.0
                    )
                };
                self.combat_log.push(info);
            }
        }

        // Show pending status effects that didn't apply (for info)
        if !packet.status_effects_to_apply.is_empty() {
            let target_max_health = self.enemy.computed_max_life();
            let applied_types: Vec<_> = result.status_effects_applied.iter()
                .map(|s| s.effect_type)
                .collect();

            for pending in &packet.status_effects_to_apply {
                if !applied_types.contains(&pending.effect_type) {
                    let chance = pending.calculate_apply_chance(target_max_health) * 100.0;
                    if chance > 0.0 {
                        self.combat_log.push(format!(
                            "  ⚪ {:?} failed to apply ({:.1}% chance, {:.0} status dmg vs {:.0} HP)",
                            pending.effect_type,
                            chance,
                            pending.status_damage,
                            target_max_health
                        ));
                    }
                }
            }
        }

        if result.is_killing_blow {
            self.combat_log.push("  💀 ENEMY DEFEATED!".to_string());
        }

        self.combat_log.push(String::new()); // Empty line for readability

        // Keep log from growing too large
        while self.combat_log.len() > 200 {
            self.combat_log.remove(0);
        }

        // Auto-scroll to bottom
        self.log_scroll = self.combat_log.len().saturating_sub(15);
    }

    pub fn tick_time(&mut self, seconds: f64) {
        self.time_elapsed += seconds;

        // Process unified effects using the new immutable API
        if !self.enemy.effects.is_empty() {
            let (new_enemy, result) = self.enemy.tick_effects(seconds);
            self.enemy = new_enemy;

            if result.dot_damage > 0.0 {
                self.combat_log.push(format!(
                    "[{:.1}s] Effects deal {:.0} damage ({:.0} HP remaining)",
                    self.time_elapsed, result.dot_damage, result.life_remaining
                ));

                if result.is_dead {
                    self.combat_log.push("  → ENEMY DEFEATED by effects!".to_string());
                }
            }

            for expired_id in &result.expired_effects {
                self.combat_log.push(format!(
                    "[{:.1}s] {} expired",
                    self.time_elapsed, expired_id
                ));
            }
        }

        // === Legacy: Process DoT ticks (for backward compatibility) ===
        if !self.enemy.active_dots.is_empty() {
            let mut configs = std::collections::HashMap::new();
            for dot_type in ["ignite", "poison", "bleed"] {
                if let Some(config) = self.dot_registry.get(dot_type) {
                    configs.insert(dot_type.to_string(), config.clone());
                }
            }

            let result = stat_core::dot::tick::process_dot_tick(
                &mut self.enemy.active_dots,
                seconds,
                false,
                &configs,
            );

            if result.total_damage > 0.0 {
                self.enemy.current_life -= result.total_damage;
                self.combat_log.push(format!(
                    "[{:.1}s] DoT deals {:.0} damage ({:.0} HP remaining)",
                    self.time_elapsed, result.total_damage, self.enemy.current_life
                ));

                if self.enemy.current_life <= 0.0 {
                    self.enemy.current_life = 0.0;
                    self.combat_log.push("  → ENEMY DEFEATED by DoT!".to_string());
                }
            }

            for expired in &result.expired_dots {
                self.combat_log
                    .push(format!("[{:.1}s] {} expired", self.time_elapsed, expired));
            }
        }

        // === Legacy: Process status effect ticks (for backward compatibility) ===
        if !self.enemy.active_status_effects.is_empty() {
            let mut total_status_damage = 0.0;
            let mut damage_by_type: Vec<(StatusEffect, f64)> = Vec::new();
            let mut expired: Vec<StatusEffect> = Vec::new();

            for effect in &mut self.enemy.active_status_effects {
                let damage = effect.tick(seconds);
                if damage > 0.0 {
                    total_status_damage += damage;
                    damage_by_type.push((effect.effect_type, damage));
                }
                if !effect.is_active() {
                    expired.push(effect.effect_type);
                }
            }

            // Apply status effect damage
            if total_status_damage > 0.0 {
                self.enemy.current_life -= total_status_damage;

                // Build damage breakdown
                let breakdown: Vec<String> = damage_by_type
                    .iter()
                    .map(|(effect, dmg)| format!("{:?}: {:.0}", effect, dmg))
                    .collect();

                self.combat_log.push(format!(
                    "[{:.1}s] Status effects deal {:.0} damage [{}] ({:.0} HP)",
                    self.time_elapsed,
                    total_status_damage,
                    breakdown.join(", "),
                    self.enemy.current_life
                ));

                if self.enemy.current_life <= 0.0 {
                    self.enemy.current_life = 0.0;
                    self.combat_log
                        .push("  → ENEMY DEFEATED by status effects!".to_string());
                }
            }

            // Remove expired effects and log
            self.enemy
                .active_status_effects
                .retain(|e| e.is_active());

            for effect_type in expired {
                self.combat_log.push(format!(
                    "[{:.1}s] {:?} expired",
                    self.time_elapsed, effect_type
                ));
            }
        }
    }

    pub fn tick(&mut self, _delta: f64) {
        // Auto-tick for real-time simulation (optional)
    }

    pub fn reset(&mut self) {
        self.enemy.current_life = self.enemy.computed_max_life();
        // Clear unified effects
        self.enemy.clear_effects();
        // Clear legacy effects (for backward compatibility)
        self.enemy.active_dots.clear();
        self.enemy.active_status_effects.clear();
        self.combat_log.clear();
        self.combat_log.push("Combat reset.".to_string());
        self.time_elapsed = 0.0;
        self.log_scroll = 0;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.current_tab = Tab::Help;
        }
    }

    /// Toggle between player and enemy equipment
    pub fn toggle_equip_target(&mut self) {
        self.equip_target = match self.equip_target {
            EquipTarget::Player => EquipTarget::Enemy,
            EquipTarget::Enemy => EquipTarget::Player,
        };
    }

    /// Get the currently selected equipment slot
    pub fn current_slot(&self) -> EquipmentSlot {
        let slots = EquipmentSlot::all();
        slots[self.selected_slot.min(slots.len() - 1)]
    }

    /// Get current equipment map based on target
    pub fn current_equipment(&self) -> &HashMap<EquipmentSlot, Item> {
        match self.equip_target {
            EquipTarget::Player => &self.player_equipment,
            EquipTarget::Enemy => &self.enemy_equipment,
        }
    }

    /// Get mutable current equipment map
    fn current_equipment_mut(&mut self) -> &mut HashMap<EquipmentSlot, Item> {
        match self.equip_target {
            EquipTarget::Player => &mut self.player_equipment,
            EquipTarget::Enemy => &mut self.enemy_equipment,
        }
    }

    /// Filter inventory to items that can go in the selected slot
    pub fn filtered_inventory(&self) -> Vec<&Item> {
        let slot = self.current_slot();
        self.inventory
            .iter()
            .filter(|item| Self::item_fits_slot(item, slot))
            .collect()
    }

    /// Check if an item can fit in a slot
    fn item_fits_slot(item: &Item, slot: EquipmentSlot) -> bool {
        match slot {
            EquipmentSlot::MainHand => matches!(
                item.class,
                ItemClass::OneHandSword
                    | ItemClass::TwoHandSword
                    | ItemClass::OneHandAxe
                    | ItemClass::TwoHandAxe
                    | ItemClass::OneHandMace
                    | ItemClass::TwoHandMace
                    | ItemClass::Bow
                    | ItemClass::Wand
                    | ItemClass::Staff
                    | ItemClass::Dagger
                    | ItemClass::Claw
            ),
            EquipmentSlot::OffHand => matches!(
                item.class,
                ItemClass::Shield
                    | ItemClass::OneHandSword
                    | ItemClass::OneHandAxe
                    | ItemClass::OneHandMace
                    | ItemClass::Wand
                    | ItemClass::Dagger
                    | ItemClass::Claw
            ),
            EquipmentSlot::Helmet => item.class == ItemClass::Helmet,
            EquipmentSlot::BodyArmour => item.class == ItemClass::BodyArmour,
            EquipmentSlot::Gloves => item.class == ItemClass::Gloves,
            EquipmentSlot::Boots => item.class == ItemClass::Boots,
            EquipmentSlot::Ring1 | EquipmentSlot::Ring2 => item.class == ItemClass::Ring,
            EquipmentSlot::Amulet => item.class == ItemClass::Amulet,
            EquipmentSlot::Belt => item.class == ItemClass::Belt,
        }
    }

    /// Equip the selected inventory item
    pub fn equip_selected(&mut self) {
        match self.equip_focus {
            EquipFocus::Slots => {
                // Unequip from current slot
                self.unequip_current_slot();
            }
            EquipFocus::Inventory => {
                let filtered = self.filtered_inventory();
                if let Some(&item) = filtered.get(self.selected_inventory) {
                    let slot = self.current_slot();
                    let item_clone = item.clone();
                    self.current_equipment_mut().insert(slot, item_clone);
                    self.rebuild_current_stats();
                }
            }
        }
    }

    /// Unequip from current slot
    pub fn unequip_current_slot(&mut self) {
        let slot = self.current_slot();
        self.current_equipment_mut().remove(&slot);
        self.rebuild_current_stats();
    }

    /// Rebuild stats for the current target after equipment change
    fn rebuild_current_stats(&mut self) {
        match self.equip_target {
            EquipTarget::Player => self.rebuild_player_stats(),
            EquipTarget::Enemy => self.rebuild_enemy_stats(),
        }
    }

    /// Rebuild player stats from equipment
    fn rebuild_player_stats(&mut self) {
        let base_stats = BaseStatsSource::new(10);
        let mut sources: Vec<Box<dyn StatSource>> = vec![Box::new(base_stats)];
        for (slot, item) in &self.player_equipment {
            sources.push(Box::new(GearSource::new(*slot, item.clone())));
        }
        self.player.rebuild_from_sources(&sources);
    }

    /// Rebuild enemy stats from equipment
    fn rebuild_enemy_stats(&mut self) {
        // Keep base enemy stats, just apply gear
        let mut enemy = StatBlock::new();
        enemy.max_life.base = 500.0;
        enemy.armour.base = 200.0;
        enemy.evasion.base = 5000.0;
        enemy.fire_resistance.base = 25.0;
        enemy.cold_resistance.base = 40.0;
        enemy.lightning_resistance.base = 15.0;
        enemy.chaos_resistance.base = -10.0;

        let mut sources: Vec<Box<dyn StatSource>> = vec![];
        for (slot, item) in &self.enemy_equipment {
            sources.push(Box::new(GearSource::new(*slot, item.clone())));
        }
        enemy.rebuild_from_sources(&sources);
        enemy.current_life = enemy.computed_max_life();
        self.enemy = enemy;
    }

    /// Calculate stat difference if an item were equipped
    pub fn preview_equip_diff(&self, item: &Item) -> Vec<(String, f64, f64)> {
        let slot = self.current_slot();

        // Create temporary equipment with the new item
        let mut temp_equipment = self.current_equipment().clone();
        temp_equipment.insert(slot, item.clone());

        // Calculate new stats
        let new_stats = match self.equip_target {
            EquipTarget::Player => {
                let base_stats = BaseStatsSource::new(10);
                let mut sources: Vec<Box<dyn StatSource>> = vec![Box::new(base_stats)];
                for (s, i) in &temp_equipment {
                    sources.push(Box::new(GearSource::new(*s, i.clone())));
                }
                let mut stats = StatBlock::new();
                stats.rebuild_from_sources(&sources);
                stats
            }
            EquipTarget::Enemy => {
                let mut stats = StatBlock::new();
                stats.max_life.base = 500.0;
                stats.armour.base = 200.0;
                stats.evasion.base = 5000.0;
                stats.fire_resistance.base = 25.0;
                stats.cold_resistance.base = 40.0;
                stats.lightning_resistance.base = 15.0;
                stats.chaos_resistance.base = -10.0;

                let mut sources: Vec<Box<dyn StatSource>> = vec![];
                for (s, i) in &temp_equipment {
                    sources.push(Box::new(GearSource::new(*s, i.clone())));
                }
                stats.rebuild_from_sources(&sources);
                stats
            }
        };

        let current_stats = match self.equip_target {
            EquipTarget::Player => &self.player,
            EquipTarget::Enemy => &self.enemy,
        };

        // Compare key stats
        let mut diffs = vec![];

        let check = |name: &str, old: f64, new: f64| -> Option<(String, f64, f64)> {
            if (new - old).abs() > 0.01 {
                Some((name.to_string(), old, new))
            } else {
                None
            }
        };

        if let Some(d) = check("Max Life", current_stats.computed_max_life(), new_stats.computed_max_life()) {
            diffs.push(d);
        }
        if let Some(d) = check("Max Mana", current_stats.max_mana.compute(), new_stats.max_mana.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Armour", current_stats.armour.compute(), new_stats.armour.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Evasion", current_stats.evasion.compute(), new_stats.evasion.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Fire Res", current_stats.fire_resistance.compute(), new_stats.fire_resistance.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Cold Res", current_stats.cold_resistance.compute(), new_stats.cold_resistance.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Light Res", current_stats.lightning_resistance.compute(), new_stats.lightning_resistance.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Phys Dmg", current_stats.global_physical_damage.compute(), new_stats.global_physical_damage.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Fire Dmg", current_stats.global_fire_damage.compute(), new_stats.global_fire_damage.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Atk Speed", current_stats.attack_speed.compute(), new_stats.attack_speed.compute()) {
            diffs.push(d);
        }
        if let Some(d) = check("Crit Chance", current_stats.critical_chance.compute(), new_stats.critical_chance.compute()) {
            diffs.push(d);
        }

        diffs
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_items() {
        let content = std::fs::read_to_string("data/items.json")
            .expect("Failed to read items.json");
        
        let result: Result<ItemsFile, _> = serde_json::from_str(&content);
        match result {
            Ok(items) => {
                println!("Loaded {} items", items.items.len());
                assert!(items.items.len() > 0, "Should have items");
            }
            Err(e) => {
                panic!("Failed to parse items.json: {}", e);
            }
        }
    }
}
