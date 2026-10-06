//! Getting an image ready to send to a vision model.
use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType};

/// Longest side we send. Bigger images are slower and cost more without helping the model.
const MAX_SIDE: u32 = 1920;
const JPEG_QUALITY: u8 = 85;
const THUMBNAIL_SIZE: u32 = 200;

/// Shrinks the image so neither side exceeds `MAX_SIDE` and returns it as JPEG bytes.
fn shrunk_jpeg(path: &Path, max_side: u32) -> Result<Vec<u8>, String> {
    let image = image::open(path).map_err(|e| format!("could not open the image: {e}"))?;

    let image = if image.width() > max_side || image.height() > max_side {
        image.resize(max_side, max_side, FilterType::Triangle)
    } else {
        image
    };

    let mut jpeg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY)
        .encode_image(&image.to_rgb8())
        .map_err(|e| format!("could not compress the image: {e}"))?;
    Ok(jpeg)
}

/// The shrunk image as a `data:` URL, the form chat APIs accept.
pub fn to_data_url(path: &Path) -> Result<String, String> {
    Ok(format!(
        "data:image/jpeg;base64,{}",
        STANDARD.encode(shrunk_jpeg(path, MAX_SIDE)?)
    ))
}

/// A small preview of the image as a `data:` URL.
pub fn thumbnail_data_url(path: &Path) -> Result<String, String> {
    Ok(format!(
        "data:image/jpeg;base64,{}",
        STANDARD.encode(shrunk_jpeg(path, THUMBNAIL_SIZE)?)
    ))
}

/// Writes the shrunk image to `destination` as a JPEG.
pub fn save_jpeg(path: &Path, destination: &Path) -> Result<(), String> {
    std::fs::write(destination, shrunk_jpeg(path, MAX_SIDE)?)
        .map_err(|e| format!("could not save the image: {e}"))
}

/// Writes raw RGBA pixels to `destination` as a PNG.
pub fn save_rgba_png(
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    destination: &Path,
) -> Result<(), String> {
    let image = image::RgbaImage::from_raw(width, height, pixels)
        .ok_or("the pixel data does not match the image size")?;
    image
        .save_with_format(destination, image::ImageFormat::Png)
        .map_err(|e| format!("could not save the image: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Saves a plain coloured PNG of the given size and returns its path.
    fn make_png(name: &str, width: u32, height: u32) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        image::RgbImage::from_pixel(width, height, image::Rgb([200, 30, 30]))
            .save(&path)
            .unwrap();
        path
    }

    /// Turns a data URL back into an image so the test can look at its size.
    fn decode(data_url: &str) -> image::DynamicImage {
        let base64_part = data_url.strip_prefix("data:image/jpeg;base64,").unwrap();
        let bytes = STANDARD.decode(base64_part).unwrap();
        image::load_from_memory(&bytes).unwrap()
    }

    #[test]
    fn small_image_keeps_its_size() {
        let path = make_png("chotu_small.png", 400, 300);
        let decoded = decode(&to_data_url(&path).unwrap());
        assert_eq!((decoded.width(), decoded.height()), (400, 300));
    }

    #[test]
    fn big_image_is_shrunk_and_keeps_its_shape() {
        let path = make_png("chotu_big.png", 3840, 1920);
        let decoded = decode(&to_data_url(&path).unwrap());
        assert_eq!((decoded.width(), decoded.height()), (1920, 960));
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(to_data_url(Path::new("/no/such/file.png")).is_err());
    }

    #[test]
    fn a_saved_copy_is_a_smaller_jpeg() {
        let source = make_png("chotu_save_source.png", 3840, 1920);
        let destination = std::env::temp_dir().join("chotu_save_copy.jpg");
        save_jpeg(&source, &destination).unwrap();

        let copy = image::open(&destination).unwrap();
        assert_eq!((copy.width(), copy.height()), (1920, 960));
        let _ = std::fs::remove_file(&destination);
    }

    #[test]
    fn thumbnail_is_shrunk_to_the_preview_size() {
        let path = make_png("chotu_thumb.png", 1000, 500);
        let decoded = decode(&thumbnail_data_url(&path).unwrap());
        assert_eq!((decoded.width(), decoded.height()), (200, 100));
    }

    #[test]
    fn save_rgba_png_writes_a_readable_png() {
        let destination = std::env::temp_dir().join("chotu_rgba.png");
        let pixels = vec![255u8; 4 * 3 * 2];

        save_rgba_png(3, 2, pixels, &destination).unwrap();

        let saved = image::open(&destination).unwrap();
        assert_eq!((saved.width(), saved.height()), (3, 2));
        let _ = std::fs::remove_file(&destination);
    }

    #[test]
    fn save_rgba_png_rejects_pixels_of_the_wrong_length() {
        let destination = std::env::temp_dir().join("chotu_rgba_bad.png");
        assert!(save_rgba_png(3, 2, vec![0u8; 5], &destination).is_err());
    }
}
