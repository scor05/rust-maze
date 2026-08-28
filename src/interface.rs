use raylib::prelude::Texture2D;

pub struct Button {
    pub start_x: u32,
    pub end_x: u32,
    pub start_y: u32,
    pub end_y: u32,
    pub active: bool,
    pub label: &'static str,
}

#[derive(Clone, Copy)]
pub struct InterfaceText {
    pub text: &'static str,
    pub x: i32,
    pub y: i32,
    pub font_size: i32,
    pub outline_size: i32,
}

pub struct Interface {
    pub buttons: Vec<Button>,
    pub text: Vec<InterfaceText>,
    pub image: Texture2D,
}

impl Button {
    pub fn new(sx: u32, ex: u32, sy: u32, ey: u32, label: &'static str) -> Button {
        Button {
            start_x: sx,
            end_x: ex,
            start_y: sy,
            end_y: ey,
            active: false,
            label,
        }
    }

    pub fn check_click(&mut self, x: u32, y: u32) -> bool {
        if x >= self.start_x && x <= self.end_x && y >= self.start_y && y <= self.end_y {
            self.active = true;
            true
        } else {
            self.active = false;
            false
        }
    }
}

impl Interface {
    pub fn new(buttons: Vec<Button>, text: Vec<InterfaceText>, img: Texture2D) -> Interface {
        Interface {
            buttons,
            text,
            image: img,
        }
    }
}
