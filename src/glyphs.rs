use cairo;
use std::cmp;
use std::fs::{File};
use std::io;
use std::path::{Path};

use glyph::{Renderer};
use render_settings::{RenderSettings};

pub struct Glyphs {
}

impl Glyphs {
    pub fn new() -> Glyphs {
        Glyphs { }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, png_path: P, render_settings: &RenderSettings, width: i32, height: i32) -> io::Result<()> {
        let surface = self.render_to_surface(render_settings, width, height);
        let mut file = File::create(png_path)?;
        match surface.write_to_png(&mut file) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(io::Error::from(io::ErrorKind::Other)),
        }
    }

    pub fn render_to_surface(&self, render_settings: &RenderSettings, width: i32, height: i32) -> cairo::ImageSurface {
        let letter_spacing = render_settings.letter_spacing as i32;
        let mut left: i32 = letter_spacing;
        let mut top: i32 = letter_spacing;
        let mut max_row_height: i32 = 0;

        let mut renderer = Renderer::new(&render_settings.library, &render_settings.face);
        renderer.set_color(&render_settings.font_color);
        renderer.set_outline(&render_settings.border_color, render_settings.border_width);

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let context = cairo::Context::new(&surface);

        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.set_operator(cairo::Operator::Source);
        context.rectangle(0.0, 0.0, width as f64, height as f64);
        context.fill();

        for character in render_settings.text.chars() {
            if character == '\n' {
                top += max_row_height + letter_spacing;
                left = letter_spacing;
                max_row_height = 0;
                continue
            }

            let rendered_glyph = renderer.render(character as usize).unwrap();
            let glyph_right = rendered_glyph.surface.get_width();
            let glyph_bottom = rendered_glyph.surface.get_height();

            context.set_operator(cairo::Operator::Over);
            context.set_source_surface(&rendered_glyph.surface, left as f64, top as f64);
            context.paint();

            left += glyph_right + letter_spacing;
            max_row_height = cmp::max(max_row_height, glyph_bottom);
        }

        surface
    }
}