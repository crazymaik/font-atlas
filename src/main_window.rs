use cairo;
use gtk::{self, Builder, ColorButton, DrawingArea, SpinButton, Window};
use gtk::prelude::*;
use std::cell::RefCell;
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
        let font_size_spin_button: SpinButton = builder.get_object("font_size").expect("Couldn't get font size spin button");
        let font_color_button: ColorButton = builder.get_object("font_color").expect("Couldn't get font color button");
        let border_width_spin_button: SpinButton = builder.get_object("border_width").expect("Couldn't get border width spin button");
        let border_color_button: ColorButton = builder.get_object("border_color").expect("Couldn't get border color button");

        window.connect_delete_event(|_,_| {
            gtk::main_quit();
            Inhibit(false)
        });

        font_color_button.set_rgba(&render_settings.borrow().font_color);

        ColorButtonExt::connect_property_rgba_notify(&font_color_button, clone!(drawing_area, render_settings => move |btn| {
            let new_color = btn.get_rgba();
            (*render_settings.borrow_mut()).font_color = new_color;
            drawing_area.queue_draw();
        }));

        font_size_spin_button.set_value(render_settings.borrow().font_size as f64);

        font_size_spin_button.connect_value_changed(clone!(drawing_area, render_settings => move |btn| {
            let new_size = btn.get_value() as isize;
            (*render_settings.borrow_mut()).font_size = new_size;
            (*render_settings.borrow()).face.set_char_size(0, new_size * 64, 0, 64).unwrap();
            drawing_area.queue_draw();
        }));

        border_color_button.set_rgba(&render_settings.borrow().border_color);

        ColorButtonExt::connect_property_rgba_notify(&border_color_button, clone!(drawing_area, render_settings => move |btn| {
            let new_color = btn.get_rgba();
            (*render_settings.borrow_mut()).border_color = new_color;
            drawing_area.queue_draw();
        }));

        border_width_spin_button.set_value(render_settings.borrow().border_width as f64);

        border_width_spin_button.connect_value_changed(clone!(drawing_area, render_settings => move |btn| {
            let new_width = btn.get_value() as isize;
            (*render_settings.borrow_mut()).border_width = new_width;
            drawing_area.queue_draw();
        }));

        drawing_area.connect_draw(clone!(render_settings => move |_, cr| {
            let render_settings = render_settings.borrow();
            let width = 800;
            let height = 800;
            let character = 65;
            let left = 50.0;
            let top = 20.0;
            let outline_glyph = RenderedGlyph::new_outline(&render_settings.library, &render_settings.face, character, &render_settings.border_color, render_settings.border_width).unwrap();
            let inside_glyph = RenderedGlyph::new(&render_settings.library, &render_settings.face, character, &render_settings.font_color).unwrap();
            let outline_left = outline_glyph.origin.0 as f64;
            let outline_top = outline_glyph.origin.1 as f64;
            let inside_left = inside_glyph.origin.0 as f64;
            let inside_top = inside_glyph.origin.1 as f64;
            let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
            let context = cairo::Context::new(&surface);

            context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            context.set_operator(cairo::Operator::Source);
            context.rectangle(0.0, 0.0, width as f64, height as f64);
            context.fill();

            context.set_operator(cairo::Operator::Source);
            context.set_source_surface(&outline_glyph.surface, left + outline_left, top + outline_top);
            context.paint();

            context.set_operator(cairo::Operator::Over);
            context.set_source_surface(&inside_glyph.surface, left + inside_left, top + inside_top);
            context.paint();

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