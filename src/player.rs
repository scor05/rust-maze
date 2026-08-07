use crate::maze::Maze;
use raylib::prelude::*;
use std::f32::consts::PI;

pub struct Player {
    pub pos: Vector2,
    pub a: f32, // ángulo del jugador
}

impl Player {
    pub fn new(xo: f32, yo: f32) -> Player {
        Self {
            pos: Vector2::new(xo, yo),
            a: PI / 2.0,
        }
    }
}

pub fn process_input(player: &mut Player, rl: &mut RaylibHandle, maze: &mut Maze) {
    const MOVE_SPEED: f32 = 10.0;
    const ROTATION_SPEED: f32 = PI / 24.0;

    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a -= ROTATION_SPEED;
    }

    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += ROTATION_SPEED;
    }

    if rl.is_key_down(KeyboardKey::KEY_UP) {
        player.pos.x += MOVE_SPEED * player.a.cos();
        player.pos.y += MOVE_SPEED * player.a.sin();
    }

    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        player.pos.x -= MOVE_SPEED * player.a.cos();
        player.pos.y -= MOVE_SPEED * player.a.sin();
    }
}
