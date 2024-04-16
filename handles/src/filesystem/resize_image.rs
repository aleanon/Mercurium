use std::{io::BufWriter, num::NonZeroU32};

use debug_print::debug_println;
use fast_image_resize as fr;
use image::{codecs::png::PngEncoder, ColorType, DynamicImage, ImageEncoder};

const RESIZE_ALGORITHM: fr::ResizeAlg = fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3);

pub fn resize_image(
    image: &DynamicImage,
    new_width: NonZeroU32,
    new_height: NonZeroU32,
) -> Option<BufWriter<Vec<u8>>> {
    let aspect_ratio = image.width() as f32 / image.height() as f32;

    let new_width = match NonZeroU32::new((new_width.get() as f32 * aspect_ratio).round() as u32) {
        Some(value) => value,
        None => {
            debug_println!(
                "{}:{} new_width x aspect ratio resulted in a zero value",
                module_path!(),
                line!()
            );
            return None;
        }
    };

    let width = match NonZeroU32::new(image.width()) {
        Some(value) => value,
        None => {
            debug_println!(
                "{}:{} Supplied image has a width of zero",
                module_path!(),
                line!()
            );
            return None;
        }
    };
    let height = match NonZeroU32::new(image.height()) {
        Some(value) => value,
        None => {
            debug_println!(
                "{}:{} Supplied image has a height of zero",
                module_path!(),
                line!()
            );
            return None;
        }
    };

    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        image.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .inspect_err(|err| {
        debug_println!(
            "{}:{} Failed to create source image for resizing: {err}",
            module_path!(),
            line!()
        );
    })
    .ok()?;

    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .inspect_err(|err| {
            debug_println!(
                "{}:{} Failed to multiply alpha in place: {err}",
                module_path!(),
                line!()
            );
        })
        .ok()?;

    let mut dst_image = fr::Image::new(new_width, new_height, fr::PixelType::U8x4);

    let mut dst_view = dst_image.view_mut();

    let mut resizer = fr::Resizer::new(RESIZE_ALGORITHM);

    resizer
        .resize(&src_image.view(), &mut dst_view)
        .inspect_err(|err| {
            debug_println!(
                "{}:{} Unable to resize image: {err}",
                module_path!(),
                line!()
            );
        })
        .ok()?;

    alpha_mul_div
        .divide_alpha_inplace(&mut dst_view)
        .inspect_err(|err| {
            debug_println!(
                "{}:{} Failed to divide image alpha: {err}",
                module_path!(),
                line!()
            );
        })
        .ok()?;

    let mut result_buf = BufWriter::new(Vec::new());
    PngEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            new_width.get(),
            new_height.get(),
            ColorType::Rgba8,
        )
        .inspect_err(|err| {
            debug_println!(
                "{}:{} Unable to write image to buffer: {err}",
                module_path!(),
                line!()
            )
        })
        .ok()?;

    Some(result_buf)
}



fn iced_resize_image(image: &DynamicImage, new_height: u32, new_width: u32) -> DynamicImage {
    image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

#[cfg(test)]
mod test {
    use std::{io::Write, path::PathBuf};

    

    use super::*;

    #[test]
    fn should_pass_png() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");

        let mut test_folder = PathBuf::from(manifest_dir);
        test_folder.push("test_data");
        let mut file_path = test_folder.clone();
        file_path.push("gc-token.png");

        let image = image::open(file_path).expect("Failed to open image file");

        let resized = resize_image(
            &image,
            NonZeroU32::new(120).unwrap(),
            NonZeroU32::new(120).unwrap(),
        )
        .expect("Failed to resize image");

        // assert_eq!(40, resized.width());
        // assert_eq!(40, resized.height());

        let mut new_file_path = test_folder.clone();
        new_file_path.push("resized");
        new_file_path.set_extension("png");

        let mut file =
            std::fs::File::create(new_file_path).expect("Unable to create new image file");
        file.write_all(resized.buffer())
            .expect("Unable to write image to file");
    }

    #[test]
    fn test_basic_resize() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");

        let mut test_folder = PathBuf::from(manifest_dir);
        test_folder.push("test_data");
        let mut file_path = test_folder.clone();
        file_path.push("ravault_logo_cropped.jpg");

        let image = image::open(file_path).expect("Failed to open image file");

        let new_size = (20, 20);
        let image = iced_resize_image(&image, new_size.0, new_size.1);
        let mut new_path = test_folder.clone();
        new_path.push("ravault_logo_cropped_20x20");
        new_path.set_extension("png");

        image.save(new_path).expect("Failed to save image");
    }
}
