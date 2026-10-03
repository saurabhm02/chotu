//! The global Cmd+/ (Ctrl+/ off macOS) shortcut. When pressed it reads the text
//! selected in the app in front, then tells the frontend what to do.
use tauri::{Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::services::selection;

fn toggle_modifier() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers::SUPER
    } else {
        Modifiers::CONTROL
    }
}

/// The plugin: on shortcut press, read the selection, then tell the frontend.
pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app_handle, shortcut, event| {
            let is_toggle_shortcut = shortcut.mods.contains(toggle_modifier())
                && shortcut.key == Code::Slash
                && event.state == ShortcutState::Pressed;

            if !is_toggle_shortcut {
                return;
            }
            log::info!("Global toggle shortcut triggered!");

            // macOS only allows the keyboard-layout lookup that simulating a
            // key press needs on the MAIN thread (from another thread the OS
            // kills the app), so the capture has to run there.
            let app = app_handle.clone();
            let _ = app_handle.run_on_main_thread(move || {
                // Must happen BEFORE we focus our own window, while the other
                // app still owns the selection.
                let selected = selection::capture();

                let Some(window) = app.get_webview_window("main") else {
                    return;
                };

                match selected {
                    // Text found: always open with it attached.
                    Some(text) => {
                        let _ = window.emit("selected-text", text);
                    }
                    // Nothing selected: behave like before (collapse/expand).
                    None => {
                        let _ = window.emit("toggle-floating-mode", ());
                    }
                }
                let _ = window.show();
                let _ = window.set_focus();
            });
        })
        .build()
}

/// Register the shortcut with the OS (call once from `setup`).
pub fn register<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut()
        .register(Shortcut::new(Some(toggle_modifier()), Code::Slash))?;
    Ok(())
}

