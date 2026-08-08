mod caster;
mod framebuffer;
mod maze;
mod player;

use crate::caster::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::{Player, process_input};
use raylib::prelude::*;
use std::f32::consts::PI;
use std::thread;
use std::time::Duration;

fn render3D(player: &mut Player, maze: &mut Maze, fb: &mut Framebuffer, block_size: usize) {
    let hh = fb.height as f32 / 2.0;
    let FOV = PI / 3.0;
    let a_0 = player.a - FOV / 2.0;
    let a_step = FOV / fb.width as f32;
    let distance_to_projection = 300.0;

    for x in 0..fb.width {
        let a = a_0 + (x as f32) * a_step;

        let int = cast_ray(a, maze, player, block_size);

        // corrección fisheye
        let a_diff = a - player.a;
        let corrected_distance = (int.dist * a_diff.cos()).max(0.001);

        let stake_height = (hh / corrected_distance) * distance_to_projection;
        let stake_top = (hh - (stake_height / 2.0)).max(0.0) as u32;
        let stake_bottom = (hh + (stake_height / 2.0)).min(fb.height as f32) as u32;

        fb.set_current_color(Color::SKYBLUE);
        for y in 0..stake_top {
            fb.set_pixel(x, y);
        }

        fb.set_current_color(Color::GRAY);
        for y in stake_bottom..fb.height {
            fb.set_pixel(x, y);
        }

        match int.object {
            '┌' | '┐' | '┘' | '└' => {
                fb.set_current_color(Color::DARKGREEN);
            }
            '╶' | '╴' | '╵' | '╷' => {
                fb.set_current_color(Color::from_hex("02e678").unwrap());
            }
            '┴' | '┬' | '├' | '┤' => {
                fb.set_current_color(Color::GREEN);
            }
            _ => {
                fb.set_current_color(Color::from_hex("02c265").unwrap());
            }
        }
        for y in stake_top..stake_bottom {
            fb.set_pixel(x, y);
        }

        fb.set_current_color(Color::LIGHTGREEN);
    }
}

fn main() -> std::io::Result<()> {
    let window_width = 1280;
    let window_height = 720;
    let block_size = 100usize;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Maze :D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);

    let mut maze = Maze::new("maze.txt", block_size)?;
    let mut player = Player::new(
        (3.0 / 2.0) * block_size as f32,
        (3.0 / 2.0) * block_size as f32,
    );

    // Image::load_image para cargar a ram
    // window.load_texture para cargar a tarjeta de video

    while !window.window_should_close() {
        process_input(&mut player, &mut window, &mut maze);
        render3D(&mut player, &mut maze, &mut framebuffer, block_size);

        framebuffer.swap_buffers(&mut window, &raylib_thread);

        // 60 frames/sec ^-1 -> 0.0166 sec/frames -> 16.67 ms/frame
        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
