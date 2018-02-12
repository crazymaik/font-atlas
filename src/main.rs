extern crate cairo;
extern crate freetype as ft;
extern crate gdk;
extern crate gtk;

mod font;
mod glyph;
mod glyphs;
#[macro_use]
mod macros;
mod main_window;
mod render_settings;

use std::cell::RefCell;
use std::rc::Rc;
use render_settings::{RenderSettings};
use main_window::{MainWindow};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let library = Rc::new(ft::Library::init().unwrap());
    let face = Rc::new(library.new_face("content/vt323-regular.ttf", 0).unwrap());
    let render_settings = Rc::new(RefCell::new(RenderSettings::new(&library, &face)));

    let main_window = MainWindow::new_from_file("content/main.glade", render_settings.clone());
    main_window.show();

    gtk::main();
}
