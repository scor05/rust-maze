use crate::helpers::HudElements;
use crate::interface::Interface;
use raylib::{ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_BONEINDICES, prelude::*};

fn draw_outlined_text(
    renderer: &mut impl RaylibDraw,
    text: &str,
    x: i32,
    y: i32,
    font_size: i32,
    outline_size: i32,
) {
    for offset_y in -outline_size..=outline_size {
        for offset_x in -outline_size..=outline_size {
            if offset_x != 0 || offset_y != 0 {
                renderer.draw_text(text, x + offset_x, y + offset_y, font_size, Color::BLACK);
            }
        }
    }

    renderer.draw_text(text, x, y, font_size, Color::WHITE);
}

// current_color sirve para pintar varios pixeles a la vez del mismo color
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    color_buffer: Vec<u8>,
    screen_texture: Texture2D,
    pub background_color: Color,
    pub current_color: Color,
}

/*
 * Para dibujar:
 *  - Escuchar input (usuario)
 *  - Limpiar frame anterior
 *  - Preparar variables / actualizarlas
 *  - Pintar al buffer
 *  - Pintar el buffer a la ventana (swap/flip buffers)
 *  - Esperar (vsync)
 * */
impl Framebuffer {
    pub fn new(
        width: u32,
        height: u32,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
    ) -> Self {
        let initial_image = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        let screen_texture = window
            .load_texture_from_image(raylib_thread, &initial_image)
            .expect("failed to create framebuffer texture");

        let mut color_buffer = vec![0; width as usize * height as usize * 4];
        for pixel in color_buffer.chunks_exact_mut(4) {
            pixel[3] = 255; // alpha
        }

        Framebuffer {
            width,
            height,
            color_buffer,
            screen_texture,
            background_color: Color::BLACK,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.color_buffer.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[
                self.background_color.r,
                self.background_color.g,
                self.background_color.b,
                self.background_color.a,
            ]);
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        self.set_pixel_color(x, y, self.current_color);
    }

    pub fn set_pixel_color(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = ((y * self.width + x) * 4) as usize;
            self.color_buffer[index] = color.r;
            self.color_buffer[index + 1] = color.g;
            self.color_buffer[index + 2] = color.b;
            self.color_buffer[index + 3] = color.a;
        }
    }

    pub fn blend_pixel_color(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height || color.a == 0 {
            return;
        }

        if color.a == 255 {
            self.set_pixel_color(x, y, color);
            return;
        }

        let index = ((y * self.width + x) * 4) as usize;
        let alpha = color.a as u16;
        let inverse_alpha = 255 - alpha;

        self.color_buffer[index] = ((color.r as u16 * alpha
            + self.color_buffer[index] as u16 * inverse_alpha)
            / 255) as u8;
        self.color_buffer[index + 1] = ((color.g as u16 * alpha
            + self.color_buffer[index + 1] as u16 * inverse_alpha)
            / 255) as u8;
        self.color_buffer[index + 2] = ((color.b as u16 * alpha
            + self.color_buffer[index + 2] as u16 * inverse_alpha)
            / 255) as u8;
        self.color_buffer[index + 3] = 255;
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn render_to_file(&self, file_path: &str) {
        let mut image =
            Image::gen_image_color(self.width as i32, self.height as i32, self.background_color);

        for (index, pixel) in self.color_buffer.chunks_exact(4).enumerate() {
            image.draw_pixel(
                (index % self.width as usize) as i32,
                (index / self.width as usize) as i32,
                Color::new(pixel[0], pixel[1], pixel[2], pixel[3]),
            );
        }

        image.export_image(file_path);
    }

    pub fn swap_buffers(
        &mut self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        viewmodel: Option<&Texture2D>,
        viewmodel_dest: Option<Rectangle>,
        hud_elements: Option<&HudElements>,
        interface: Option<&Interface>, // None = in game, no renderiza interfaz.
    ) {
        let win_height = window.get_render_height();
        let win_width = window.get_render_width();

        if let Some(int) = interface {
            let mut renderer = window.begin_drawing(raylib_thread);
            let src = Rectangle::new(0.0, 0.0, int.image.width as f32, int.image.height as f32);
            let dest = Rectangle::new(0.0, 0.0, win_width as f32, win_height as f32);

            renderer.draw_texture_pro(
                &int.image,
                src,
                dest,
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );

            let scale_x = win_width as f32 / int.image.width as f32;
            let scale_y = win_height as f32 / int.image.height as f32;
            let text_scale = scale_x.min(scale_y);

            for t in int.text.iter() {
                let scaled_x = (t.x as f32 * scale_x).round() as i32;
                let scaled_y = (t.y as f32 * scale_y).round() as i32;
                let scaled_font_size = ((t.font_size as f32 * text_scale).round() as i32).max(1);
                let scaled_outline_size = (t.outline_size as f32 * text_scale).round() as i32;

                draw_outlined_text(
                    &mut renderer,
                    t.text,
                    scaled_x,
                    scaled_y,
                    scaled_font_size,
                    scaled_outline_size,
                );
            }
            return;
        }

        let viewmodel = viewmodel.expect("in-game rendering requires a viewmodel texture");
        let viewmodel_dest =
            viewmodel_dest.expect("in-game rendering requires a viewmodel destination");
        let hud_elements = hud_elements.expect("in-game rendering requires HUD elements");

        let fps_text = format!("FPS: {}", window.get_fps());
        let ammo_text = format!("AMMO: {}/{}", hud_elements.ammo, hud_elements.mag);
        let ammo_font_size = 48;
        let ammo_margin = 18;
        let ammo_width = window.measure_text(&ammo_text, ammo_font_size);
        let ammo_x = win_width - ammo_width - ammo_margin;
        let ammo_y = win_height - ammo_font_size - ammo_margin;

        let controls_text = "Movement: WASD\nShoot: LMB\nSprint: LSHIFT\nReload: R\nDefuse: E";
        let controls_text_size = 20;
        let controls_text_width = window.measure_text(controls_text, controls_text_size);
        let controls_x = win_width - controls_text_width - 10;
        let controls_y = controls_text_size - 10;

        self.screen_texture
            .update_texture(&self.color_buffer)
            .expect("failed to update framebuffer texture");

        let mut renderer = window.begin_drawing(raylib_thread);

        let defuse_counter = hud_elements.defuse_counter;
        let current_defused = hud_elements.current_defused;
        let defusing = hud_elements.defusing;
        let defused_current_site = hud_elements.defused_current_site;

        let defuse_text = format!("Defusing: {defuse_counter}");
        let defuse_text_size = 35;

        let bomb_text = format!("Bombs Defused: {current_defused}/2");
        let bomb_text_size = 35;

        let source = Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32);
        let dest = Rectangle::new(0.0, 0.0, win_width as f32, win_height as f32);

        renderer.draw_texture_pro(
            &self.screen_texture,
            source,
            dest,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        let viewmodel_source = Rectangle::new(
            0.0,
            0.0,
            viewmodel.width() as f32,
            viewmodel.height() as f32,
        );

        renderer.draw_texture_pro(
            viewmodel,
            viewmodel_source,
            viewmodel_dest,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        if defusing && !defused_current_site {
            draw_outlined_text(
                &mut renderer,
                &defuse_text,
                win_width / 2 - 40,
                defuse_text_size + 10,
                defuse_text_size,
                2,
            );
        } else if defusing && defused_current_site {
            draw_outlined_text(
                &mut renderer,
                "Site already defused.",
                win_width / 2 - 40,
                40,
                35,
                2,
            );
        }

        draw_outlined_text(
            &mut renderer,
            &bomb_text,
            10,
            win_height - bomb_text_size - 20,
            bomb_text_size,
            2,
        );
        draw_outlined_text(&mut renderer, &fps_text, 12, 10, 18, 2);
        draw_outlined_text(&mut renderer, &ammo_text, ammo_x, ammo_y, ammo_font_size, 2);
        draw_outlined_text(
            &mut renderer,
            &controls_text,
            controls_x,
            controls_y,
            controls_text_size,
            2,
        );
    }
}
