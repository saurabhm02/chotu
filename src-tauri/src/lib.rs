pub mod commands;
pub mod config;
pub mod db;
pub mod models;
pub mod services;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    let builder = builder.plugin(services::shortcut::plugin()).setup(|app| {
        services::panel::init(app);
        services::shortcut::register(app)?;
        db::init(app);
        Ok(())
    });

    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::chat::ask_ai,
            commands::web::ask_web,
            commands::window::keep_window_on_screen,
            commands::ocr::extract_text,
            commands::screen::capture_screen,
            commands::screen::capture_region,
            commands::history::chat_create,
            commands::history::chat_add_message,
            commands::history::chat_list,
            commands::history::chat_messages,
            commands::history::chat_rename,
            commands::history::chat_delete,
            commands::history::chat_generate_title,
            commands::history::store_attachments,
            commands::history::attachment_data_url,
            commands::images::paste_image,
            commands::images::preview_image,
            commands::images::view_image,
            commands::chat::ask_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
