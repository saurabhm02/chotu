//! The global Cmd+/ (Ctrl+/ off macOS) shortcut that toggles minimized mode.
use tauri::{Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

fn toggle_modifier() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers::SUPER
    } else {
        Modifiers::CONTROL
    }
}

/// The plugin: on shortcut press, tell the frontend to toggle and focus the window.
pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app_handle, shortcut, event| {
            let is_toggle_shortcut = shortcut.mods.contains(toggle_modifier())
                && shortcut.key == Code::Slash
                && event.state == ShortcutState::Pressed;

            if is_toggle_shortcut {
                log::info!("Global toggle shortcut triggered!");

                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("toggle-floating-mode", ());
                    let _ = window.set_focus();
                }
            }
        })
        .build()
}

/// Register the shortcut with the OS (call once from `setup`).
pub fn register<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut()
        .register(Shortcut::new(Some(toggle_modifier()), Code::Slash))?;
    Ok(())
}
