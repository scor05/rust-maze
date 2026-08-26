use raylib::prelude::Color;
use raylib::prelude::Image;

pub struct CpuTexture {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl CpuTexture {
    pub fn from_image(image: &Image) -> Self {
        Self {
            width: image.width() as usize,
            height: image.height() as usize,
            // convertir solo una vez en vez de llamar a raylib::get_color() cada frame
            pixels: image.get_image_data().iter().copied().collect(),
        }
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let x = (u.rem_euclid(1.0) * self.width as f32) as usize;
        let y = (v.rem_euclid(1.0) * self.height as f32) as usize;

        self.pixels[y.min(self.height - 1) * self.width + x.min(self.width - 1)]
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

pub struct WorldTextures {
    pub box_texture: CpuTexture,
    pub floor: CpuTexture,
    pub bricks: CpuTexture,
    pub concrete: CpuTexture,
    pub wall1: CpuTexture,
    pub wall2: CpuTexture,
    pub defuse_site: CpuTexture,
}

impl WorldTextures {
    pub fn wall_for(&self, symbol: char) -> &CpuTexture {
        match symbol {
            'B' => &self.box_texture,
            '┌' | '┐' | '┘' | '└' => &self.bricks,
            '┴' | '┬' | '├' | '┤' => &self.concrete,
            '╶' | '╴' | '╵' | '╷' => &self.wall2,
            _ => &self.wall1,
        }
    }

    pub fn floor_color(&self, symbol: char, u: f32, v: f32) -> Color {
        let floor_color = self.floor.sample(u, v);

        if symbol == '1' || symbol == '2' {
            let site_color = self.defuse_site.sample(u, v);
            Color::color_alpha_blend(&floor_color, &site_color, &Color::WHITE)
        } else {
            floor_color
        }
    }
}
