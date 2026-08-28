use crate::bullet::Bullet;
use crate::caster::{cast_bullet_ray, cast_ray};
use crate::enemy::Enemy;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::{
    BOMB_COLOR, BOX_COLOR, BULLET_COLOR, BULLET_SPEED, FLOOR_COLOR, FOV, OOB_COLOR,
    VIEWMODEL_HEIGHT_RATIO, VIEWMODEL_RIGHT_OFFSET, WALL_COLOR,
};
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    MapSelect,
    InGame,
    Victory,
}

pub struct PixelColor {
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

pub struct HudElements {
    pub ammo: i8,
    pub mag: i8,
    pub defuse_counter: f32,
    pub defusing: bool,
    pub current_defused: u8,
    pub defused_current_site: bool,
}

// el mapa en sí nunca cambia, saltar los checks de renderizar pixeles estáticos cada frame y solo
// renderizarlo una vez
pub fn get_static_radar_pixels(
    maze: &Maze,
    map_size: usize,
    x: usize,
    y: usize,
) -> Vec<PixelColor> {
    let mut static_pixels: Vec<PixelColor> = Vec::new();
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
                '1' | '2' => {
                    static_pixels.push(PixelColor {
                        x: (x + map_x) as u32,
                        y: (y + map_y) as u32,
                        color: (BOMB_COLOR),
                    });
                }
                'B' => {
                    static_pixels.push(PixelColor {
                        x: (x + map_x) as u32,
                        y: (y + map_y) as u32,
                        color: (BOX_COLOR),
                    });
                }
                ' ' | 'B' | 'P' | 'T' => {
                    static_pixels.push(PixelColor {
                        x: (x + map_x) as u32,
                        y: (y + map_y) as u32,
                        color: (FLOOR_COLOR),
                    });
                }
                '-' => {
                    static_pixels.push(PixelColor {
                        x: (x + map_x) as u32,
                        y: (y + map_y) as u32,
                        color: (OOB_COLOR),
                    });
                }
                _ => static_pixels.push(PixelColor {
                    x: (x + map_x) as u32,
                    y: (y + map_y) as u32,
                    color: (WALL_COLOR),
                }),
            }
        }
    }

    static_pixels
}

pub fn draw_filled_circle(
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

pub fn viewmodel_destination(
    texture: &Texture2D,
    screen_width: i32,
    screen_height: i32,
) -> Rectangle {
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

pub fn draw_bullets(
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
