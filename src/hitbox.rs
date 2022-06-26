use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
pub enum Hitbox {
    Rectangle(Vec2),
}

impl Hitbox {
    pub fn check_collision(
        x_hitbox: &Hitbox,
        x_position: Vec2,
        y_hitbox: &Hitbox,
        y_position: Vec2,
    ) -> bool {
        match (x_hitbox, y_hitbox) {
            (&Hitbox::Rectangle(dim_x), &Hitbox::Rectangle(dim_y)) => {
                collide(x_position.extend(0.), dim_x, y_position.extend(0.), dim_y).is_some()
            }
        }
    }

    pub fn new_rectangle(dimensions: Vec2) -> Hitbox {
        Hitbox::Rectangle(dimensions)
    }
}
