use crate::bullet::{
    bullet_movement, on_collision_bullet, spawn_bullets, EnemyBulletMarker, PlayerBulletBundle,
};
use crate::common::{EntityType, TextureWrapper};
use crate::enemy::EnemyMarker;
use crate::health_system::{DeathEvent, HealthData, ModifyHealth, ReadDeaths, TakeDamageEvent};
use crate::hitbox::Hitbox;
use crate::move_system::{
    CollisionEvent, HandleCollisionEvents, ModifyVelocity, MoveObjectType,
    MoveSystemObjectWithVelocity, VelocityVector,
};
use crate::{hitbox, player, AppState};
use bevy::prelude::*;
use std::borrow::BorrowMut;

const PLAYER_START_SPEED: f32 = 100.;
const BULLET_START_SPEED: f32 = 15.;
const PLAYER_START_HEALTH: usize = 42;

const PLAYER_BULLET_SIZE: f32 = 5.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(control_player.label(ModifyVelocity))
                .with_system(bullet_movement.label(ModifyVelocity))
                .with_system(control_bullets)
                .with_system(
                    on_collision_bullet
                        .label(HandleCollisionEvents)
                        .label(ModifyHealth),
                )
                .with_system(
                    player_takes_damage
                        .label(ModifyHealth)
                        .label(HandleCollisionEvents),
                )
                .with_system(player_dies.label(ReadDeaths)),
        )
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(spawn_player))
        .add_system_set(SystemSet::on_exit(AppState::GameEnded).with_system(spawn_player))
        .add_system_set(SystemSet::on_exit(AppState::PlayerDied).with_system(spawn_player))
        .add_system_set(SystemSet::on_enter(AppState::GameEnded).with_system(despawn_player))
        .add_system_set(SystemSet::on_enter(AppState::PlayerDied).with_system(despawn_player));
    }
}

#[derive(Component, Copy, Clone)]
pub struct PlayerMarker;

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
            health_data: HealthData::new_health(PLAYER_START_HEALTH),
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
    let texture_wrapper = wrapper.unwrap();
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
}

pub fn despawn_player(mut commands: Commands, player_query: Query<Entity, With<PlayerMarker>>) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn control_player(
    mut player: Query<(&mut VelocityVector, &Speed), With<PlayerMarker>>,
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
    players: Query<Entity, With<PlayerMarker>>,
    enemies: Query<Entity, With<EnemyMarker>>,
    bullets: Query<Entity, With<EnemyBulletMarker>>,
) {
    for collision in collision_reade.iter() {
        if let (Ok(player), Ok(_)) = (
            players.get(collision.object_id),
            bullets.get(collision.collided_with_id),
        ) {
            damage_writer.send(TakeDamageEvent {
                id: player,
                amount: 1,
            });
        } else if let (Ok(player), Ok(_)) = (
            players.get(collision.object_id),
            enemies.get(collision.collided_with_id),
        ) {
            damage_writer.send(TakeDamageEvent {
                id: player,
                amount: 1,
            });
        }
    }
}

fn player_dies(
    mut death_reader: EventReader<DeathEvent>,
    mut players: Query<(Entity, &mut Sprite), With<PlayerMarker>>,
    mut state: ResMut<State<AppState>>,
) {
    for death in death_reader.iter() {
        if let Ok((_, mut sprite)) = players.get_mut(death.id) {
            info!("Zed's Dead");
            sprite.color = Color::PINK;
            state
                .set(AppState::PlayerDied)
                .expect("Couldn't switch state from InGame");
        }
    }
}

pub fn control_bullets(
    mut commands: Commands,
    mut player: Query<&Transform, With<PlayerMarker>>,
    mut player_timer: ResMut<Timer>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    player_timer.tick(time.delta());
    if let Ok(player_tf) = player.get_single_mut() {
        let shoot_direction = get_direction_from_keyboard(
            &keyboard_input,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
        );

        if shoot_direction != Vec2::new(0., 0.) {
            let bullet_size = Vec2::new(PLAYER_BULLET_SIZE, PLAYER_BULLET_SIZE);
            let bullet_bundle = PlayerBulletBundle::new(
                hitbox::Hitbox::new_rectangle(bullet_size),
                shoot_direction * BULLET_START_SPEED,
            );
            let sprite_bundle = SpriteBundle {
                sprite: Sprite {
                    color: Color::BLUE,
                    custom_size: Some(bullet_size),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(player_tf.translation.x, player_tf.translation.y, 2.0),
                    ..Default::default()
                },
                ..Default::default()
            };
            spawn_bullets(
                commands.borrow_mut(),
                bullet_bundle,
                sprite_bundle,
                player_timer.borrow_mut(),
            );
        }
    }
}
