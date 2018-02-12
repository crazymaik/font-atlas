use cairo;
use std::cmp;
use std::fs::{File};
use std::io;
use std::path::{Path};

use font;
use glyph::{Renderer};
use render_settings::{RenderSettings};

pub struct GlyphInfo {
    codepoint: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    xoffset: i32,
    yoffset: i32,
    xadvance: i32,
}

pub struct Glyphs {
}

impl Glyphs {
    pub fn new() -> Glyphs {
        Glyphs { }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P, render_settings: &RenderSettings, width: i32, height: i32) -> io::Result<()> {
        let png_filepath = path.as_ref().with_extension("png");
        let fnt_filepath = path.as_ref().with_extension("fnt");
        let (info, surface) = self.render_to_surface(render_settings, width, height);
        let mut png_file = File::create(&png_filepath)?;
        match surface.write_to_png(&mut png_file) {
            Ok(_) => (),
            Err(_) => return Err(io::Error::from(io::ErrorKind::Other)),
        }
        let mut fnt_file = File::create(&fnt_filepath)?;

        let padding = render_settings.letter_padding as u32;
        let metrics = render_settings.face.size_metrics().expect("Need metrics");
        let line_height = metrics.height as u32 / 64;
        let base = metrics.ascender as i32 / 64;

        let font_file = font::FontFile {
            info: font::InfoTag {
                face: "".to_string(),
                size: render_settings.font_size as u32,
                bold: false,
                italic: false,
                charset: "".to_string(),
                unicode: false,
                stretch_h: 100,
                smooth: false,
                aa: false,
                padding: [padding, padding, padding, padding],
                spacing: [0, 0],
                outline: 0,
            },
            common: font::CommonTag {
                line_height: line_height,
                base: base,
                scale_w: width as u32,
                scale_h: height as u32,
                pages: 1,
                packed: false,
                alpha_channel: 0,
                red_channel: 0,
                green_channel: 0,
                blue_channel: 0,
            },
            page: font::PageTag {
                id: 0,
                file: png_filepath.file_name().expect("").to_string_lossy().to_string(),
            },
            chars: info.iter().map(|g| {
                font::CharTag {
                    id: g.codepoint,
                    x: g.x,
                    y: g.y,
                    width: g.width,
                    height: g.height,
                    xoffset: g.xoffset,
                    yoffset: (line_height as i32) - g.yoffset,
                    xadvance: g.xadvance,
                    page: 0,
                    chnl: 15,
                }
            }).collect(),
        };

        font_file.write(&mut fnt_file)?;

        Ok(())
    }

    pub fn render_to_surface(&self, render_settings: &RenderSettings, width: i32, height: i32) -> (Vec<GlyphInfo>, cairo::ImageSurface) {
        let letter_padding = render_settings.letter_padding as i32;
        let mut left: i32 = letter_padding;
        let mut top: i32 = letter_padding;
        let mut max_row_height: i32 = 0;

        let mut renderer = Renderer::new(&render_settings.library, &render_settings.face);
        renderer.set_color(&render_settings.font_color);
        renderer.set_outline(&render_settings.border_color, render_settings.border_width);

        let mut info = Vec::new();

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let context = cairo::Context::new(&surface);

        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.set_operator(cairo::Operator::Source);
        context.rectangle(0.0, 0.0, width as f64, height as f64);
        context.fill();

        for character in render_settings.text.chars() {
            if character == '\n' {
                top += max_row_height + letter_padding;
                left = letter_padding;
                max_row_height = 0;
                continue
            }

            let rendered_glyph = renderer.render(character as usize).unwrap();
            let glyph_right = rendered_glyph.surface.get_width();
            let glyph_bottom = rendered_glyph.surface.get_height();

            info.push(GlyphInfo {
                codepoint: rendered_glyph.codepoint,
                x: left as u32,
                y: top as u32,
                width: rendered_glyph.surface.get_width() as u32,
                height: rendered_glyph.surface.get_height() as u32,
                xoffset: rendered_glyph.offset.0,
                yoffset: rendered_glyph.offset.1,
                xadvance: rendered_glyph.advance.0,
            });

            context.set_operator(cairo::Operator::Over);
            context.set_source_surface(&rendered_glyph.surface, left as f64, top as f64);
            context.paint();

            left += glyph_right + letter_padding;
            max_row_height = cmp::max(max_row_height, glyph_bottom);
        }

        (info, surface)
    }
}