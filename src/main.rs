mod bullet;
mod caster;
mod framebuffer;
mod maze;
mod player;

use crate::bullet::Bullet;
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
const BACKGROUND_MUSIC_PATH: &str = "./assets/audio/Rip & Tear - Mick Gordon (128k).mp3";
const GUN_SOUND_PATH: &str = "./assets/audio/deagle.mp3";
const EMPTY_GUN_SOUND_PATH: &str = "./assets/audio/empty_gun.mp3";
const RELOAD_SOUND_PATH: &str = "./assets/audio/gun_reload.mp3";
const VIEWMODEL_PATH: &str = "./assets/sprites/viewmodel.png";
const VIEWMODEL_RELOAD_PATH: &str = "./assets/sprites/viewmodel_reload.png";
const VIEWMODEL_HEIGHT_RATIO: f32 = 0.62;
const VIEWMODEL_RIGHT_OFFSET: f32 = 110.0;
const BULLET_SIZE: f32 = 2.5;
const BULLET_COLOR: Color = Color::new(84, 84, 84, 255);
const BULLET_INITIAL_OFFSET: f32 = 10.0;
const BULLET_SPEED: f32 = 2000.0;

fn draw_bullets(
    player: &mut Player,
    maze: &mut Maze,
    fb: &mut Framebuffer,
    bullets: &mut Vec<Bullet>,
    dt: f32,
) {
    let projection_distance = (fb.width as f32 / 2.0) / (FOV / 2.0).tan();

    let forward_x = player.a.cos();
    let forward_y = player.a.sin();

    let right_x = -player.a.sin();
    let right_y = player.a.cos();

    for b in bullets.iter_mut() {
        let travel_distance = BULLET_SPEED * dt;

        if cast_bullet_ray(b.a, maze, b, travel_distance).is_some() {
            b.active = false;
            continue;
        }

        b.pos.x += travel_distance * b.a.cos();
        b.pos.y += travel_distance * b.a.sin();

        let relative_x = b.pos.x - player.pos.x;
        let relative_y = b.pos.y - player.pos.y;

        let depth = relative_x * forward_x + relative_y * forward_y;
        let sideways = relative_x * right_x + relative_y * right_y;

        if depth <= 0.0 {
            continue;
        }

        // no dibujar balas que estén detrás de una pared
        let distance_to_bullet = relative_x.hypot(relative_y);
        let angle_to_bullet = relative_y.atan2(relative_x);
        let distance_to_wall = cast_ray(angle_to_bullet, maze, player).dist;

        if distance_to_wall < distance_to_bullet - b.radius {
            continue;
        }

        let screen_x = fb.width as f32 / 2.0 + sideways * projection_distance / depth;
        let screen_y = fb.height as f32 / 2.0;
        let screen_radius = b.radius * projection_distance / depth;

        if b.active {
            draw_filled_circle(
                fb,
                screen_x as i32,
                screen_y as i32,
                screen_radius as i32,
                BULLET_COLOR,
            );
        }
    }

    bullets.retain(|b| b.active);
}

fn draw_filled_circle(
    fb: &mut Framebuffer,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: Color,
) {
    for offset_x in -radius..=radius {
        let squared_height = radius * radius - offset_x * offset_x; // r^2 - x^2 = y^2

        let half_height = (squared_height as f32).sqrt() as i32;

        let pixel_x = center_x + offset_x;

        for pixel_y in center_y - half_height..=center_y + half_height {
            if pixel_x >= 0 && pixel_y >= 0 {
                fb.set_current_color(color);
                fb.set_pixel(pixel_x as u32, pixel_y as u32);
            }
        }
    }
}

fn viewmodel_destination(texture: &Texture2D, screen_width: i32, screen_height: i32) -> Rectangle {
    let target_height = screen_height as f32 * VIEWMODEL_HEIGHT_RATIO;
    let scale = target_height / texture.height() as f32;
    let target_width = texture.width() as f32 * scale;

    Rectangle::new(
        screen_width as f32 - target_width - VIEWMODEL_RIGHT_OFFSET,
        screen_height as f32 - target_height,
        target_width,
        target_height,
    )
}

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

    let audio = RaylibAudio::init_audio_device().expect("audio init fail");
    let music = audio
        .new_sound(BACKGROUND_MUSIC_PATH)
        .expect("failed to load background soung");

    let gun = audio
        .new_sound(GUN_SOUND_PATH)
        .expect("failed to load gun sound effects");

    let empty_gun = audio
        .new_sound(EMPTY_GUN_SOUND_PATH)
        .expect("failed to load empty gun sound effects");

    let reload = audio
        .new_sound(RELOAD_SOUND_PATH)
        .expect("failed to load gun reload sound effects");

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Maze :D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let viewmodel_image =
        Image::load_image(VIEWMODEL_PATH).expect("failed to load regular viewmodel image");
    let viewmodel_texture = window
        .load_texture_from_image(&raylib_thread, &viewmodel_image)
        .expect("failed to upload regular viewmodel texture");

    let viewmodel_reload_image =
        Image::load_image(VIEWMODEL_RELOAD_PATH).expect("failed to load reload viewmodel image");
    let viewmodel_reload_texture = window
        .load_texture_from_image(&raylib_thread, &viewmodel_reload_image)
        .expect("failed to upload reload viewmodel texture");

    // lock cursor para solo la pantalla
    window.disable_cursor();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);

    music.play();

    let mut maze = Maze::new("maze.txt", block_size)?;
    let mut player = Player::new(
        (3.0 / 2.0) * block_size as f32,
        (3.0 / 2.0) * block_size as f32,
    );
    let mut current_gun_cooldown = 0.0;
    let mut current_reload_cooldown = 0.0;
    let mut bullets: Vec<Bullet> = Vec::new();

    // Image::load_image para cargar a ram
    // window.load_texture para cargar a tarjeta de video

    while !window.window_should_close() {
        if !music.is_playing() {
            music.play()
        }

        let dt = window.get_frame_time();
        process_input(&mut player, &mut window, &mut maze);

        render3D(&mut player, &mut maze, &mut framebuffer);

        if player.ammo == 0
            && window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            && !player.reloading
        {
            empty_gun.play();
        }

        if player.reloading {
            current_reload_cooldown += window.get_frame_time();
            if !reload.is_playing() {
                reload.play();
            }
        }

        if player.fired && player.ammo > 0 && !player.reloading {
            if current_gun_cooldown == 0.0 {
                let initial = Vector2::new(
                    player.pos.x + BULLET_INITIAL_OFFSET * player.a.cos(),
                    player.pos.y + BULLET_INITIAL_OFFSET * player.a.sin(),
                );
                let bullet = Bullet {
                    pos: initial,
                    radius: BULLET_SIZE,
                    color: BULLET_COLOR,
                    a: player.a,
                    active: true,
                };
                gun.play();
                bullets.push(bullet);
                player.ammo -= 1;
            }
            current_gun_cooldown += window.get_frame_time();
        }

        draw_bullets(&mut player, &mut maze, &mut framebuffer, &mut bullets, dt);

        if current_reload_cooldown >= player.reload_time {
            player.ammo = 7;
            player.reloading = false;
            current_reload_cooldown = 0.0;
        }

        if current_gun_cooldown >= player.gun_cooldown {
            player.fired = !player.fired;
            current_gun_cooldown = 0.0;
        }

        render2D(
            &mut player,
            &mut maze,
            &mut framebuffer,
            250usize,
            50usize,
            50usize,
        );

        let active_viewmodel = if player.reloading {
            &viewmodel_reload_texture
        } else {
            &viewmodel_texture
        };
        let viewmodel_dest = viewmodel_destination(
            active_viewmodel,
            window.get_render_width(),
            window.get_render_height(),
        );

        framebuffer.swap_buffers(
            &mut window,
            &raylib_thread,
            active_viewmodel,
            viewmodel_dest,
        );

        thread::sleep(Duration::from_millis(MS));
    }

    Ok(())
}
