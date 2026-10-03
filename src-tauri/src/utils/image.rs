//! Getting an image ready to send to a vision model.
use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType};

/// Longest side we send. Bigger images are slower and cost more without helping the model.
const MAX_SIDE: u32 = 1920;
const JPEG_QUALITY: u8 = 85;

/// Reads an image file, shrinks it so neither side is over `MAX_SIDE`, saves it as
/// a JPEG in memory, and returns it as a `data:` URL, which is how a chat API
/// accepts an image inside a request.

pub fn to_data_url(path: &Path) -> Result<String, String> {
    let image = image::open(path).map_err(|e| format!("Could not open the image: {e}"))?;

    let image = if image.width() > MAX_SIDE || image.height() > MAX_SIDE {
        image.resize(MAX_SIDE, MAX_SIDE, FilterType::Triangle)
    } else {
        image
    };

    let mut jpeg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY)
        .encode_image(&image.to_rgb8())
        .map_err(|e| format!("could not compress the image: {e}"))?;

    Ok(format!("data:image/jpeg;base64,{}", STANDARD.encode(&jpeg)))
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
        let path = make_png("ty_small.png", 400, 300);
        let decoded = decode(&to_data_url(&path).unwrap());
        assert_eq!((decoded.width(), decoded.height()), (400, 300));
    }

    #[test]
    fn big_image_is_shrunk_and_keeps_its_shape() {
        let path = make_png("ty_big.png", 3840, 1920);
        let decoded = decode(&to_data_url(&path).unwrap());
        assert_eq!((decoded.width(), decoded.height()), (1920, 960));
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(to_data_url(Path::new("/no/such/file.png")).is_err());
    }
}
