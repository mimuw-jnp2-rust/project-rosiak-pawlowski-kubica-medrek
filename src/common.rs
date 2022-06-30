use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Position of an entity.
#[derive(Component, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Position(pub f32, pub f32);

// Type of the entity.
#[derive(Clone, Copy, PartialEq)]
pub enum EntityType {
    Wall,
    Floor,
    Player,
    Enemy,
    Bones,
}

// Information about given texture.
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
    TextureInfo {
        path: "bones.png",
        owner_type: EntityType::Bones,
    },
];

pub struct TextureWrapper {
    pub texture: Handle<Image>,
    pub owner_type: EntityType,
}

// Loads textures.
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
