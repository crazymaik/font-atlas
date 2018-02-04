use ft;

pub struct RenderSettings {
    pub library: ft::Library,
    pub face: ft::Face<'static>,
    pub border_width: isize,
    pub char_size: isize,
}

impl RenderSettings {
    pub fn new(library: ft::Library, face: ft::Face<'static>) -> RenderSettings {
        let default_char_size = 128;

        face.set_char_size(0, default_char_size*64, 0, 64).unwrap();

        RenderSettings {
            library: library,
            face: face,
            border_width: 0,
            char_size: default_char_size,
        }
    }
}