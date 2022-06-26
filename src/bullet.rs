use crate::health_system::{DeathEvent, HealthData, ReadDeaths, TakeDamageEvent};
use crate::move_system::{
    CollisionEvent, MoveObjectType, MoveSystemObjectWithVelocity, VelocityVector,
};
use crate::player::Speed;
use crate::{AppState, Hitbox};
use bevy::prelude::*;

const BULLET_START_SPEED: f32 = 10.;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(despawn_bullets_after_collision.label(ReadDeaths)),
        );
    }
}

#[derive(Component, Copy, Clone)]
pub struct PlayerBulletMarker;

#[derive(Component, Copy, Clone)]
pub struct EnemyBulletMarker;

#[derive(Component)]
pub struct DirectionVector(pub Vec2);

#[derive(Bundle)]
pub struct PlayerBulletBundle {
    marker: PlayerBulletMarker,
    speed: Speed,
    direction: DirectionVector,
    #[bundle]
    move_system_bundle: MoveSystemObjectWithVelocity,
    health_data: HealthData,
}

impl PlayerBulletBundle {
    pub fn new(hitbox: Hitbox, vel: Vec2) -> PlayerBulletBundle {
        PlayerBulletBundle {
            marker: PlayerBulletMarker,
            speed: Speed(BULLET_START_SPEED),
            direction: DirectionVector(vel),
            move_system_bundle: MoveSystemObjectWithVelocity::new_with_vel_0(
                MoveObjectType::PlayerBullet,
                hitbox,
            ),
            health_data: HealthData::new_health(1),
        }
    }
}

#[derive(Bundle)]
pub struct EnemyBulletBundle {
    marker: EnemyBulletMarker,
    speed: Speed,
    direction: DirectionVector,
    #[bundle]
    move_system_bundle: MoveSystemObjectWithVelocity,
    health_data: HealthData,
}

impl EnemyBulletBundle {
    pub fn new(hitbox: Hitbox, vel: Vec2) -> EnemyBulletBundle {
        EnemyBulletBundle {
            marker: EnemyBulletMarker,
            speed: Speed(BULLET_START_SPEED),
            direction: DirectionVector(vel),
            move_system_bundle: MoveSystemObjectWithVelocity::new_with_vel_0(
                MoveObjectType::EnemyBullet,
                hitbox,
            ),
            health_data: HealthData::new_health(1),
        }
    }
}

pub fn spawn_bullets<B>(
    commands: &mut Commands,
    bullet_bundle: B,
    sprite_bundle: SpriteBundle,
    timer: &mut ResMut<Timer>,
) where
    B: Bundle,
{
    if timer.finished() {
        commands
            .spawn_bundle(bullet_bundle)
            .insert_bundle(sprite_bundle);
        timer.reset();
    }
}

pub fn bullet_movement(
    mut query: Query<
        (&DirectionVector, &mut VelocityVector, &Speed),
        Or<(With<EnemyBulletMarker>, With<PlayerBulletMarker>)>,
    >,
) {
    for (direction, mut vel, &Speed(speed)) in query.iter_mut() {
        let new_vel = direction.0 * speed;
        vel.0 += new_vel;
    }
}

pub fn on_collision_bullet(
    mut collision_reader: EventReader<CollisionEvent>,
    query_bullet: Query<Entity, Or<(With<EnemyBulletMarker>, With<PlayerBulletMarker>)>>,
    mut damage_writer: EventWriter<TakeDamageEvent>,
) {
    for collision in collision_reader.iter() {
        let bullet = query_bullet.get(collision.object_id);
        if let Ok(bullet_entity) = bullet {
            // commands.entity(bullet_entity).despawn();
            damage_writer.send(TakeDamageEvent {
                id: bullet_entity,
                amount: 1,
            });
        }
    }
}

pub fn despawn_bullets_after_collision(
    mut commands: Commands,
    mut death_reader: EventReader<DeathEvent>,
    bullets: Query<Entity, Or<(With<EnemyBulletMarker>, With<PlayerBulletMarker>)>>,
) {
    for death in death_reader.iter() {
        if let Ok(bullet) = bullets.get(death.id) {
            commands.entity(bullet).despawn();
        }
    }
}
