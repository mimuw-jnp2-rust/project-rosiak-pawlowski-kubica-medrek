use crate::AppState;
use bevy::prelude::*;
use std::cmp;
use std::cmp::min;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamageEvent>();
        app.add_event::<DeathEvent>();
        app.add_event::<HealEvent>();
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(health_system.after(ModifyHealth).before(ReadDeaths)),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub struct ModifyHealth;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub struct ReadDeaths;

pub struct TakeDamageEvent {
    pub id: Entity,
    pub amount: usize,
}

pub struct HealEvent {
    pub id: Entity,
    pub amount: usize,
}

pub struct DeathEvent {
    pub id: Entity,
}

#[derive(Component, Clone)]
pub struct HealthData {
    pub max_health: usize,
    pub current_health: usize,
}

impl HealthData {
    pub fn new_healthy(max_health: usize) -> HealthData {
        HealthData {
            max_health,
            current_health: max_health,
        }
    }
}

impl HealthData {
    fn heal(&mut self, amount: usize) {
        self.current_health = min(self.max_health, self.current_health + amount);
    }

    // Decreases health, if health goes to zero returns true
    fn take_damage(&mut self, amount: usize) -> bool {
        if self.current_health <= amount {
            self.current_health = 0;
            return true;
        }
        self.current_health -= amount;
        false
    }
}

fn health_system(
    mut damage_reader: EventReader<TakeDamageEvent>,
    mut heal_reader: EventReader<HealEvent>,
    mut death_writer: EventWriter<DeathEvent>,
    mut entities: Query<(&mut HealthData)>,
) {
    for heal_event in heal_reader.iter() {
        if let Ok(mut to_heal) = entities.get_mut(heal_event.id) {
            to_heal.heal(heal_event.amount);
        }
    }

    for damage_event in damage_reader.iter() {
        if let Ok(mut to_damage) = entities.get_mut(damage_event.id) {
            if to_damage.take_damage(damage_event.amount) {
                death_writer.send(DeathEvent {
                    id: damage_event.id,
                });
            }
        }
    }
}
