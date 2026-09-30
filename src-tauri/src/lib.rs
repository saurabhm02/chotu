use tauri::Manager;

pub mod adapters;
pub mod ai;
pub mod commands;
pub mod types;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app_handle, shortcut, event| {
                    use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

                    let target_modifier = if cfg!(target_os = "macos") {
                        Modifiers::SUPER
                    } else {
                        Modifiers::CONTROL
                    };

                    let is_toggle_shortcut = shortcut.mods.contains(target_modifier)
                        && shortcut.key == Code::Slash
                        && event.state == ShortcutState::Pressed;

                    if is_toggle_shortcut {
                        log::info!("Global toggle shortcut triggered!");

                        if let Some(window) = app_handle.get_webview_window("main") {
                            if let Ok(true) = window.is_visible() {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

                let target_modifier = if cfg!(target_os = "macos") {
                    Modifiers::SUPER
                } else {
                    Modifiers::CONTROL
                };

                let toggle_shortcut = Shortcut::new(Some(target_modifier), Code::Slash);
                app.global_shortcut().register(toggle_shortcut)?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::ai::ask_ai,
            commands::websearch::ask_web
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
