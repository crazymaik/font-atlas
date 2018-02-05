use cairo;
use std::cmp;
use std::fs::{File};
use std::io;
use std::path::{Path};

use render_settings::{RenderSettings};
use rendered_glyph::{RenderedGlyph};

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

            let codepoint = character as usize;
            let outline_glyph = RenderedGlyph::new_outline(&render_settings.library, &render_settings.face, codepoint, &render_settings.border_color, render_settings.border_width).unwrap();
            let inside_glyph = RenderedGlyph::new(&render_settings.library, &render_settings.face, codepoint, &render_settings.font_color).unwrap();

            let outline_top = outline_glyph.origin.1 + outline_glyph.surface.get_height();
            let inside_top = inside_glyph.origin.1 + inside_glyph.surface.get_height();
            let glyph_top = cmp::max(outline_top, inside_top);

            let outline_left = outline_glyph.origin.0;
            let inside_left = inside_glyph.origin.0;
            let glyph_left = cmp::min(outline_left, inside_left);

            let outline_right = cmp::max(0, outline_glyph.origin.0) + outline_glyph.surface.get_width();
            let inside_right = cmp::max(0, inside_glyph.origin.0) + inside_glyph.surface.get_width();
            let glyph_right = cmp::max(outline_right, inside_right);

            let outline_bottom = cmp::max(outline_glyph.surface.get_height(), inside_glyph.surface.get_height());

            context.set_operator(cairo::Operator::Over);
            context.set_source_surface(&outline_glyph.surface, (left - glyph_left + outline_left) as f64, (top + glyph_top - outline_top) as f64);
            context.paint();

            context.set_operator(cairo::Operator::Over);
            context.set_source_surface(&inside_glyph.surface, (left - glyph_left + inside_left) as f64, (top + glyph_top - inside_top) as f64);
            context.paint();

            left += glyph_right + letter_spacing;
            max_row_height = cmp::max(max_row_height, outline_bottom);
        }

        surface
    }
}