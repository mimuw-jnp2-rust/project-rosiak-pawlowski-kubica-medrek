use crate::bullet::{bullet_movement, on_collision_bullet, EnemyBulletBundle};
use crate::common::{EntityType, TextureWrapper};
use crate::health_system::{DeathEvent, HealthData, ModifyHealth, ReadDeaths, TakeDamageEvent};
use crate::hitbox::Hitbox;
use crate::move_system::MoveObjectType;
use crate::move_system::{
    CollisionEvent, HandleCollisionEvents, ModifyVelocity, MoveSystemObjectWithVelocity,
    VelocityVector,
};
use crate::player::{PlayerMarker, Speed};
use crate::{hitbox, AppState, Level, WINDOW_HEIGHT, WINDOW_WIDTH};
use bevy::ecs::event::Events;
use bevy::prelude::*;
use std::time::Duration;

use rand::{thread_rng, Rng};

const BULLET_START_SPEED: f32 = 10.;
const ENEMY_BULLET_SIZE: f32 = 5.;

const SIMPLE_ENEMY_OPTIONS: SpawnEnemyOptions = SpawnEnemyOptions {
    attack_ai: EnemyAttackAiType::None,
    size: (30., 30.),
    health: 3,
    speed: 42.,
};

const SHOOTING_ENEMY_OPTIONS: SpawnEnemyOptions = SpawnEnemyOptions {
    attack_ai: EnemyAttackAiType::RegularSingleShot(0.742),
    size: (30., 30.),
    health: 5,
    speed: 20.,
};

const SLOW_BIG_ENEMY_OPTIONS: SpawnEnemyOptions = SpawnEnemyOptions {
    attack_ai: EnemyAttackAiType::None,
    size: (50., 50.),
    health: 10,
    speed: 7.,
};

const BOSS_OPTIONS: SpawnEnemyOptions = SpawnEnemyOptions {
    attack_ai: EnemyAttackAiType::StarShaped(1., 7),
    size: (70., 70.),
    health: 20,
    speed: 15.,
};

const ENEMIES_ON_LEVEL: &[&[(usize, SpawnEnemyOptions)]] = &[
    &[(3, SIMPLE_ENEMY_OPTIONS)],
    &[(3, SHOOTING_ENEMY_OPTIONS), (2, SIMPLE_ENEMY_OPTIONS)],
    &[
        (1, BOSS_OPTIONS),
        (2, SIMPLE_ENEMY_OPTIONS),
        (2, SLOW_BIG_ENEMY_OPTIONS),
    ],
];

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Events::<SpawnEnemies>::default())
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(move_enemies.label(ModifyVelocity))
                    .with_system(enemies_take_damage.label(ModifyHealth))
                    .with_system(despawn_dead_enemies.label(ReadDeaths))
                    .with_system(control_bullets)
                    .with_system(bullet_movement.label(ModifyVelocity))
                    .with_system(
                        on_collision_bullet
                            .label(HandleCollisionEvents)
                            .label(ModifyHealth),
                    ),
            )
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_enemies))
            .add_system_to_stage(CoreStage::PostUpdate, spawn_enemies);
    }
}

#[derive(Component, Copy, Clone)]
pub struct EnemyMarker;

#[derive(Component, Clone, Copy)]
pub enum EnemyAttackAiType {
    None,
    RegularSingleShot(f32),
    StarShaped(f32, u16),
}

#[derive(Component, Clone)]
pub enum EnemyAttackAI {
    None,
    RegularSingleShot(Timer),
    StarShaped(Timer, u16),
}

// Create timer and tick it by random time from range (0, seconds).
fn get_timer_with_random_offset(seconds: f32) -> Timer {
    let mut timer = Timer::new(Duration::from_secs_f32(seconds), false);
    let offset = thread_rng().gen::<f32>() * seconds;
    timer.tick(Duration::from_secs_f32(offset));
    timer
}

impl EnemyAttackAI {
    pub fn new(from: EnemyAttackAiType) -> EnemyAttackAI {
        match from {
            EnemyAttackAiType::None => EnemyAttackAI::None,
            EnemyAttackAiType::RegularSingleShot(seconds) => {
                EnemyAttackAI::RegularSingleShot(get_timer_with_random_offset(seconds))
            }
            EnemyAttackAiType::StarShaped(seconds, amount) => {
                EnemyAttackAI::StarShaped(get_timer_with_random_offset(seconds), amount)
            }
        }
    }
}

#[derive(Bundle)]
pub struct EnemyBundle {
    marker: EnemyMarker,
    speed: Speed,
    #[bundle]
    move_system_bundle: MoveSystemObjectWithVelocity,
    health_data: HealthData,
    attack_ai: EnemyAttackAI,
}

impl EnemyBundle {
    pub fn new(
        hitbox: Hitbox,
        max_health: usize,
        attack_ai: EnemyAttackAI,
        speed: f32,
    ) -> EnemyBundle {
        EnemyBundle {
            marker: EnemyMarker,
            speed: Speed(speed),
            move_system_bundle: MoveSystemObjectWithVelocity::new_with_vel_0(
                MoveObjectType::Enemy,
                hitbox,
            ),
            health_data: HealthData::new_health(max_health),
            attack_ai,
        }
    }
}

// Spawn create enemy with given options and texture and place it in given location.
fn spawn_enemy(
    commands: &mut Commands,
    x: f32,
    y: f32,
    texture: Handle<Image>,
    options: SpawnEnemyOptions,
) {
    let (width, height) = options.size;
    commands
        .spawn_bundle(EnemyBundle::new(
            hitbox::Hitbox::new_rectangle(Vec2::new(width, height)),
            options.health,
            EnemyAttackAI::new(options.attack_ai),
            options.speed,
        ))
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(width, height)),
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

#[derive(Copy, Clone)]
pub struct SpawnEnemyOptions {
    attack_ai: EnemyAttackAiType,
    size: (f32, f32),
    health: usize,
    speed: f32,
}

pub struct SpawnEnemies {
    pub amount: usize,
    pub options: SpawnEnemyOptions,
}

fn get_random_free_position(other_objects: &Vec<(Vec2, Hitbox)>, hitbox: Hitbox) -> Vec2 {
    let get_random_position = || {
        Vec2::new(
            rand::thread_rng().gen_range(-(WINDOW_WIDTH / 2. - 50.)..(WINDOW_WIDTH / 2. - 50.)) as f32,
            rand::thread_rng().gen_range(-(WINDOW_HEIGHT / 2. - 50.)..(WINDOW_HEIGHT / 2. - 50.)) as f32,
        )
    };

    let check_position = |pos| {
        for (other_pos, other_hitbox) in other_objects.iter() {
            if Hitbox::check_collision(&hitbox, pos, other_hitbox, *other_pos) {
                // No place for object of this shape in this position.
                return false;
            }
        }
        true
    };

    loop {
        let random_position = get_random_position();
        if check_position(random_position) {
            return random_position;
        }
    }
}

// Read request to spawn enemy and realize theme.
fn spawn_enemies(
    mut commands: Commands,
    mut spawn_event: EventReader<SpawnEnemies>,
    other_objects: Query<(&Transform, &Hitbox, &MoveObjectType)>,
    textures: Res<Vec<TextureWrapper>>,
) {
    if spawn_event.is_empty() {
        return;
    }
    if let Some(enemy_texture) = textures.iter().find(|&x| x.owner_type == EntityType::Enemy) {
        let mut other_objects: Vec<_> = other_objects
            .iter()
            .filter(|(_, _, t)| {
                **t == MoveObjectType::Enemy
                    || **t == MoveObjectType::Player
                    || **t == MoveObjectType::Obstacle
            })
            .map(|(transform, hitbox, _)| (transform.translation.truncate(), hitbox.clone()))
            .collect();

        for request in spawn_event.iter() {
            let hitbox =
                Hitbox::new_rectangle(Vec2::new(request.options.size.0, request.options.size.1));
            for _ in 0..request.amount {
                let position = get_random_free_position(&other_objects, hitbox);
                other_objects.push((position, hitbox));
                spawn_enemy(
                    &mut commands,
                    position.x,
                    position.y,
                    enemy_texture.texture.clone(),
                    request.options,
                );
            }
        }
    }
}

// Update enemies velocity vectors so that move system can move them correctly.
fn move_enemies(
    mut enemies: Query<(&Transform, &mut VelocityVector, &Speed), With<EnemyMarker>>,
    player: Query<&Transform, With<PlayerMarker>>,
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

// Request spawning enemies at the beginning of each level.
fn setup_enemies(mut enemies: EventWriter<SpawnEnemies>, level: ResMut<Level>) {
    for to_spawn in ENEMIES_ON_LEVEL[level.current - 1].iter() {
        enemies.send(SpawnEnemies {
            amount: to_spawn.0,
            options: to_spawn.1,
        });
    }
}

// Despawn dead enemies and spawn bones texture in their place.
fn despawn_dead_enemies(
    mut commands: Commands,
    mut death_reader: EventReader<DeathEvent>,
    enemies: Query<(Entity, &Transform, &Sprite), With<EnemyMarker>>,
    textures: Res<Vec<TextureWrapper>>,
) {
    let texture_wrapper = textures
        .iter()
        .find(|&x| x.owner_type == EntityType::Bones)
        .unwrap();
    for death in death_reader.iter() {
        if let Ok((enemy, transform, sprite)) = enemies.get(death.id) {
            let x = transform.translation.x;
            let y = transform.translation.y;
            // Spawn bones in place of dead enemy.
            if let Some(enemy_size) = sprite.custom_size {
                commands.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(enemy_size),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.75),
                        ..Default::default()
                    },
                    texture: texture_wrapper.texture.clone(),
                    ..Default::default()
                });
            }
            commands.entity(enemy).despawn();
        }
    }
}

// Damage enemies after colliding with player bullet.
fn enemies_take_damage(
    mut collision_reader: EventReader<CollisionEvent>,
    mut damage_writer: EventWriter<TakeDamageEvent>,
    enemies: Query<Entity, With<EnemyMarker>>,
) {
    for collision in collision_reader.iter().filter(|col| {
        col.object_type == MoveObjectType::Enemy
            && col.collided_with_type == MoveObjectType::PlayerBullet
    }) {
        if let Ok(enemy) = enemies.get(collision.object_id) {
            damage_writer.send(TakeDamageEvent {
                id: enemy,
                amount: 1,
            });
        }
    }
}

// Auxiliary function spawning enemy bullet.
fn spawn_enemy_bullet(commands: &mut Commands, direction: Vec2, position: Vec2) {
    let bullet_size = Vec2::new(ENEMY_BULLET_SIZE, ENEMY_BULLET_SIZE);
    let bullet_bundle = EnemyBulletBundle::new(
        hitbox::Hitbox::new_rectangle(bullet_size),
        direction * BULLET_START_SPEED,
    );
    let sprite_bundle = SpriteBundle {
        sprite: Sprite {
            color: Color::CRIMSON,
            custom_size: Some(bullet_size),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(position.x, position.y, 2.0),
            ..Default::default()
        },
        ..Default::default()
    };
    commands
        .spawn_bundle(bullet_bundle)
        .insert_bundle(sprite_bundle);
}

// Spawn enemy bullets accordingly to each enemy attack AI.
pub fn control_bullets(
    mut commands: Commands,
    mut enemy: Query<(&Transform, &mut EnemyAttackAI), With<EnemyMarker>>,
    player_tf: Query<&Transform, With<PlayerMarker>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_tf.get_single() {
        let player_position = player_transform.translation.truncate();
        for (enemy_tf, mut attack_ai) in enemy.iter_mut() {
            let enemy_position = enemy_tf.translation.truncate();
            match attack_ai.as_mut() {
                EnemyAttackAI::None => (),
                EnemyAttackAI::RegularSingleShot(ref mut shot_cool_down) => {
                    shot_cool_down.tick(time.delta());
                    if shot_cool_down.finished() {
                        shot_cool_down.reset();
                        let shoot_direction = Vec2::new(
                            player_position.x - enemy_position.x,
                            player_position.y - enemy_position.y,
                        )
                        .normalize_or_zero();
                        if shoot_direction != Vec2::ZERO {
                            spawn_enemy_bullet(&mut commands, shoot_direction, enemy_position);
                        }
                    }
                }
                EnemyAttackAI::StarShaped(ref mut shot_cool_down, amount) => {
                    let amount = *amount;
                    shot_cool_down.tick(time.delta());
                    if shot_cool_down.finished() {
                        shot_cool_down.reset();
                        let angle = (360. as f32).to_radians() / amount as f32;
                        for i in 0..amount {
                            let degrees_direction = angle * i as f32;
                            let direction =
                                Vec2::new(degrees_direction.sin(), degrees_direction.cos())
                                    .normalize_or_zero();
                            let bullet_position = enemy_position + direction * ENEMY_BULLET_SIZE;
                            spawn_enemy_bullet(&mut commands, direction, bullet_position);
                        }
                    }
                }
            }
        }
    }
}
