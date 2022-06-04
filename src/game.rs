use crate::health_system::HealthPlugin;
use crate::{
    map, AppState, EnemyPlugin, LoadMap, MoveSystemPlugin, PlayerPlugin, RenderMap, UnrenderMap,
    WinSize,
};
use bevy::prelude::*;
use std::time::Duration;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(spawn_map))
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(spawn_camera))
            .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(despawn_map))
            .add_plugin(MoveSystemPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(EnemyPlugin)
            .add_plugin(HealthPlugin);
    }
}

fn spawn_map(mut load_map: EventWriter<LoadMap>, mut render_map: EventWriter<RenderMap>) {
    load_map.send(map::LoadMap(1));
    render_map.send(map::RenderMap(1));
}

fn despawn_map() {
    todo!();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
