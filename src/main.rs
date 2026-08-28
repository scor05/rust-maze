mod bullet;
mod caster;
mod enemy;
mod framebuffer;
mod helpers;
mod interface;
mod map;
mod maze;
mod player;
mod world_textures;

use crate::bullet::Bullet;
use crate::caster::*;
use crate::enemy::{Enemy, draw_enemies};
use crate::framebuffer::Framebuffer;
use crate::helpers::*;
use crate::interface::*;
use crate::map::Map;
use crate::maze::Maze;
use crate::player::{Player, process_ingame_input};
use crate::world_textures::{CpuTexture, WorldTextures};
use raylib::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;

// colores
const FLOOR_COLOR: Color = Color::GRAY;
const OOB_COLOR: Color = Color::BLACK;
const BOX_COLOR: Color = Color::new(77, 33, 12, 0xff);
const WALL_COLOR: Color = Color::new(251, 255, 219, 0xff);
const BOMB_COLOR: Color = Color::new(225, 255, 0, 255);
const SKY_COLOR: Color = Color::SKYBLUE;
const ENEMY_COLOR: Color = Color::new(255, 0, 0, 255);

// settings
const FOV: f32 = PI / 3.0;
const FPS: u32 = 120;
const MAP_SIZE: usize = 250;
const MAP_POS: usize = 35;
const BULLET_SIZE: f32 = 2.5;
const BULLET_SPEED: f32 = 8000.0;
const BULLET_INITIAL_OFFSET: f32 = 10.0;
const BULLET_COLOR: Color = Color::new(255, 40, 40, 255);
const BLOCK_SIZE: usize = 100;
const WALL_PROJECTION_DISTANCE: f32 = 300.0;

// audios
const PLAYING_MUSIC_PATH: &str = "./assets/audio/Rip & Tear - Mick Gordon (128k).mp3";
const MAIN_MENU_MUSIC_PATH: &str = "./assets/audio/main_menu.mp3";
const VICTORY_SOUND_PATH: &str = "./assets/audio/victory.mp3";
const GUN_SOUND_PATH: &str = "./assets/audio/deagle.mp3";
const EMPTY_GUN_SOUND_PATH: &str = "./assets/audio/empty_gun.mp3";
const RELOAD_SOUND_PATH: &str = "./assets/audio/gun_reload.mp3";
const DEFUSE_PATH: &str = "./assets/audio/defuse.mp3";

// viewmodel
const VIEWMODEL_PATH: &str = "./assets/sprites/viewmodel.png";
const VIEWMODEL_RELOAD_PATH: &str = "./assets/sprites/viewmodel_reload.png";
const VIEWMODEL_HEIGHT_RATIO: f32 = 0.62;
const VIEWMODEL_RIGHT_OFFSET: f32 = 110.0;

// texturas
const BOX_TEXTURE_PATH: &str = "./assets/sprites/box.png";
const FLOOR_TEXTURE_PATH: &str = "./assets/sprites/sand_floor.png";
const BRICKS_TEXTURE_PATH: &str = "./assets/sprites/sand_bricks.png";
const CONCRETE_TEXTURE_PATH: &str = "./assets/sprites/sand_concrete.png";
const WALL1_TEXTURE_PATH: &str = "./assets/sprites/sand_wall1.png";
const WALL2_TEXTURE_PATH: &str = "./assets/sprites/sand_wall2.png";
const DEFUSE_SITE_TEXTURE_PATH: &str = "./assets/sprites/defuse_site.png";
const ENEMY_TEXTURE_PATH: &str = "./assets/sprites/t_model.png";

// menus
const MAIN_MENU_SPRITE_PATH: &str = "./assets/sprites/titlescreen.png";
const MAP_SELECT_SPRITE_PATH: &str = "./assets/sprites/map_select.png";
const VICTORY_SCREEN_SPRITE_PATH: &str = "./assets/sprites/win_screen.jpg";

// mapas
const DE_DUST2: &str = "./dust2.txt";
const DE_MIRAGE: &str = "./mirage.txt";
const DE_CACHE: &str = "./cache.txt";
const DE_INFERNO: &str = "./inferno.txt";

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
    let framebuffer_width = (window_width as f32 / 1.35) as u32; // downscaling
    let framebuffer_height = (window_height as f32 / 1.35) as u32;

    // init
    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Maze :D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    let mut framebuffer = Framebuffer::new(
        framebuffer_width,
        framebuffer_height,
        &mut window,
        &raylib_thread,
    );

    // audios
    let audio = RaylibAudio::init_audio_device().expect("audio init fail");
    let main_menu_music = audio
        .new_sound(MAIN_MENU_MUSIC_PATH)
        .expect("failed to load background soung");
    let playing_music = audio
        .new_sound(PLAYING_MUSIC_PATH)
        .expect("failed to load background soung");
    let victory_sound = audio
        .new_sound(VICTORY_SOUND_PATH)
        .expect("failed to load defuse sound effects");
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

    // viewmodel
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

    // texturas
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

    // texturas de menú
    let main_menu_image =
        Image::load_image(MAIN_MENU_SPRITE_PATH).expect("failed to load main menu image");
    let main_menu_texture = window
        .load_texture_from_image(&raylib_thread, &main_menu_image)
        .expect("failed to upload main menu screen texture");
    let map_select_image =
        Image::load_image(MAP_SELECT_SPRITE_PATH).expect("failed to load map select image");
    let map_select_texture = window
        .load_texture_from_image(&raylib_thread, &map_select_image)
        .expect("failed to upload map select texture");
    let victory_image =
        Image::load_image(VICTORY_SCREEN_SPRITE_PATH).expect("failed to load victory-screen image");
    let victory_screen_texture = window
        .load_texture_from_image(&raylib_thread, &victory_image)
        .expect("failed to upload victory-screen texture");

    window.set_target_fps(FPS);

    // cargar mapas
    let de_dust2 = Map::new(DE_DUST2.to_string(), BLOCK_SIZE, "Dust 2")?;
    let de_mirage = Map::new(DE_MIRAGE.to_string(), BLOCK_SIZE, "Mirage")?;
    let de_inferno = Map::new(DE_INFERNO.to_string(), BLOCK_SIZE, "Inferno")?;
    let de_cache = Map::new(DE_CACHE.to_string(), BLOCK_SIZE, "Cache")?;
    let mut maps = HashMap::new();
    maps.insert(de_dust2.name, de_dust2);
    maps.insert(de_mirage.name, de_mirage);
    maps.insert(de_inferno.name, de_inferno);
    maps.insert(de_cache.name, de_cache);
    let mut current_map = &maps["Inferno"];

    // botones de interfaces
    let main_menu_buttons: Vec<Button> = vec![
        Button::new(458, 960, 507, 549, "play"),       // play
        Button::new(459, 852, 567, 623, "map select"), // map Select
        Button::new(458, 960, 642, 684, "exit"),       // exit Game
    ];
    let map_select_buttons: Vec<Button> = vec![
        Button::new(18, 270, 19, 312, "random"),    // random
        Button::new(304, 555, 20, 312, "cache"),    // cache
        Button::new(588, 840, 20, 312, "dust 2"),   // dust 2
        Button::new(871, 1122, 20, 312, "inferno"), // inferno
        Button::new(1155, 1407, 20, 312, "mirage"), // mirage
        Button::new(42, 387, 906, 999, "back"),     // back
    ];
    let victory_screen_buttons: Vec<Button> = vec![
        Button::new(554, 763, 453, 528, "return"), // return to menu
    ];

    // textos de interfaces
    let map_select_text: Vec<InterfaceText> = vec![InterfaceText {
        text: current_map.name,
        x: 1115,
        y: 927,
        font_size: 50,
        outline_size: 0,
    }];

    // interfaces
    let mut main_menu: Interface = Interface::new(main_menu_buttons, vec![], main_menu_texture);
    let mut map_select: Interface =
        Interface::new(map_select_buttons, map_select_text, map_select_texture);
    let mut victory: Interface =
        Interface::new(victory_screen_buttons, vec![], victory_screen_texture);

    // init jugador/enemigos/mapa
    let mut maze = current_map.maze.clone();
    let mut game_state = GameState::MainMenu;
    let mut cursor_state = game_state;
    let mut player = Player::new(current_map.spawn_location.x, current_map.spawn_location.y);
    let mut enemies = Enemy::from_maze(&maze);
    let mut bullets: Vec<Bullet> = Vec::new();
    window.enable_cursor();

    // cooldowns/checks
    let mut current_gun_cooldown = 0.0;
    let mut current_reload_cooldown = 0.0;
    let mut current_defuse_counter = 0.0;
    let mut has_defused = false;
    let mut defused_a = false;
    let mut defused_b = false;
    let mut current_defused: u8 = 0;

    // Image::load_image para cargar a ram
    // window.load_texture para cargar a tarjeta de video

    'game_loop: while !window.window_should_close() {
        if game_state != cursor_state {
            match game_state {
                GameState::InGame => {
                    window.disable_cursor();
                    maze = current_map.maze.clone();
                    player =
                        Player::new(current_map.spawn_location.x, current_map.spawn_location.y);
                    enemies = Enemy::from_maze(&maze);
                    main_menu_music.stop();
                    playing_music.play();
                    defused_a = false;
                    defused_b = false;
                }
                GameState::Victory => {
                    window.enable_cursor();
                    playing_music.stop();
                    victory_sound.play();
                }
                GameState::MainMenu | GameState::MapSelect => {
                    window.enable_cursor();
                    playing_music.stop();
                    victory_sound.stop();
                    if !main_menu_music.is_playing() {
                        main_menu_music.play();
                    }
                }
            }
            cursor_state = game_state;
        }

        match game_state {
            GameState::MainMenu => {
                framebuffer.swap_buffers(
                    &mut window,
                    &raylib_thread,
                    None,
                    None,
                    None,
                    Some(&main_menu),
                );

                if !main_menu_music.is_playing() {
                    main_menu_music.play();
                }

                if window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    let mouse_pos = window.get_mouse_position();
                    let image_x = mouse_pos.x * main_menu.image.width as f32
                        / window.get_render_width().max(1) as f32;
                    let image_y = mouse_pos.y * main_menu.image.height as f32
                        / window.get_render_height().max(1) as f32;

                    for b in main_menu.buttons.iter_mut() {
                        if b.check_click(image_x as u32, image_y as u32) {
                            match b.label {
                                "play" => {
                                    game_state = GameState::InGame;
                                }
                                "map select" => {
                                    game_state = GameState::MapSelect;
                                }
                                "exit" => {
                                    break 'game_loop;
                                }
                                _ => {}
                            }
                            continue;
                        }
                    }
                }
            }
            GameState::MapSelect => {
                if !main_menu_music.is_playing() {
                    main_menu_music.play();
                }

                if window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    let mouse_pos = window.get_mouse_position();
                    let image_x = mouse_pos.x * map_select.image.width as f32
                        / window.get_render_width().max(1) as f32;
                    let image_y = mouse_pos.y * map_select.image.height as f32
                        / window.get_render_height().max(1) as f32;

                    for b in map_select.buttons.iter_mut() {
                        if b.check_click(image_x as u32, image_y as u32) {
                            let mut selected_random = false;
                            match b.label {
                                "random" => {
                                    selected_random = true;
                                    let random_index: i32 =
                                        window.get_random_value(0..=((maps.len() - 1) as i32));
                                    if let Some(&random_key) =
                                        maps.keys().nth(random_index as usize)
                                    {
                                        if let Some(random_val) = maps.get(&random_key) {
                                            current_map = random_val;
                                        }
                                    }
                                }
                                "mirage" => {
                                    current_map = &maps["Mirage"];
                                }
                                "inferno" => {
                                    current_map = &maps["Inferno"];
                                }
                                "dust 2" => {
                                    current_map = &maps["Dust 2"];
                                }
                                "cache" => {
                                    current_map = &maps["Cache"];
                                }
                                "back" => {
                                    game_state = GameState::MainMenu;
                                }
                                _ => {}
                            }
                            map_select.text[0].text = if selected_random {
                                "Random"
                            } else {
                                current_map.name
                            };
                            continue;
                        }
                    }
                }

                framebuffer.swap_buffers(
                    &mut window,
                    &raylib_thread,
                    None,
                    None,
                    None,
                    Some(&map_select),
                );
            }
            GameState::Victory => {
                framebuffer.swap_buffers(
                    &mut window,
                    &raylib_thread,
                    None,
                    None,
                    None,
                    Some(&victory),
                );

                if window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    let mouse_pos = window.get_mouse_position();
                    let image_x = mouse_pos.x * victory.image.width as f32
                        / window.get_render_width().max(1) as f32;
                    let image_y = mouse_pos.y * victory.image.height as f32
                        / window.get_render_height().max(1) as f32;

                    for b in victory.buttons.iter_mut() {
                        if b.check_click(image_x as u32, image_y as u32) {
                            match b.label {
                                "return" => {
                                    game_state = GameState::MainMenu;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            GameState::InGame => {
                if !playing_music.is_playing() {
                    playing_music.play()
                }

                let dt = window.get_frame_time();

                maze = current_map.maze.clone();
                let mut static_radar_pixels =
                    get_static_radar_pixels(&maze, MAP_SIZE, MAP_POS, MAP_POS);

                let was_reloading = player.reloading;
                let reload_was_pressed = window.is_key_pressed(KeyboardKey::KEY_R);

                if window.is_key_pressed(KeyboardKey::KEY_Q) {
                    game_state = GameState::MainMenu;
                }

                if defused_a && defused_b {
                    game_state = GameState::Victory;
                }

                process_ingame_input(&mut player, &mut window, &mut maze);

                if player.reloading && !was_reloading {
                    reload.play();
                } else if reload_was_pressed && !was_reloading && player.mag == 0 {
                    empty_gun.play();
                }

                let wall_depths =
                    render3D(&mut player, &mut maze, &mut framebuffer, &world_textures);

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

                let needed_ammo = player.max_ammo - player.ammo;
                let transfer_ammo = needed_ammo.min(player.mag);
                if player.reloading && player.ammo < player.max_ammo && player.mag > 0 {
                    current_reload_cooldown += window.get_frame_time();
                }

                let grid_x = (player.pos.x / maze.block_size as f32) as usize;
                let grid_y = (player.pos.y / maze.block_size as f32) as usize;
                let player_on_site =
                    maze.grid[grid_y][grid_x] == '1' || maze.grid[grid_y][grid_x] == '2';
                let defused_current_site = (maze.grid[grid_y][grid_x] == '1' && defused_a)
                    || (maze.grid[grid_y][grid_x] == '2' && defused_b);
                let not_defused_current_site = (maze.grid[grid_y][grid_x] == '1' && !defused_a)
                    || (maze.grid[grid_y][grid_x] == '2' && !defused_b);

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
                    player.mag -= transfer_ammo;
                    player.ammo += transfer_ammo;
                    player.reloading = false;
                    current_reload_cooldown = 0.0;
                }

                if current_gun_cooldown >= player.gun_cooldown {
                    player.fired = !player.fired;
                    current_gun_cooldown = 0.0;
                }

                if current_defuse_counter >= player.defuse_time {
                    if maze.grid[grid_y][grid_x] == '1' {
                        defused_a = true;
                    } else if maze.grid[grid_y][grid_x] == '2' {
                        defused_b = true;
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

                let hud_elements = HudElements {
                    ammo: player.ammo,
                    mag: player.mag,
                    defuse_counter: (current_defuse_counter * 100.0).round() / 100.0,
                    defusing: player.defusing && player_on_site,
                    current_defused,
                    defused_current_site,
                };

                framebuffer.swap_buffers(
                    &mut window,
                    &raylib_thread,
                    Some(active_viewmodel),
                    Some(viewmodel_dest),
                    Some(&hud_elements),
                    None,
                );
            }
        }
    }

    Ok(())
}
