use crate::bullet::Bullet;
use crate::caster::cast_ray;
use crate::maze::Maze;
use raylib::prelude::*;
use std::f32::consts::PI;

const GUN_SOUND_PATH: &str = "./assets/audio/gunshot.mp3";

pub struct Player {
    pub pos: Vector2,
    pub a: f32, // ángulo del jugador
    pub bullets: Vec<Bullet>,
    pub fired: bool,
    pub gun_cooldown: f32,
}

impl Player {
    pub fn new(xo: f32, yo: f32) -> Player {
        Self {
            pos: Vector2::new(xo, yo),
            a: PI / 2.0,
            bullets: Vec::new(),
            fired: false,
            gun_cooldown: 0.75,
        }
    }
}

pub fn process_input(player: &mut Player, rl: &mut RaylibHandle, maze: &mut Maze) {
    const MOVE_SPEED: f32 = 250.0;
    const ROTATION_SPEED: f32 = PI / 0.75;
    const COLLISION_MARGIN: f32 = 1.0;

    // delta time para medir tiempo entre frames y usar eso
    // en vez de actualizar la lógica cada frame
    let dt = rl.get_frame_time();
    let move_step = MOVE_SPEED * dt;
    let rot_step = ROTATION_SPEED * dt;
    // delta retorna cambio de mouse pos desde el frame pasado
    let mouse_delta = rl.get_mouse_delta().scale(1.0 / 400.0);
    player.a += mouse_delta.x;

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        player.fired = true;
    }

    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a -= rot_step;
    }

    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += rot_step;
    }

    if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
        let int = cast_ray(player.a, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN {
            player.pos.x += move_step * player.a.cos();
            player.pos.y += move_step * player.a.sin();
        }
    }

    if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
        let int = cast_ray(player.a + PI, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN {
            player.pos.x -= move_step * player.a.cos();
            player.pos.y -= move_step * player.a.sin();
        }
    }

    if rl.is_key_down(KeyboardKey::KEY_A) {
        let left = player.a - PI / 2.0;
        let int = cast_ray(left, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN {
            player.pos.x += move_step * left.cos();
            player.pos.y += move_step * left.sin();
        }
    }

    if rl.is_key_down(KeyboardKey::KEY_D) {
        let right = player.a + PI / 2.0;
        let int = cast_ray(right, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN {
            player.pos.x += move_step * right.cos();
            player.pos.y += move_step * right.sin();
        }
    }
}
