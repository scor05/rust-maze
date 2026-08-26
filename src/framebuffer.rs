use raylib::prelude::*;

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
        viewmodel: &Texture2D,
        viewmodel_dest: Rectangle,
    ) {
        self.screen_texture
            .update_texture(&self.color_buffer)
            .expect("failed to update framebuffer texture");

        let win_height = window.get_render_height();
        let win_width = window.get_render_width();

        let mut renderer = window.begin_drawing(raylib_thread);

        // renderizar el framebuffer a un tamaño menor que la pantalla pa que sea más rápido
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
    }
}
