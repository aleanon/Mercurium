use std::path::Path;

use image::DynamicImage;



pub fn open_image(path: &Path) -> Option<DynamicImage> {
    if let Ok(reader) = image::io::Reader::open(path) {
        if let Ok(new_reader) = reader.with_guessed_format() {
           return new_reader.decode().ok()
        }
    }
    None
}