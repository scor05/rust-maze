use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::world_textures::CpuTexture;
use raylib::prelude::Vector2;

const ENEMY_RADIUS_RATIO: f32 = 0.25;
const ENEMY_HEIGHT_RATIO: f32 = 0.9;

pub struct Enemy {
    pub pos: Vector2,
    pub radius: f32,
    pub active: bool,
}

impl Enemy {
    pub fn from_maze(maze: &Maze) -> Vec<Self> {
        let block_size = maze.block_size as f32;

        maze.grid
            .iter()
            .enumerate()
            .flat_map(|(row, cells)| {
                cells
                    .iter()
                    .enumerate()
                    .filter(|(_, symbol)| **symbol == 'T')
                    .map(move |(column, _)| Self {
                        pos: Vector2::new(
                            (column as f32 + 0.5) * block_size,
                            (row as f32 + 0.5) * block_size,
                        ),
                        radius: block_size * ENEMY_RADIUS_RATIO,
                        active: true,
                    })
            })
            .collect()
    }

    pub fn first_hit_by_bullet(
        enemies: &[Self],
        bullet_position: Vector2,
        bullet_direction: Vector2,
        bullet_radius: f32,
        max_distance: f32,
    ) -> Option<(usize, f32)> {
        enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| enemy.active)
            .filter_map(|(index, enemy)| {
                let to_enemy = enemy.pos - bullet_position;
                let distance_along_ray = to_enemy.dot(bullet_direction);
                let combined_radius = enemy.radius + bullet_radius;

                if distance_along_ray < -combined_radius
                    || distance_along_ray > max_distance + combined_radius
                {
                    return None;
                }

                let sideways_distance_squared =
                    to_enemy.length_sqr() - distance_along_ray * distance_along_ray;

                if sideways_distance_squared > combined_radius * combined_radius {
                    return None;
                }

                let distance_to_circle_edge =
                    (combined_radius * combined_radius - sideways_distance_squared.max(0.0)).sqrt();
                let hit_distance = (distance_along_ray - distance_to_circle_edge).max(0.0);

                Some((index, hit_distance))
            })
            .min_by(|(_, first_distance), (_, second_distance)| {
                first_distance.total_cmp(second_distance)
            })
    }
}

pub fn draw_enemies(
    enemies: &[Enemy],
    player: &Player,
    framebuffer: &mut Framebuffer,
    texture: &CpuTexture,
    wall_depths: &[f32],
    fov: f32,
    wall_projection_distance: f32,
) {
    let forward = Vector2::new(player.a.cos(), player.a.sin());
    let right = Vector2::new(-player.a.sin(), player.a.cos());
    let screen_projection_distance = (framebuffer.width as f32 / 2.0) / (fov / 2.0).tan();
    let half_height = framebuffer.height as f32 / 2.0;

    let mut visible_enemies: Vec<(&Enemy, f32, f32)> = enemies
        .iter()
        .filter(|enemy| enemy.active)
        .filter_map(|enemy| {
            let relative = enemy.pos - player.pos;
            let depth = relative.dot(forward);

            (depth > 0.01).then(|| (enemy, depth, relative.dot(right)))
        })
        .collect();

    visible_enemies.sort_by(|first, second| second.1.total_cmp(&first.1));

    for (_, depth, sideways) in visible_enemies {
        let cell_screen_height = half_height * wall_projection_distance / depth;
        let sprite_height = cell_screen_height * ENEMY_HEIGHT_RATIO;
        let sprite_width = sprite_height * texture.aspect_ratio();
        let center_x =
            framebuffer.width as f32 / 2.0 + sideways * screen_projection_distance / depth;
        let sprite_left = center_x - sprite_width / 2.0;
        let sprite_top = half_height + cell_screen_height / 2.0 - sprite_height;
        let sprite_right = sprite_left + sprite_width;
        let sprite_bottom = sprite_top + sprite_height;

        if sprite_right <= 0.0
            || sprite_left >= framebuffer.width as f32
            || sprite_bottom <= 0.0
            || sprite_top >= framebuffer.height as f32
        {
            continue;
        }

        let first_x = sprite_left.max(0.0) as u32;
        let last_x = sprite_right.min(framebuffer.width as f32).ceil() as u32;
        let first_y = sprite_top.max(0.0) as u32;
        let last_y = sprite_bottom.min(framebuffer.height as f32).ceil() as u32;

        for x in first_x..last_x {
            if depth >= wall_depths[x as usize] {
                continue;
            }

            let texture_u = (x as f32 + 0.5 - sprite_left) / sprite_width;

            for y in first_y..last_y {
                let texture_v = (y as f32 + 0.5 - sprite_top) / sprite_height;
                let color = texture.sample(texture_u, texture_v);

                if color.a != 0 {
                    framebuffer.blend_pixel_color(x, y, color);
                }
            }
        }
    }
}
