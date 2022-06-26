use crate::common::{EntityType, Position, TextureWrapper};
use crate::hitbox::Hitbox;
use crate::move_system::{MoveObjectType, MoveSystemObject};
use crate::parser::{MapId, ParsedEntity, Parser};
use crate::player::PlayerMarker;
use crate::AppState;
use bevy::ecs::event::Events;
use bevy::prelude::*;
use std::collections::HashMap;

type VecIter<'a, T> = std::slice::Iter<'a, T>;

/*
    The idea:
        reading data from a file and parsing it,
        then storing it in a hashmap to avoid
        having to perform I/O when something
        is necessary at the moment -- this way
        is a lot faster.
*/

// Labels for systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
enum MapLabel {
    Load,
    Unload,
    Remove,
    Render,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapStorage::new())
            .insert_resource(Events::<LoadMap>::default())
            .insert_resource(Events::<UnloadMap>::default())
            .insert_resource(Events::<RenderMap>::default())
            .insert_resource(Events::<RemoveMap>::default())
            .add_system(remove_map.label(MapLabel::Remove))
            .add_system(unload_map.label(MapLabel::Unload).after(MapLabel::Remove))
            .add_system(load_map.label(MapLabel::Load).after(MapLabel::Unload))
            .add_system(render_map.label(MapLabel::Render).after(MapLabel::Load))
            .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(despawn_map));
    }
}

// An entity being part of a map.
#[derive(Bundle, Clone)]
struct MapEntity {
    #[bundle]
    move_system: MoveSystemObject,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl MapEntity {
    fn new(parsed_entity: ParsedEntity, texture: &Handle<Image>) -> MapEntity {
        MapEntity {
            move_system: MoveSystemObject::new(parsed_entity.move_type, parsed_entity.hitbox),
            sprite_bundle: Self::get_sprite_bundle(&parsed_entity, texture),
        }
    }

    // Placeholder for the time being.
    fn get_sprite_bundle(parsed_entity: &ParsedEntity, texture: &Handle<Image>) -> SpriteBundle {
        SpriteBundle {
            sprite: Sprite {
                custom_size: match parsed_entity.hitbox {
                    Hitbox::Rectangle(v) => Some(v),
                },
                ..Default::default()
            },
            texture: (*texture).clone(),
            transform: Transform {
                translation: Vec3::new(parsed_entity.position.0, parsed_entity.position.1, 0.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

// Structure storing contents making for a map.
#[derive(Clone)]
struct Map {
    entities: Vec<MapEntity>,
}

impl Map {
    fn new() -> Map {
        Map { entities: vec![] }
    }

    fn insert(&mut self, map_entity: MapEntity) {
        self.entities.push(map_entity);
    }

    fn iter(&self) -> VecIter<'_, MapEntity> {
        self.entities.iter()
    }
}

// Structure storing loaded maps.
struct MapStorage {
    maps: HashMap<MapId, Map>,
}

impl MapStorage {
    fn new() -> MapStorage {
        MapStorage {
            maps: HashMap::new(),
        }
    }

    fn insert(&mut self, id: MapId, map: Map) {
        self.maps.insert(id, map);
    }

    fn get(&self, id: MapId) -> Option<&Map> {
        self.maps.get(&id)
    }

    fn remove(&mut self, id: MapId) -> Option<Map> {
        self.maps.remove(&id)
    }

    fn contains(&self, id: MapId) -> bool {
        self.maps.contains_key(&id)
    }
}

// Message for asking to load the map of a given ID.
pub struct LoadMap(pub MapId);

// Message for asking to unload the map of a given ID.
pub struct UnloadMap(pub MapId);

// Message for asking to render the map of a given ID.
pub struct RenderMap(pub MapId);

// Message for asking to remove the map.
pub struct RemoveMap;

fn get_texture<'a>(
    object_type: MoveObjectType,
    textures: &'a Res<Vec<TextureWrapper>>,
) -> std::option::Option<&'a Handle<Image>> {
    let find_texture = |entity_type: EntityType| {
        textures
            .iter()
            .find(|&x| x.owner_type == entity_type)
            .map(|texture| &texture.texture)
    };

    match object_type {
        MoveObjectType::Obstacle => find_texture(EntityType::Wall),
        MoveObjectType::Floor => find_texture(EntityType::Floor),
        _ => panic!(),
    }
}

// Reads a map from the drive and returns an option of the map with the given ID.
fn fetch_map(id: MapId, textures: &Res<Vec<TextureWrapper>>) -> Option<Map> {
    if let Some(parser) = Parser::new(id) {
        let mut map = Map::new();
        for parsed_entity in parser.iter() {
            if let Some(texture) = get_texture(parsed_entity.move_type, textures) {
                map.insert(MapEntity::new((*parsed_entity).clone(), texture));
            }
        }
        Some(map)
    } else {
        warn!("[fetch_map] There is no map of id {}.", id);
        None
    }
}

// Loads maps from the disc and stores them in the map storage for the future.
fn load_map(
    mut map_ids: EventReader<LoadMap>,
    mut maps: ResMut<MapStorage>,
    textures: Res<Vec<TextureWrapper>>,
) {
    for LoadMap(id) in map_ids.iter() {
        if !maps.contains(*id) {
            if let Some(map) = fetch_map(*id, &textures) {
                maps.insert(*id, map);
            } else {
                warn!(
                    "[load_map] Reading from a file corresponding to map {} has failed.",
                    id
                );
            }
        } else {
            info!("[load_map] The map of id {} has already been loaded.", id);
        }
    }
}

// Removes maps from the map storage. It does NOT remove the currently displayed one.
fn unload_map(mut map_ids: EventReader<UnloadMap>, mut maps: ResMut<MapStorage>) {
    for UnloadMap(id) in map_ids.iter() {
        maps.remove(*id);
    }
}

// Renders a map with given ID. If a map hasn't been loaded yet, it loads it first.
fn render_map(
    mut commands: Commands,
    mut map_ids: EventReader<RenderMap>,
    maps: ResMut<MapStorage>,
) {
    let mut render = |map: &Map| {
        for map_entity in map.iter() {
            commands.spawn_bundle(map_entity.clone());
        }
    };

    for RenderMap(id) in map_ids.iter() {
        if let Some(map) = maps.get(*id) {
            render(map);
        } else {
            info!(
                "[render_map] The map of id {} has not been loaded. Fetching the map...",
                id
            );
        }
    }
}

// Removes the map.
fn remove_map(
    mut commands: Commands,
    mut request: EventReader<RemoveMap>,
    entities: Query<Entity, With<Position>>,
) {
    if request.iter().next().is_some() {
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// Despawns currently displayed map (and other entities that are not the player).
fn despawn_map(
    mut commands: Commands,
    entities: Query<Entity, (With<Transform>, Without<PlayerMarker>)>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}
