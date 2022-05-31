use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use std::time::Duration;

use crate::common::{EntityType, TextureWrapper};
use crate::enemy::EnemyMarker;
use crate::health_system::{DeathEvent, HealthData, ModifyHealth, ReadDeaths, TakeDamageEvent};
use crate::hitbox::Hitbox;
use crate::move_system::MoveObjectType::{Enemy, PlayerBullet};
use crate::move_system::{
    CollisionEvent, HandleCollisionEvents, ModifyVelocity, MoveObjectType,
    MoveSystemObjectWithVelocity, VelocityVector,
};
use crate::window::WinSize;
use crate::{hitbox, player, AppState, Player};

const PLAYER_START_SPEED: f32 = 100.;
const BULLET_START_SPEED: f32 = 5.;
const PLAYER_START_HEALTH: usize = 42;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(control_player.label(ModifyVelocity))
                .with_system(bullet_movement.label(ModifyVelocity))
                .with_system(control_bullets)
                .with_system(on_collision_bullet.label(HandleCollisionEvents))
                .with_system(player_takes_damage.label(ModifyHealth))
                .with_system(player_dies.label(ReadDeaths)),
        )
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(spawn_player));
    }
}

#[derive(Component, Copy, Clone)]
pub struct PlayerMarker;

#[derive(Component, Copy, Clone)]
pub struct BulletMarker;

#[derive(Component)]
pub struct DirectionVector(pub Vec2);

#[derive(Component, Copy, Clone)]
pub struct Speed(pub f32);

#[derive(Bundle)]
pub struct PlayerBundle {
    marker: PlayerMarker,
    speed: Speed,
    #[bundle]
    move_system_bundle: MoveSystemObjectWithVelocity,
    health_data: HealthData,
}

impl PlayerBundle {
    pub fn new(hitbox: Hitbox) -> PlayerBundle {
        PlayerBundle {
            marker: PlayerMarker,
            speed: Speed(PLAYER_START_SPEED),
            move_system_bundle: MoveSystemObjectWithVelocity::new_with_vel_0(
                MoveObjectType::Player,
                hitbox,
            ),
            health_data: HealthData::new_healthy(PLAYER_START_HEALTH),
        }
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    marker: BulletMarker,
    speed: Speed,
    direction: DirectionVector,
    #[bundle]
    move_system_bundle: MoveSystemObjectWithVelocity,
}

impl BulletBundle {
    pub fn new(hitbox: Hitbox, vel: Vec2) -> BulletBundle {
        BulletBundle {
            marker: BulletMarker,
            speed: Speed(BULLET_START_SPEED),
            direction: DirectionVector(vel),
            move_system_bundle: MoveSystemObjectWithVelocity::new_with_vel_0(
                MoveObjectType::PlayerBullet,
                hitbox,
            ),
        }
    }
}

fn get_direction_from_keyboard(
    keyboard_input: &Res<Input<KeyCode>>,
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
) -> Vec2 {
    let mut x = 0.;
    if keyboard_input.pressed(left) {
        x -= 1.;
    }
    if keyboard_input.pressed(right) {
        x += 1.;
    }
    let mut y = 0.;
    if keyboard_input.pressed(down) {
        y -= 1.;
    }
    if keyboard_input.pressed(up) {
        y += 1.;
    }
    Vec2::new(x, y).normalize_or_zero()
}

pub fn spawn_player(mut commands: Commands, textures: Res<Vec<TextureWrapper>>) {
    let wrapper = textures
        .iter()
        .find(|&x| x.owner_type == EntityType::Player);
    if let Some(texture_wrapper) = wrapper {
        // Spawning player.
        commands
            .spawn_bundle(player::PlayerBundle::new(hitbox::Hitbox::new_rectangle(
                Vec2::new(30., 30.),
            )))
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(30., 30.)),
                    ..Default::default()
                },
                texture: texture_wrapper.texture.clone(),
                transform: Transform {
                    translation: Vec3::new(200., 0., 1.0),
                    ..Default::default()
                },
                ..Default::default()
            });
    } else {
        panic!()
    }
}

pub fn control_player(
    mut player: Query<(&mut VelocityVector, &Speed), (With<PlayerMarker>)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Ok((mut vel, &Speed(speed))) = player.get_single_mut() {
        let direction = get_direction_from_keyboard(
            &keyboard_input,
            KeyCode::W,
            KeyCode::S,
            KeyCode::A,
            KeyCode::D,
        );
        let new_vel = direction * speed;
        vel.0 += new_vel;
    }
}

fn player_takes_damage(
    mut collision_reade: EventReader<CollisionEvent>,
    mut damage_writer: EventWriter<TakeDamageEvent>,
    mut players: Query<(Entity), (With<PlayerMarker>)>,
    mut enemies: Query<(Entity), (With<EnemyMarker>)>,
    mut bullets: Query<(Entity), (With<BulletMarker>)>,
) {
    for collision in collision_reade.iter() {
        if let (Ok(player), Ok(bullet)) = (
            players.get(collision.object_id),
            bullets.get(collision.collided_with_id),
        ) {
            damage_writer.send(TakeDamageEvent {
                id: player,
                amount: 1,
            });
        } else if let (Ok(player), Ok(enemie)) = (
            players.get(collision.object_id),
            enemies.get(collision.collided_with_id),
        ) {
            // Now its the same as for collision with bullet,
            // but in the futer calculating damage amount might be different
            damage_writer.send(TakeDamageEvent {
                id: player,
                amount: 1,
            });
        }
    }
}

fn player_dies(
    mut commands: Commands,
    mut death_reader: EventReader<DeathEvent>,
    mut players: Query<(Entity), (With<PlayerMarker>)>,
) {
    for death in death_reader.iter() {
        if let Ok(plyer) = players.get(death.id) {
            info!("Zed's Dead");
        }
    }
}

pub fn control_bullets(
    mut commands: Commands,
    mut player: Query<&Transform, With<PlayerMarker>>,
    mut timer: ResMut<Timer>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    timer.tick(time.delta());
    if let Ok(mut player_tf) = player.get_single_mut() {
        let shoot_direction = get_direction_from_keyboard(
            &keyboard_input,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
        );

        if shoot_direction != Vec2::new(0., 0.) {
            let mut spawn_bullets = || {
                commands
                    .spawn_bundle(BulletBundle::new(
                        hitbox::Hitbox::new_rectangle(Vec2::new(7., 25.)),
                        shoot_direction * BULLET_START_SPEED,
                    ))
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::from([30., 130., 240.]),
                            custom_size: Some(Vec2::new(7., 25.)),
                            ..Default::default()
                        },
                        transform: Transform {
                            translation: Vec3::new(
                                player_tf.translation.x,
                                player_tf.translation.y,
                                2.0,
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
            };

            // przyda się do wyświetlania kiedyś potencjalnie, fajnie wiedzieć kiedy można strzelić
            // println!("Cooldown percent: {}", 200. * timer.elapsed_secs());
            if timer.finished() {
                spawn_bullets();
                timer.reset();
            }
        }
    }
}

// W przyszłości można wyciągnąć do pliku bullet.rs

fn bullet_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<
        (
            Entity,
            &DirectionVector,
            &mut VelocityVector,
            &Speed,
            &Transform,
        ),
        (With<BulletMarker>),
    >,
) {
    for (bullet_entity, direction, mut vel, &Speed(speed), bullet_tf) in query.iter_mut() {
        let new_vel = direction.0 * speed;
        vel.0 += new_vel;

        // Delete entity if it's outside of the window.
        let translation = &bullet_tf.translation;
        if translation.y > win_size.height || translation.x > win_size.width {
            commands.entity(bullet_entity).despawn();
        }
    }
}

fn on_collision_bullet(
    mut commands: Commands,
    mut collision_reader: EventReader<CollisionEvent>,
    query_bullet: Query<(Entity), (With<BulletMarker>)>,
    query_entity: Query<(Entity), (Or<(With<EnemyMarker>, With<PlayerMarker>)>)>, // , With<EnemyMarker>
) {
    for collision in collision_reader
        .iter()
        .filter(|c_ev| c_ev.object_type == PlayerBullet)
    {
        let bullet = query_bullet.get(collision.object_id);
        // let entity = query_entity.get(collision.collided_with_id);
        if let Ok((bullet_entity)) = bullet {
            // Enemie and Player should handle their damage themselves
            // if collision.collided_with_type == Player {
            //     // Decrement health for player.
            // } else if collision.collided_with_type == Enemy {
            //     // Decrement health for enemy.
            // }
            commands.entity(bullet_entity).despawn(); /* TODO May despawn bullet before other systems
                                                       * that need to access it, will have
                                                       * a chance to do so. */
            // TODO Consider giving bullet 1 HP and handling its death
        }
    }
}
