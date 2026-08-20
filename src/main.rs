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

const SKY_COLOR: Color = Color::SKYBLUE;
const FLOOR_COLOR: Color = Color::GRAY;
const CORNER_COLOR: Color = Color::DARKGREEN;
const T_COLOR: Color = Color::GREEN;
const WALL_COLOR: Color = Color::new(0x02, 0xc2, 0x65, 0xff);
const EDGE_COLOR: Color = Color::new(0x02, 0xe6, 0x78, 0xff);
const FOV: f32 = PI / 3.0;
const FPS: u64 = 500;
// FPS frames/sec ^-1 -> sec/frames * 1000 -> ms/frame
const MS: u64 = 1000 / FPS;

fn render3D(player: &mut Player, maze: &mut Maze, fb: &mut Framebuffer) {
    let hh = fb.height as f32 / 2.0;
    let a_0 = player.a - FOV / 2.0;
    let a_step = FOV / fb.width as f32;
    let distance_to_projection = 75.0;

    for x in 0..fb.width {
        let a = a_0 + (x as f32) * a_step;

        let int = cast_ray(a, maze, player);

        // corrección fisheye
        let a_diff = a - player.a;
        let corrected_distance = (int.dist * a_diff.cos()).max(0.001);

        let stake_height = (hh / corrected_distance) * distance_to_projection;
        let stake_top = (hh - (stake_height / 2.0)).max(0.0) as u32;
        let stake_bottom = (hh + (stake_height / 2.0)).min(fb.height as f32) as u32;

        fb.set_current_color(SKY_COLOR);
        for y in 0..stake_top {
            fb.set_pixel(x, y);
        }

        fb.set_current_color(FLOOR_COLOR);
        for y in stake_bottom..fb.height {
            fb.set_pixel(x, y);
        }

        match int.object {
            '┌' | '┐' | '┘' | '└' => {
                fb.set_current_color(CORNER_COLOR);
            }
            '╶' | '╴' | '╵' | '╷' => {
                fb.set_current_color(EDGE_COLOR);
            }
            '┴' | '┬' | '├' | '┤' => {
                fb.set_current_color(T_COLOR);
            }
            _ => {
                fb.set_current_color(WALL_COLOR);
            }
        }
        for y in stake_top..stake_bottom {
            fb.set_pixel(x, y);
        }
    }
}

fn render2D(
    player: &mut Player,
    maze: &mut Maze,
    fb: &mut Framebuffer,
    map_size: usize,
    x: usize,
    y: usize,
) {
    let maze_height = maze.grid.len();
    let maze_width = maze.grid.iter().map(Vec::len).max().unwrap();

    let scale_factor = map_size as f32 / maze_width.max(maze_height) as f32;
    let rendered_width = (maze_width as f32 * scale_factor) as usize;
    let rendered_height = (maze_height as f32 * scale_factor) as usize;

    for map_y in 0..rendered_height {
        let grid_y = ((map_y as f32 / scale_factor) as usize).min(maze_height - 1);

        for map_x in 0..rendered_width {
            let grid_x = ((map_x as f32 / scale_factor) as usize).min(maze_width - 1);
            let cell = maze.grid[grid_y][grid_x];

            match cell {
                '┌' | '┐' | '┘' | '└' => {
                    fb.set_current_color(CORNER_COLOR);
                }
                '╶' | '╴' | '╵' | '╷' => {
                    fb.set_current_color(EDGE_COLOR);
                }
                '┴' | '┬' | '├' | '┤' => {
                    fb.set_current_color(T_COLOR);
                }
                ' ' => {
                    fb.set_current_color(FLOOR_COLOR);
                }
                _ => fb.set_current_color(WALL_COLOR),
            }

            fb.set_pixel((x + map_x) as u32, (y + map_y) as u32);
        }
    }

    let player_x = player.pos.x * (scale_factor / maze.block_size as f32) + x as f32;
    let player_y = player.pos.y * (scale_factor / maze.block_size as f32) + y as f32;
    let player_size = 2; // rect de 2*playersize + 1

    // dibujar fov 2d
    fb.set_current_color(Color::YELLOW);
    let ray_amount = 50;
    let a_0 = player.a - FOV / 2.0;
    let a_inc = FOV / ray_amount as f32;
    let max_dist = (player_size + 1) * 12;
    let min_dist = player_size + 1;
    let ray_steps = 50;

    for r in 0..ray_amount {
        let a = a_0 + a_inc * r as f32;
        let mut int = cast_ray(a, maze, player);
        int.dist = (max_dist as f32)
            .min(int.dist * (scale_factor / maze.block_size as f32))
            .max(min_dist as f32);
        let d_inc = int.dist / ray_steps as f32;
        for d in 0..ray_steps {
            fb.set_pixel(
                (player_x + d as f32 * d_inc * a.cos()) as u32,
                (player_y + d as f32 * d_inc * a.sin()) as u32,
            );
        }
    }

    // dibujar cuadradito de jugador
    fb.set_current_color(Color::RED);
    for x in player_x as u32 - player_size..player_x as u32 + player_size {
        for y in player_y as u32 - player_size..player_y as u32 + player_size {
            fb.set_pixel(x, y);
        }
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

    // lock cursor para solo la pantalla
    window.disable_cursor();

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
        render3D(&mut player, &mut maze, &mut framebuffer);
        render2D(
            &mut player,
            &mut maze,
            &mut framebuffer,
            250usize,
            50usize,
            50usize,
        );

        framebuffer.swap_buffers(&mut window, &raylib_thread);

        thread::sleep(Duration::from_millis(MS));
    }

    Ok(())
}
