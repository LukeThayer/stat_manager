//! Integration test: Generate item -> Apply currency -> Equip -> Attack
//!
//! This test validates the full flow from loot generation to combat resolution.

use loot_core::{Config, Generator, Item};
use loot_core::currency::apply_currency;
use loot_core::types::{Rarity, DamageType};
use stat_core::{
    combat::resolve_damage,
    damage::{calculate_damage, DamagePacketGenerator},
    source::{BaseStatsSource, GearSource, StatSource},
    stat_block::StatBlock,
    types::EquipmentSlot,
};
use std::path::Path;

/// Helper to print a separator
fn separator(title: &str) {
    println!("\n{}", "=".repeat(60));
    println!("  {}", title);
    println!("{}\n", "=".repeat(60));
}

/// Helper to print item details
fn print_item(item: &Item) {
    println!("  Name: {} ({})", item.name, item.base_name);
    println!("  Rarity: {:?}", item.rarity);
    println!("  Class: {:?}", item.class);

    if let Some(ref damage) = item.damage {
        println!("  Weapon Damage:");
        for dmg in &damage.damages {
            println!("    - {:?}: {}-{}", dmg.damage_type, dmg.min, dmg.max);
        }
        println!("    - Attack Speed: {:.2}", damage.attack_speed);
        println!("    - Crit Chance: {:.1}%", damage.critical_chance);
    }

    if let Some(ref implicit) = item.implicit {
        println!("  Implicit: {} ({:?}: {})", implicit.name, implicit.stat, implicit.value);
    }

    if !item.prefixes.is_empty() {
        println!("  Prefixes:");
        for prefix in &item.prefixes {
            println!("    - {} ({:?}: {} [T{}])", prefix.name, prefix.stat, prefix.value, prefix.tier);
        }
    }

    if !item.suffixes.is_empty() {
        println!("  Suffixes:");
        for suffix in &item.suffixes {
            println!("    - {} ({:?}: {} [T{}])", suffix.name, suffix.stat, suffix.value, suffix.tier);
        }
    }
}

/// Helper to print stat block summary
fn print_stats(name: &str, stats: &StatBlock) {
    println!("  {} Stats:", name);
    println!("    Life: {:.0}/{:.0}", stats.current_life, stats.computed_max_life());
    println!("    Mana: {:.0}/{:.0}", stats.current_mana, stats.computed_max_mana());

    let (phys_min, phys_max) = stats.weapon_damage(DamageType::Physical);
    if phys_max > 0.0 {
        println!("    Weapon Physical: {:.0}-{:.0}", phys_min, phys_max);
    }

    let (fire_min, fire_max) = stats.weapon_damage(DamageType::Fire);
    if fire_max > 0.0 {
        println!("    Weapon Fire: {:.0}-{:.0}", fire_min, fire_max);
    }

    println!("    Attack Speed: {:.2}", stats.computed_attack_speed());
    println!("    Crit Chance: {:.1}%", stats.weapon_crit_chance);
    println!("    Armour: {:.0}", stats.armour.compute());
    println!("    Fire Resist: {:.0}%", stats.fire_resistance.compute());
}

#[test]
fn test_full_item_to_combat_flow() {
    separator("INTEGRATION TEST: Item Generation -> Currency -> Equip -> Combat");

    // =========================================================================
    // STEP 1: Load the loot generator config
    // =========================================================================
    separator("STEP 1: Loading Loot Generator Config");

    let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../loot_generator/config");

    println!("  Config path: {:?}", config_path);

    let config = Config::load_from_dir(&config_path)
        .expect("Failed to load loot generator config");

    println!("  Loaded {} base types", config.base_types.len());
    println!("  Loaded {} affixes", config.affixes.len());
    println!("  Loaded {} currencies", config.currencies.len());

    let generator = Generator::new(config);

    // =========================================================================
    // STEP 2: Generate a normal weapon
    // =========================================================================
    separator("STEP 2: Generating Normal Weapon");

    let seed: u64 = 42;
    let mut rng = Generator::make_rng(seed);

    let mut item = generator
        .generate("iron_sword", seed)
        .expect("Failed to generate iron_sword");

    println!("  Generated normal item (seed: {}):", seed);
    print_item(&item);
    assert_eq!(item.rarity, Rarity::Normal);
    assert!(item.prefixes.is_empty());
    assert!(item.suffixes.is_empty());

    // =========================================================================
    // STEP 3: Apply Orb of Transmutation (Normal -> Magic)
    // =========================================================================
    separator("STEP 3: Applying Orb of Transmutation");

    let transmute = generator.config().currencies.get("transmute")
        .expect("Transmute currency not found");

    println!("  Using: {} - {}", transmute.name, transmute.description);

    let mut currency_rng = Generator::make_rng(123);
    apply_currency(&generator, &mut item, transmute, &mut currency_rng)
        .expect("Failed to apply transmute");

    println!("\n  After transmutation:");
    print_item(&item);
    assert_eq!(item.rarity, Rarity::Magic);
    assert!(item.prefixes.len() + item.suffixes.len() >= 1);

    // =========================================================================
    // STEP 4: Apply Regal Orb (Magic -> Rare)
    // =========================================================================
    separator("STEP 4: Applying Regal Orb");

    let regal = generator.config().currencies.get("regal")
        .expect("Regal currency not found");

    println!("  Using: {} - {}", regal.name, regal.description);

    let mut currency_rng = Generator::make_rng(456);
    apply_currency(&generator, &mut item, regal, &mut currency_rng)
        .expect("Failed to apply regal");

    println!("\n  After regal orb:");
    print_item(&item);
    assert_eq!(item.rarity, Rarity::Rare);
    assert!(item.prefixes.len() + item.suffixes.len() >= 3);

    // =========================================================================
    // STEP 5: Create player and equip the item
    // =========================================================================
    separator("STEP 5: Creating Player and Equipping Item");

    let mut player = StatBlock::new();

    // Apply base stats
    let base_stats = BaseStatsSource::new(1);
    let mut accumulator = stat_core::stat_block::StatAccumulator::new();
    base_stats.apply(&mut accumulator);
    accumulator.apply_to(&mut player);

    println!("  Player before equipping:");
    print_stats("Player", &player);

    // Create gear source from the item
    let gear_source = GearSource::new(EquipmentSlot::MainHand, item.clone());

    println!("\n  Equipping: {} to MainHand slot", item.name);

    // Rebuild stats with gear
    let sources: Vec<Box<dyn StatSource>> = vec![
        Box::new(base_stats),
        Box::new(gear_source),
    ];
    player.rebuild_from_sources(&sources);

    // Set current life/mana to max
    player.current_life = player.computed_max_life();
    player.current_mana = player.computed_max_mana();

    println!("\n  Player after equipping:");
    print_stats("Player", &player);

    // Verify weapon damage was applied
    let (phys_min, phys_max) = player.weapon_damage(DamageType::Physical);
    assert!(phys_max > 0.0, "Weapon should provide physical damage");

    // =========================================================================
    // STEP 6: Create enemy
    // =========================================================================
    separator("STEP 6: Creating Enemy");

    let mut enemy = StatBlock::new();
    enemy.max_life.base = 500.0;
    enemy.current_life = 500.0;
    enemy.fire_resistance.base = 25.0;
    enemy.armour.base = 100.0;

    print_stats("Enemy", &enemy);

    // =========================================================================
    // STEP 7: Select a skill and attack
    // =========================================================================
    separator("STEP 7: Combat - Attacking Enemy");

    // Use Heavy Strike for bleed chance
    let skill = DamagePacketGenerator {
        id: "heavy_strike".to_string(),
        name: "Heavy Strike".to_string(),
        weapon_effectiveness: 1.0,
        damage_effectiveness: 1.5,
        attack_speed_modifier: 0.85,
        tags: vec![stat_core::types::SkillTag::Attack, stat_core::types::SkillTag::Melee],
        status_conversions: stat_core::damage::SkillStatusConversions {
            physical_to_bleed: 0.70,
            ..Default::default()
        },
        ..Default::default()
    };

    println!("  Skill: {} ({}% weapon effectiveness, {}% damage effectiveness)",
        skill.name,
        skill.weapon_effectiveness * 100.0,
        skill.damage_effectiveness * 100.0
    );
    println!("  Status Conversions: 70% Physical -> Bleed");

    let mut attack_rng = Generator::make_rng(789);

    // Perform multiple attacks
    for attack_num in 1..=5 {
        println!("\n  --- Attack #{} ---", attack_num);

        let packet = calculate_damage(
            &player,
            &skill,
            "player".to_string(),
            &mut attack_rng,
        );

        println!("  Damage Packet:");
        println!("    Total Damage: {:.0}", packet.total_damage());
        println!("    Critical: {}", if packet.is_critical { "YES!" } else { "No" });
        if packet.is_critical {
            println!("    Crit Multiplier: {:.2}x", packet.crit_multiplier);
        }
        for dmg in &packet.damages {
            println!("    {:?}: {:.0}", dmg.damage_type, dmg.amount);
        }

        if !packet.status_effects_to_apply.is_empty() {
            println!("    Pending Status Effects:");
            for status in &packet.status_effects_to_apply {
                let chance = status.calculate_apply_chance(enemy.computed_max_life()) * 100.0;
                println!("      {:?}: {:.0} status dmg ({:.1}% chance, {:.0} DPS)",
                    status.effect_type, status.status_damage, chance, status.dot_dps);
            }
        }

        // Resolve damage
        let result = resolve_damage(&mut enemy, &packet);

        println!("  Combat Result:");
        println!("    Damage Dealt: {:.0}", result.total_damage);
        println!("    Reduced by Armour: {:.0}", result.damage_reduced_by_armour);
        println!("    Reduced by Resists: {:.0}", result.damage_reduced_by_resists);
        println!("    Enemy HP: {:.0}/{:.0}", enemy.current_life, enemy.computed_max_life());

        if !result.effects_applied.is_empty() {
            println!("    Effects Applied:");
            for effect in &result.effects_applied {
                if let stat_core::types::EffectType::Ailment { dot_dps, .. } = &effect.effect_type {
                    if *dot_dps > 0.0 {
                        println!("      {}: {:.0} DPS for {:.1}s",
                            effect.name, dot_dps, effect.duration_remaining);
                    } else {
                        println!("      {}: {:.1}s duration",
                            effect.name, effect.duration_remaining);
                    }
                } else {
                    println!("      {}: {:.1}s duration",
                        effect.name, effect.duration_remaining);
                }
            }
        }

        if result.is_killing_blow {
            println!("    ENEMY DEFEATED!");
            break;
        }
    }

    // =========================================================================
    // STEP 8: Process effect ticks (using unified Effect system)
    // =========================================================================
    if !enemy.effects.is_empty() && enemy.is_alive() {
        separator("STEP 8: Processing Effect Ticks");

        println!("  Active Effects on Enemy:");
        for effect in &enemy.effects {
            if let stat_core::types::EffectType::Ailment { dot_dps, .. } = &effect.effect_type {
                if *dot_dps > 0.0 {
                    println!("    {}: {:.0} DPS, {:.1}s remaining",
                        effect.name, dot_dps, effect.duration_remaining);
                } else {
                    println!("    {}: {:.1}s remaining",
                        effect.name, effect.duration_remaining);
                }
            } else {
                println!("    {}: {:.1}s remaining",
                    effect.name, effect.duration_remaining);
            }
        }

        // Simulate 5 seconds of effect ticks
        println!("\n  Simulating 5 seconds of effect damage...\n");

        for second in 1..=5 {
            let (new_enemy, tick_result) = enemy.tick_effects(1.0);
            enemy = new_enemy;

            if tick_result.dot_damage > 0.0 {
                println!("  [{}s] Effects deal {:.0} damage (Enemy HP: {:.0})",
                    second, tick_result.dot_damage, tick_result.life_remaining.max(0.0));
            }

            if tick_result.is_dead {
                println!("  ENEMY DEFEATED by effects!");
                break;
            }

            if enemy.effects.is_empty() {
                println!("  All effects expired.");
                break;
            }
        }
    }

    // =========================================================================
    // SUMMARY
    // =========================================================================
    separator("TEST COMPLETE - SUMMARY");

    println!("  Item Journey:");
    println!("    1. Generated normal Iron Sword");
    println!("    2. Transmuted to Magic (gained affixes)");
    println!("    3. Regaled to Rare (gained more affixes)");
    println!("    4. Equipped on player");
    println!("    5. Used Heavy Strike with 70% bleed conversion");
    println!("    6. Attacked enemy, applied bleeds, dealt damage over time");
    println!("\n  Final State:");
    println!("    Enemy HP: {:.0}/{:.0}", enemy.current_life.max(0.0), enemy.computed_max_life());
    println!("    Enemy Status: {}", if enemy.is_alive() { "Alive" } else { "Defeated" });

    println!("\n  Test passed successfully!");
}
