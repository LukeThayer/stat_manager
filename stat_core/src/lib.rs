//! stat_core - Core stat management library for game entities
//!
//! This library provides:
//! - StatBlock: Aggregated stats from multiple sources
//! - DamagePacketGenerator: Skill/ability damage configuration
//! - DamagePacket: Calculated damage output
//! - Damage Resolution: Processing incoming damage against defenses

pub mod combat;
pub mod config;
pub mod damage;
pub mod defense;
pub mod dot;
pub mod source;
pub mod stat_block;
pub mod types;

// Re-export core types for convenience
pub use combat::{CombatResult, DamageTaken};
pub use defense::calculate_damage_cap;
pub use damage::{
    BaseDamage, DamagePacket, DamagePacketGenerator, DotApplication, FinalDamage, PendingDoT,
    PendingStatusEffect,
};
pub use dot::{ActiveDoT, DotConfig, DotStacking};
pub use source::{BaseStatsSource, BuffSource, GearSource, SkillTreeSource, StatSource};
pub use stat_block::{StatAccumulator, StatBlock, StatValue, StatusConversions, StatusEffectStats, StatusEffectData};
pub use types::{ActiveBuff, ActiveStatusEffect, EquipmentSlot, SkillNodeId, SkillTag};
pub use config::default_skills;

// Re-export loot_core types for convenience
pub use loot_core::types::{Attribute, DamageType, DefenseType, ItemClass, Rarity, StatType, StatusEffect};
pub use loot_core::item::Modifier;
pub use loot_core::Item;
