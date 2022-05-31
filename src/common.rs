use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Position of an entity.
///
/// Maintenance note:
///     Might be replaced by Bevy's Vec2 or Vec3.
///     The 3-dimensional type would probably be better
///     since we *would* like to have entities
///     displayed properly, i.e. a player *on* a floor tile, etc.
///     no matter what the order of the rendered entities is.
#[derive(Component, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Position(pub f32, pub f32);

#[derive(Clone, Copy, PartialEq)]
pub enum EntityType {
    Wall,
    Floor,
    Player,
    Enemy,
}

struct TextureInfo {
    path: &'static str,
    owner_type: EntityType,
}

const TEXTURES: &[TextureInfo] = &[
    TextureInfo {
        path: "wall.png",
        owner_type: EntityType::Wall,
    },
    TextureInfo {
        path: "floor.png",
        owner_type: EntityType::Floor,
    },
    TextureInfo {
        path: "enemy.png",
        owner_type: EntityType::Enemy,
    },
    TextureInfo {
        path: "player.png",
        owner_type: EntityType::Player,
    },
];

pub struct TextureWrapper {
    pub texture: Handle<Image>,
    pub owner_type: EntityType,
}

pub fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    let load_texture = |path: &str, owner_type: EntityType| {
        let texture = asset_server.load(path);
        TextureWrapper {
            texture,
            owner_type,
        }
    };

    let textures: Vec<TextureWrapper> = TEXTURES
        .iter()
        .map(|info| load_texture(info.path, info.owner_type))
        .collect();

    commands.insert_resource(textures);
}
