//! Example Game - A minimal TUI game demonstrating stat_core and loot_core integration
//!
//! This game shows:
//! - Generating enemies with random gear (loot_core)
//! - Combat using damage calculations (stat_core)
//! - Loot drops (items + currency)
//! - Inventory management and crafting
//! - Equipment swapping

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use loot_core::currency::apply_currency;
use loot_core::{Config, Generator, Item};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use stat_core::{
    combat::resolve_damage,
    damage::{calculate_damage, calculate_skill_dps, DamagePacketGenerator},
    dot::DotRegistry,
    source::{BaseStatsSource, GearSource, StatSource},
    stat_block::StatBlock,
    DamageType, EquipmentSlot, ItemClass, Rarity,
};
use loot_core::item::{Defenses, DamageValue, WeaponDamage};
use std::collections::HashMap;
use std::io;
use std::path::Path;

/// Current screen in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Combat,
    Inventory,
    Equipment,
}

/// Main game state
struct GameState {
    // Player
    player: StatBlock,
    player_equipment: HashMap<EquipmentSlot, Item>,
    inventory: Vec<Item>,
    currency: HashMap<String, u32>,
    skills: Vec<DamagePacketGenerator>,
    selected_skill: usize,

    // Enemy
    enemy: StatBlock,
    enemy_equipment: HashMap<EquipmentSlot, Item>,
    enemy_max_hp: f64,
    enemy_skill: DamagePacketGenerator,

    // Game
    time: f64,
    kills: u32,
    messages: Vec<String>,

    // UI
    screen: Screen,
    selected_index: usize,
    equipment_slot_index: usize,

    // Systems
    generator: Generator,
    dot_registry: DotRegistry,
    rng: ChaCha8Rng,
}

/// Create the 4 player skills
fn create_skills() -> Vec<DamagePacketGenerator> {
    use stat_core::damage::BaseDamage;
    use stat_core::types::SkillTag;

    vec![
        // 1. Basic Attack - simple weapon attack
        DamagePacketGenerator {
            id: "basic_attack".to_string(),
            name: "Basic Attack".to_string(),
            base_damages: vec![],
            weapon_effectiveness: 1.0,
            damage_effectiveness: 1.0,
            attack_speed_modifier: 1.0,
            base_crit_chance: 0.0,
            crit_multiplier_bonus: 0.0,
            tags: vec![SkillTag::Attack, SkillTag::Melee],
            hits_per_attack: 1,
            ..Default::default()
        },
        // 2. Heavy Strike - 150% damage, 20% slower
        DamagePacketGenerator {
            id: "heavy_strike".to_string(),
            name: "Heavy Strike".to_string(),
            base_damages: vec![],
            weapon_effectiveness: 1.5,
            damage_effectiveness: 1.0,
            attack_speed_modifier: 0.8,
            base_crit_chance: 0.0,
            crit_multiplier_bonus: 0.0,
            tags: vec![SkillTag::Attack, SkillTag::Melee],
            hits_per_attack: 1,
            ..Default::default()
        },
        // 3. Double Strike - hits twice at 70% damage each
        DamagePacketGenerator {
            id: "double_strike".to_string(),
            name: "Double Strike".to_string(),
            base_damages: vec![],
            weapon_effectiveness: 0.7,
            damage_effectiveness: 1.0,
            attack_speed_modifier: 0.9,
            base_crit_chance: 0.0,
            crit_multiplier_bonus: 0.0,
            tags: vec![SkillTag::Attack, SkillTag::Melee],
            hits_per_attack: 2,
            ..Default::default()
        },
        // 4. Elemental Strike - adds flat fire damage, +50% crit multiplier
        DamagePacketGenerator {
            id: "elemental_strike".to_string(),
            name: "Elemental Strike".to_string(),
            base_damages: vec![BaseDamage::new(DamageType::Fire, 10.0, 20.0)],
            weapon_effectiveness: 1.0,
            damage_effectiveness: 1.0,
            attack_speed_modifier: 1.0,
            base_crit_chance: 5.0,
            crit_multiplier_bonus: 0.5,
            tags: vec![SkillTag::Attack, SkillTag::Melee, SkillTag::Fire],
            hits_per_attack: 1,
            ..Default::default()
        },
    ]
}

impl GameState {
    /// Create a fallback weapon when no config is available
    fn create_fallback_weapon() -> Item {
        Item {
            base_type_id: "iron_sword".to_string(),
            name: "Iron Sword".to_string(),
            base_name: "Iron Sword".to_string(),
            class: ItemClass::OneHandSword,
            rarity: Rarity::Normal,
            tags: vec!["melee".to_string(), "physical".to_string()],
            requirements: loot_core::types::Requirements::default(),
            implicit: None,
            prefixes: vec![],
            suffixes: vec![],
            defenses: Defenses::default(),
            damage: Some(WeaponDamage {
                damages: vec![DamageValue {
                    damage_type: DamageType::Physical,
                    min: 8,
                    max: 15,
                }],
                attack_speed: 1.3,
                critical_chance: 5.0,
                spell_efficiency: 0.0,
            }),
        }
    }

    fn new() -> Self {
        // Load config from example_game's config directory
        let config_paths = ["example_game/config", "../example_game/config"];

        let (config, config_path) = config_paths
            .iter()
            .find_map(|p| {
                let path = Path::new(p);
                if path.exists() {
                    match Config::load_from_dir(path) {
                        Ok(cfg) => Some((cfg, p.to_string())),
                        Err(e) => {
                            eprintln!("Error loading config from '{}': {}", p, e);
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                eprintln!("ERROR: Could not find config directory.");
                eprintln!("Looked in:");
                for p in &config_paths {
                    eprintln!("  - {}", p);
                }
                eprintln!();
                eprintln!("Make sure you run the game from the stat_manager directory:");
                eprintln!("  cd /path/to/stat_manager");
                eprintln!("  cargo run -p example_game");
                std::process::exit(1);
            });

        let generator = Generator::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let dot_registry = DotRegistry::with_defaults();

        // Create player with starting weapon
        let mut player_equipment = HashMap::new();
        let inventory = Vec::new();

        // Try to generate a starting weapon, use fallback if not available
        let weapon = generator
            .generate_normal("iron_sword", &mut rng)
            .map(|mut w| {
                generator.make_magic(&mut w, &mut rng);
                w
            })
            .unwrap_or_else(Self::create_fallback_weapon);
        player_equipment.insert(EquipmentSlot::MainHand, weapon);

        // Build player stats
        let mut player = StatBlock::new();
        let base_stats = BaseStatsSource::new(10);
        let mut sources: Vec<Box<dyn StatSource>> = vec![Box::new(base_stats)];
        for (slot, item) in &player_equipment {
            sources.push(Box::new(GearSource::new(*slot, item.clone())));
        }
        player.rebuild_from_sources(&sources);
        player.current_life = player.computed_max_life();

        // Generate first enemy
        let enemy = StatBlock::new();
        let enemy_equipment = HashMap::new();

        let welcome_msg = format!("Welcome! Config: {}", config_path);
        let skills = create_skills();

        let mut state = GameState {
            player,
            player_equipment,
            inventory,
            currency: HashMap::new(),
            skills,
            selected_skill: 0,
            enemy,
            enemy_equipment,
            enemy_max_hp: 100.0,
            enemy_skill: DamagePacketGenerator::basic_attack(),
            time: 0.0,
            kills: 0,
            messages: vec![welcome_msg],
            screen: Screen::Combat,
            selected_index: 0,
            equipment_slot_index: 0,
            generator,
            dot_registry,
            rng,
        };

        state.spawn_enemy();
        state
    }

    fn spawn_enemy(&mut self) {
        // Pick a random weapon for the enemy
        let weapon_bases: Vec<&String> = self.generator.base_type_ids();
        let weapon_bases: Vec<_> = weapon_bases
            .into_iter()
            .filter(|id| {
                self.generator
                    .get_base_type(id)
                    .map(|bt| bt.class.is_weapon())
                    .unwrap_or(false)
            })
            .collect();

        let mut enemy = StatBlock::new();
        let mut enemy_equipment = HashMap::new();

        if !weapon_bases.is_empty() {
            let base_id = weapon_bases.choose(&mut self.rng).unwrap();
            if let Some(mut weapon) = self.generator.generate_normal(base_id, &mut self.rng) {
                // 50% chance to make magic
                if self.rng.gen_bool(0.5) {
                    self.generator.make_magic(&mut weapon, &mut self.rng);
                }
                enemy_equipment.insert(EquipmentSlot::MainHand, weapon);
            }
        }

        // Build enemy stats from gear
        let mut sources: Vec<Box<dyn StatSource>> = vec![];
        for (slot, item) in &enemy_equipment {
            sources.push(Box::new(GearSource::new(*slot, item.clone())));
        }

        // Set base enemy stats (scales slightly with kills)
        let scale = 1.0 + (self.kills as f64 * 0.1);
        enemy.max_life.base = 100.0 * scale;
        enemy.armour.base = 50.0 * scale;
        enemy.fire_resistance.base = 10.0;

        enemy.rebuild_from_sources(&sources);
        enemy.current_life = enemy.computed_max_life();

        self.enemy_max_hp = enemy.computed_max_life();
        self.enemy = enemy;
        self.enemy_equipment = enemy_equipment;

        // Assign a random skill to the enemy
        let skill_idx = self.rng.gen_range(0..self.skills.len());
        self.enemy_skill = self.skills[skill_idx].clone();
    }

    fn attack(&mut self) {
        if !self.enemy.is_alive() {
            self.messages.push("Enemy already dead!".to_string());
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

        let result = resolve_damage(&mut self.enemy, &packet, &self.dot_registry);

        // Advance time by attack interval (modified by skill)
        let attack_speed = self.player.attack_speed.compute() * self.player.weapon_attack_speed;
        let effective_speed = attack_speed * skill.attack_speed_modifier;
        self.time += 1.0 / effective_speed.max(0.1);

        // Log the attack
        let crit_str = if packet.is_critical { " CRIT!" } else { "" };
        let hits_str = if skill.hits_per_attack > 1 {
            format!(" ({}x)", skill.hits_per_attack)
        } else {
            String::new()
        };
        self.messages.push(format!(
            "{}: {:.0} damage{}{}",
            skill.name, result.total_damage, hits_str, crit_str
        ));

        // Check for kill
        if !self.enemy.is_alive() {
            self.kills += 1;
            self.messages
                .push(format!("Enemy #{} defeated!", self.kills));
            self.roll_loot();
            self.spawn_enemy();
        }

        // Keep message log reasonable
        while self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }

    fn roll_loot(&mut self) {
        // 30% chance for item drop
        if self.rng.gen_bool(0.3) {
            let base_ids: Vec<&String> = self.generator.base_type_ids();
            if let Some(base_id) = base_ids.choose(&mut self.rng) {
                if let Some(item) = self.generator.generate_normal(base_id, &mut self.rng) {
                    self.messages.push(format!("Dropped: {}", item.name));
                    self.inventory.push(item);
                }
            }
        }

        // 50% chance for currency drop
        if self.rng.gen_bool(0.5) {
            // Use actual currency IDs from config
            let currencies = [
                "transmute", "augment", "alchemy", "chaos", "scour", "regal", "exalt",
            ];
            if let Some(currency_id) = currencies.choose(&mut self.rng) {
                *self.currency.entry(currency_id.to_string()).or_insert(0) += 1;
                self.messages.push(format!("Dropped: {}", currency_id));
            }
        }
    }

    fn rebuild_player_stats(&mut self) {
        let base_stats = BaseStatsSource::new(10);
        let mut sources: Vec<Box<dyn StatSource>> = vec![Box::new(base_stats)];
        for (slot, item) in &self.player_equipment {
            sources.push(Box::new(GearSource::new(*slot, item.clone())));
        }
        self.player.rebuild_from_sources(&sources);
        self.player.current_life = self
            .player
            .current_life
            .min(self.player.computed_max_life());
    }

    fn apply_craft(&mut self, currency_id: &str) -> Result<(), String> {
        let count = self.currency.get(currency_id).copied().unwrap_or(0);
        if count == 0 {
            return Err(format!("No {} available", currency_id));
        }

        if self.selected_index >= self.inventory.len() {
            return Err("No item selected".to_string());
        }

        let currency_config = self
            .generator
            .config()
            .currencies
            .get(currency_id)
            .ok_or_else(|| format!("Unknown currency: {}", currency_id))?
            .clone();

        let item = &mut self.inventory[self.selected_index];
        match apply_currency(&self.generator, item, &currency_config, &mut self.rng) {
            Ok(()) => {
                *self.currency.get_mut(currency_id).unwrap() -= 1;
                Ok(())
            }
            Err(e) => Err(format!("{}", e)),
        }
    }

    fn equip_selected(&mut self) {
        if self.selected_index >= self.inventory.len() {
            return;
        }

        let item = &self.inventory[self.selected_index];
        if let Some(slot) = item_to_slot(item) {
            let item = self.inventory.remove(self.selected_index);

            // Unequip current item if any
            if let Some(old_item) = self.player_equipment.remove(&slot) {
                self.inventory.push(old_item);
            }

            self.player_equipment.insert(slot, item);
            self.rebuild_player_stats();
            self.messages.push("Item equipped!".to_string());

            // Adjust selection
            if self.selected_index >= self.inventory.len() && self.selected_index > 0 {
                self.selected_index -= 1;
            }
        } else {
            self.messages.push("Cannot equip this item.".to_string());
        }
    }

    fn unequip_selected(&mut self) {
        let slots = EquipmentSlot::all();
        if self.equipment_slot_index >= slots.len() {
            return;
        }

        let slot = slots[self.equipment_slot_index];
        if let Some(item) = self.player_equipment.remove(&slot) {
            self.inventory.push(item);
            self.rebuild_player_stats();
            self.messages.push("Item unequipped!".to_string());
        }
    }

    fn calculate_dps(&self) -> f64 {
        let skill = &self.skills[self.selected_skill];
        calculate_skill_dps(&self.player, skill, &self.dot_registry)
    }
}

fn item_to_slot(item: &Item) -> Option<EquipmentSlot> {
    match item.class {
        ItemClass::OneHandSword
        | ItemClass::OneHandAxe
        | ItemClass::OneHandMace
        | ItemClass::TwoHandSword
        | ItemClass::TwoHandAxe
        | ItemClass::TwoHandMace
        | ItemClass::Bow
        | ItemClass::Staff
        | ItemClass::Wand
        | ItemClass::Dagger
        | ItemClass::Claw => Some(EquipmentSlot::MainHand),
        ItemClass::Shield => Some(EquipmentSlot::OffHand),
        ItemClass::Helmet => Some(EquipmentSlot::Helmet),
        ItemClass::BodyArmour => Some(EquipmentSlot::BodyArmour),
        ItemClass::Gloves => Some(EquipmentSlot::Gloves),
        ItemClass::Boots => Some(EquipmentSlot::Boots),
        ItemClass::Ring => Some(EquipmentSlot::Ring1),
        ItemClass::Amulet => Some(EquipmentSlot::Amulet),
        ItemClass::Belt => Some(EquipmentSlot::Belt),
    }
}

fn draw(f: &mut Frame, state: &GameState) {
    match state.screen {
        Screen::Combat => draw_combat(f, state),
        Screen::Inventory => draw_inventory(f, state),
        Screen::Equipment => draw_equipment(f, state),
    }
}

fn draw_combat(f: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(4), // Skills
            Constraint::Length(5), // Messages
            Constraint::Length(3), // Controls
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Example Game - Kill enemies, collect loot, get stronger")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content: Player vs Enemy
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Player stats
    let player_hp = state.player.current_life;
    let player_max_hp = state.player.computed_max_life();
    let dps = state.calculate_dps();
    let weapon_name = state
        .player_equipment
        .get(&EquipmentSlot::MainHand)
        .map(|i| i.name.as_str())
        .unwrap_or("Unarmed");

    let player_text = vec![
        Line::from(vec![Span::styled(
            "Player",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!("HP: {:.0}/{:.0}", player_hp, player_max_hp)),
        Line::from(format!("DPS: {:.1}", dps)),
        Line::from(format!("Weapon: {}", weapon_name)),
        Line::from(""),
        Line::from(format!("Time: {:.1}s", state.time)),
        Line::from(format!("Kills: {}", state.kills)),
        Line::from(format!("Gold: {}", state.currency.values().sum::<u32>())),
    ];
    let player_widget =
        Paragraph::new(player_text).block(Block::default().borders(Borders::ALL).title("Player"));
    f.render_widget(player_widget, main_chunks[0]);

    // Enemy stats
    let enemy_hp = state.enemy.current_life;
    let enemy_max_hp = state.enemy_max_hp;
    let hp_pct = (enemy_hp / enemy_max_hp * 10.0) as usize;
    let hp_bar = format!(
        "[{}{}]",
        "█".repeat(hp_pct.min(10)),
        "░".repeat(10 - hp_pct.min(10))
    );
    let enemy_weapon = state
        .enemy_equipment
        .get(&EquipmentSlot::MainHand)
        .map(|i| i.name.as_str())
        .unwrap_or("Unarmed");

    let enemy_text = vec![
        Line::from(vec![Span::styled(
            format!("Enemy #{}", state.kills + 1),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!(
            "HP: {} {:.0}/{:.0}",
            hp_bar, enemy_hp, enemy_max_hp
        )),
        Line::from(format!("Weapon: {}", enemy_weapon)),
        Line::from(format!("Skill: {}", state.enemy_skill.name)),
    ];
    let enemy_widget =
        Paragraph::new(enemy_text).block(Block::default().borders(Borders::ALL).title("Enemy"));
    f.render_widget(enemy_widget, main_chunks[1]);

    // Skills
    let skill_spans: Vec<Span> = state
        .skills
        .iter()
        .enumerate()
        .flat_map(|(i, skill)| {
            let key = format!("[{}] ", i + 1);
            let style = if i == state.selected_skill {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            vec![
                Span::styled(key, style),
                Span::styled(skill.name.clone(), style),
                Span::raw("    "),
            ]
        })
        .collect();
    let skills_line = Line::from(skill_spans);
    let selected_skill = &state.skills[state.selected_skill];
    let skill_desc = match state.selected_skill {
        0 => "100% weapon damage",
        1 => "150% damage, 20% slower",
        2 => "70% damage x2 hits",
        3 => "+10-20 fire, +5% crit, +50% crit multi",
        _ => "",
    };
    let skills_widget = Paragraph::new(vec![skills_line, Line::from(format!("{}: {}", selected_skill.name, skill_desc))])
        .block(Block::default().borders(Borders::ALL).title("Skills (1-4 to select)"));
    f.render_widget(skills_widget, chunks[2]);

    // Messages
    let messages: Vec<ListItem> = state
        .messages
        .iter()
        .map(|m| ListItem::new(m.as_str()))
        .collect();
    let messages_widget =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Combat Log"));
    f.render_widget(messages_widget, chunks[3]);

    // Controls
    let controls = Paragraph::new("[SPACE] Attack    [I] Inventory    [E] Equipment    [Q] Quit")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[4]);
}

fn draw_inventory(f: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    // Inventory list
    let items: Vec<ListItem> = state
        .inventory
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let rarity_color = match item.rarity {
                Rarity::Normal => Color::White,
                Rarity::Magic => Color::Blue,
                Rarity::Rare => Color::Yellow,
                Rarity::Unique => Color::Magenta,
            };
            let prefix = if i == state.selected_index {
                "> "
            } else {
                "  "
            };
            let style = if i == state.selected_index {
                Style::default()
                    .fg(rarity_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(rarity_color)
            };
            ListItem::new(format!("{}{} ({:?})", prefix, item.name, item.rarity)).style(style)
        })
        .collect();

    let inv_title = format!("Inventory ({} items)", state.inventory.len());
    let inv_widget =
        List::new(items).block(Block::default().borders(Borders::ALL).title(inv_title));
    f.render_widget(inv_widget, chunks[0]);

    // Right side: item details + currency
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(16)])
        .split(chunks[1]);

    // Item details
    let detail_text = if let Some(item) = state.inventory.get(state.selected_index) {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                item.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(format!("{:?}", item.class)),
            Line::from(""),
        ];

        // Show damage if weapon
        if let Some(ref dmg) = item.damage {
            for d in &dmg.damages {
                lines.push(Line::from(format!(
                    "{:?} Damage: {}-{}",
                    d.damage_type, d.min, d.max
                )));
            }
            lines.push(Line::from(format!("Attack Speed: {:.2}", dmg.attack_speed)));
        }

        // Show defenses if armor
        if item.defenses.has_any() {
            if let Some(arm) = item.defenses.armour {
                lines.push(Line::from(format!("Armour: {}", arm)));
            }
            if let Some(ev) = item.defenses.evasion {
                lines.push(Line::from(format!("Evasion: {}", ev)));
            }
        }

        // Show implicit
        if let Some(ref imp) = item.implicit {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                imp.display(),
                Style::default().fg(Color::Cyan),
            )]));
        }

        // Show mods
        if !item.prefixes.is_empty() || !item.suffixes.is_empty() {
            lines.push(Line::from(""));
            for m in &item.prefixes {
                lines.push(Line::from(vec![Span::styled(
                    m.display(),
                    Style::default().fg(Color::Blue),
                )]));
            }
            for m in &item.suffixes {
                lines.push(Line::from(vec![Span::styled(
                    m.display(),
                    Style::default().fg(Color::Blue),
                )]));
            }
        }

        lines
    } else {
        vec![Line::from("No item selected")]
    };

    let detail_widget =
        Paragraph::new(detail_text).block(Block::default().borders(Borders::ALL).title("Details"));
    f.render_widget(detail_widget, right_chunks[0]);

    // Currency and controls
    let mut currency_lines = vec![Line::from(vec![Span::styled(
        "Currency:",
        Style::default().add_modifier(Modifier::BOLD),
    )])];
    for (id, count) in &state.currency {
        currency_lines.push(Line::from(format!("  {}: {}", id, count)));
    }
    currency_lines.push(Line::from(""));
    currency_lines.push(Line::from(vec![Span::styled(
        "Crafting Keys:",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    currency_lines.push(Line::from("[T] Transmute  Normal->Magic"));
    currency_lines.push(Line::from("[R] Alchemy    Normal->Rare"));
    currency_lines.push(Line::from("[G] Regal      Magic->Rare"));
    currency_lines.push(Line::from("[A] Augment    Magic +affix"));
    currency_lines.push(Line::from("[X] Exalt      Rare +affix"));
    currency_lines.push(Line::from("[C] Chaos      Rare reroll"));
    currency_lines.push(Line::from("[S] Scour      ->Normal"));
    currency_lines.push(Line::from(""));
    currency_lines.push(Line::from("[Enter] Equip  [Esc] Back"));

    let currency_widget = Paragraph::new(currency_lines)
        .block(Block::default().borders(Borders::ALL).title("Actions"));
    f.render_widget(currency_widget, right_chunks[1]);
}

fn draw_equipment(f: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    // Equipment slots
    let slots = EquipmentSlot::all();
    let items: Vec<ListItem> = slots
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            let item_name = state
                .player_equipment
                .get(slot)
                .map(|item| {
                    let rarity_indicator = match item.rarity {
                        Rarity::Normal => "",
                        Rarity::Magic => " (Magic)",
                        Rarity::Rare => " (Rare)",
                        Rarity::Unique => " (Unique)",
                    };
                    format!("{}{}", item.name, rarity_indicator)
                })
                .unwrap_or_else(|| "[Empty]".to_string());

            let prefix = if i == state.equipment_slot_index {
                "> "
            } else {
                "  "
            };
            let style = if i == state.equipment_slot_index {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("{}{:?}: {}", prefix, slot, item_name)).style(style)
        })
        .collect();

    let equip_widget =
        List::new(items).block(Block::default().borders(Borders::ALL).title("Equipment"));
    f.render_widget(equip_widget, chunks[0]);

    // Item details for selected slot
    let slots = EquipmentSlot::all();
    let detail_text = if let Some(slot) = slots.get(state.equipment_slot_index) {
        if let Some(item) = state.player_equipment.get(slot) {
            let mut lines = vec![
                Line::from(vec![Span::styled(
                    item.name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from(format!("{:?} ({:?})", item.class, item.rarity)),
                Line::from(""),
            ];

            // Show damage if weapon
            if let Some(ref dmg) = item.damage {
                for d in &dmg.damages {
                    lines.push(Line::from(format!(
                        "{:?} Damage: {}-{}",
                        d.damage_type, d.min, d.max
                    )));
                }
                lines.push(Line::from(format!("Attack Speed: {:.2}", dmg.attack_speed)));
            }

            // Show defenses
            if item.defenses.has_any() {
                if let Some(arm) = item.defenses.armour {
                    lines.push(Line::from(format!("Armour: {}", arm)));
                }
            }

            // Show mods
            if let Some(ref imp) = item.implicit {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    imp.display(),
                    Style::default().fg(Color::Cyan),
                )]));
            }
            for m in &item.prefixes {
                lines.push(Line::from(vec![Span::styled(
                    m.display(),
                    Style::default().fg(Color::Blue),
                )]));
            }
            for m in &item.suffixes {
                lines.push(Line::from(vec![Span::styled(
                    m.display(),
                    Style::default().fg(Color::Blue),
                )]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from("[U] Unequip  [Esc] Back"));

            lines
        } else {
            vec![
                Line::from("Empty slot"),
                Line::from(""),
                Line::from("[Esc] Back"),
            ]
        }
    } else {
        vec![Line::from("No slot selected")]
    };

    let detail_widget =
        Paragraph::new(detail_text).block(Block::default().borders(Borders::ALL).title("Details"));
    f.render_widget(detail_widget, chunks[1]);
}

fn main() -> io::Result<()> {
    // Create game state before terminal setup so panics are visible
    let mut state = GameState::new();

    // Setup terminal
    if let Err(e) = enable_raw_mode() {
        eprintln!("Error: Cannot enable raw mode: {}", e);
        eprintln!("This game requires a terminal. Run it directly, not piped or in a non-TTY context.");
        return Err(e);
    }

    let mut stdout = io::stdout();
    if let Err(e) = execute!(stdout, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        eprintln!("Error: Cannot enter alternate screen: {}", e);
        return Err(e);
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            let _ = disable_raw_mode();
            eprintln!("Error: Cannot create terminal: {}", e);
            return Err(e);
        }
    };

    // Main loop
    loop {
        terminal.draw(|f| draw(f, &state))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match state.screen {
                    Screen::Combat => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => state.attack(),
                        KeyCode::Char('1') => state.selected_skill = 0,
                        KeyCode::Char('2') => state.selected_skill = 1,
                        KeyCode::Char('3') => state.selected_skill = 2,
                        KeyCode::Char('4') => state.selected_skill = 3,
                        KeyCode::Char('i') => {
                            state.screen = Screen::Inventory;
                            state.selected_index = 0;
                        }
                        KeyCode::Char('e') => {
                            state.screen = Screen::Equipment;
                            state.equipment_slot_index = 0;
                        }
                        _ => {}
                    },
                    Screen::Inventory => match key.code {
                        KeyCode::Esc => state.screen = Screen::Combat,
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 {
                                state.selected_index -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index < state.inventory.len().saturating_sub(1) {
                                state.selected_index += 1;
                            }
                        }
                        KeyCode::Enter => state.equip_selected(),
                        KeyCode::Char('t') => match state.apply_craft("transmute") {
                            Ok(()) => state.messages.push("Transmuted to Magic!".to_string()),
                            Err(e) => state.messages.push(e),
                        },
                        KeyCode::Char('a') => match state.apply_craft("augment") {
                            Ok(()) => state.messages.push("Augmented!".to_string()),
                            Err(e) => state.messages.push(e),
                        },
                        KeyCode::Char('r') => match state.apply_craft("alchemy") {
                            Ok(()) => state.messages.push("Alchemized to Rare!".to_string()),
                            Err(e) => state.messages.push(e),
                        },
                        KeyCode::Char('c') => match state.apply_craft("chaos") {
                            Ok(()) => state.messages.push("Chaos rerolled!".to_string()),
                            Err(e) => state.messages.push(e),
                        },
                        KeyCode::Char('s') => match state.apply_craft("scour") {
                            Ok(()) => state.messages.push("Scoured to Normal!".to_string()),
                            Err(e) => state.messages.push(e),
                        },
                        KeyCode::Char('g') => match state.apply_craft("regal") {
                            Ok(()) => state.messages.push("Regaled to Rare!".to_string()),
                            Err(e) => state.messages.push(e),
                        },
                        KeyCode::Char('x') => match state.apply_craft("exalt") {
                            Ok(()) => state.messages.push("Exalted!".to_string()),
                            Err(e) => state.messages.push(e),
                        },
                        _ => {}
                    },
                    Screen::Equipment => match key.code {
                        KeyCode::Esc => state.screen = Screen::Combat,
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.equipment_slot_index > 0 {
                                state.equipment_slot_index -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let max = EquipmentSlot::all().len();
                            if state.equipment_slot_index < max - 1 {
                                state.equipment_slot_index += 1;
                            }
                        }
                        KeyCode::Char('u') => state.unequip_selected(),
                        _ => {}
                    },
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
