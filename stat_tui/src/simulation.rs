//! Combat simulation utilities

use stat_core::{
    combat::resolve_damage,
    damage::{calculate_damage, DamagePacketGenerator},
    stat_block::StatBlock,
};
use rand::Rng;

/// Run a DPS simulation
pub struct DpsSimulation {
    pub total_damage: f64,
    pub total_time: f64,
    pub hit_count: u32,
    pub crit_count: u32,
    pub kill_time: Option<f64>,
}

impl DpsSimulation {
    /// Simulate attacking a target for a duration
    pub fn run(
        attacker: &StatBlock,
        defender: &mut StatBlock,
        skill: &DamagePacketGenerator,
        duration: f64,
        rng: &mut impl Rng,
    ) -> Self {
        let mut result = DpsSimulation {
            total_damage: 0.0,
            total_time: 0.0,
            hit_count: 0,
            crit_count: 0,
            kill_time: None,
        };

        // Get attack speed
        let attack_time = if skill.is_attack() {
            1.0 / (attacker.computed_attack_speed() * skill.attack_speed_modifier)
        } else {
            1.0 / (attacker.computed_cast_speed() * skill.attack_speed_modifier)
        };

        let mut time = 0.0;
        let initial_life = defender.current_life;

        while time < duration && defender.is_alive() {
            // Generate attack
            let packet = calculate_damage(
                attacker,
                skill,
                "simulator".to_string(),
                rng,
            );

            result.hit_count += 1;
            if packet.is_critical {
                result.crit_count += 1;
            }

            // Apply damage
            let (new_defender, combat_result) = resolve_damage(defender, &packet);
            *defender = new_defender;
            result.total_damage += combat_result.total_damage;

            // Check for kill
            if !defender.is_alive() && result.kill_time.is_none() {
                result.kill_time = Some(time);
            }

            time += attack_time;
        }

        result.total_time = time.min(duration);

        // Restore defender for repeated tests
        defender.current_life = initial_life;
        defender.clear_effects();

        result
    }

    /// Calculate DPS
    pub fn dps(&self) -> f64 {
        if self.total_time > 0.0 {
            self.total_damage / self.total_time
        } else {
            0.0
        }
    }

    /// Calculate crit rate
    pub fn crit_rate(&self) -> f64 {
        if self.hit_count > 0 {
            self.crit_count as f64 / self.hit_count as f64 * 100.0
        } else {
            0.0
        }
    }

    /// Calculate average damage per hit
    pub fn avg_damage(&self) -> f64 {
        if self.hit_count > 0 {
            self.total_damage / self.hit_count as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_dps_simulation() {
        let mut attacker = StatBlock::new();
        attacker.weapon_physical_min = 50.0;
        attacker.weapon_physical_max = 50.0;
        attacker.weapon_attack_speed = 1.0;

        let mut defender = StatBlock::new();
        defender.max_life.base = 10000.0;
        defender.current_life = 10000.0;

        let skill = DamagePacketGenerator::basic_attack();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let result = DpsSimulation::run(
            &attacker,
            &mut defender,
            &skill,
            10.0,
            &mut rng,
        );

        assert!(result.hit_count > 0);
        assert!(result.total_damage > 0.0);
        assert!(result.dps() > 0.0);
    }
}
