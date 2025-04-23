use deps::*;
    
use fast_image_resize::{images::Image, FilterType, IntoImageView, ResizeAlg, ResizeOptions, Resizer};

use std::io::{BufWriter, Cursor};

use image::{codecs::png::PngEncoder, DynamicImage, ImageEncoder};

const IMAGE_STANDARD_WIDTH: u32 = 150;
const IMAGE_STANDARD_HEIGHT: u32 = 150;
const IMAGE_SMALL_WIDTH: u32 = 40;
const IMAGE_SMALL_HEIGHT: u32 = 40;
const FILTER_TYPE: image::imageops::FilterType = image::imageops::FilterType::Lanczos3;
const RESIZE_ALGORITHM: ResizeAlg = ResizeAlg::Convolution(FilterType::Lanczos3);

/// This will produce an invalid image if dimensions are bigger than 60x60 pixels.
/// Returns None if the resize fails
pub fn fast_resize(
    image: &DynamicImage,
    new_width: u32,
    new_height: u32,
    resize_algo: ResizeAlg,
) -> Option<BufWriter<Vec<u8>>> {

    let mut dst_image = Image::new(
        new_width,
        new_height,
        image.pixel_type().unwrap(),
    );

    let mut resizer = Resizer::new();
    let resize_options = ResizeOptions::new()
        .resize_alg(resize_algo);

    resizer.resize(image, &mut dst_image, Some(&resize_options)).ok()?;

    let mut result_buf = BufWriter::new(Vec::new());
    PngEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            new_width,
            new_height,
            image.color().into(),
        ).ok()?;

    Some(result_buf)
}

pub fn resize_standard_dimensions(image: &DynamicImage) -> Option<Vec<u8>> {
    fast_resize(image, IMAGE_STANDARD_WIDTH, IMAGE_STANDARD_HEIGHT, ResizeAlg::Convolution(FilterType::Lanczos3))
        .and_then(|buffer| buffer.into_inner().ok())
    // image.resize(IMAGE_STANDARD_WIDTH, IMAGE_STANDARD_HEIGHT, FILTER_TYPE)
}

pub fn resize_small_dimensions(image: &DynamicImage) -> Option<Vec<u8>> {
    fast_resize(image, IMAGE_SMALL_WIDTH, IMAGE_SMALL_HEIGHT, RESIZE_ALGORITHM)
        .and_then(|buffer|buffer.into_inner().ok())
    // let reader = image::ImageReader::new(Cursor::new(buffer.into_inner().ok()?));
    // let with_guessed_format = reader.with_guessed_format().ok()?;
    // with_guessed_format.decode().ok()
    // image.resize(IMAGE_SMALL_WIDTH, IMAGE_SMALL_HEIGHT, FILTER_TYPE)
}


pub fn resize_standard_dimensions_from_bytes(image: &Vec<u8>) -> Option<Vec<u8>> {
    let reader = image::ImageReader::new(Cursor::new(image));
    let with_guessed_format = reader.with_guessed_format().ok()?;
    let image = with_guessed_format.decode().ok()?;

    resize_standard_dimensions(&image)
}

pub fn resize_small_dimensions_from_bytes(image: &Vec<u8>) -> Option<Vec<u8>> {
    let reader = image::ImageReader::new(Cursor::new(image));
    let with_guessed_format = reader.with_guessed_format().ok()?;
    let image = with_guessed_format.decode().ok()?;

    resize_small_dimensions(&image)
    // let mut encoded_small = BufWriter::new(Cursor::new(Vec::new()));
    // resize_small_dimensions(&image)
    //         .write_to(&mut encoded_small, image::ImageFormat::Png)
    //         .ok()?;        
}