use crate::move_system::MoveObjectType::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Add;

use crate::hitbox::Hitbox;
use crate::player::PlayerMarker;

/*
    This system moves object accordingly to value of their VelocityVector, this vector has to be
    set by other system every frame, because it is cleared (set to ZERO) at the beginning of event
    loop iteration. Systems that modify VelocityVector should have label ModifyVelocity, this will
    ensure that the execute after clearing vector and before moving objects.

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
#[derive(Component, Copy, Clone)]
pub struct VelocityVector(pub Vec2); 

#[derive(Component, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum MoveObjectType {
    Obstacle,
    Floor,
    Player,
    Enemy,
    PlayerBullet,
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
fn clear_velocity_vector(mut vector_query: Query<(&mut VelocityVector), (With<MoveSystemMarker>)>) {
    for (mut vector) in vector_query.iter_mut() {
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
        _ => false,
    }
}

// Send information about collision to other systems, but ignore its effect on moving objects.
fn allow_overlap(type_1: &MoveObjectType, type_2: &MoveObjectType) -> bool {
    match (type_1, type_2) {
        _ => false,
    }
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
        (With<MoveSystemMarker>),
    >,
    mut collision_writer: EventWriter<CollisionEvent>,
) {
    let delta_time = time.delta().as_secs_f32();
    let to_move_iterator = to_move_query
        .iter_mut()
        .filter(|(_, __, &x, _, _)| x != MoveObjectType::Floor);
    let mut to_move_vec = vec![];
    for entry in to_move_iterator {
        to_move_vec.push(entry);
    }
    let mut no_collision = vec![true; to_move_vec.len()];
    let mut future_position = vec![];
    for v in to_move_vec.iter() {
        let mut pos = v.0.translation.truncate();
        let vel_option = v.3;
        match vel_option {
            None => (),
            Some(vel) => pos += (vel.0 * delta_time),
        }
        future_position.push(pos);
    }
    for i in 0..to_move_vec.len() {
        for j in (i + 1)..to_move_vec.len() {
            let (_, first_hitbox, first_type, _, first_id) = to_move_vec[i];
            let (_, second_hitbox, second_type, _, second_id) = to_move_vec[j];
            if ignore_collision(first_type, second_type) {
                continue;
            }
            if Hitbox::check_collision(
                first_hitbox,
                future_position[i],
                second_hitbox,
                future_position[j],
            ) {
                if !allow_overlap(first_type, second_type) {
                    no_collision[i] = false;
                    no_collision[j] = false;
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
    for i in 0..to_move_vec.len() {
        if no_collision[i] {
            to_move_vec[i].0.translation =
                future_position[i].extend(to_move_vec[i].0.translation.z);
        }
    }
}
