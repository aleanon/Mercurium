use std::path::PathBuf;

use debug_print::debug_println;
use image::DynamicImage;

pub fn save_image(image: &DynamicImage, path: &PathBuf) {
    let mut path = path.clone();
    path.set_extension("png");
    if let Err(err) = image.save_with_format(path, image::ImageFormat::Png) {
        debug_println!(
            "{}:{} Unable to save image to file: {err}",
            module_path!(),
            line!()
        )
    }
    // let rgba = image.to_rgb8();
    // let mut encoded_image = BufWriter::new(Vec::with_capacity(rgba.as_raw().len()));

    // if let Ok(()) = rgba.write_with_encoder(PngEncoder::new(&mut encoded_image)) {
    //     if let Ok(mut file) = std::fs::File::create(path) {
    //         if let Ok(_) = file.write_all(encoded_image.buffer()) {
    //             debug_println!(
    //                 "{}:{} successfully saved image, path: {:?}",
    //                 module_path!(),
    //                 line!(),
    //                 path
    //             );
    //         } else {
    //             debug_println!(
    //                 "{}:{} Failed to save image, path: {:?}",
    //                 module_path!(),
    //                 line!(),
    //                 path
    //             )
    //         }
    //     } else {
    //         debug_println!(
    //             "{}:{} Unable to save image to disk: {:?}",
    //             module_path!(),
    //             line!(),
    //             path
    //         )
    //     }
    // } else {
    //     debug_println!("{}:{} Unable to encode image", module_path!(), line!())
    // }
}
