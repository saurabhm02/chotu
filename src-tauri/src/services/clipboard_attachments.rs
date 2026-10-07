use std::path::Path;

use crate::utils::images;

/// Saves the clipboard image into `folder` and returns its path. `None` when the
/// clipboard holds no image.
#[cfg(target_os = "macos")]
pub fn save_image(folder: &Path) -> Result<Option<String>, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    // A file copied in Finder also puts a picture of its icon on the clipboard. Look for
    // the file first, so the photo is pasted and not its icon.
    if let Ok(files) = clipboard.get().file_list() {
        if !files.is_empty() {
            return save_first_image_file(&files, folder).map(Some);
        }
    }

    let image = match clipboard.get_image() {
        Ok(image) => image,
        Err(arboard::Error::ContentNotAvailable) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };

    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let path = folder.join(format!("paste-{}.png", uuid::Uuid::new_v4().simple()));
    images::save_rgba_png(
        image.width as u32,
        image.height as u32,
        image.bytes.into_owned(),
        &path,
    )?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[cfg(not(target_os = "macos"))]
pub fn save_image(_folder: &Path) -> Result<Option<String>, String> {
    Err("pasting images is only supported on macOS".to_string())
}

/// Saves the first of `files` that is an image into `folder`. The others are skipped.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn save_first_image_file(files: &[std::path::PathBuf], folder: &Path) -> Result<String, String> {
    files
        .iter()
        .find_map(|file| save_image_file(file, folder).ok())
        .ok_or_else(|| "the copied file is not an image".to_string())
}

/// Copies an image file into `folder` as a JPEG of at most 1920 pixels, and returns the
/// new path. Fails when the file is not an image the app can read.
fn save_image_file(source: &Path, folder: &Path) -> Result<String, String> {
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let path = folder.join(format!("paste-{}.jpg", uuid::Uuid::new_v4().simple()));
    images::save_jpeg(source, &path)?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chotu-{label}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_image(dir: &Path, name: &str, width: u32, height: u32) -> PathBuf {
        let path = dir.join(name);
        image::RgbImage::from_pixel(width, height, image::Rgb([10, 120, 200]))
            .save(&path)
            .unwrap();
        path
    }

    #[test]
    fn a_jpeg_file_is_saved_as_a_readable_jpeg_in_the_folder() {
        let source = temp_dir("source");
        let folder = temp_dir("folder");
        let photo = write_image(&source, "photo.jpg", 300, 200);

        let saved = save_image_file(&photo, &folder).unwrap();

        assert!(Path::new(&saved).starts_with(&folder));
        let copy = image::open(&saved).unwrap();
        assert_eq!((copy.width(), copy.height()), (300, 200));
    }

    #[test]
    fn a_big_photo_is_shrunk_to_1920_pixels() {
        let source = temp_dir("source");
        let folder = temp_dir("folder");
        let photo = write_image(&source, "big.png", 3840, 1920);

        let saved = save_image_file(&photo, &folder).unwrap();

        let copy = image::open(&saved).unwrap();
        assert_eq!((copy.width(), copy.height()), (1920, 960));
    }

    #[test]
    fn a_file_that_is_not_an_image_is_refused() {
        let source = temp_dir("source");
        let folder = temp_dir("folder");
        let text = source.join("notes.txt");
        std::fs::write(&text, "not an image").unwrap();

        assert!(save_image_file(&text, &folder).is_err());
    }

    #[test]
    fn the_first_image_in_a_list_of_files_is_used() {
        let source = temp_dir("source");
        let folder = temp_dir("folder");
        let text = source.join("notes.txt");
        std::fs::write(&text, "not an image").unwrap();
        let photo = write_image(&source, "photo.jpg", 50, 40);

        let saved = save_first_image_file(&[text, photo], &folder).unwrap();

        assert!(image::open(&saved).is_ok());
    }

    #[test]
    fn a_list_without_images_is_an_error() {
        let source = temp_dir("source");
        let folder = temp_dir("folder");
        let text = source.join("notes.txt");
        std::fs::write(&text, "not an image").unwrap();

        assert!(save_first_image_file(&[text], &folder).is_err());
    }
}
