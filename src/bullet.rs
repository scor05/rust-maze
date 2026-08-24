use raylib::prelude::Color;
use raylib::prelude::Vector2;

pub struct Bullet {
    pub pos: Vector2,
    pub radius: f32,
    pub color: Color,
    pub a: f32,
    pub active: bool,
}
