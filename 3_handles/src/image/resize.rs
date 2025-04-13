use deps::*;

use std::{io::BufWriter, num::NonZeroU32};

use debug_print::debug_println;
use fast_image_resize as fr;
use image::{codecs::png::PngEncoder, DynamicImage, ExtendedColorType, ImageEncoder};

const IMAGE_STANDARD_WIDTH: u32 = 150;
const IMAGE_STANDARD_HEIGHT: u32 = 150;
const IMAGE_SMALL_WIDTH: u32 = 40;
const IMAGE_SMALL_HEIGHT: u32 = 40;
const FILTER_TYPE: image::imageops::FilterType = image::imageops::FilterType::Lanczos3;
const RESIZE_ALGORITHM: fr::ResizeAlg = fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3);

/// This will produce an invalid image if dimensions are bigger than 60x60 pixels.
/// Returns None if the resize fails
pub fn fast_resize(
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
            ExtendedColorType::Rgba8,
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

pub fn resize_standard_dimensions(image: &DynamicImage) -> DynamicImage {
    image.resize(IMAGE_STANDARD_WIDTH, IMAGE_STANDARD_HEIGHT, FILTER_TYPE)
}

pub fn resize_small_dimensions(image: &DynamicImage) -> DynamicImage {
    image.resize(IMAGE_STANDARD_WIDTH, IMAGE_STANDARD_HEIGHT, FILTER_TYPE)
}
