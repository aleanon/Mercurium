use image::ImageFormat;

pub fn get_extension(format: &ImageFormat) -> &str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Gif => "gif",
        ImageFormat::WebP => "webp",
        ImageFormat::Pnm => "ppm",
        ImageFormat::Tiff => "tiff",
        ImageFormat::Tga => "tga",
        ImageFormat::Dds => "dds",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Ico => "ico",
        ImageFormat::Hdr => "hdr",
        ImageFormat::OpenExr => "exr",
        ImageFormat::Farbfeld => "ff",
        // According to: https://aomediacodec.github.io/av1-avif/#mime-registration
        ImageFormat::Avif => "avif",
        ImageFormat::Qoi => "qoi",
        _ => "none"
    }
}