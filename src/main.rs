extern crate cairo;
extern crate freetype as ft;
extern crate gtk;

#[macro_use]
mod macros;
mod rendered_glyph;
mod render_settings;
mod main_window;

use std::cell::RefCell;
use std::rc::Rc;
use render_settings::{RenderSettings};
use main_window::{MainWindow};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let library = ft::Library::init().unwrap();
    let face: ft::Face = library.new_face("content/vt323-regular.ttf", 0).unwrap();
    let render_settings = Rc::new(RefCell::new(RenderSettings::new(library, face)));

    let main_window = MainWindow::new_from_file("content/main.glade", render_settings.clone());
    main_window.show();

    gtk::main();
}
