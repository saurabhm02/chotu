use tauri::{PhysicalPosition, WebviewWindow};

/// Slide the window just far enough that a `width` x `height` (logical px) window
/// sits fully inside the usable area of the monitor it is on.
///
/// The target size is passed in rather than read back from the window, because
/// the OS applies a just-requested resize asynchronously.
#[tauri::command]
pub fn keep_window_on_screen(window: WebviewWindow, width: f64, height: f64) {
    if let Err(e) = fit_on_screen(&window, width, height) {
        log::warn!("keep_window_on_screen failed: {e}");
    }
}

fn fit_on_screen(window: &WebviewWindow, width: f64, height: f64) -> tauri::Result<()> {
    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };
    let scale = window.scale_factor()?;
    let pos = window.outer_position()?;
    let area = monitor.work_area();

    let w = (width * scale).round() as i32;
    let h = (height * scale).round() as i32;
    let min_x = area.position.x;
    let min_y = area.position.y;
    let max_x = (min_x + area.size.width as i32 - w).max(min_x);
    let max_y = (min_y + area.size.height as i32 - h).max(min_y);

    let x = pos.x.clamp(min_x, max_x);
    let y = pos.y.clamp(min_y, max_y);
    log::debug!("fit: pos=({},{}) -> ({x},{y}) size={w}x{h} area=({min_x},{min_y},{max_x},{max_y})", pos.x, pos.y);
    if (x, y) != (pos.x, pos.y) {
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}
