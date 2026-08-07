use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
    pub dist: f32,
    pub object: char,
}

// castea un rayo en la dirección de la cámara
pub fn cast_ray(angle: f32, maze: &Maze, player: &Player, block_size: usize) -> Intersect {
    let mut d = 0.0;

    // loop es como un while
    loop {
        let c = d * angle.cos();
        let s = d * angle.sin();

        let x = (player.pos.x + c) as usize;
        let y = (player.pos.y + s) as usize;

        // para encontrar el índice del array (maze) con lo que chocó
        // como cada "bloque" es de block_size*block_size, se obtiene así:
        let i = x / block_size;
        let j = y / block_size;

        if maze.grid[j][i] != ' ' {
            return Intersect {
                dist: d,
                object: maze.grid[j][i],
            };
        }

        d += 0.1;
    }
}
