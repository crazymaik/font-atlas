use cairo;
use ft;
use ft::freetype_sys as fts;
use gdk::{RGBA};
use std::ptr;
use std::mem;

pub struct RenderedGlyph {
    pub surface: cairo::ImageSurface,
    pub origin: (i32, i32),
}

impl RenderedGlyph {
    pub fn new(_library: &ft::Library, face: &ft::Face, character: usize, color: &RGBA) -> ft::FtResult<RenderedGlyph> {
        face.load_char(character, ft::face::LoadFlag::DEFAULT)?;

        let glyph = face.glyph().get_glyph()?;
        let cbox = glyph.get_cbox(fts::FT_GLYPH_BBOX_PIXELS);
        let bitmap_glyph = glyph.to_bitmap(ft::RenderMode::Normal, None)?;

        let surface = RenderedGlyph::bitmap_glyph_to_surface(bitmap_glyph, color)?;

        Ok(RenderedGlyph {
            surface: surface,
            origin: (cbox.xMin as i32, cbox.yMin as i32),
        })
    }
    
    pub fn new_outline(library: &ft::Library, face: &ft::Face, character: usize, color: &RGBA, border_width: isize) -> ft::FtResult<RenderedGlyph> {

        let mut stroker: ft::freetype_sys::FT_Stroker = ptr::null_mut();
        unsafe {
            fts::FT_Stroker_New(library.raw(), &mut stroker);
            fts::FT_Stroker_Set(stroker, (border_width * 64) as fts::FT_Fixed, fts::FT_STROKER_LINECAP_ROUND, fts::FT_STROKER_LINEJOIN_ROUND, 0);    
        }

        face.load_char(character, ft::face::LoadFlag::DEFAULT)?;

        let mut glyph = face.glyph().get_glyph()?;

        unsafe {
            let p: *const fts::FT_GlyphRec = glyph.raw() as *const fts::FT_GlyphRec;
            let mut gp: fts::FT_Glyph = mem::transmute_copy(&p);
            fts::FT_Glyph_StrokeBorder(&mut gp, stroker, false as fts::FT_Bool, false as fts::FT_Bool);
            glyph = ft::Glyph::from_raw(library.raw(), gp);
        }

        let cbox = glyph.get_cbox(fts::FT_GLYPH_BBOX_PIXELS);
        let bitmap_glyph = glyph.to_bitmap(ft::RenderMode::Normal, None).unwrap();

        let surface = RenderedGlyph::bitmap_glyph_to_surface(bitmap_glyph, color)?;

        Ok(RenderedGlyph {
            surface: surface,
            origin: (cbox.xMin as i32, cbox.yMin as i32),
        })
    }

    pub fn bitmap_glyph_to_surface(glyph: ft::BitmapGlyph, color: &RGBA) -> ft::FtResult<cairo::ImageSurface> {
        let bitmap = glyph.bitmap();
        let width = bitmap.width() as usize;
        let height = bitmap.rows() as usize;
        let stride = cairo::Format::ARgb32.stride_for_width(bitmap.width() as u32).unwrap_or(width as i32 * 4) as usize;
        let out_size = stride * bitmap.rows() as usize;
        let mut out: Vec<u8> = Vec::with_capacity(out_size);
        let red = (color.red * 255.0) as u8;
        let green = (color.green * 255.0) as u8;
        let blue = (color.blue * 255.0) as u8;

        for y in 0..height {
            for x in 0..width {
                let value = bitmap.buffer()[y * width + x];

                if value > 0 {
                    out.push(blue);
                    out.push(green);
                    out.push(red);
                } else {
                    out.push(0);
                    out.push(0);
                    out.push(0);
                }
                out.push(value);
            }

            for _ in (width*4)..stride {
                out.push(0);
            }
        }

        let data = out.into_boxed_slice();
        if let Ok(surface) = cairo::ImageSurface::create_for_data(data, |_| (), cairo::Format::ARgb32, width as i32, height as i32, stride as i32) {
            return Ok(surface);
        } else {
            return Err(ft::Error::Unknown);
        }
    }
}
