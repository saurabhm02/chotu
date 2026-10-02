pub mod commands;
pub mod config;
pub mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    let builder = builder
        .plugin(services::shortcut::plugin())
        .setup(|app| {
            services::shortcut::register(app)?;
            Ok(())
        });

    builder
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::chat::ask_ai,
            commands::web::ask_web,
            commands::window::keep_window_on_screen
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
