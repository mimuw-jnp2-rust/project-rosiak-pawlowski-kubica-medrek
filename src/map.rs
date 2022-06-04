use crate::common::{EntityType, Position, TextureWrapper};
use crate::hitbox::Hitbox;
use crate::move_system::{MoveObjectType, MoveSystemObject};
use crate::parser::{MapId, ParsedEntity, Parser};
use bevy::ecs::event::Events;
use bevy::prelude::*;
use std::collections::HashMap;

/*
    (Un)loading and (un)rendering the contents making for a map.

    The idea:
        reading data from a file and parsing it,
        then storing it in a hashmap to avoid
        having to perform I/O when something
        is necessary at the moment -- this way
        is a lot faster.

        Example:
        you can not only load a map that's currently
        being displayed, but also those incident to it.
        Once a map gets "out of scope", i.e. the player
        cannot move to it from the current one, you can
        unload that map. This way, you can control the RAM
        usage and performance of the program. It also allows
        to make use of parallelism. You can be loading incident
        maps while the player is dealing with enemies in
        the current room.

    Systems to use:
        load_map: read a map from the disc and store it in a hashmap.
        unload_map: remove a map from the hashmap.
        render_map: display one of the maps currently stored in the hashmap.
        unrender_map: make a map disappear.

    All these functions rely on IDs of the maps.

*/

pub struct MapPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
enum MapLabel {
    Load,
    Unload,
    Unrender,
    Render,
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapStorage::new())
            .insert_resource(Events::<LoadMap>::default())
            .insert_resource(Events::<UnloadMap>::default())
            .insert_resource(Events::<RenderMap>::default())
            .insert_resource(Events::<UnrenderMap>::default())
            .add_system(unrender_map.label(MapLabel::Unrender))
            .add_system(unload_map.label(MapLabel::Unload).after(MapLabel::Unrender))
            .add_system(load_map.label(MapLabel::Load).after(MapLabel::Unload))
            .add_system(render_map.label(MapLabel::Render).after(MapLabel::Load));
    }
}

type VecIter<'a, T> = std::slice::Iter<'a, T>;
type HashMapIter<'a, K, V> = std::collections::hash_map::Iter<'a, K, V>;

/// An entity being part of a map.
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

    /// Placeholder for the time being.
    fn get_sprite_bundle(parsed_entity: &ParsedEntity, texture: &Handle<Image>) -> SpriteBundle {
        SpriteBundle {
            sprite: Sprite {
                custom_size: match parsed_entity.hitbox {
                    Hitbox::Rectangle(v) => Some(v), // Vec2 implements the Copy trait.
                    Hitbox::Circle(_) => panic!(),
                },
                ..Default::default()
            },
            texture: texture.clone(),
            transform: Transform {
                translation: Vec3::new(parsed_entity.position.0, parsed_entity.position.1, 0.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Placeholder for the time being.
    fn get_color(object_type: MoveObjectType) -> Color {
        match object_type {
            MoveObjectType::Obstacle => Color::rgb(0.7, 0.2, 0.5),
            MoveObjectType::Floor => Color::rgb(0.3, 0.1, 0.7),
            _ => panic!(),
        }
    }
}

/// Structure storing contents making for a map.
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

/// Structure storing loaded maps.
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

    fn iter(&self) -> HashMapIter<'_, MapId, Map> {
        self.maps.iter()
    }
}

/// Message for asking to load the map of a given ID.
pub struct LoadMap(pub MapId);
/// Message for asking to unload the map of a given ID.
pub struct UnloadMap(pub MapId);
/// Message for asking to render the map of a given ID.
pub struct RenderMap(pub MapId);
/// Message for asking to unrender the map.
pub struct UnrenderMap;

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

/// Description:
///     Reads a map from the drive and returns Some of it.
///     If it fails, returns None.
///
/// Arguments:
///     id : id of the map to be loaded.
///
/// Return:
///     The map of the given ID.
///
/// Maintenance notes:
///     None
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
        eprintln!("[fetch_map] There is no map of id {}.", id);
        None
    }
}

/// Description:
///     Loads maps from the disc and stores them in
///     the map storage for the future.
///     To load a map, just put a LoadMap tuple struct
///     with the ID of the map you wn to load
///     in the EventWriter<LoadMap>.
///    
/// Arguments:
///     map_ids : IDs of the maps to be loaded,
///     maps    : the map storage.
///    
/// Return:
///     None
///    
/// Maintenance notes:
///     Loading the maps can be changed to be happening concurrently.
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
                eprintln!(
                    "[load_map] Reading from a file corresponding to map {} has failed.",
                    id
                );
            }
        } else {
            eprintln!("[load_map] The map of id {} has already been loaded.", id);
        }
    }
}

/// Description:
///     Removes maps from the map storage. It does NOT
///     unrender the currently displayed one even if
///     it's among those to dispose.
///     To unload a map, put a tuple struct UnloadMap
///     with the ID of the map you want to unload
///     into the EventWriter<UnloadMap>.
///    
/// Arguments:
///     map_ids : IDs of the maps to be unloaded,
///     maps    : the map storage.
///    
/// Return:
///     None
///    
/// Maintenance notes:
///     None
fn unload_map(mut map_ids: EventReader<UnloadMap>, mut maps: ResMut<MapStorage>) {
    for UnloadMap(id) in map_ids.iter() {
        maps.remove(*id);
    }
}

/// Description:
///     Renders maps of given IDs. If a map has not been loaded,
///     it first loads it.
///     To render a map, put a tuple struct RenderMap
///     with the ID of the map you want to render
///     into the EventWriter<RenderMap>.
///
/// Arguments:
///     commands : commands (for rendering purposes),
///     map_ids  : IDs of the maps to be rendered,
///     maps     : the map storage.
///
/// Return:
///     None
///
/// Maintenance notes:
///     None
fn render_map(
    mut commands: Commands,
    mut map_ids: EventReader<RenderMap>,
    mut maps: ResMut<MapStorage>,
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
            eprintln!(
                "[render_map] The map of id {} has not been loaded. Fetching the map...",
                id
            );
        }
    }
}

/// Description:
///     Loads maps from the disc and stores them in
///     the map storage for the future.
///     To unrender the currently displayed map,
///     put a struct UnrenderMap into
///     the EventWriter<UnrenderMap>.
///
/// Arguments:
///     commands : commands (for despawning purposes)
///     request  : just a label to know that the system should unrender the map,
///     entities : query storing entities to be removed.
///
/// Return:
///     None
///
/// Maintenance notes:
///     None
fn unrender_map(
    mut commands: Commands,
    mut request: EventReader<UnrenderMap>,
    entities: Query<Entity, With<Position>>,
) {
    if request.iter().next().is_some() {
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
    }
}
