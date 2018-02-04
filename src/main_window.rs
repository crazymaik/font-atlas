use cairo;
use gtk::{self, Builder, ColorButton, DrawingArea, FontButton, SpinButton, TextView, Window};
use gtk::prelude::*;
use std::cell::RefCell;
use std::cmp;
use std::path::{Path};
use std::rc::Rc;

use rendered_glyph::{RenderedGlyph};
use render_settings::{RenderSettings};

pub struct MainWindow {
    window: Window,
}

impl MainWindow {

    pub fn new_from_file<P: AsRef<Path>>(file_path: P, render_settings: Rc<RefCell<RenderSettings>>) -> MainWindow {
        let builder = Builder::new_from_file(file_path);
        let window: Window = builder.get_object("main_window").expect("Couldn't get main window");
        let drawing_area: DrawingArea = builder.get_object("drawing_area").expect("Couldn't get drawing area");

        window.connect_delete_event(|_,_| {
            gtk::main_quit();
            Inhibit(false)
        });

        let text_field: TextView = builder.get_object("text").expect("Couldn't get text field");
        text_field.get_buffer().expect("No text buffer").set_text(&render_settings.borrow().text);
        text_field.get_buffer().expect("No text buffer").connect_property_text_notify(clone!(drawing_area, render_settings => move |buffer| {
            let text: String = buffer.get_text(&buffer.get_start_iter(), &buffer.get_end_iter(), false).expect("text");
            (*render_settings.borrow_mut()).text = text;
            drawing_area.queue_draw();
        }));

        let font_color_button: ColorButton = builder.get_object("font_color").expect("Couldn't get font color button");
        font_color_button.set_rgba(&render_settings.borrow().font_color);
        ColorButtonExt::connect_property_rgba_notify(&font_color_button, clone!(drawing_area, render_settings => move |btn| {
            let new_color = btn.get_rgba();
            (*render_settings.borrow_mut()).font_color = new_color;
            drawing_area.queue_draw();
        }));

        let font_size_spin_button: SpinButton = builder.get_object("font_size").expect("Couldn't get font size spin button");
        font_size_spin_button.set_value(render_settings.borrow().font_size as f64);
        font_size_spin_button.connect_value_changed(clone!(drawing_area, render_settings => move |btn| {
            let new_size = btn.get_value() as isize;
            (*render_settings.borrow_mut()).font_size = new_size;
            (*render_settings.borrow()).face.set_char_size(0, new_size * 64, 0, 64).unwrap();
            drawing_area.queue_draw();
        }));

        let font_letter_spacing_button: SpinButton = builder.get_object("font_letter_spacing").expect("Couldn't get font letter spacing button");
        font_letter_spacing_button.set_value(render_settings.borrow().letter_spacing as f64);
        font_letter_spacing_button.connect_value_changed(clone!(drawing_area, render_settings => move |btn| {
            let new_spacing = btn.get_value() as isize;
            (*render_settings.borrow_mut()).letter_spacing = new_spacing;
            drawing_area.queue_draw();
        }));

        let border_color_button: ColorButton = builder.get_object("border_color").expect("Couldn't get border color button");
        border_color_button.set_rgba(&render_settings.borrow().border_color);
        ColorButtonExt::connect_property_rgba_notify(&border_color_button, clone!(drawing_area, render_settings => move |btn| {
            let new_color = btn.get_rgba();
            (*render_settings.borrow_mut()).border_color = new_color;
            drawing_area.queue_draw();
        }));

        let border_width_spin_button: SpinButton = builder.get_object("border_width").expect("Couldn't get border width spin button");
        border_width_spin_button.set_value(render_settings.borrow().border_width as f64);
        border_width_spin_button.connect_value_changed(clone!(drawing_area, render_settings => move |btn| {
            let new_width = btn.get_value() as isize;
            (*render_settings.borrow_mut()).border_width = new_width;
            drawing_area.queue_draw();
        }));

        drawing_area.connect_draw(clone!(drawing_area, render_settings => move |_, cr| {
            let render_settings = render_settings.borrow();
            let letter_spacing = render_settings.letter_spacing as i32;
            let width = drawing_area.get_allocated_width();
            let height = drawing_area.get_allocated_height();
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

            cr.set_operator(cairo::Operator::Over);
            cr.set_source_surface(&surface, 0.0, 0.0);
            cr.paint();

            Inhibit(false)
        }));

        MainWindow {
            window: window,
        }
    }

    pub fn show(&self) {
        self.window.show();
    }
}