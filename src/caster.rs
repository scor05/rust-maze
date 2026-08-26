use crate::bullet::Bullet;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
    pub dist: f32,
    pub object: char,
}

// castea un rayo en la dirección de la cámara
pub fn cast_ray(angle: f32, maze: &Maze, player: &Player) -> Intersect {
    let mut d = 0.0;
    let direction_x = angle.cos();
    let direction_y = angle.sin();

    // loop es como un while
    loop {
        let c = d * direction_x;
        let s = d * direction_y;

        let world_x = player.pos.x + c;
        let world_y = player.pos.y + s;

        match maze.cell_at_world(world_x, world_y) {
            Some(object) if Maze::is_solid_symbol(object) => {
                return Intersect { dist: d, object };
            }
            Some(_) => {}
            None => {
                return Intersect {
                    dist: d,
                    object: '#',
                };
            }
        }

        d += 0.5;
    }
}

pub fn cast_bullet_ray(
    angle: f32,
    maze: &Maze,
    bullet: &Bullet,
    max_distance: f32,
) -> Option<Intersect> {
    let mut d = 0.0;
    let max_distance = max_distance.max(0.0);
    let direction_x = angle.cos();
    let direction_y = angle.sin();

    loop {
        let world_x = bullet.pos.x + d * direction_x;
        let world_y = bullet.pos.y + d * direction_y;

        if world_x < 0.0 || world_y < 0.0 {
            return Some(Intersect {
                dist: d,
                object: '#',
            });
        }

        match maze.cell_at_world(world_x, world_y) {
            Some(object) if Maze::is_solid_symbol(object) => {
                return Some(Intersect { dist: d, object });
            }
            Some(_) => {}
            None => {
                return Some(Intersect {
                    dist: d,
                    object: '#',
                });
            }
        }

        if d >= max_distance {
            return None;
        }

        d = (d + 0.5).min(max_distance);
    }
}
