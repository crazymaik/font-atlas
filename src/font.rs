//! Module for writing a bitmap font.
//! For more info see the [file format specification][1] and [text rendering page][2].
//!
//! [1]: http://www.angelcode.com/products/bmfont/doc/file_format.html
//! [2]: http://www.angelcode.com/products/bmfont/doc/render_text.html
//!

use std::io;

pub struct InfoTag {
    pub face: String,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub charset: String,
    pub unicode: bool,
    pub stretch_h: u32,
    pub smooth: bool,
    pub aa: bool,
    pub padding: [u32; 4],
    pub spacing: [u32; 2],
    pub outline: u32,
}

impl InfoTag {
    fn write_to(&self, out: &mut PrintWriter) -> io::Result<()> {
        out.write_str("info face=")?;
        out.write_string(&self.face)?;
        out.write_str(" size=")?;
        out.write_u32(self.size)?;
        out.write_str(" bold=")?;
        out.write_bool(self.bold)?;
        out.write_str(" italic=")?;
        out.write_bool(self.italic)?;
        out.write_str(" charset=")?;
        out.write_string(&self.charset)?;
        out.write_str(" unicode=")?;
        out.write_bool(self.unicode)?;
        out.write_str(" stretchH=")?;
        out.write_u32(self.stretch_h)?;
        out.write_str(" smooth=")?;
        out.write_bool(self.smooth)?;
        out.write_str(" aa=")?;
        out.write_bool(self.aa)?;
        out.write_str(" padding=")?;
        out.write_u32(self.padding[0])?;
        out.write_str(",")?;
        out.write_u32(self.padding[1])?;
        out.write_str(",")?;
        out.write_u32(self.padding[2])?;
        out.write_str(",")?;
        out.write_u32(self.padding[3])?;
        out.write_str(" spacing=")?;
        out.write_u32(self.spacing[0])?;
        out.write_str(",")?;
        out.write_u32(self.spacing[1])?;
        out.write_str(" outline=")?;
        out.write_u32(self.outline)?;
        out.write_str("\n")?;
        Ok(())
    }
}

pub struct CommonTag {
    pub line_height: u32,
    pub base: i32,
    pub scale_w: u32,
    pub scale_h: u32,
    pub pages: u32,
    pub packed: bool,
    pub alpha_channel: u8,
    pub red_channel: u8,
    pub green_channel: u8,
    pub blue_channel: u8,
}

impl CommonTag {
    fn write_to(&self, out: &mut PrintWriter) -> io::Result<()> {
        out.write_str("common lineHeight=")?;
        out.write_u32(self.line_height)?;
        out.write_str(" base=")?;
        out.write_i32(self.base)?;
        out.write_str(" scaleW=")?;
        out.write_u32(self.scale_w)?;
        out.write_str(" scaleH=")?;
        out.write_u32(self.scale_h)?;
        out.write_str(" pages=")?;
        out.write_u32(self.pages)?;
        out.write_str(" packed=")?;
        out.write_bool(self.packed)?;
        out.write_str(" alphaChnl=")?;
        out.write_u8(self.alpha_channel)?;
        out.write_str(" redChnl=")?;
        out.write_u8(self.red_channel)?;
        out.write_str(" greenChnl=")?;
        out.write_u8(self.green_channel)?;
        out.write_str(" blueChnl=")?;
        out.write_u8(self.blue_channel)?;
        out.write_str("\n")?;
        Ok(())
    }
}

pub struct PageTag {
    pub id: u32,
    pub file: String,
}

impl PageTag {
    fn write_to(&self, out: &mut PrintWriter) -> io::Result<()> {
        out.write_str("page id=")?;
        out.write_u32(self.id)?;
        out.write_str(" file=")?;
        out.write_string(&self.file)?;
        out.write_str("\n")?;
        Ok(())
    }
}

pub struct CharsTag {
    pub count: u32,
}

impl CharsTag {
    fn write_to(&self, out: &mut PrintWriter) -> io::Result<()> {
        out.write_str("chars count=")?;
        out.write_u32(self.count)?;
        out.write_str("\n")?;
        Ok(())
    }
}

pub struct CharTag {
    /// Character id, usually the Unicode codepoint
    pub id: usize,
    /// Offset of left glyph edge in bitmap font
    pub x: u32,
    /// Offset of top glyph edge in bitmap font
    pub y: u32,
    /// Width of the rendered glyph
    pub width: u32,
    /// Height of the rendered glyph
    pub height: u32,
    /// Horizontal offset that should be added to the text cursor to find the left edge position of the glyph
    pub xoffset: i32,
    /// Distance from the top of the cell (i.e. top of the line) to the top of the bitmap.
    pub yoffset: i32,
    /// Distance to move the text cursor forward to render the next glyph
    pub xadvance: i32,
    /// Identifies the bitmap font page containing this character
    pub page: u32,
    pub chnl: u8,
}

impl CharTag {
    fn write_to(&self, out: &mut PrintWriter) -> io::Result<()> {
        out.write_str("char id=")?;
        out.write_usize(self.id)?;
        out.write_str(" x=")?;
        out.write_u32(self.x)?;
        out.write_str(" y=")?;
        out.write_u32(self.y)?;
        out.write_str(" width=")?;
        out.write_u32(self.width)?;
        out.write_str(" height=")?;
        out.write_u32(self.height)?;
        out.write_str(" xoffset=")?;
        out.write_i32(self.xoffset)?;
        out.write_str(" yoffset=")?;
        out.write_i32(self.yoffset)?;
        out.write_str(" xadvance=")?;
        out.write_i32(self.xadvance)?;
        out.write_str(" page=")?;
        out.write_u32(self.page)?;
        out.write_str(" chnl=")?;
        out.write_u8(self.chnl)?;
        out.write_str("\n")?;
        Ok(())
    }
}

pub struct FontFile {
    pub info: InfoTag,
    pub common: CommonTag,
    pub page: PageTag,
    pub chars: Vec<CharTag>,
}

impl FontFile {
    pub fn write(&self, write: &mut io::Write) -> io::Result<()> {
        let mut out = PrintWriter::new(write);

        self.info.write_to(&mut out)?;
        self.common.write_to(&mut out)?;
        self.page.write_to(&mut out)?;
        
        CharsTag {
            count: self.chars.len() as u32,
        }.write_to(&mut out)?;

        for char in &self.chars {
            char.write_to(&mut out)?;
        }

        Ok(())
    }
}

struct PrintWriter<'a> {
    writer: &'a mut io::Write,
}

impl<'a> PrintWriter<'a> {
    fn new(writer: &'a mut io::Write) -> PrintWriter<'a> {
        PrintWriter {
            writer: writer,
        }
    }

    fn write_str(&mut self, str: &str) -> io::Result<()> {
        self.writer.write_all(str.as_bytes())
    }

    fn write_string(&mut self, string: &String) -> io::Result<()> {
        self.writer.write_all(string.as_bytes())
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        let value = if value { 1 } else { 0 };
        self.writer.write_all(value.to_string().as_bytes())
    }

    fn write_u8(&mut self, value: u8) -> io::Result<()> {
        self.writer.write_all(value.to_string().as_bytes())
    }

    fn write_i32(&mut self, value: i32) -> io::Result<()> {
        self.writer.write_all(value.to_string().as_bytes())
    }

    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        self.writer.write_all(value.to_string().as_bytes())
    }

    fn write_usize(&mut self, value: usize) -> io::Result<()> {
        self.writer.write_all(value.to_string().as_bytes())
    }
}
