use ft;
use gdk::{RGBA};
use std::default::Default;

pub struct RenderSettings {
    pub library: ft::Library,
    pub face: ft::Face<'static>,
    pub border_color: RGBA,
    pub border_width: isize,
    pub font_color: RGBA,
    pub font_size: isize,
    pub letter_spacing: isize,
    pub text: String,
}

impl RenderSettings {
    pub fn new(library: ft::Library, face: ft::Face<'static>) -> RenderSettings {
        let default_font_size = 128;

        face.set_char_size(0, default_font_size*64, 0, 64).unwrap();

        RenderSettings {
            library: library,
            face: face,
            border_color: RGBA::black(),
            border_width: 4,
            font_color: RGBA::white(),
            font_size: default_font_size,
            letter_spacing: 2,
            text: String::from("AaBbCcDd"),
        }
    }

    pub fn reset(&mut self) {
        self.border_color = RGBA::black();
        self.border_width = 4;
        self.font_color = RGBA::white();
        self.font_size = 128;
        self.face.set_char_size(0, self.font_size*64, 0, 64).unwrap();
        self.letter_spacing = 2;
        self.text = String::from("AaBbCcDd");
    }
}