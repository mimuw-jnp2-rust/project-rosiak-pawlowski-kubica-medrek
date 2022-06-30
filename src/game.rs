use crate::bullet::BulletPlugin;
use crate::enemy::{EnemyMarker, EnemyPlugin, SpawnEnemies};
use crate::health_system::{HealEvent, HealthPlugin};
use crate::player::{PlayerMarker, PlayerPlugin};
use crate::{map, AppState, Level, LoadMap, MoveSystemPlugin, RenderMap, MAX_LEVEL};
use bevy::prelude::*;

// After finishing a level player gets healed by PLAYER_BOOST_HEALTH health points.
const PLAYER_BOOST_HEALTH: usize = 20;

// Game camera entity saved in resources to despawn it easily.
struct GameCamera {
    camera_entity: Entity,
}

// Plugin responsible for handling the course of the game.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(reset_level_number))
            .add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(spawn_map)
                    .with_system(spawn_camera),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame).with_system(check_if_player_finished_level),
            )
            .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(despawn_camera))
            .add_system_set(SystemSet::on_enter(AppState::NextLevel).with_system(next_level))
            .add_plugin(MoveSystemPlugin)
            .add_plugin(HealthPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(EnemyPlugin)
            .add_plugin(BulletPlugin);
    }
}

// System spawning the game camera and saving it in resources.
fn spawn_camera(mut commands: Commands) {
    let camera_entity = commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .id();
    commands.insert_resource(GameCamera { camera_entity });
}

// System despawning the game camera saved in resources.
fn despawn_camera(mut commands: Commands, game_camera: Res<GameCamera>) {
    commands
        .entity(game_camera.camera_entity)
        .despawn_recursive();
}

// Spawns map for given level.
fn spawn_map(
    mut load_map: EventWriter<LoadMap>,
    mut render_map: EventWriter<RenderMap>,
    level: ResMut<Level>,
) {
    load_map.send(map::LoadMap(level.current.try_into().unwrap()));
    render_map.send(map::RenderMap(level.current.try_into().unwrap()));
}

// System responsible for handling finishing a level.
fn check_if_player_finished_level(
    mut state: ResMut<State<AppState>>,
    player: Query<Entity, With<PlayerMarker>>,
    enemies: Query<Entity, With<EnemyMarker>>,
    mut spawn_event: EventReader<SpawnEnemies>,
    mut healing_writer: EventWriter<HealEvent>,
) {
    let no_spawned_enemies = enemies.is_empty();
    let mut no_enemies_to_spawn = true;

    // Check if there are any enemies left to spawn.
    for _event in spawn_event.iter() {
        no_enemies_to_spawn = false;
    }

    // Player finished the level.
    if no_spawned_enemies && no_enemies_to_spawn {
        // Heal the player.
        if let Ok(player_entity) = player.get_single() {
            healing_writer.send(HealEvent {
                id: player_entity,
                amount: PLAYER_BOOST_HEALTH,
            });
        }

        // Go to next level.
        state
            .set(AppState::NextLevel)
            .expect("Couldn't switch state to InGame");
    }
}

// After winning the game the level is set to the default one in case someone wants to play again.
fn reset_level_number(mut level: ResMut<Level>) {
    level.current = 1;
}

// If there are any levels left, then goes to next level, else ends the game.
fn next_level(mut state: ResMut<State<AppState>>, mut level: ResMut<Level>) {
    if level.current < MAX_LEVEL {
        level.current += 1;
        state
            .set(AppState::InGame)
            .expect("Couldn't switch state to InGame");
    } else {
        state
            .set(AppState::GameEnded)
            .expect("Couldn't switch state to GameEnded");
    }
}
