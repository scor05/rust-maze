use raylib::prelude::Color;
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
}
