//! Core types specific to stat_manager

use serde::{Deserialize, Serialize};

/// Equipment slot for gear
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Helmet,
    BodyArmour,
    Gloves,
    Boots,
    Ring1,
    Ring2,
    Amulet,
    Belt,
}

impl EquipmentSlot {
    /// Get all equipment slots
    pub fn all() -> &'static [EquipmentSlot] {
        &[
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Helmet,
            EquipmentSlot::BodyArmour,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Ring1,
            EquipmentSlot::Ring2,
            EquipmentSlot::Amulet,
            EquipmentSlot::Belt,
        ]
    }
}

/// Skill tags for damage scaling and categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillTag {
    // Damage source
    Attack,
    Spell,
    // Damage types
    Physical,
    Fire,
    Cold,
    Lightning,
    Chaos,
    Elemental,
    // Delivery
    Melee,
    Ranged,
    Projectile,
    // Area
    Aoe,
}

/// Identifier for a skill tree node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillNodeId(pub String);

impl From<&str> for SkillNodeId {
    fn from(s: &str) -> Self {
        SkillNodeId(s.to_string())
    }
}

impl From<String> for SkillNodeId {
    fn from(s: String) -> Self {
        SkillNodeId(s)
    }
}

/// Active buff/debuff on an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBuff {
    /// Buff identifier
    pub buff_id: String,
    /// Display name
    pub name: String,
    /// Time remaining in seconds
    pub duration_remaining: f64,
    /// Current stack count
    pub stacks: u32,
    /// Whether this is a debuff (negative effect)
    pub is_debuff: bool,
}

/// Active status effect on an entity (freeze, chill, burn, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStatusEffect {
    /// The type of status effect
    pub effect_type: loot_core::types::StatusEffect,
    /// Time remaining in seconds
    pub duration_remaining: f64,
    /// Current stack count
    pub stacks: u32,
    /// Effect magnitude (e.g., slow percentage)
    pub magnitude: f64,
    /// Damage per second for damaging statuses (Poison, Bleed, Burn)
    pub dot_dps: f64,
    /// Source entity ID that applied this effect
    pub source_id: String,
}

impl ActiveStatusEffect {
    pub fn new(
        effect_type: loot_core::types::StatusEffect,
        duration: f64,
        magnitude: f64,
        source_id: String,
    ) -> Self {
        ActiveStatusEffect {
            effect_type,
            duration_remaining: duration,
            stacks: 1,
            magnitude,
            dot_dps: 0.0,
            source_id,
        }
    }

    /// Create a new status effect with DoT damage
    pub fn new_with_dot(
        effect_type: loot_core::types::StatusEffect,
        duration: f64,
        magnitude: f64,
        dot_dps: f64,
        source_id: String,
    ) -> Self {
        ActiveStatusEffect {
            effect_type,
            duration_remaining: duration,
            stacks: 1,
            magnitude,
            dot_dps,
            source_id,
        }
    }

    /// Check if the effect is still active
    pub fn is_active(&self) -> bool {
        self.duration_remaining > 0.0 && self.stacks > 0
    }

    /// Check if this is a damaging status effect
    pub fn is_damaging(&self) -> bool {
        use loot_core::types::StatusEffect;
        matches!(
            self.effect_type,
            StatusEffect::Poison | StatusEffect::Bleed | StatusEffect::Burn
        )
    }

    /// Get damage for a tick (damage = dot_dps * delta * stacks)
    pub fn tick_damage(&self, delta: f64) -> f64 {
        self.dot_dps * delta * self.stacks as f64
    }

    /// Tick the effect duration, returns damage dealt this tick
    pub fn tick(&mut self, delta: f64) -> f64 {
        let damage = if self.is_damaging() {
            self.tick_damage(delta)
        } else {
            0.0
        };
        self.duration_remaining -= delta;
        damage
    }

    /// Add a stack (also increases DoT damage proportionally)
    pub fn add_stack(&mut self, max_stacks: u32) {
        if self.stacks < max_stacks {
            self.stacks += 1;
        }
    }

    /// Refresh duration and update dot_dps if new is higher
    pub fn refresh(&mut self, duration: f64, dot_dps: f64) {
        self.duration_remaining = duration;
        if dot_dps > self.dot_dps {
            self.dot_dps = dot_dps;
        }
    }
}
