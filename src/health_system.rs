use crate::bullet::{EnemyBulletMarker, PlayerBulletMarker};
use crate::AppState;
use bevy::prelude::*;
use bevy::utils::HashSet;
use std::cmp::min;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamageEvent>();
        app.add_event::<DeathEvent>();
        app.add_event::<HealEvent>();
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(health_system.after(ModifyHealth).before(ReadDeaths))
                .with_system(display_health),
        );
    }
}

// Tag component used to label systems that modify health.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub struct ModifyHealth;

// Tag component used to label systems that read death events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub struct ReadDeaths;

// Types of events related to health.

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

// Struct holding information about health of an entity.
#[derive(Component, Clone)]
pub struct HealthData {
    pub max_health: usize,
    pub current_health: usize,
}

impl HealthData {
    pub fn new_health(max_health: usize) -> HealthData {
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

    // Decreases health, returns true if health goes to zero.
    fn take_damage(&mut self, amount: usize) -> bool {
        if self.current_health <= amount {
            self.current_health = 0;
            return true;
        }
        self.current_health -= amount;
        false
    }
}

// System responsible for editing health level for entities based on events.
fn health_system(
    mut damage_reader: EventReader<TakeDamageEvent>,
    mut heal_reader: EventReader<HealEvent>,
    mut death_writer: EventWriter<DeathEvent>,
    mut entities: Query<&mut HealthData>,
) {
    for heal_event in heal_reader.iter() {
        if let Ok(mut to_heal) = entities.get_mut(heal_event.id) {
            to_heal.heal(heal_event.amount);
        }
    }

    let mut dead_entities = HashSet::new();
    for damage_event in damage_reader.iter() {
        if let Ok(mut to_damage) = entities.get_mut(damage_event.id) {
            if to_damage.take_damage(damage_event.amount) {
                dead_entities.insert(damage_event.id);
            }
        }
    }
    for id in dead_entities {
        death_writer.send(DeathEvent { id });
    }
}

// Sets red tint representing remaining health.
fn display_health(
    mut entities_with_health: Query<
        (&mut Sprite, &HealthData),
        (Without<PlayerBulletMarker>, Without<EnemyBulletMarker>),
    >,
) {
    for (mut sprite, health_data) in entities_with_health.iter_mut() {
        let health_percentage = health_data.current_health as f32 / health_data.max_health as f32;
        let color = Color::WHITE * health_percentage + Color::RED * (1. - health_percentage);
        sprite.color = color;
    }
}
