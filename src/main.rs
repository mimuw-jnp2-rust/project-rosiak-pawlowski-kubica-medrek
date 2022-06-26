use crate::common::load_textures;
use crate::game::GamePlugin;
use crate::hitbox::Hitbox;
use crate::main_menu::MenuPlugin;
use crate::move_system::MoveSystemPlugin;
use bevy::prelude::*;
use bevy::window::WindowResizeConstraints;
use map::{LoadMap, MapPlugin, RenderMap};
use std::time::Duration;

mod bullet;
mod common;
mod enemy;
mod game;
mod health_system;
mod hitbox;
mod main_menu;
mod map;
mod move_system;
mod parser;
mod player;

// Window setup information.
const WINDOW_TITLE: &str = "super game";
const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 600.0;
const WINDOW_MIN_WIDTH: f32 = 300.0;
const WINDOW_MIN_HEIGHT: f32 = 300.0;
const WINDOW_MAX_WIDTH: f32 = f32::INFINITY;
const WINDOW_MAX_HEIGHT: f32 = f32::INFINITY;

// Highest level number.
const MAX_LEVEL: usize = 3;

// States of the game.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
    NextLevel,
    GameEnded,
    PlayerDied,
}

// Current level number.
pub struct Level {
    current: usize,
}

fn main() {
    let mut player_timer = Timer::from_seconds(0.5, false);
    player_timer.tick(Duration::new(1, 0));

    App::new()
        .add_state(AppState::MainMenu)
        .insert_resource(WindowDescriptor {
            title: WINDOW_TITLE.to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resize_constraints: WindowResizeConstraints {
                min_width: WINDOW_MIN_WIDTH,
                min_height: WINDOW_MIN_HEIGHT,
                max_width: WINDOW_MAX_WIDTH,
                max_height: WINDOW_MAX_HEIGHT,
            },
            ..Default::default()
        })
        .insert_resource(player_timer)
        .insert_resource(Level { current: 1 })
        .add_plugins(DefaultPlugins)
        .add_plugin(MenuPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(MapPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_textures)
        .add_system_set(SystemSet::on_update(AppState::InGame).with_system(resize_window))
        .run();
}

// System responsible for resizing window while in game, it scales the map and entities.
fn resize_window(mut projection: Query<&mut OrthographicProjection>, mut windows: ResMut<Windows>) {
    let window = windows.primary_mut();
    if let Ok(mut projection_unwrapped) = projection.get_single_mut() {
        projection_unwrapped.scale =
            (WINDOW_HEIGHT / window.height()).max(WINDOW_WIDTH / window.width());
    }
}
