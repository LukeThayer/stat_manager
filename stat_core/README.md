# stat_core

A comprehensive game mechanics library for stat management, damage calculation, and combat resolution in action RPGs. Built in Rust with a focus on correctness, configurability, and the triple-modifier (Flat/Increased/More) stacking model.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Core Concepts](#core-concepts)
  - [StatValue: The Triple-Modifier Model](#statvalue-the-triple-modifier-model)
  - [StatBlock: Character State](#statblock-character-state)
  - [Stat Sources](#stat-sources)
- [Skills & Damage](#skills--damage)
  - [DamagePacketGenerator (Skills)](#damagepacketgenerator-skills)
  - [Skill Fields Reference](#skill-fields-reference)
  - [Damage Conversions](#damage-conversions)
  - [Status Effect Conversions](#status-effect-conversions)
- [Damage Calculation](#damage-calculation)
  - [Calculation Flow](#calculation-flow)
  - [Formulas](#formulas)
- [Status Effects & DoTs](#status-effects--dots)
  - [Status Effects](#status-effects)
  - [DoT System](#dot-system)
  - [DoT Stacking Rules](#dot-stacking-rules)
- [Defense Mechanics](#defense-mechanics)
- [Configuration](#configuration)
- [Examples](#examples)

---

## Overview

`stat_core` provides:

- **StatBlock**: Aggregated character statistics from multiple sources (gear, buffs, skill tree, etc.)
- **DamagePacketGenerator**: Configurable skill/ability definitions
- **DamagePacket**: Calculated damage output with type breakdown, crits, penetration, and pending status effects
- **DoT System**: Damage-over-time with configurable stacking rules
- **Combat Resolution**: Applying damage against defenses (armour, evasion, resistances)
- **Triple-Modifier Model**: Flat → Increased → More modifier stacking (like Path of Exile)

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stat_core = { path = "../stat_core" }  # Or from your registry
```

Required dependencies:
- `serde` (with `derive` feature)
- `rand`
- `loot_core` (for item/modifier types)

---

## Quick Start

```rust
use stat_core::{
    StatBlock, DamagePacketGenerator, DotRegistry,
    damage::calculate_damage,
};
use rand::thread_rng;

fn main() {
    // Create a character with some stats
    let mut player = StatBlock::new();
    player.weapon_physical_min = 50.0;
    player.weapon_physical_max = 100.0;
    player.weapon_attack_speed = 1.5;
    player.global_physical_damage.add_increased(0.50); // 50% increased

    // Load or create a skill
    let skill = DamagePacketGenerator::basic_attack();

    // Calculate damage
    let dot_registry = DotRegistry::with_defaults();
    let mut rng = thread_rng();
    let damage_packet = calculate_damage(
        &player,
        &skill,
        "player".to_string(),
        &dot_registry,
        &mut rng,
    );

    println!("Total damage: {}", damage_packet.total_damage());
    println!("Is critical: {}", damage_packet.is_critical);
}
```

---

## Architecture

```
stat_core/
├── src/
│   ├── lib.rs           # Public API exports
│   ├── types.rs         # Core enums (EquipmentSlot, SkillTag, etc.)
│   ├── stat_block/      # Character stat aggregation
│   │   ├── mod.rs       # StatBlock definition
│   │   ├── stat_value.rs    # Triple-modifier StatValue
│   │   ├── computed.rs      # Computed stat helpers
│   │   └── aggregator.rs    # StatAccumulator for building stats
│   ├── damage/          # Damage calculation system
│   │   ├── mod.rs
│   │   ├── generator.rs     # DamagePacketGenerator (skills)
│   │   ├── calculation.rs   # Core damage formulas
│   │   └── packet.rs        # DamagePacket output
│   ├── defense/         # Defense mechanics
│   │   ├── armour.rs
│   │   ├── evasion.rs
│   │   └── resistance.rs
│   ├── dot/             # Damage-over-Time system
│   │   ├── mod.rs       # DotRegistry
│   │   ├── types.rs     # DotConfig, DotStacking
│   │   ├── active.rs    # ActiveDoT
│   │   └── tick.rs      # DoT tick processing
│   ├── combat/          # Combat resolution
│   │   ├── resolution.rs
│   │   └── result.rs
│   ├── source/          # Stat providers
│   │   ├── base_stats.rs
│   │   ├── gear.rs
│   │   ├── buff.rs
│   │   └── skill_tree.rs
│   └── config/          # Configuration loading
│       ├── skills.rs
│       └── dots.rs
└── config/
    └── skills.toml      # Default skill definitions
```

---

## Core Concepts

### StatValue: The Triple-Modifier Model

`StatValue` implements the industry-standard modifier stacking system:

```rust
pub struct StatValue {
    pub base: f64,       // Base value (from character level, skill, etc.)
    pub flat: f64,       // Sum of all "+X" additions
    pub increased: f64,  // Sum of all "% increased" (additive with each other)
    pub more: Vec<f64>,  // List of "% more" multipliers (multiplicative)
}
```

**Computation Formula:**
```
Final = (base + flat) × (1 + increased) × ∏(1 + more[i])
```

**Example:**
```rust
let mut damage = StatValue::new(100.0);  // 100 base damage
damage.add_flat(20.0);                    // +20 flat damage
damage.add_increased(0.50);               // 50% increased damage
damage.add_increased(0.30);               // 30% increased damage (stacks additively)
damage.add_more(0.20);                    // 20% more damage
damage.add_more(0.15);                    // 15% more damage (stacks multiplicatively)

let final_damage = damage.compute();
// = (100 + 20) × (1 + 0.50 + 0.30) × (1 + 0.20) × (1 + 0.15)
// = 120 × 1.80 × 1.20 × 1.15
// = 298.08
```

### StatBlock: Character State

`StatBlock` is the complete representation of a character's stats:

```rust
pub struct StatBlock {
    // === Resources ===
    pub max_life: StatValue,
    pub current_life: f64,
    pub max_mana: StatValue,
    pub current_mana: f64,
    pub max_energy_shield: f64,
    pub current_energy_shield: f64,

    // === Attributes ===
    pub strength: StatValue,
    pub dexterity: StatValue,
    pub intelligence: StatValue,
    pub constitution: StatValue,
    pub wisdom: StatValue,
    pub charisma: StatValue,

    // === Defenses ===
    pub armour: StatValue,
    pub evasion: StatValue,
    pub fire_resistance: StatValue,      // -100% to +100%
    pub cold_resistance: StatValue,
    pub lightning_resistance: StatValue,
    pub chaos_resistance: StatValue,

    // === Offense (Global) ===
    pub accuracy: StatValue,
    pub global_physical_damage: StatValue,
    pub global_fire_damage: StatValue,
    pub global_cold_damage: StatValue,
    pub global_lightning_damage: StatValue,
    pub global_chaos_damage: StatValue,
    pub attack_speed: StatValue,
    pub cast_speed: StatValue,
    pub critical_chance: StatValue,
    pub critical_multiplier: StatValue,   // Base 1.5x (150%)

    // === Penetration ===
    pub fire_penetration: StatValue,
    pub cold_penetration: StatValue,
    pub lightning_penetration: StatValue,
    pub chaos_penetration: StatValue,

    // === Recovery ===
    pub life_regen: StatValue,
    pub mana_regen: StatValue,
    pub life_leech: StatValue,
    pub mana_leech: StatValue,

    // === Weapon Stats ===
    pub weapon_physical_min: f64,
    pub weapon_physical_max: f64,
    pub weapon_fire_min: f64,
    pub weapon_fire_max: f64,
    // ... other weapon damage types
    pub weapon_attack_speed: f64,
    pub weapon_crit_chance: f64,

    // === Active Effects ===
    pub active_dots: Vec<ActiveDoT>,
    pub active_buffs: Vec<ActiveBuff>,
    pub active_status_effects: Vec<ActiveStatusEffect>,

    // === Status Effect Configuration ===
    pub status_effect_stats: StatusEffectData,
}
```

### Stat Sources

Stats are aggregated from multiple sources with priority ordering:

| Source | Priority | Description |
|--------|----------|-------------|
| `BaseStatsSource` | -100 | Character base stats, level scaling |
| `GearSource` | 0 | Equipment modifiers |
| `SkillTreeSource` | 100 | Passive skill tree |
| `BuffSource` | 200 | Temporary buffs/debuffs |

```rust
use stat_core::{StatBlock, StatAccumulator, GearSource, StatSource};

let mut accumulator = StatAccumulator::new();

// Apply sources in priority order
let gear = GearSource::from_items(&equipped_items);
gear.apply(&mut accumulator);

// Build final stats
let mut stats = StatBlock::new();
accumulator.apply_to(&mut stats);
```

---

## Skills & Damage

### DamagePacketGenerator (Skills)

Skills are defined using `DamagePacketGenerator`:

```rust
pub struct DamagePacketGenerator {
    pub id: String,                    // Unique identifier
    pub name: String,                  // Display name
    pub base_damages: Vec<BaseDamage>, // Skill's own damage
    pub weapon_effectiveness: f64,     // How much weapon damage to use
    pub damage_effectiveness: f64,     // Scaling for added damage
    pub attack_speed_modifier: f64,    // Speed multiplier
    pub base_crit_chance: f64,         // Skill's crit chance
    pub crit_multiplier_bonus: f64,    // Added crit multiplier
    pub tags: Vec<SkillTag>,           // Categorization tags
    pub status_conversions: SkillStatusConversions,
    pub damage_conversions: DamageConversions,
    pub type_effectiveness: DamageTypeEffectiveness,
    pub hits_per_attack: u32,          // Multi-hit skills
    pub can_chain: bool,               // Projectile chaining
    pub chain_count: u32,
    pub pierce_chance: f64,
}
```

### Skill Fields Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | String | Required | Unique skill identifier |
| `name` | String | Required | Display name |
| `base_damages` | Vec | `[]` | Skill's inherent damage (not from weapon) |
| `weapon_effectiveness` | f64 | `1.0` | **Portion of weapon damage used**. `0.0` = spell (no weapon), `1.0` = full attack, `0.5` = half weapon damage |
| `damage_effectiveness` | f64 | `1.0` | **Multiplier for all damage scaling**. Affects added damage from gear. `1.5` = 150% of added damage applies |
| `attack_speed_modifier` | f64 | `1.0` | Multiplies attack/cast speed. `0.85` = 15% slower, `1.1` = 10% faster |
| `base_crit_chance` | f64 | `0.0` | Skill's base crit % (added to weapon crit for attacks) |
| `crit_multiplier_bonus` | f64 | `0.0` | Added to crit multiplier (base is 1.5x) |
| `tags` | Vec | `[]` | Skill tags for scaling (see below) |
| `status_conversions` | Struct | Default | % of damage converted to status effects |
| `damage_conversions` | Struct | Default | Convert damage types before scaling |
| `type_effectiveness` | Struct | All 1.0 | Per-type damage multipliers |
| `hits_per_attack` | u32 | `1` | Hits per use (e.g., Double Strike = 2) |
| `can_chain` | bool | `false` | Whether projectiles chain |
| `chain_count` | u32 | `0` | Number of chains |
| `pierce_chance` | f64 | `0.0` | Chance to pierce (0.0-1.0) |

#### Skill Tags

```rust
pub enum SkillTag {
    Attack,     // Uses weapon, scales with attack modifiers
    Spell,      // Doesn't use weapon, scales with spell modifiers
    Physical,   // Deals physical damage
    Fire,       // Deals fire damage
    Cold,       // Deals cold damage
    Lightning,  // Deals lightning damage
    Chaos,      // Deals chaos damage
    Elemental,  // Scales with elemental modifiers
    Melee,      // Close range
    Ranged,     // Long range
    Projectile, // Fires projectiles
    Aoe,        // Area of effect
}
```

#### Weapon Effectiveness Explained

`weapon_effectiveness` determines how much of your weapon's damage the skill uses:

| Value | Meaning | Example Skills |
|-------|---------|----------------|
| `0.0` | Pure spell - no weapon damage | Fireball, Ice Nova |
| `0.5` | Half weapon damage | Blade Vortex |
| `0.91` | Slightly reduced (multi-hit balance) | Double Strike |
| `1.0` | Full weapon damage | Basic Attack, Heavy Strike |
| `1.3+` | More than 100% weapon damage | Ground Slam |

#### Damage Effectiveness Explained

`damage_effectiveness` multiplies ALL damage scaling, including:
- Flat added damage from gear
- Base damage from the skill
- Per-type effectiveness bonuses

| Value | Meaning | Use Case |
|-------|---------|----------|
| `0.5` | Half effectiveness | Very fast/multi-hit skills |
| `1.0` | Normal | Standard skills |
| `1.5` | 150% effectiveness | Slow, heavy-hitting skills |

### Damage Conversions

Convert damage from one type to another **before** damage scaling is applied.

```rust
pub struct DamageConversions {
    pub physical_to_fire: f64,       // % of physical → fire
    pub physical_to_cold: f64,
    pub physical_to_lightning: f64,
    pub physical_to_chaos: f64,
    pub lightning_to_fire: f64,
    pub lightning_to_cold: f64,
    pub cold_to_fire: f64,
    pub fire_to_chaos: f64,
}
```

**Conversion Order:** Physical → Lightning → Cold → Fire → Chaos

This order matters! Converted damage can be converted again in sequence:
- Physical (50% to Lightning) → Lightning (50% to Cold) → Cold (50% to Fire)
- Starting with 100 Physical: 50 Physical + 25 Lightning + 12.5 Cold + 12.5 Fire

**Example - Molten Strike:**
```toml
[skills.damage_conversions]
physical_to_fire = 0.6  # 60% of physical becomes fire
```

### Status Effect Conversions

Convert hit damage into status effect applications.

```rust
pub struct SkillStatusConversions {
    pub physical_to_poison: f64,    // Physical damage → Poison chance
    pub chaos_to_poison: f64,       // Chaos damage → Poison chance
    pub physical_to_bleed: f64,     // Physical damage → Bleed
    pub fire_to_burn: f64,          // Fire damage → Burn
    pub cold_to_freeze: f64,        // Cold damage → Freeze
    pub cold_to_chill: f64,         // Cold damage → Chill
    pub lightning_to_static: f64,   // Lightning damage → Static/Shock
    pub chaos_to_fear: f64,         // Chaos damage → Fear
    pub physical_to_slow: f64,      // Physical damage → Slow
    pub cold_to_slow: f64,          // Cold damage → Slow
}
```

These combine **additively** with player stat conversions:
```
Total Conversion = Skill Conversion + Player Stat Conversion
```

### Type Effectiveness

Per-damage-type multipliers applied after conversion:

```rust
pub struct DamageTypeEffectiveness {
    pub physical: f64,   // Default 1.0 (100%)
    pub fire: f64,
    pub cold: f64,
    pub lightning: f64,
    pub chaos: f64,
}
```

**Example - Infernal Blow:**
```toml
[skills.type_effectiveness]
fire = 1.6      # 160% fire damage effectiveness
physical = 0.5  # Only 50% physical effectiveness
```

---

## Damage Calculation

### Calculation Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    DAMAGE CALCULATION FLOW                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. GATHER BASE DAMAGE                                          │
│     ├── Skill base damages (rolled randomly)                    │
│     └── Weapon damage × weapon_effectiveness (if attack)        │
│                          ↓                                       │
│  2. APPLY DAMAGE CONVERSIONS                                    │
│     Physical → Lightning → Cold → Fire → Chaos                  │
│                          ↓                                       │
│  3. APPLY DAMAGE SCALING (per type)                             │
│     base × increased_mult × more_mult × damage_eff × type_eff   │
│                          ↓                                       │
│  4. CALCULATE CRITICAL STRIKE                                   │
│     ├── Crit chance: (base + flat) × increased × more           │
│     └── If crit: damage × crit_multiplier                       │
│                          ↓                                       │
│  5. SET PENETRATION & ACCURACY                                  │
│     From attacker's stats                                       │
│                          ↓                                       │
│  6. CALCULATE STATUS EFFECTS                                    │
│     ├── Status damage = Σ(hit_damage × conversion%)             │
│     ├── Apply chance = status_damage / target_max_health        │
│     └── DoT DPS = base_dot% × status_damage × (1 + dot_increased)│
│                          ↓                                       │
│  7. OUTPUT: DamagePacket                                        │
│     damages[], is_critical, penetration, status_effects[]       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Formulas

#### Damage Scaling
```
Scaled Damage = Base × (1 + Increased%) × ∏(1 + More%) × DamageEffectiveness × TypeEffectiveness
```

Where:
- **Base** = skill base damage + (weapon damage × weapon_effectiveness)
- **Increased** = sum of all "% increased [type] damage" modifiers
- **More** = product of all "% more [type] damage" modifiers
- **DamageEffectiveness** = skill's `damage_effectiveness` value
- **TypeEffectiveness** = skill's per-type multiplier

#### Critical Strike Chance
```
Crit Chance = (BaseWeaponCrit + SkillBaseCrit + FlatCrit) × (1 + Increased%) × ∏(1 + More%)
```

Clamped to [0%, 100%].

#### Critical Strike Damage
```
Crit Damage = Base Damage × (1.5 + CritMultiplierBonus + IncreasedCritMulti)
```

Base crit multiplier is 150% (1.5x).

#### DPS Calculation
```
Hit DPS = AvgDamage × (1 + (CritMulti - 1) × CritChance) × AttackSpeed × HitsPerAttack
DoT DPS = StatusDamage × BaseDotPercent × (1 + DotIncreased) × AttackSpeed
Total DPS = Hit DPS + DoT DPS
```

#### Status Effect Application
```
Status Damage = Σ(HitDamage[type] × (SkillConversion[type] + PlayerConversion[type]))
Apply Chance = min(1.0, StatusDamage / TargetMaxHealth)
DoT DPS = BaseDotPercent × StatusDamage × (1 + DotIncreased%)
```

---

## Status Effects & DoTs

### Status Effects

8 status effects are supported:

| Status | Damage Type | Effect | DoT? |
|--------|-------------|--------|------|
| **Poison** | Chaos | Damage over time | Yes (20% base) |
| **Bleed** | Physical | DoT, 2x while moving | Yes (20% base) |
| **Burn** | Fire | Damage over time | Yes (25% base) |
| **Freeze** | Cold | Immobilization | No |
| **Chill** | Cold | Slow effect | No |
| **Static** | Lightning | Shock (increased damage taken) | No |
| **Fear** | Chaos | Flee/debuff | No |
| **Slow** | Physical/Cold | Movement penalty | No |

### DoT System

DoT configuration via `DotConfig`:

```rust
pub struct DotConfig {
    pub id: String,                // "burn", "poison", "bleed"
    pub name: String,
    pub damage_type: DamageType,
    pub stacking: DotStacking,
    pub base_duration: f64,        // Seconds
    pub tick_rate: f64,            // Seconds between ticks
    pub base_damage_percent: f64,  // % of status damage → DPS
    pub max_stacks: u32,
    pub stack_effectiveness: f64,  // Additional stack multiplier
    pub moving_multiplier: f64,    // Extra damage while moving
}
```

Default DoT configurations:

| DoT | Duration | Tick Rate | Base DMG % | Stacking | Special |
|-----|----------|-----------|------------|----------|---------|
| Burn | 4.0s | 0.5s | 25% | Strongest Only | - |
| Poison | 2.0s | 0.33s | 20% | Unlimited | - |
| Bleed | 5.0s | 1.0s | 20% | Limited (8, 50%) | 2x while moving |

### DoT Stacking Rules

```rust
pub enum DotStacking {
    StrongestOnly,  // Only the highest damage instance deals damage
    Unlimited,      // All instances stack independently
    Limited {
        max_stacks: u32,           // Maximum number of stacks
        stack_effectiveness: f64,  // Additional stacks at reduced effectiveness
    },
}
```

**StrongestOnly** (Burn, Freeze, Chill, Fear, Slow):
- Only the strongest instance deals damage
- Weaker applications refresh duration if stronger

**Unlimited** (Poison):
- Every application adds a new instance
- All deal full damage independently

**Limited** (Bleed, Static):
- First application is at full effectiveness
- Additional stacks up to `max_stacks` at `stack_effectiveness`
- At max: refreshes oldest stack

---

## Defense Mechanics

### Armour (Physical Reduction)

```
Reduction% = Armour / (Armour + ARMOUR_CONSTANT × Damage)
```

Armour is more effective against many small hits than few large hits.

### Evasion

```
Chance to Evade = Evasion / (Evasion + Accuracy)
Damage Cap = EVASION_SCALE_FACTOR × Evasion / Accuracy
```

If damage exceeds cap, always hits.

### Resistance

```
Final Damage = Hit Damage × (1 - EffectiveResist%)
Effective Resist = Resist - Penetration
```

Resistance capped at 100% (immunity) and -200% (triple damage).

---

## Configuration

### Skills TOML Format

```toml
[[skills]]
id = "fireball"
name = "Fireball"
tags = ["spell", "fire", "projectile", "aoe"]
weapon_effectiveness = 0.0    # Pure spell
damage_effectiveness = 1.0
attack_speed_modifier = 1.0
base_crit_chance = 6.0
crit_multiplier_bonus = 0.0

[[skills.base_damages]]
type = "fire"
min = 100
max = 180

[skills.damage_conversions]
physical_to_fire = 0.0        # No conversions

[skills.status_conversions]
fire_to_burn = 0.50           # 50% fire → burn

[skills.type_effectiveness]
fire = 1.3                    # 130% fire effectiveness
```

### Loading Skills

```rust
use stat_core::config::default_skills;

let skills = default_skills();
let fireball = skills.iter().find(|s| s.id == "fireball").unwrap();
```

---

## Examples

### Example 1: Basic Attack Calculation

```rust
use stat_core::{StatBlock, DamagePacketGenerator, DotRegistry};
use stat_core::damage::calculate_damage;
use rand::thread_rng;

let mut player = StatBlock::new();
player.weapon_physical_min = 50.0;
player.weapon_physical_max = 100.0;
player.global_physical_damage.add_increased(0.5); // 50% increased

let skill = DamagePacketGenerator::basic_attack();
let dot_registry = DotRegistry::with_defaults();
let mut rng = thread_rng();

let packet = calculate_damage(&player, &skill, "player".to_string(), &dot_registry, &mut rng);

// Expected: 75 avg × 1.5 = 112.5 avg damage
println!("Damage: {}", packet.total_damage());
```

### Example 2: Elemental Conversion Skill

```rust
use stat_core::{DamagePacketGenerator, DamageConversions, DamageTypeEffectiveness};
use stat_core::types::SkillTag;

let molten_strike = DamagePacketGenerator {
    id: "molten_strike".to_string(),
    name: "Molten Strike".to_string(),
    weapon_effectiveness: 1.0,
    damage_effectiveness: 1.0,
    hits_per_attack: 3,
    tags: vec![SkillTag::Attack, SkillTag::Melee, SkillTag::Fire],
    damage_conversions: DamageConversions {
        physical_to_fire: 0.6, // 60% phys → fire
        ..Default::default()
    },
    type_effectiveness: DamageTypeEffectiveness {
        fire: 1.4, // 140% fire effectiveness
        ..Default::default()
    },
    ..Default::default()
};
```

### Example 3: DPS Calculation

```rust
use stat_core::{StatBlock, DamagePacketGenerator, DotRegistry};
use stat_core::damage::calculate_skill_dps;

let player = StatBlock::new();
let skill = DamagePacketGenerator::basic_attack();
let dot_registry = DotRegistry::with_defaults();

let dps = calculate_skill_dps(&player, &skill, &dot_registry);
println!("Skill DPS: {:.1}", dps);
```

### Example 4: Aggregating Stats from Gear

```rust
use stat_core::{StatBlock, StatAccumulator, GearSource, StatSource};

let mut accumulator = StatAccumulator::new();

// Apply gear stats
let gear = GearSource::from_items(&my_equipped_items);
gear.apply(&mut accumulator);

// Build final stat block
let mut player = StatBlock::new();
accumulator.apply_to(&mut player);
```

---

## License

MIT License - see workspace Cargo.toml for details.
