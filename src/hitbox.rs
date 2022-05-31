use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Copy, Serialize, Deserialize)]
pub enum Hitbox {
    Rectangle(Vec2),
    Circle(f32), // currently not used
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
            (&Hitbox::Circle(r_x), &Hitbox::Circle(r_y)) => todo!(),
            (&Hitbox::Rectangle(dim_x), &Hitbox::Circle(r_y)) => todo!(),
            (&Hitbox::Circle(_), &Hitbox::Rectangle(_)) => {
                Hitbox::check_collision(y_hitbox, y_position, x_hitbox, x_position)
            }
        }
    }

    pub fn new_rectangle(dimensions: Vec2) -> Hitbox {
        Hitbox::Rectangle(dimensions)
    }
}
