use crate::move_system::MoveObjectType::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::min;

use crate::hitbox::Hitbox;
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

/*
    This system moves object accordingly to value of their VelocityVector, this vector has to be
    set by other system every frame, because it is cleared (set to ZERO) at the beginning of event
    loop iteration. Systems that modify VelocityVector should have label ModifyVelocity, this will
    ensure that they execute after clearing vector and before moving objects.

    When move_system detects collision of two objects it sends event of type CollisionEvent
*/

pub struct MoveSystemPlugin;

impl Plugin for MoveSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEvent>()
            .add_system(
                clear_velocity_vector
                    .label(ClearVelocity)
                    .before(ModifyVelocity),
            )
            .add_system(
                move_system
                    .after(ModifyVelocity)
                    .after(ClearVelocity)
                    .before(HandleCollisionEvents),
            );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
struct ClearVelocity;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub struct ModifyVelocity;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub struct HandleCollisionEvents;

pub struct CollisionEvent {
    pub object_id: Entity,
    pub object_type: MoveObjectType,
    pub collided_with_id: Entity,
    pub collided_with_type: MoveObjectType,
}

// Add to moving objects and static obstacles.
#[derive(Component, Copy, Clone)]
pub struct MoveSystemMarker;

// Add only to moving objects.
#[derive(Component, Copy, Clone, Debug)]
pub struct VelocityVector(pub Vec2);

#[derive(Component, Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum MoveObjectType {
    Obstacle,
    Floor,
    Player,
    Enemy,
    PlayerBullet,
    EnemyBullet,
}

#[derive(Bundle, Copy, Clone)]
pub struct MoveSystemObject {
    marker: MoveSystemMarker,
    object_type: MoveObjectType,
    hitbox: Hitbox,
}

impl MoveSystemObject {
    pub fn new(object_type: MoveObjectType, hitbox: Hitbox) -> MoveSystemObject {
        MoveSystemObject {
            marker: MoveSystemMarker,
            object_type,
            hitbox,
        }
    }
}

#[derive(Bundle, Copy, Clone)]
pub struct MoveSystemObjectWithVelocity {
    #[bundle]
    object_bundle: MoveSystemObject,
    velocity: VelocityVector,
}

impl MoveSystemObjectWithVelocity {
    pub fn new(
        object_type: MoveObjectType,
        hitbox: Hitbox,
        vel: Vec2,
    ) -> MoveSystemObjectWithVelocity {
        MoveSystemObjectWithVelocity {
            object_bundle: MoveSystemObject::new(object_type, hitbox),
            velocity: VelocityVector(vel),
        }
    }

    pub fn new_with_vel_0(
        object_type: MoveObjectType,
        hitbox: Hitbox,
    ) -> MoveSystemObjectWithVelocity {
        MoveSystemObjectWithVelocity::new(object_type, hitbox, Vec2::new(0., 0.))
    }
}

// Clear velocity vectors before other systems start to modify it.
fn clear_velocity_vector(mut vector_query: Query<&mut VelocityVector, With<MoveSystemMarker>>) {
    for mut vector in vector_query.iter_mut() {
        vector.0 = Vec2::ZERO;
    }
}

// Completely ignore collision - collision has no effect on moving objects
// and information about it is not passed to other systems.
fn ignore_collision(type_1: &MoveObjectType, type_2: &MoveObjectType) -> bool {
    match (type_1, type_2) {
        (PlayerBullet, PlayerBullet) => false,
        (Player, PlayerBullet) | (PlayerBullet, Player) => true,
        (Player, Floor) | (Floor, Player) => true,
        (Enemy, Floor) | (Floor, Enemy) => true,
        (PlayerBullet, Floor) | (Floor, PlayerBullet) => true,
        (Enemy, EnemyBullet) | (EnemyBullet, Enemy) => true,
        (EnemyBullet, Floor) | (Floor, EnemyBullet) => true,
        (Enemy, Enemy) => true,
        _ => false,
    }
}

// Send information about collision to other systems, but ignore its effect on moving objects.
fn allow_overlap(type_1: &MoveObjectType, type_2: &MoveObjectType) -> bool {
    match (type_1, type_2) {
        _ => false,
    }
}

// ZONE_SIZE hase to be bigger then size of any object using move system.
const ZONE_SIZE: f32 = 100.;
const ZONES_HORIZONTALLY: usize = (WINDOW_WIDTH / ZONE_SIZE) as usize + 1;
const ZONES_VERTICALLY: usize = (WINDOW_HEIGHT / ZONE_SIZE) as usize + 1;

fn get_zone(position: &Vec2) -> (usize, usize) {
    let x = position.x + WINDOW_WIDTH / 2.;
    let y = position.y + WINDOW_HEIGHT / 2.;
    let x = (x / ZONE_SIZE) as usize;
    let y = (y / ZONE_SIZE) as usize;
    (min(x, ZONES_HORIZONTALLY - 1), min(y, ZONES_VERTICALLY - 1))
}

// Try to move objects accordingly to their velocity vectors,
// after other systems modified those vectors.
fn move_system(
    time: Res<Time>,
    mut to_move_query: Query<
        (
            &mut Transform,
            &Hitbox,
            &MoveObjectType,
            Option<&VelocityVector>,
            Entity,
        ),
        With<MoveSystemMarker>,
    >,
    mut collision_writer: EventWriter<CollisionEvent>,
) {
    let delta_time = time.delta().as_secs_f32();

    let mut zones: [[Vec<_>; ZONES_VERTICALLY]; ZONES_HORIZONTALLY] = Default::default();
    // Put every object to proper zone.
    for object in to_move_query
        .iter_mut()
        .filter(|(_, _, t, _, _)| **t != MoveObjectType::Floor)
    {
        let current_position = object.0.translation.truncate();
        let (x, y) = get_zone(&current_position);
        let future_position = if let Some(velocity) = object.3 {
            current_position + velocity.0 * delta_time
        } else {
            current_position
        };
        zones[x][y].push((object, future_position, false));
    }

    let mut zones_len = [[0; ZONES_VERTICALLY]; ZONES_HORIZONTALLY];
    for i in 0..ZONES_HORIZONTALLY {
        for j in 0..ZONES_VERTICALLY {
            zones_len[i][j] = zones[i][j].len();
        }
    }
    for i in 0..ZONES_HORIZONTALLY {
        for j in 0..ZONES_VERTICALLY {
            let current_zone = (i, j);
            let mut bordering_zones = vec![];
            for (x, y) in [(1, 0), (0, 1), (1, 1)] {
                let (x, y) = (i + x, j + y);
                if x < ZONES_HORIZONTALLY && y < ZONES_VERTICALLY {
                    bordering_zones.push((x, y));
                }
            }
            let current_zone_len = zones[current_zone.0][current_zone.1].len();
            for i in 0..current_zone_len {
                let rest_of_current =
                    ((i + 1)..current_zone_len).map(|i| (current_zone.0, current_zone.1, i));
                let other_zones = bordering_zones.iter_mut().flat_map(|(x, y)| {
                    let (x, y) = (*x, *y);
                    // Can't use zones here, because it creates borrow
                    // and prevents creating mutable borrow later.
                    (0..zones_len[x][y]).map(move |i| (x, y, i))
                });
                let obj1 = (current_zone.0, current_zone.1, i);
                for obj2 in rest_of_current.chain(other_zones) {
                    let ((_, first_hitbox, first_type, _, first_id), first_fut_poss, _) =
                        zones[obj1.0][obj1.1][obj1.2];
                    let ((_, second_hitbox, second_type, _, second_id), second_fut_poss, _) =
                        zones[obj2.0][obj2.1][obj2.2];
                    if !ignore_collision(first_type, second_type)
                        && Hitbox::check_collision(
                            first_hitbox,
                            first_fut_poss,
                            second_hitbox,
                            second_fut_poss,
                        )
                    {
                        if !allow_overlap(first_type, second_type) {
                            zones[obj1.0][obj1.1][obj1.2].2 = true;
                            zones[obj2.0][obj2.1][obj2.2].2 = true;
                        }
                        collision_writer.send(CollisionEvent {
                            object_id: first_id,
                            object_type: *first_type,
                            collided_with_id: second_id,
                            collided_with_type: *second_type,
                        });
                        collision_writer.send(CollisionEvent {
                            object_id: second_id,
                            object_type: *second_type,
                            collided_with_id: first_id,
                            collided_with_type: *first_type,
                        });
                    }
                }
            }
        }
    }
    for ((transform, _, _, _, _), fut_pos, collision) in zones.iter_mut().flatten().flatten() {
        if !*collision {
            transform.translation = fut_pos.extend(transform.translation.z);
        }
    }
}
