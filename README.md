# stat_manager

A Rust library for game stat management, damage calculation, and combat resolution. Designed for RPG-style games with complex stat interactions.

## Quick Start

```rust
use stat_core::prelude::*;
use loot_core::{Config, Generator};

fn main() {
    // Load item generator and registries
    let config = Config::load("path/to/config/").unwrap();
    let generator = Generator::new(config);
    let dot_registry = DotRegistry::new();
    let skills = default_skills();

    // Create a player and equip items
    let mut player = StatBlock::with_id("player");
    player.equip(
        EquipmentSlot::MainHand,
        generator.generate("iron_sword", 12345).unwrap(),
    );

    // Create an enemy
    let mut enemy = StatBlock::with_id("goblin");
    enemy.max_life.base = 200.0;
    enemy.current_life = 200.0;

    // Attack!
    let packet = player.attack(&skills["heavy_strike"], &dot_registry);
    let result = enemy.receive_damage(&packet, &dot_registry);

    println!("Dealt {} damage!", result.total_damage);
    println!("Enemy has {} life remaining", enemy.current_life);
}
```

## Core Concepts

### StatBlock

The central data structure representing an entity's complete stat state:
- **Resources**: life, mana, energy shield
- **Attributes**: strength, dexterity, intelligence, etc.
- **Defenses**: armour, evasion, resistances
- **Offense**: accuracy, damage bonuses, crit chance/multiplier

```rust
let mut character = StatBlock::with_id("hero");
character.max_life.base = 100.0;
character.strength.base = 20.0;
```

### Equipment

Equip items directly to a StatBlock. Stats are automatically recalculated:

```rust
let weapon = generator.generate("iron_sword", seed).unwrap();
player.equip(EquipmentSlot::MainHand, weapon);

// Stats are now updated with weapon bonuses
println!("Attack speed: {}", player.attack_speed.compute());
```

### Skills & Damage

Load skills from TOML config and generate damage packets:

```rust
let skills = default_skills();
let fireball = &skills["fireball"];

// Generate a damage packet (rolls damage, crit, etc.)
let packet = player.attack(fireball, &dot_registry);

// Apply to target
let result = enemy.receive_damage(&packet, &dot_registry);
```

### Buffs

Apply temporary buffs that modify stats:

```rust
use stat_core::BuffSource;
use loot_core::types::StatType;

let damage_buff = BuffSource::new(
    "rage".to_string(),
    "Rage".to_string(),
    10.0,  // duration
    false, // not a debuff
)
.with_modifier(StatType::IncreasedPhysicalDamage, 50.0, false);

player.apply_buff(damage_buff);
```

## Stat Formula

Stats use the **Flat → Increased → More** multiplicative stacking model:

```
Final = (base + flat) × (1 + sum(increased%)) × product(1 + more%)
```

- `flat`: Additive bonuses (e.g., +10 life)
- `increased`: Additive percentage bonuses (e.g., 20% + 30% = 50% increased)
- `more`: Multiplicative percentage bonuses (e.g., 20% more × 30% more = 1.2 × 1.3)

## Project Structure

```
stat_manager/
├── stat_core/       # Core library
├── stat_tui/        # Interactive TUI for testing
└── example_game/    # Minimal game example
```

## Development

```bash
# Enter development shell
nix develop

# Build
cargo build

# Test
cargo test

# Run TUI
cargo run -p stat_tui

# Run example game
cargo run -p example_game
```

## License

MIT
