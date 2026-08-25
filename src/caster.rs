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

    // loop es como un while
    loop {
        let c = d * angle.cos();
        let s = d * angle.sin();

        let x = (player.pos.x + c) as usize;
        let y = (player.pos.y + s) as usize;

        // para encontrar el índice del array (maze) con lo que chocó
        // como cada "bloque" es de block_size*block_size, se obtiene así:
        let i = x / maze.block_size;
        let j = y / maze.block_size;

        if maze.grid[j][i] != ' ' {
            return Intersect {
                dist: d,
                object: maze.grid[j][i],
            };
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

        let column = world_x as usize / maze.block_size;
        let row = world_y as usize / maze.block_size;

        match maze.grid.get(row).and_then(|maze_row| maze_row.get(column)) {
            Some(' ') => {}
            Some(&object) => {
                return Some(Intersect { dist: d, object });
            }
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
