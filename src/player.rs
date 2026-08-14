use crate::caster::cast_ray;
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
    const MOVE_SPEED: f32 = 250.0;
    const ROTATION_SPEED: f32 = PI / 1.25;
    const COLLISION_MARGIN: f32 = 1.0;

    // delta time para medir tiempo entre frames y usar eso
    // en vez de actualizar la lógica cada frame
    let dt = rl.get_frame_time();
    let move_step = MOVE_SPEED * dt;
    let rot_step = ROTATION_SPEED * dt;

    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a -= rot_step;
    }

    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += rot_step;
    }

    if rl.is_key_down(KeyboardKey::KEY_UP) {
        let int = cast_ray(player.a, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN {
            player.pos.x += move_step * player.a.cos();
            player.pos.y += move_step * player.a.sin();
        }
    }

    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        let int = cast_ray(player.a + PI, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN {
            player.pos.x -= move_step * player.a.cos();
            player.pos.y -= move_step * player.a.sin();
        }
    }
}
