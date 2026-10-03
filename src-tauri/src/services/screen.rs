//! Taking a screenshot of the display. On macOS this uses the built-in
//! `screencapture` tool and saves a PNG in the app's cache folder.

#[cfg(target_os = "macos")]
mod mac {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use crate::models::screen::ScreenShot;
    use tauri::{AppHandle, Manager, Runtime};

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }
    /// Without the Screen Recording permission macOS silently gives a screenshot
    /// with no app windows in it. So check first, and trigger the system prompt
    fn ensure_screen_permission() -> Result<(), String> {
        if unsafe { CGPreflightScreenCaptureAccess() } {
            return Ok(());
        }

        unsafe { CGRequestScreenCaptureAccess() };
        Err("Screen Recording permission is needed. Allow it in System Settings → Privacy & Security → Screen & System Audio Recording, then restart TY.".to_string())
    }

    pub async fn capture_display<R: Runtime>(app: &AppHandle<R>) -> Result<ScreenShot, String> {
        ensure_screen_permission()?;

        let id = uuid::Uuid::new_v4().simple().to_string();
        let path = screen_dir(app)?.join(format!("{id}.png"));

        // Hide TY so it isn't in the picture, and always bring it back after
        let window = app.get_webview_window("main");
        if let Some(window) = &window {
            let _ = window.hide();
        }

        // Give macOS a moment to really remove the window from the screen
        tokio::time::sleep(Duration::from_millis(250)).await;

        let captured = take_screenshot(&path).await;

        if let Some(window) = &window {
            let _ = window.show();
            let _ = window.set_focus();
        }
        captured?;

        let (width, height) = png_size(&path)?;
        log::info!("screen captured: {} ({width}x{height})", path.display());

        Ok(ScreenShot {
            id,
            image_path: path.to_string_lossy().into_owned(),
            width,
            height,
        })
    }

    fn screen_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
        let dir = app
            .path()
            .app_cache_dir()
            .map_err(|e| e.to_string())?
            .join("screens");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }

    /// `-x` = no camera sound, `-t png` = PNG file. Captures the main display
    async fn take_screenshot(path: &Path) -> Result<(), String> {
        let status = tokio::process::Command::new("screencapture")
            .args(["-x", "-t", "png"])
            .arg(path)
            .status()
            .await
            .map_err(|e| format!("could not run screencapture: {e}"))?;

        if !status.success() {
            return Err(format!("screencapture failed ({status})"));
        }
        Ok(())
    }

    /// Width and height straight from the PNG header (two big-endian numbers at
    /// bytes 16..24), so we don't need an image library just for this.
    fn png_size(path: &Path) -> Result<(u32, u32), String> {
        use std::io::Read;

        let mut header = [0u8; 24];
        std::fs::File::open(path)
            .and_then(|mut file| file.read_exact(&mut header))
            .map_err(|e| format!("could not read the screenshot: {e}"))?;

        if &header[1..4] != b"PNG" {
            return Err("the screenshot is not a PNG file".to_string());
        }
        let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
        let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
        Ok((width, height))
    }
}

#[cfg(target_os = "macos")]
pub use mac::capture_display;

#[cfg(not(target_os = "macos"))]
pub async fn capture_display<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
) -> Result<crate::models::screen::ScreenShot, String> {
    Err("Screen capture only works on macOS for now.".to_string())
}

