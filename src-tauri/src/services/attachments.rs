//! Stores the images attached to chat messages in the app's data folder.
//! Screenshots live in the cache folder, which macOS may clear.

use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine};
use tauri::{AppHandle, Manager};

use crate::utils::images;

/// Whether `file` is inside `folder`. Both paths are resolved first, so `..` and
/// symlinks cannot escape.
pub fn is_inside(file: &Path, folder: &Path) -> bool {
    match (file.canonicalize(), folder.canonicalize()) {
        (Ok(file), Ok(folder)) => file.starts_with(folder),
        _ => false,
    }
}

/// Copies each screenshot into `storage_dir` as a shrunk JPEG and returns the paths of
/// the copies. Only files inside `screenshots_dir` are accepted.
pub fn store_screenshots(
    storage_dir: &Path,
    screenshots_dir: &Path,
    message_id: i64,
    screenshots: &[String],
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(storage_dir).map_err(|e| e.to_string())?;

    let mut stored = Vec::new();
    for (index, screenshot) in screenshots.iter().enumerate() {
        let screenshot = Path::new(screenshot);
        if !is_inside(screenshot, screenshots_dir) {
            return Err("not a screenshot taken by this app".to_string());
        }
        let destination = storage_dir.join(format!("message-{message_id}-{index}.jpg"));
        images::save_jpeg(screenshot, &destination)?;
        stored.push(destination.to_string_lossy().into_owned());
    }
    Ok(stored)
}

/// Reads a stored image as a `data:` URL for an `<img>`. Only files inside
/// `storage_dir` are served.
pub fn read_data_url(storage_dir: &Path, path: &str) -> Result<String, String> {
    let path = Path::new(path);
    if !is_inside(path, storage_dir) {
        return Err("not a stored attachment".to_string());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("could not read the image: {e}"))?;
    Ok(format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes)))
}

pub fn storage_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("attachments"))
}

pub fn screenshots_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    Ok(cache_dir.join("screens"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chotu-{label}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_png(dir: &Path, name: &str) -> String {
        let path = dir.join(name);
        image::RgbImage::from_pixel(300, 200, image::Rgb([10, 120, 200]))
            .save(&path)
            .unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn store_screenshots_returns_the_path_of_each_copy() {
        let screenshots = temp_dir("screens");
        let storage = temp_dir("storage");
        let screenshot = write_png(&screenshots, "shot.png");

        let stored = store_screenshots(&storage, &screenshots, 7, &[screenshot]).unwrap();

        assert_eq!(stored.len(), 1);
        assert!(stored[0].ends_with("message-7-0.jpg"));
        assert!(Path::new(&stored[0]).is_file());
    }

    #[test]
    fn store_screenshots_rejects_files_outside_the_screenshots_dir() {
        let screenshots = temp_dir("screens");
        let elsewhere = temp_dir("elsewhere");
        let storage = temp_dir("storage");
        let stranger = write_png(&elsewhere, "other.png");

        assert!(store_screenshots(&storage, &screenshots, 1, &[stranger]).is_err());
    }

    #[test]
    fn read_data_url_returns_a_jpeg_data_url() {
        let screenshots = temp_dir("screens");
        let storage = temp_dir("storage");
        let screenshot = write_png(&screenshots, "shot.png");
        let stored = store_screenshots(&storage, &screenshots, 1, &[screenshot]).unwrap();

        let url = read_data_url(&storage, &stored[0]).unwrap();

        assert!(url.starts_with("data:image/jpeg;base64,"));
    }

    #[test]
    fn read_data_url_rejects_paths_outside_the_storage_dir() {
        let storage = temp_dir("storage");
        let elsewhere = temp_dir("elsewhere");
        let stranger = write_png(&elsewhere, "other.png");

        assert!(read_data_url(&storage, &stranger).is_err());
        assert!(read_data_url(&storage, "/etc/passwd").is_err());
    }

    #[test]
    fn read_data_url_rejects_dot_dot_escapes() {
        let storage = temp_dir("storage");
        let escape = format!("{}/../../etc/passwd", storage.display());

        assert!(read_data_url(&storage, &escape).is_err());
    }
}
