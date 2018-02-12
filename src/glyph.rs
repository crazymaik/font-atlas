use cairo::{Format, ImageSurface};
use ft;
use ft::freetype_sys as fts;
use gdk::{RGBA};
use std::cmp;
use std::mem;
use std::os::raw::c_long;
use std::ptr;
use std::rc::{Rc};

pub struct RenderedGlyph {
    /// The rendered glyph
    pub surface: ImageSurface,
    /// Unicode codepoint
    pub codepoint: usize,
    // Distance to move the text cursor forward to render the next glyph, in pixels
    pub advance: (i32, i32),
    // Offset of top-left corner from baseline, in pixels
    pub offset: (i32, i32),
}

pub struct Renderer {
    library: Rc<ft::Library>,
    face: Rc<ft::Face<'static>>,
    color: RGBA,
    outline_stroker: Option<Stroker>,
    outline_color: RGBA,
    outline_width: isize,
}

impl Renderer {

    pub fn new(library: &Rc<ft::Library>, face: &Rc<ft::Face<'static>>) -> Renderer {
        Renderer {
            library: library.clone(),
            face: face.clone(),
            color: RGBA::black(),
            outline_stroker: None,
            outline_color: RGBA::black(),
            outline_width: 0,
        }
    }

    pub fn set_color(&mut self, color: &RGBA) {
        self.color = *color;
    }

    pub fn set_outline(&mut self, color: &RGBA, width: isize) {
        self.outline_color = *color;
        self.outline_width = width;
        self.outline_stroker = if width > 0 {
            let stroker = Stroker::new(self.library.raw()).expect("Stroker");
            stroker.set((width * 64) as fts::FT_Fixed, fts::FT_STROKER_LINECAP_ROUND, fts::FT_STROKER_LINEJOIN_ROUND, 0);
            Some(stroker)
        } else {
            None
        };
    }

    pub fn render(&self, codepoint: usize) -> ft::FtResult<RenderedGlyph> {
        self.face.load_char(codepoint, ft::face::LoadFlag::DEFAULT)?;

        let glyph = self.face.glyph().get_glyph()?;

        let mut bitmaps = Vec::new();
        let mut glyph_bbox = None;
        let mut glyph_advance_x = None;
        let mut glyph_advance_y = None;

        if let Some(stroker) = self.outline_stroker.as_ref() {
            let glyph = unsafe {
                let p: *const fts::FT_GlyphRec = glyph.raw() as *const fts::FT_GlyphRec;
                let mut gp: fts::FT_Glyph = mem::transmute_copy(&p);
                fts::FT_Glyph_StrokeBorder(&mut gp, stroker.raw_mut(), false as fts::FT_Bool, false as fts::FT_Bool);
                ft::Glyph::from_raw(self.library.raw(), gp)
            };

            let bbox = glyph.get_cbox(fts::FT_GLYPH_BBOX_PIXELS);
            let bitmap_glyph = glyph.to_bitmap(ft::RenderMode::Normal, None)?;

            glyph_bbox = Some(bbox);
            glyph_advance_x = Some(glyph.advance_x());
            glyph_advance_y = Some(glyph.advance_y());

            bitmaps.push((bitmap_glyph, bbox, &self.outline_color));
        }

        {
            let bbox = glyph.get_cbox(fts::FT_GLYPH_BBOX_PIXELS);
            let bitmap_glyph = glyph.to_bitmap(ft::RenderMode::Normal, None)?;

            glyph_bbox = glyph_bbox.or(Some(bbox));
            glyph_advance_x = glyph_advance_x.or(Some(glyph.advance_x()));
            glyph_advance_y = glyph_advance_y.or(Some(glyph.advance_y()));

            bitmaps.push((bitmap_glyph, bbox, &self.color));
        }

        let surface = Renderer::bitmaps_to_surface(bitmaps)?;

        Ok(RenderedGlyph {
            surface: surface,
            codepoint: codepoint,
            advance: ((glyph_advance_x.unwrap() >> 16) as i32, (glyph_advance_y.unwrap() >> 16) as i32),
            offset: (glyph_bbox.unwrap().xMin as i32, glyph_bbox.unwrap().yMax as i32),
        })
    }

    fn bitmaps_to_surface(bitmaps: Vec<(ft::BitmapGlyph, ft::BBox, &RGBA)>) -> ft::FtResult<ImageSurface> {
        let min_box = ft::BBox {
            xMin: c_long::max_value(),
            yMin: c_long::max_value(),
            xMax: c_long::min_value(),
            yMax: c_long::min_value(),
        };
        let bounding_box = bitmaps.iter().fold(min_box, |acc, &(_,bbox,_)| { Renderer::union(&acc, &bbox) });

        let width = bounding_box.xMax - bounding_box.xMin;
        let height = bounding_box.yMax - bounding_box.yMin;
        let mut out: Vec<f32> = Vec::with_capacity((width * height * 4) as usize);
        out.resize((width * height * 4) as usize, 0.0);

        for (bitmap, bbox, color) in bitmaps {
            let b = bitmap.bitmap();
            let buffer = b.buffer();
            let red = color.red as f32;
            let green = color.green as f32;
            let blue = color.blue as f32;

            let bitmap_width = bbox.xMax - bbox.xMin;
            let bitmap_height = bbox.yMax - bbox.yMin;

            let offset_x = bbox.xMin - bounding_box.xMin;
            let offset_y = -(bbox.yMax - bounding_box.yMax);

            for y in 0..bitmap_height {
                for x in 0..bitmap_width {
                    let value = buffer[(y * bitmap_width + x) as usize];

                    let offset = ((y + offset_y) * (width*4) + (x + offset_x) * 4) as usize;
                    if value > 0 {
                        let old_value = out[offset + 3];
                        let source_alpha = value as f32 / 255.0;
                        if old_value > 0.0 {
                            let dest_blue = out[offset + 0];
                            let dest_green = out[offset + 1];
                            let dest_red = out[offset + 2];
                            let dest_alpha = out[offset + 3]; 
                            let new_alpha = source_alpha + dest_alpha * (1.0 - source_alpha);
                            out[offset + 0] = (blue + dest_blue * (1.0 - source_alpha)) / new_alpha;
                            out[offset + 1] = (green + dest_green * (1.0 - source_alpha)) / new_alpha;
                            out[offset + 2] = (red + dest_red * (1.0 - source_alpha)) / new_alpha;
                            out[offset + 3] = new_alpha;
                        } else {
                            out[offset + 0] = blue;
                            out[offset + 1] = green;
                            out[offset + 2] = red;
                            out[offset + 3] = source_alpha;
                        }
                    }
                }
            }
        }

        Renderer::f32_image_to_image_surface(&out, width as u32, height as u32)
    }

    fn f32_image_to_image_surface(buffer: &Vec<f32>, width: u32, height: u32) -> ft::FtResult<ImageSurface> {
        let stride = Format::ARgb32.stride_for_width(width).unwrap_or(width as i32 * 4) as u32;
        let out_size = (stride * height) as usize;
        let mut out: Vec<u8> = Vec::with_capacity(out_size);
        out.resize(out_size, 0u8);

        for y in 0..height {
            let dest_offset_y = (y * stride) as usize;
            let src_offset_y = y * width * 4;
            for x in 0..width {
                let dest_offset = dest_offset_y + x as usize * 4;
                let src_offset = (src_offset_y + x * 4) as usize;
                out[dest_offset + 0] = (buffer[src_offset + 0] * 255.0) as u8;
                out[dest_offset + 1] = (buffer[src_offset + 1] * 255.0) as u8;
                out[dest_offset + 2] = (buffer[src_offset + 2] * 255.0) as u8;
                out[dest_offset + 3] = (buffer[src_offset + 3] * 255.0) as u8;
            }
        }

        let data = out.into_boxed_slice();

        if let Ok(surface) = ImageSurface::create_for_data(data, |_| (), Format::ARgb32, width as i32, height as i32, stride as i32) {
            return Ok(surface);
        } else {
            return Err(ft::Error::Unknown);
        }
    }

    fn union(a: &ft::BBox, b: &ft::BBox) -> ft::BBox {
        ft::BBox {
            xMin: cmp::min(a.xMin, b.xMin),
            yMin: cmp::min(a.yMin, b.yMin),
            xMax: cmp::max(a.xMax, b.xMax),
            yMax: cmp::max(a.yMax, b.yMax),
        }
    }
}

struct Stroker {
    library_raw: fts::FT_Library,
    raw: fts::FT_Stroker,
}

impl Stroker {
    fn new(library_raw: fts::FT_Library) -> ft::FtResult<Self> {
        let mut stroker: fts::FT_Stroker = ptr::null_mut();

        let err = unsafe {
            fts::FT_Stroker_New(library_raw, &mut stroker)
        };

        if err == fts::FT_Err_Ok {
            Ok(unsafe { Self::from_raw(library_raw, stroker) })
        } else {
            Err(err.into())
        }
    }

    unsafe fn from_raw(library_raw: fts::FT_Library, raw: fts::FT_Stroker) -> Self {
        fts::FT_Reference_Library(library_raw);
        Stroker {
            library_raw: library_raw,
            raw: raw,
        }
    }

    fn set(&self, radius: fts::FT_Fixed, line_cap: fts::FT_Stroker_LineCap, line_join: fts::FT_Stroker_LineCap, miter_limit: fts::FT_Fixed) {
        unsafe {
            fts::FT_Stroker_Set(self.raw, radius, line_cap, line_join, miter_limit);
        }
    }

    fn raw(&self) -> &fts::FT_StrokerRec {
        unsafe {
            &*self.raw
        }
    }

    fn raw_mut(&self) -> &mut fts::FT_StrokerRec {
        unsafe {
            &mut *self.raw
        }
    }
}

impl Drop for Stroker {

    fn drop(&mut self) {
        let err = unsafe {
            fts::FT_Stroker_Done(self.raw);
            fts::FT_Done_Library(self.library_raw)
        };
        if err != fts::FT_Err_Ok {
            panic!("Failed to drop library");
        }
    }
}
