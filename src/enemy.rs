use bevy::ecs::event::Events;
use bevy::prelude::*;

use crate::common::{EntityType, TextureWrapper};
use crate::move_system::{
    CollisionEvent, ModifyVelocity, MoveSystemObjectWithVelocity, VelocityVector,
};
use crate::player::{BulletMarker, PlayerMarker, Speed};

use crate::health_system::{DeathEvent, HealthData, ModifyHealth, ReadDeaths, TakeDamageEvent};
use crate::hitbox::Hitbox;
use crate::move_system::MoveObjectType;
use crate::{hitbox, player, AppState, Player};

use rand::Rng;

const ENEMY_START_SPEED: f32 = 20.0;
const ENEMY_SIZE: f32 = 20.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub struct SpawnEnemy;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Events::<SpawnEnemies>::default())
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(spawn_enemies.label(SpawnEnemy))
                    .with_system(move_enemies.label(ModifyVelocity))
                    .with_system(enemies_take_damage.label(ModifyHealth))
                    .with_system(despawn_dead_enemies.label(ReadDeaths)),
            )
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_enemies));
    }
}

#[derive(Component, Copy, Clone)]
pub struct EnemyMarker;

#[derive(Bundle)]
pub struct EnemyBundle {
    marker: EnemyMarker,
    speed: Speed,
    #[bundle]
    move_system_bundle: MoveSystemObjectWithVelocity,
    health_data: HealthData,
}

impl EnemyBundle {
    pub fn new(hitbox: Hitbox, max_health: usize) -> EnemyBundle {
        EnemyBundle {
            marker: EnemyMarker,
            speed: Speed(ENEMY_START_SPEED),
            move_system_bundle: MoveSystemObjectWithVelocity::new_with_vel_0(
                MoveObjectType::Enemy,
                hitbox,
            ),
            health_data: HealthData::new_healthy(max_health),
        }
    }
}

fn spawn_enemy(commands: &mut Commands, x: f32, y: f32, texture: Handle<Image>) {
    commands
        .spawn_bundle(EnemyBundle::new(
            hitbox::Hitbox::new_rectangle(Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
            20,
        ))
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(x, y, 1.0),
                ..Default::default()
            },
            texture,
            ..Default::default()
        });
}

pub struct SpawnEnemies(pub usize);

fn spawn_enemies(
    mut commands: Commands,
    mut spawn_event: EventReader<SpawnEnemies>,
    player: Query<(&Transform), (With<PlayerMarker>)>,
    textures: Res<Vec<TextureWrapper>>,
) {
    let get_random_position = || {
        (
            rand::thread_rng().gen_range(-400.0..400.0) as f32,
            rand::thread_rng().gen_range(-220.0..220.0) as f32,
        )
    };

    let get_position = |x: f32, y: f32| loop {
        let (x_, y_) = get_random_position();
        let (x_diff, y_diff) = (x - x_, y - y_);
        if x_diff.abs() + y_diff.abs() > 100.0 {
            return (x_, y_);
        }
    };

    if let Some(enemy_texture) = textures.iter().find(|&x| x.owner_type == EntityType::Enemy) {
        let mut spawn_one = |position: (f32, f32)| {
            spawn_enemy(
                &mut commands,
                position.0,
                position.1,
                enemy_texture.texture.clone(),
            );
        };

        let enemies_count = spawn_event.iter().map(|SpawnEnemies(count)| count).sum();

        if let Ok(player_transform) = player.get_single() {
            let (player_x, player_y) = (
                player_transform.translation.x,
                player_transform.translation.y,
            );
            for _ in 0..enemies_count {
                spawn_one(get_position(player_x, player_y));
            }
        } else {
            for _ in 0..enemies_count {
                spawn_one(get_random_position());
            }
        }
    } else {
        panic!()
    }
}

fn move_enemies(
    mut enemies: Query<(&Transform, &mut VelocityVector, &Speed), (With<EnemyMarker>)>,
    player: Query<(&Transform), (With<PlayerMarker>)>,
) {
    if let Ok(player_transform) = player.get_single() {
        let player_position = (
            player_transform.translation.x,
            player_transform.translation.y,
        );
        for (position, mut vel, &Speed(speed)) in enemies.iter_mut() {
            let enemy_position = (position.translation.x, position.translation.y);
            let translation = Vec2::new(
                player_position.0 - enemy_position.0,
                player_position.1 - enemy_position.1,
            )
            .normalize_or_zero();
            vel.0 += translation * speed;
        }
    }
}

fn setup_enemies(mut enemies: EventWriter<SpawnEnemies>) {
    enemies.send(SpawnEnemies(3));
}

fn despawn_dead_enemies(
    mut commands: Commands,
    mut death_reader: EventReader<DeathEvent>,
    mut enemies: Query<(Entity), (With<EnemyMarker>)>,
) {
    for death in death_reader.iter() {
        if let Ok(enemy) = enemies.get(death.id) {
            commands.entity(enemy).despawn();
        }
    }
}

fn enemies_take_damage(
    mut collision_reade: EventReader<CollisionEvent>,
    mut damage_writer: EventWriter<TakeDamageEvent>,
    mut enemies: Query<(Entity), (With<EnemyMarker>)>,
    mut bullets: Query<(Entity), (With<BulletMarker>)>,
) {
    for collision in collision_reade.iter() {
        if let (Ok(enemie), Ok(bullet)) = (
            enemies.get(collision.object_id),
            bullets.get(collision.collided_with_id),
        ) {
            damage_writer.send(TakeDamageEvent {
                id: enemie,
                amount: 1,
            });
        }
    }
}
