use std::fs;
use std::io;

pub struct Maze {
    pub block_size: usize,
    pub grid: Vec<Vec<char>>,
}

impl Maze {
    pub fn new(filename: &str, size: usize) -> io::Result<Self> {
        let grid = Self::read_maze(filename)?;

        Ok(Self {
            block_size: size,
            grid,
        })
    }

    // Result es un enum así
    // Result {
    //      Ok(T),
    //      Err(E),
    // }
    pub fn read_maze(file_path: &str) -> io::Result<Vec<Vec<char>>> {
        let contents = fs::read_to_string(file_path)?;

        // map funciona igual que en JS
        // se le pasa una función y al array al que se le llama se ejecuta
        // esa función por cada elemento. Se pone el parámetro de la función
        // con | | y luego se pueden poner {} para la función en sí o si es corta
        // dejarla solo así.
        let grid = contents
            .lines() // separa el texto en líneas
            .map(|line| line.chars().collect()) // chars separa por caracteres
            // collect junta caracteres en vector
            .collect(); // junta los vectores en un vector

        Ok(grid)
    }

    pub fn is_walkable_symbol(symbol: char) -> bool {
        matches!(symbol, ' ' | 'P' | 'T' | '1' | '2')
    }

    pub fn is_solid_symbol(symbol: char) -> bool {
        !Self::is_walkable_symbol(symbol)
    }

    pub fn cell_at_world(&self, world_x: f32, world_y: f32) -> Option<char> {
        if !world_x.is_finite() || !world_y.is_finite() || world_x < 0.0 || world_y < 0.0 {
            return None;
        }

        let column = world_x as usize / self.block_size;
        let row = world_y as usize / self.block_size;

        self.grid
            .get(row)
            .and_then(|maze_row| maze_row.get(column))
            .copied()
    }
}
