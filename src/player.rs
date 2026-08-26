use crate::bullet::Bullet;
use crate::caster::cast_ray;
use crate::maze::Maze;
use raylib::prelude::*;
use std::f32::consts::PI;

pub struct Player {
    pub pos: Vector2,
    pub a: f32, // ángulo del jugador
    pub fired: bool,
    pub gun_cooldown: f32,
    pub ammo: u8,
    pub reloading: bool,
    pub reload_time: f32,
    pub defusing: bool,
    pub defuse_time: f32,
}

impl Player {
    pub fn new(xo: f32, yo: f32) -> Player {
        Self {
            pos: Vector2::new(xo, yo),
            a: PI / 2.0,
            fired: false,
            gun_cooldown: 0.35,
            ammo: 7,
            reloading: false,
            reload_time: 2.2,
            defusing: false,
            defuse_time: 5.0,
        }
    }
}

pub fn process_input(player: &mut Player, rl: &mut RaylibHandle, maze: &mut Maze) {
    const MOVE_SPEED: f32 = 300.0;
    const ROTATION_SPEED: f32 = PI / 0.75;
    const COLLISION_MARGIN: f32 = 6.0;

    // delta time para medir tiempo entre frames y usar eso
    // en vez de actualizar la lógica cada frame
    let dt = rl.get_frame_time();
    let move_step = MOVE_SPEED * dt;
    let mut running_speed_mult = 1.0;

    if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
        running_speed_mult = 2.5;
    } else {
        running_speed_mult = 1.0;
    }

    let rot_step = ROTATION_SPEED * dt;
    // delta retorna cambio de mouse pos desde el frame pasado
    let mouse_delta = rl.get_mouse_delta().scale(1.0 / 500.0);
    player.a += mouse_delta.x;

    let grid_x = (player.pos.x / maze.block_size as f32) as usize;
    let grid_y = (player.pos.y / maze.block_size as f32) as usize;
    let on_site = maze.grid[grid_y][grid_x] == '1' || maze.grid[grid_y][grid_x] == '2';
    if rl.is_key_down(KeyboardKey::KEY_E) && on_site {
        player.defusing = true;
    } else if !rl.is_key_down(KeyboardKey::KEY_E) {
        player.defusing = false;
    }

    if rl.is_key_pressed(KeyboardKey::KEY_R) && !player.reloading && player.ammo < 7 {
        player.reloading = true;
    }

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
        if int.dist >= move_step + COLLISION_MARGIN * running_speed_mult {
            player.pos.x += move_step * player.a.cos() * running_speed_mult;
            player.pos.y += move_step * player.a.sin() * running_speed_mult;
        }
    }

    if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
        let int = cast_ray(player.a + PI, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN * running_speed_mult {
            player.pos.x -= move_step * player.a.cos() * running_speed_mult;
            player.pos.y -= move_step * player.a.sin() * running_speed_mult;
        }
    }

    if rl.is_key_down(KeyboardKey::KEY_A) {
        let left = player.a - PI / 2.0;
        let int = cast_ray(left, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN * running_speed_mult {
            player.pos.x += move_step * left.cos() * running_speed_mult;
            player.pos.y += move_step * left.sin() * running_speed_mult;
        }
    }

    if rl.is_key_down(KeyboardKey::KEY_D) {
        let right = player.a + PI / 2.0;
        let int = cast_ray(right, maze, player);
        if int.dist >= move_step + COLLISION_MARGIN * running_speed_mult {
            player.pos.x += move_step * right.cos() * running_speed_mult;
            player.pos.y += move_step * right.sin() * running_speed_mult;
        }
    }
}
