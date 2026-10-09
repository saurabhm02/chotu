use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Runtime};

use crate::services::panel;

const SHOW_ID: &str = "show";
const QUIT_ID: &str = "quit";
const TRAY_ICON: &[u8] = include_bytes!("../../icons/tray-icon.png");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TrayAction {
    Show,
    Quit,
}

fn action_for(menu_id: &str) -> Option<TrayAction> {
    match menu_id {
        SHOW_ID => Some(TrayAction::Show),
        QUIT_ID => Some(TrayAction::Quit),
        _ => None,
    }
}

fn run_action<R: Runtime>(app: &AppHandle<R>, action: TrayAction) {
    match action {
        TrayAction::Show => panel::show_main_window(app),
        TrayAction::Quit => app.exit(0),
    }
}

pub fn init<R: Runtime>(app: &tauri::App<R>) {
    if let Err(e) = build_tray(app) {
        log::error!("could not create the menu bar icon: {e}");
    }
}

fn build_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, SHOW_ID, "Show Chotu", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit Chotu", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::new()
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(true)
        .tooltip("Chotu")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if let Some(action) = action_for(event.id().as_ref()) {
                run_action(app, action);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                run_action(tray.app_handle(), TrayAction::Show);
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_show_item_shows_chotu() {
        assert_eq!(action_for("show"), Some(TrayAction::Show));
    }

    #[test]
    fn the_quit_item_quits() {
        assert_eq!(action_for("quit"), Some(TrayAction::Quit));
    }

    #[test]
    fn an_unknown_item_does_nothing() {
        assert_eq!(action_for("settings"), None);
        assert_eq!(action_for(""), None);
    }

    #[test]
    fn the_icon_file_is_a_44_pixel_picture_with_a_see_through_background() {
        let icon = image::load_from_memory(TRAY_ICON).unwrap();
        assert_eq!((icon.width(), icon.height()), (44, 44));
        assert!(icon.color().has_alpha());
    }
}
