use cairo;
use gtk::{self, AboutDialog, ApplicationWindow, Builder, ColorButton, DrawingArea, FileChooserAction, FileChooserDialog, FontButton, ImageMenuItem, ResponseType, SpinButton, TextView};
use gtk::prelude::*;
use std::cell::RefCell;
use std::option::{Option};
use std::path::{Path};
use std::rc::Rc;

use glyphs::{Glyphs};
use render_settings::{RenderSettings};

pub struct MainWindow {
    window: ApplicationWindow,
}

impl MainWindow {

    pub fn new_from_file<P: AsRef<Path>>(file_path: P, render_settings: Rc<RefCell<RenderSettings>>) -> MainWindow {
        let builder = Builder::new_from_file(file_path);
        let window: ApplicationWindow = builder.get_object("main_window").expect("Couldn't get main window");
        let drawing_area: DrawingArea = builder.get_object("drawing_area").expect("Couldn't get drawing area");

        window.connect_delete_event(|_,_| {
            gtk::main_quit();
            Inhibit(false)
        });

        let new_menu_item: ImageMenuItem = builder.get_object("new_action").expect("Couldn't get new menu item");
        new_menu_item.connect_activate(clone!(drawing_area, render_settings => move |_| {
            (*render_settings.borrow_mut()).reset();
            drawing_area.queue_draw();
        }));

        let save_as_menu_item: ImageMenuItem = builder.get_object("save_as_action").expect("Couldn't get save as menu item");
        save_as_menu_item.connect_activate(clone!(drawing_area, render_settings, window => move |_| {
            let file_chooser = FileChooserDialog::new(Some("Save as..."), Some(&window), FileChooserAction::Save);
            file_chooser.add_buttons(&[
                ("Save", ResponseType::Ok.into()),
                ("Cancel", ResponseType::Cancel.into())
            ]);
            if file_chooser.run() == ResponseType::Ok.into() {
                let filename = file_chooser.get_filename().expect("Couldn't get filename");
                let png_filepath = filename.with_extension("png");
                let render_settings = render_settings.borrow();
                let width = drawing_area.get_allocated_width();
                let height = drawing_area.get_allocated_height();
                let glyphs = Glyphs::new();
                glyphs.write_to_file(png_filepath, &render_settings, width, height).expect("Succeeds");
            }
            file_chooser.destroy();
        }));

        let quit_menu_item: ImageMenuItem = builder.get_object("quit_action").expect("Couldn't get quit menu item");
        quit_menu_item.connect_activate(clone!(window => move |_| {
            window.close();
        }));

        let about_menu_item: ImageMenuItem = builder.get_object("about").expect("Couldn't get about menu item");
        about_menu_item.connect_activate(clone!(window => move |_| {
            let d = AboutDialog::new();
            d.set_title("About");
            d.set_authors(&["Michael Zoech"]);
            d.set_program_name("font-atlas");
            d.set_website(Some("https://github.com/crazymaik/font-atlas"));
            d.set_transient_for(Some(&window));
            d.run();
            d.destroy();
        }));

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
            let width = drawing_area.get_allocated_width();
            let height = drawing_area.get_allocated_height();

            let glyphs = Glyphs::new();

            let surface = glyphs.render_to_surface(&render_settings, width, height);

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