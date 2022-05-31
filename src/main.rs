// !!! To get rid of !!!
#![allow(unused)]

use std::time::Duration;
// For the basic funcionality of Bevy.
use bevy::prelude::*;
// For setting up the minimum acceptable size of the window.
use bevy::window::{WindowResizeConstraints, WindowResized};
// For getting diagnostic data about framerate.
use crate::hitbox::Hitbox;
use crate::window::WinSize;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
// use bevy::ecs::event::Events;

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
mod window;

use crate::common::load_textures;
use crate::enemy::EnemyPlugin;
use crate::game::GamePlugin;
use crate::main_menu::MainMenuPlugin;
use crate::move_system::MoveObjectType::Player;
use crate::move_system::MoveSystemPlugin;
use crate::player::{PlayerBundle, PlayerPlugin};
use map::{LoadMap, MapPlugin, RenderMap, UnloadMap, UnrenderMap};
use parser::MapId;

/*================
    CONSTANTS
================*/

const WINDOW_TITLE: &str = "TEST [WIP]";
const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 600.0;

const WINDOW_MIN_WIDTH: f32 = 300.0;
const WINDOW_MIN_HEIGHT: f32 = 300.0;
const WINDOW_MAX_WIDTH: f32 = f32::INFINITY;
const WINDOW_MAX_HEIGHT: f32 = f32::INFINITY;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
    // maybe Paused,
}

/*===================
   ACTUAL PROGRAM
====================*/

fn main() {
    // Adding timer to resources.
    let mut timer = Timer::from_seconds(0.5, false);
    timer.tick(Duration::new(1, 0));

    App::new()
        .insert_resource(Msaa { samples: 4 }) // ???
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
        .insert_resource(WinSize {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        })
        .insert_resource(timer)
        .add_plugins(DefaultPlugins)
        // Diagnostic information about framerate.
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        //Custom plugins.
        .add_plugin(MainMenuPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(EnemyPlugin)
        /*
        // Framerate control so as to lower CPU usage.
        // [Remember to comment/uncomment the dependency in Cargo.toml]
        .add_plugin(bevy_framepace::FramepacePlugin::default())
        */
        .add_startup_system_to_stage(StartupStage::PreStartup, load_textures)
        .add_system(resize_window)
        .add_plugin(MapPlugin)
        .run();
}

/// Checks whether the window has been resized and if that's the case,
/// modifies the scale of the projection so that everything fits the window.
fn resize_window(
    mut projection: Query<&mut OrthographicProjection>,
    mut resize_event: EventReader<WindowResized>,
) {
    let max = |x: f32, y: f32| {
        if x < y {
            y
        } else {
            x
        }
    };

    if let Some(w) = resize_event.iter().next() {
        projection.single_mut().scale = max(WINDOW_HEIGHT / w.height, WINDOW_WIDTH / w.width);
    }
}
