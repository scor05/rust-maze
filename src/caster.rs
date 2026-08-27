use crate::bullet::Bullet;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
    pub dist: f32,
    pub object: char,
}

// castea un rayo en la dirección de la cámara
// usa DDA para castear en vez de ir en steps, se estaba lageueando
// mucho con d += 0.5
pub fn cast_ray(angle: f32, maze: &Maze, player: &Player) -> Intersect {
    let direction_x = angle.cos();
    let direction_y = angle.sin();
    let block_size = maze.block_size as f32;

    if player.pos.x < 0.0 || player.pos.y < 0.0 {
        return Intersect {
            dist: 0.0,
            object: '#',
        };
    }

    let mut grid_x = (player.pos.x / block_size).floor() as isize;
    let mut grid_y = (player.pos.y / block_size).floor() as isize;

    match maze
        .grid
        .get(grid_y as usize)
        .and_then(|row| row.get(grid_x as usize))
        .copied()
    {
        Some(object) if Maze::is_solid_symbol(object) => {
            return Intersect { dist: 0.0, object };
        }
        Some(_) => {}
        None => {
            return Intersect {
                dist: 0.0,
                object: '#',
            };
        }
    }

    // distancia con respecto al rayo de múltiplos de block_size + la distancia inicial

    // si la dirección > EPSILON (minimo número mayor que 1, 1.19e-7) se va a +x, si < epsilon va a -x
    // y si es 0 el vector no se usa (porque no tiene el componente r(t)).

    // step_x itera sobre maze.grid, delta es el cambio en múltiplos de block_size
    // side_x son los componentes iniciales del vector dirección del rayo (normalizados ya)
    let (step_x, delta_x, mut side_x) = if direction_x > f32::EPSILON {
        let next_boundary_x = (grid_x + 1) as f32 * block_size;
        (
            1,                                              // step
            block_size / direction_x,                       // delta
            (next_boundary_x - player.pos.x) / direction_x, // side
        )
    } else if direction_x < -f32::EPSILON {
        let next_boundary_x = grid_x as f32 * block_size;
        (
            -1,
            block_size / -direction_x,
            (player.pos.x - next_boundary_x) / -direction_x,
        )
    } else {
        (0, f32::INFINITY, f32::INFINITY)
    };

    let (step_y, delta_y, mut side_y) = if direction_y > f32::EPSILON {
        let next_boundary_y = (grid_y + 1) as f32 * block_size;
        (
            1,
            block_size / direction_y,
            (next_boundary_y - player.pos.y) / direction_y,
        )
    } else if direction_y < -f32::EPSILON {
        let next_boundary_y = grid_y as f32 * block_size;
        (
            -1,
            block_size / -direction_y,
            (player.pos.y - next_boundary_y) / -direction_y,
        )
    } else {
        (0, f32::INFINITY, f32::INFINITY)
    };

    loop {
        let distance = if side_x < side_y {
            let distance = side_x;
            side_x += delta_x;
            grid_x += step_x;
            distance
        } else {
            let distance = side_y;
            side_y += delta_y;
            grid_y += step_y;
            distance
        };

        if grid_x < 0 || grid_y < 0 {
            return Intersect {
                dist: distance,
                object: '#',
            };
        }

        match maze
            .grid
            .get(grid_y as usize)
            .and_then(|row| row.get(grid_x as usize))
            .copied()
        {
            Some(object) if Maze::is_solid_symbol(object) => {
                return Intersect {
                    dist: distance,
                    object,
                };
            }
            Some(_) => {}
            None => {
                return Intersect {
                    dist: distance,
                    object: '#',
                };
            }
        }
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
