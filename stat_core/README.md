# stat_core

A game mechanics library for stat management, damage calculation, and combat resolution in action RPGs. Built with the triple-modifier (Flat/Increased/More) stacking model.

## Quick Start

```rust
use stat_core::{StatBlock, DamagePacketGenerator, damage::calculate_damage, combat::resolve_damage};
use rand::thread_rng;

fn main() {
    // Create attacker with weapon stats
    let mut attacker = StatBlock::new();
    attacker.weapon_physical_min = 50.0;
    attacker.weapon_physical_max = 100.0;
    attacker.global_physical_damage.add_increased(0.50); // 50% increased damage

    // Create defender
    let mut defender = StatBlock::new();
    defender.max_life.base = 500.0;
    defender.current_life = 500.0;
    defender.armour.base = 200.0;

    // Attack with basic attack
    let skill = DamagePacketGenerator::basic_attack();
    let mut rng = thread_rng();
    let packet = calculate_damage(&attacker, &skill, "attacker".into(), &mut rng);
    let result = resolve_damage(&mut defender, &packet);

    println!("Dealt {} damage, defender has {} HP remaining",
        result.total_damage, defender.current_life);
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

```
1. calculate_damage() → DamagePacket
   - Roll weapon/skill damage
   - Apply damage conversions
   - Apply scaling (increased/more)
   - Calculate crit
   - Generate pending status effects

2. resolve_damage() → CombatResult
   - Apply resistances
   - Apply armour (physical only)
   - Apply evasion cap
   - Damage ES, then life
   - Roll status effect applications
   - Return damage breakdown
```

---

## Stat Sources

Build stats from multiple sources:

```rust
use stat_core::{StatBlock, source::{BaseStatsSource, GearSource, StatSource}};

let mut player = StatBlock::new();
let sources: Vec<Box<dyn StatSource>> = vec![
    Box::new(BaseStatsSource::new(10)),  // Level 10 base stats
    Box::new(GearSource::new(slot, item)),
];
player.rebuild_from_sources(&sources);
```

| Source | Priority | Description |
|--------|----------|-------------|
| `BaseStatsSource` | -100 | Level-based stats |
| `GearSource` | 0 | Equipment |
| `SkillTreeSource` | 100 | Passives |
| `BuffSource` | 200 | Temporary effects |

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
    StatBlock, DamagePacketGenerator, Effect,
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
        // Attack
        let packet = calculate_damage(&player, &skill, "player".into(), &mut rng);
        let result = resolve_damage(&mut enemy, &packet);

        println!("[{:.1}s] {} damage → {:.0} HP",
            time, result.total_damage, enemy.current_life);

        // Tick effects
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
