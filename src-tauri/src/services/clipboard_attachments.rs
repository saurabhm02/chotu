use std::path::Path;

/// Saves the clipboard image into `folder` and returns its path. `None` when the
/// clipboard holds no image.
#[cfg(target_os = "macos")]
pub fn save_image(folder: &Path) -> Result<Option<String>, String> {
    use crate::utils::images;

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

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
