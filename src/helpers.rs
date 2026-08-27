use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use raylib::prelude::*;

const FLOOR_COLOR: Color = Color::GRAY;
const OOB_COLOR: Color = Color::BLACK;
const BOX_COLOR: Color = Color::new(77, 33, 12, 0xff);
const WALL_COLOR: Color = Color::new(251, 255, 219, 0xff);
const BOMB_COLOR: Color = Color::new(225, 255, 0, 255);
const VIEWMODEL_HEIGHT_RATIO: f32 = 0.62;
const VIEWMODEL_RIGHT_OFFSET: f32 = 110.0;

pub struct PixelColor {
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

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
