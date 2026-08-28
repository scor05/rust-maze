use crate::maze::Maze;
use raylib::prelude::Vector2;
use std::io;

#[derive(Clone)]
pub struct Map {
    pub path: String,
    pub spawn_location: Vector2,
    pub maze: Maze,
    pub name: &'static str,
}

impl Map {
    pub fn new(path: String, block_size: usize, name: &'static str) -> io::Result<Self> {
        let maze = Maze::new(&path, block_size)?;

        let (spawn_row, spawn_column) = maze
            .grid
            .iter()
            .enumerate()
            .find_map(|(row, cells)| {
                cells
                    .iter()
                    .position(|&symbol| symbol == 'P')
                    .map(|column| (row, column))
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("map '{path}' does not contain a player spawn ('P')"),
                )
            })?;

        let spawn_location = Vector2::new(
            (spawn_column as f32 + 0.5) * block_size as f32,
            (spawn_row as f32 + 0.5) * block_size as f32,
        );

        Ok(Self {
            path,
            spawn_location,
            maze,
            name,
        })
    }
}
