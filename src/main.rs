mod bullet;
mod caster;
mod enemy;
mod framebuffer;
mod helpers;
mod map;
mod maze;
mod player;
mod world_textures;

use crate::bullet::Bullet;
use crate::caster::*;
use crate::enemy::{Enemy, draw_enemies};
use crate::framebuffer::Framebuffer;
use crate::helpers::*;
use crate::map::Map;
use crate::maze::Maze;
use crate::player::{Player, process_input};
use crate::world_textures::{CpuTexture, WorldTextures};
use raylib::prelude::*;
use std::f32::consts::PI;

const SKY_COLOR: Color = Color::SKYBLUE;
const ENEMY_COLOR: Color = Color::new(255, 0, 0, 255);
const FOV: f32 = PI / 3.0;
const FPS: u32 = 120;
const MAP_SIZE: usize = 250;
const MAP_POS: usize = 35;
const BACKGROUND_MUSIC_PATH: &str = "./assets/audio/Rip & Tear - Mick Gordon (128k).mp3";
const GUN_SOUND_PATH: &str = "./assets/audio/deagle.mp3";
const EMPTY_GUN_SOUND_PATH: &str = "./assets/audio/empty_gun.mp3";
const RELOAD_SOUND_PATH: &str = "./assets/audio/gun_reload.mp3";
const VIEWMODEL_PATH: &str = "./assets/sprites/viewmodel.png";
const VIEWMODEL_RELOAD_PATH: &str = "./assets/sprites/viewmodel_reload.png";
const DEFUSE_PATH: &str = "./assets/audio/defuse.mp3";
const BOX_TEXTURE_PATH: &str = "./assets/sprites/box.png";
const FLOOR_TEXTURE_PATH: &str = "./assets/sprites/sand_floor.png";
const BRICKS_TEXTURE_PATH: &str = "./assets/sprites/sand_bricks.png";
const CONCRETE_TEXTURE_PATH: &str = "./assets/sprites/sand_concrete.png";
const WALL1_TEXTURE_PATH: &str = "./assets/sprites/sand_wall1.png";
const WALL2_TEXTURE_PATH: &str = "./assets/sprites/sand_wall2.png";
const DEFUSE_SITE_TEXTURE_PATH: &str = "./assets/sprites/defuse_site.png";
const ENEMY_TEXTURE_PATH: &str = "./assets/sprites/t_model.png";
const DE_DUST2: &str = "./dust2.txt";
const DE_MIRAGE: &str = "./mirage.txt";
const DE_CACHE: &str = "./cache.txt";
const DE_INFERNO: &str = "./inferno.txt";
const BULLET_SIZE: f32 = 2.5;
const BULLET_COLOR: Color = Color::new(255, 40, 40, 255);
const BULLET_INITIAL_OFFSET: f32 = 10.0;
const BULLET_SPEED: f32 = 8000.0;
const BLOCK_SIZE: usize = 100;
const WALL_PROJECTION_DISTANCE: f32 = 200.0;

fn draw_bullets(
    player: &mut Player,
    maze: &mut Maze,
    fb: &mut Framebuffer,
    bullets: &mut Vec<Bullet>,
    enemies: &mut [Enemy],
    dt: f32,
) {
    let projection_distance = (fb.width as f32 / 2.0) / (FOV / 2.0).tan();

    let forward_x = player.a.cos();
    let forward_y = player.a.sin();

    let right_x = -player.a.sin();
    let right_y = player.a.cos();

    for b in bullets.iter_mut() {
        let travel_distance = BULLET_SPEED * dt;
        let bullet_direction = Vector2::new(b.a.cos(), b.a.sin());
        let wall_hit = cast_bullet_ray(b.a, maze, b, travel_distance);
        let enemy_hit =
            Enemy::first_hit_by_bullet(enemies, b.pos, bullet_direction, b.radius, travel_distance);

        if let Some((enemy_index, enemy_distance)) = enemy_hit
            && wall_hit
                .as_ref()
                .is_none_or(|wall| enemy_distance < wall.dist)
        {
            enemies[enemy_index].active = false;
            b.active = false;
            continue;
        }

        if wall_hit.is_some() {
            b.active = false;
            continue;
        }

        b.pos += bullet_direction * travel_distance;

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

fn render3D(
    player: &mut Player,
    maze: &mut Maze,
    fb: &mut Framebuffer,
    textures: &WorldTextures,
) -> Vec<f32> {
    let hh = fb.height as f32 / 2.0;
    let a_0 = player.a - FOV / 2.0;
    let a_step = FOV / fb.width as f32;
    let distance_to_projection = WALL_PROJECTION_DISTANCE;
    let floor_projection_scale = hh * distance_to_projection / 2.0;
    let block_size = maze.block_size as f32;
    let mut wall_depths = vec![f32::INFINITY; fb.width as usize];

    for x in 0..fb.width {
        let a = a_0 + (x as f32) * a_step;
        let ray_cos = a.cos();
        let ray_sin = a.sin();

        let int = cast_ray(a, maze, player);

        // corrección fisheye
        let a_diff = a - player.a;
        let corrected_distance = (int.dist * a_diff.cos()).max(0.001);
        wall_depths[x as usize] = corrected_distance;

        let stake_height = (hh / corrected_distance) * distance_to_projection;
        let stake_top_unclipped = hh - stake_height / 2.0;
        let stake_bottom_unclipped = hh + stake_height / 2.0;
        let stake_top = stake_top_unclipped.max(0.0) as u32;
        let stake_bottom = stake_bottom_unclipped.min(fb.height as f32) as u32;

        fb.set_current_color(SKY_COLOR);
        for y in 0..stake_top {
            fb.set_pixel(x, y);
        }

        let cos_difference = a_diff.cos();
        let floor_distance_scale = floor_projection_scale / cos_difference;
        for y in stake_bottom..fb.height {
            let distance_from_horizon = y as f32 + 0.5 - hh;
            let floor_distance = floor_distance_scale / distance_from_horizon;
            let world_x = player.pos.x + floor_distance * ray_cos;
            let world_y = player.pos.y + floor_distance * ray_sin;
            let texture_u = world_x.rem_euclid(block_size) / block_size;
            let texture_v = world_y.rem_euclid(block_size) / block_size;
            let floor_symbol = maze.cell_at_world(world_x, world_y).unwrap_or(' ');
            let color = textures.floor_color(floor_symbol, texture_u, texture_v);

            fb.set_pixel_color(x, y, color);
        }

        let hit_x = player.pos.x + int.dist * ray_cos;
        let hit_y = player.pos.y + int.dist * ray_sin;
        let local_x = hit_x.rem_euclid(block_size);
        let local_y = hit_y.rem_euclid(block_size);
        let distance_to_vertical_edge = local_x.min(block_size - local_x);
        let distance_to_horizontal_edge = local_y.min(block_size - local_y);
        let texture_u = if distance_to_vertical_edge < distance_to_horizontal_edge {
            local_y / block_size
        } else {
            local_x / block_size
        };
        let wall_texture = textures.wall_for(int.object);

        let texture_v_step = 1.0 / stake_height;
        let mut texture_v = (stake_top as f32 + 0.5 - stake_top_unclipped) * texture_v_step;
        for y in stake_top..stake_bottom {
            let color = wall_texture.sample(texture_u, texture_v);

            fb.set_pixel_color(x, y, color);
            texture_v += texture_v_step;
        }
    }

    wall_depths
}

fn render2D(
    player: &mut Player,
    maze: &mut Maze,
    fb: &mut Framebuffer,
    enemies: &mut Vec<Enemy>,
    static_pixels: &mut Vec<PixelColor>,
) {
    let maze_height = maze.grid.len();
    let maze_width = maze.grid.iter().map(Vec::len).max().unwrap();

    let scale_factor = MAP_SIZE as f32 / maze_width.max(maze_height) as f32;

    // mapa
    for p in static_pixels {
        fb.set_current_color(p.color);
        fb.set_pixel(p.x, p.y);
    }

    let player_x = player.pos.x * (scale_factor / maze.block_size as f32) + MAP_POS as f32;
    let player_y = player.pos.y * (scale_factor / maze.block_size as f32) + MAP_POS as f32;
    let player_size = 2; // rect de 2*playersize + 1

    // dibujar fov 2d
    fb.set_current_color(Color::new(0, 255, 255, 255));
    let ray_amount = 50;
    let a_0 = player.a - FOV / 2.0;
    let a_inc = FOV / ray_amount as f32;
    let max_dist = (player_size + 1) * 4;
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
    fb.set_current_color(Color::BLUE);
    for x in player_x as u32 - player_size..player_x as u32 + player_size {
        for y in player_y as u32 - player_size..player_y as u32 + player_size {
            fb.set_pixel(x, y);
        }
    }

    // enemigos
    let alive_enemies = enemies.iter().filter(|e| e.active);
    fb.set_current_color(ENEMY_COLOR);
    for e in alive_enemies {
        let enemy_x = e.pos.x * (scale_factor / maze.block_size as f32) + MAP_POS as f32;
        let enemy_y = e.pos.y * (scale_factor / maze.block_size as f32) + MAP_POS as f32;

        // usar player_size para que los enemigos sean del mismo tamaño que el jugador
        for x in enemy_x as u32 - player_size..enemy_x as u32 + player_size {
            for y in enemy_y as u32 - player_size..enemy_y as u32 + player_size {
                fb.set_pixel(x, y);
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let window_width = 1280;
    let window_height = 720;
    // downscaling
    let framebuffer_width = (window_width as f32 / 1.25) as u32;
    let framebuffer_height = (window_height as f32 / 1.25) as u32;

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

    let defuse = audio
        .new_sound(DEFUSE_PATH)
        .expect("failed to load defuse sound effects");

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

    let box_image = Image::load_image(BOX_TEXTURE_PATH).expect("failed to load box texture");
    let floor_image = Image::load_image(FLOOR_TEXTURE_PATH).expect("failed to load floor texture");
    let bricks_image =
        Image::load_image(BRICKS_TEXTURE_PATH).expect("failed to load bricks texture");
    let concrete_image =
        Image::load_image(CONCRETE_TEXTURE_PATH).expect("failed to load concrete texture");
    let wall1_image = Image::load_image(WALL1_TEXTURE_PATH).expect("failed to load wall1 texture");
    let wall2_image = Image::load_image(WALL2_TEXTURE_PATH).expect("failed to load wall2 texture");
    let defuse_site_image =
        Image::load_image(DEFUSE_SITE_TEXTURE_PATH).expect("failed to load defuse-site texture");
    let enemy_image = Image::load_image(ENEMY_TEXTURE_PATH).expect("failed to load enemy texture");
    let enemy_texture = CpuTexture::from_image(&enemy_image);

    let world_textures = WorldTextures {
        box_texture: CpuTexture::from_image(&box_image),
        floor: CpuTexture::from_image(&floor_image),
        bricks: CpuTexture::from_image(&bricks_image),
        concrete: CpuTexture::from_image(&concrete_image),
        wall1: CpuTexture::from_image(&wall1_image),
        wall2: CpuTexture::from_image(&wall2_image),
        defuse_site: CpuTexture::from_image(&defuse_site_image),
    };

    // lock cursor para solo la pantalla
    window.disable_cursor();
    window.set_target_fps(FPS);

    let mut framebuffer = Framebuffer::new(
        framebuffer_width,
        framebuffer_height,
        &mut window,
        &raylib_thread,
    );

    music.play();

    let de_dust2 = Map::new(DE_DUST2.to_string(), BLOCK_SIZE)?;
    let de_mirage = Map::new(DE_MIRAGE.to_string(), BLOCK_SIZE)?;
    let de_inferno = Map::new(DE_INFERNO.to_string(), BLOCK_SIZE)?;
    let de_cache = Map::new(DE_CACHE.to_string(), BLOCK_SIZE)?;

    let current_map = de_inferno;
    let mut maze = current_map.maze;
    let mut player = Player::new(current_map.spawn_location.x, current_map.spawn_location.y);
    let mut enemies = Enemy::from_maze(&maze);
    let mut current_gun_cooldown = 0.0;
    let mut current_reload_cooldown = 0.0;
    let mut current_defuse_counter = 0.0;
    let mut has_defused = false;
    let mut defused_A = false;
    let mut defused_B = false;
    let mut current_defused: u8 = 0;
    let mut bullets: Vec<Bullet> = Vec::new();

    // Image::load_image para cargar a ram
    // window.load_texture para cargar a tarjeta de video

    let mut static_radar_pixels = get_static_radar_pixels(&maze, MAP_SIZE, MAP_POS, MAP_POS);

    while !window.window_should_close() {
        if !music.is_playing() {
            music.play()
        }

        let dt = window.get_frame_time();
        let was_reloading = player.reloading;
        process_input(&mut player, &mut window, &mut maze);

        if player.reloading && !was_reloading {
            reload.play();
        }

        let wall_depths = render3D(&mut player, &mut maze, &mut framebuffer, &world_textures);

        draw_enemies(
            &enemies,
            &player,
            &mut framebuffer,
            &enemy_texture,
            &wall_depths,
            FOV,
            WALL_PROJECTION_DISTANCE,
        );

        if player.ammo == 0
            && window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            && !player.reloading
        {
            empty_gun.play();
        }

        if player.reloading {
            current_reload_cooldown += window.get_frame_time();
        }

        let grid_x = (player.pos.x / maze.block_size as f32) as usize;
        let grid_y = (player.pos.y / maze.block_size as f32) as usize;
        let player_on_site = maze.grid[grid_y][grid_x] == '1' || maze.grid[grid_y][grid_x] == '2';
        let defused_current_site = (maze.grid[grid_y][grid_x] == '1' && defused_A)
            || (maze.grid[grid_y][grid_x] == '2' && defused_B);
        let not_defused_current_site = (maze.grid[grid_y][grid_x] == '1' && !defused_A)
            || (maze.grid[grid_y][grid_x] == '2' && !defused_B);

        if player.defusing && player_on_site && not_defused_current_site {
            current_defuse_counter += window.get_frame_time();
            if !has_defused {
                has_defused = true;
                if !defuse.is_playing() {
                    defuse.play();
                }
            }
        } else {
            current_defuse_counter = 0.0;
            has_defused = false;
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

        draw_bullets(
            &mut player,
            &mut maze,
            &mut framebuffer,
            &mut bullets,
            &mut enemies,
            dt,
        );

        if current_reload_cooldown >= player.reload_time {
            player.ammo = 7;
            player.reloading = false;
            current_reload_cooldown = 0.0;
        }

        if current_gun_cooldown >= player.gun_cooldown {
            player.fired = !player.fired;
            current_gun_cooldown = 0.0;
        }

        if current_defuse_counter >= player.defuse_time {
            if maze.grid[grid_y][grid_x] == '1' {
                defused_A = true;
            } else if maze.grid[grid_y][grid_x] == '2' {
                defused_B = true;
            }
            current_defused += 1;
            player.defusing = false;
            has_defused = false;
            current_defuse_counter = 0.0;
        }

        render2D(
            &mut player,
            &mut maze,
            &mut framebuffer,
            &mut enemies,
            &mut static_radar_pixels,
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
            player.ammo,
            (current_defuse_counter * 100.0).round() / 100.0,
            player.defusing && player_on_site,
            current_defused,
            defused_current_site,
        );
    }

    Ok(())
}
