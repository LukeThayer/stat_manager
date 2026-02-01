# stat_core

A game mechanics library for stat management, damage calculation, and combat resolution in action RPGs. Built with the triple-modifier (Flat/Increased/More) stacking model.

## Quick Start

```rust
use stat_core::{
    StatBlock, DamagePacketGenerator,
    damage::calculate_damage,
    combat::resolve_damage,
    source::{BaseStatsSource, GearSource, StatSource},
    types::EquipmentSlot,
};
use loot_core::{Config, Generator};
use rand::thread_rng;
use std::path::Path;

fn main() {
    // Load loot generator and create a weapon
    let config = Config::load_from_dir(Path::new("config")).unwrap();
    let generator = Generator::new(config);
    let weapon = generator.generate("iron_sword", 42).unwrap();

    // Build player stats from sources
    let base_stats = BaseStatsSource::new(10);  // Level 10
    let gear = GearSource::new(EquipmentSlot::MainHand, weapon);

    let sources: Vec<Box<dyn StatSource>> = vec![
        Box::new(base_stats),
        Box::new(gear),
    ];

    let mut player = StatBlock::new();
    player.rebuild_from_sources(&sources);
    player.current_life = player.computed_max_life();

    // Create enemy
    let mut enemy = StatBlock::new();
    enemy.max_life.base = 500.0;
    enemy.current_life = 500.0;
    enemy.armour.base = 200.0;

    // Attack
    let skill = DamagePacketGenerator::basic_attack();
    let mut rng = thread_rng();
    let packet = calculate_damage(&player, &skill, "player".into(), &mut rng);
    let (enemy, result) = resolve_damage(&enemy, &packet);

    println!("Dealt {} damage, enemy has {} HP remaining",
        result.total_damage, enemy.current_life);
}
```

## Installation

```toml
[dependencies]
stat_core = { path = "../stat_core" }
```

---

## Core Concepts

### StatValue: Triple-Modifier Model

All stats use `StatValue` with the formula:

```
Final = (base + flat) × (1 + increased) × ∏(1 + more[i])
```

```rust
let mut damage = StatValue::new(100.0);
damage.add_flat(20.0);           // +20 flat
damage.add_increased(0.50);      // 50% increased (additive with other increased)
damage.add_more(0.20);           // 20% more (multiplicative)
// Result: (100 + 20) × 1.50 × 1.20 = 216
```

### StatBlock

Complete character state including:
- **Resources**: life, mana, energy shield
- **Attributes**: strength, dexterity, intelligence, etc.
- **Defenses**: armour, evasion, resistances
- **Offense**: damage stats, crit, penetration
- **Weapon**: min/max damage, attack speed, crit chance
- **Effects**: active buffs, debuffs, and ailments

---

## Skills

Skills are defined with `DamagePacketGenerator`:

```rust
let fireball = DamagePacketGenerator {
    id: "fireball".into(),
    name: "Fireball".into(),
    base_damages: vec![BaseDamage::new(DamageType::Fire, 100.0, 150.0)],
    weapon_effectiveness: 0.0,    // Spell - no weapon damage
    damage_effectiveness: 1.0,    // Full scaling
    base_crit_chance: 6.0,
    tags: vec![SkillTag::Spell, SkillTag::Fire],
    ..Default::default()
};
```

Key fields:
| Field | Description |
|-------|-------------|
| `weapon_effectiveness` | 0.0 = spell, 1.0 = full weapon damage |
| `damage_effectiveness` | Multiplier for all damage scaling |
| `damage_conversions` | Convert physical → elemental before scaling |
| `status_conversions` | % of damage that applies status effects |

---

## Unified Effect System

All buffs, debuffs, and ailments use a single `Effect` type:

```rust
use stat_core::{Effect, StatusEffect};

// Create a poison effect using helper constructor
let poison = Effect::poison(50.0, "attacker"); // 50 DPS

// Apply to target
enemy.add_effect(poison);

// Tick effects over time (immutable API)
let (new_enemy, result) = enemy.tick_effects(1.0);
println!("DoT dealt {} damage", result.dot_damage);
```

### Ailment Helpers

```rust
Effect::poison(dps, source)   // Chaos DoT, unlimited stacking
Effect::bleed(dps, source)    // Physical DoT, limited stacking
Effect::burn(dps, source)     // Fire DoT, strongest only
Effect::freeze(mag, source)   // Immobilize
Effect::chill(mag, source)    // Slow
Effect::shock(mag, source)    // Increased damage taken
Effect::fear(mag, source)     // Flee
Effect::slow(mag, source)     // Movement penalty
```

### Stacking Rules

| Mode | Behavior | Used By |
|------|----------|---------|
| `StrongestOnly` | Only strongest applies | Burn, Freeze, Chill |
| `Unlimited` | All instances stack | Poison |
| `Limited` | Up to max stacks | Bleed, Shock |

---

## Defense Mechanics

### Armour
Reduces physical damage with diminishing returns:
```
Reduction% = Armour / (Armour + 10 × Damage)
```

### Evasion
Caps maximum damage from a single hit:
```
Damage Cap = Accuracy / (1 + Evasion/1000)
```

### Resistance
Flat percentage reduction (capped at 75%):
```
Final = Damage × (1 - (Resistance - Penetration))
```

---

## Combat Flow

Both functions use immutable APIs that return new state:

```
1. calculate_damage(&attacker, &skill, source_id, &mut rng) → DamagePacket
   - Roll weapon/skill damage
   - Apply damage conversions
   - Apply scaling (increased/more)
   - Calculate crit
   - Generate pending status effects

2. resolve_damage(&defender, &packet) → (StatBlock, CombatResult)
   - Apply resistances
   - Apply armour (physical only)
   - Apply evasion cap
   - Damage ES, then life
   - Roll status effect applications
   - Return new defender state + damage breakdown
```

---

## Stat Sources

The `StatSource` trait allows modular stat building. Sources are sorted by priority (lowest first) and applied in order to a `StatAccumulator`, which then finalizes into a `StatBlock`.

### How It Works

```rust
use stat_core::{
    StatBlock,
    stat_block::StatAccumulator,
    source::{BaseStatsSource, GearSource, StatSource},
    types::EquipmentSlot,
};
use loot_core::{Config, Generator};

// 1. Create sources
let base_stats = BaseStatsSource::new(10);  // Level 10

let config = Config::load_from_dir("config").unwrap();
let generator = Generator::new(config);
let sword = generator.generate("iron_sword", 42).unwrap();
let helmet = generator.generate("iron_helmet", 43).unwrap();

let main_hand = GearSource::new(EquipmentSlot::MainHand, sword);
let head = GearSource::new(EquipmentSlot::Helmet, helmet);

// 2. Collect into trait objects
let sources: Vec<Box<dyn StatSource>> = vec![
    Box::new(base_stats),
    Box::new(main_hand),
    Box::new(head),
];

// 3. Rebuild stats (sorts by priority, applies in order)
let mut player = StatBlock::new();
player.rebuild_from_sources(&sources);

// 4. Set current resources to max
player.current_life = player.computed_max_life();
player.current_mana = player.computed_max_mana();
```

### Priority Order

Sources are applied from lowest to highest priority. This ensures base stats are set before equipment modifies them.

| Source | Priority | What It Provides |
|--------|----------|------------------|
| `BaseStatsSource` | -100 | Life (+12/level), mana (+6/level), base attributes (10 each) |
| `GearSource` | 0 | Weapon damage, armour, evasion, affixes from items |
| `SkillTreeSource` | 100 | Passive skill bonuses |
| Custom buffs | 200+ | Temporary stat modifications |

### BaseStatsSource

Provides level-scaled base stats:

```rust
let base = BaseStatsSource::new(level);
// Level 1:  62 life, 46 mana
// Level 10: 170 life, 100 mana
// Formula: 50 + (12 × level) life, 40 + (6 × level) mana
```

### GearSource

Converts `loot_core::Item` into stat modifiers:

```rust
use loot_core::currency::apply_currency;

// Generate and craft an item
let mut sword = generator.generate("iron_sword", 42).unwrap();
let transmute = generator.config().currencies.get("transmute").unwrap();
apply_currency(&generator, &mut sword, transmute, &mut rng).unwrap();

// Equip it
let gear = GearSource::new(EquipmentSlot::MainHand, sword);
```

GearSource handles:
- **Weapon stats**: Base damage, attack speed, crit chance
- **Armour/Evasion**: From body armour, shields, helmets
- **Affixes**: Both prefixes and suffixes with their stat modifiers
- **Local vs Global scope**: Local modifiers (e.g., "increased physical damage" on weapons) apply to the item; global modifiers apply to all stats

### Custom Sources

Implement `StatSource` for custom stat providers:

```rust
use stat_core::{source::StatSource, stat_block::StatAccumulator};

struct AuraSource {
    increased_damage: f64,
}

impl StatSource for AuraSource {
    fn id(&self) -> &str { "aura" }
    fn priority(&self) -> i32 { 150 }  // After gear, before temporary buffs

    fn apply(&self, acc: &mut StatAccumulator) {
        acc.global_physical_damage.add_increased(self.increased_damage);
    }
}
```

---

## Configuration

Load skills from TOML:

```toml
[[skills]]
id = "heavy_strike"
name = "Heavy Strike"
tags = ["attack", "melee", "physical"]
weapon_effectiveness = 1.0
damage_effectiveness = 1.5

[skills.status_conversions]
physical_to_bleed = 0.25
```

```rust
use stat_core::default_skills;

let skills = default_skills();
let heavy_strike = skills.get("heavy_strike").unwrap();
```

---

## Full Example: Combat Loop

```rust
use stat_core::{
    StatBlock, DamagePacketGenerator,
    damage::calculate_damage,
    combat::resolve_damage,
};
use rand::thread_rng;

fn main() {
    let mut player = StatBlock::new();
    player.weapon_physical_min = 80.0;
    player.weapon_physical_max = 120.0;
    player.weapon_attack_speed = 1.4;
    player.global_physical_damage.add_increased(0.75);

    let mut enemy = StatBlock::new();
    enemy.max_life.base = 1000.0;
    enemy.current_life = 1000.0;
    enemy.armour.base = 500.0;
    enemy.fire_resistance.base = 30.0;

    let skill = DamagePacketGenerator::basic_attack();
    let mut rng = thread_rng();
    let mut time = 0.0;

    while enemy.is_alive() && time < 10.0 {
        // Attack (immutable - returns new enemy state)
        let packet = calculate_damage(&player, &skill, "player".into(), &mut rng);
        let (new_enemy, result) = resolve_damage(&enemy, &packet);
        enemy = new_enemy;

        println!("[{:.1}s] {} damage → {:.0} HP",
            time, result.total_damage, enemy.current_life);

        // Tick effects (also immutable)
        if !enemy.effects.is_empty() {
            let (new_enemy, tick) = enemy.tick_effects(0.5);
            enemy = new_enemy;
            if tick.dot_damage > 0.0 {
                println!("  DoT: {} damage", tick.dot_damage);
            }
        }

        time += 1.0 / player.computed_attack_speed();
    }

    println!("Combat ended at {:.1}s - Enemy {}",
        time, if enemy.is_alive() { "survived" } else { "defeated" });
}
```

---

## License

MIT
