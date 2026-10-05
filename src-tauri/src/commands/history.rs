use crate::db::Db;
use crate::models::chat::NewMessage;
use crate::services::history;
use tauri::{AppHandle, Manager};

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[tauri::command]
pub fn chat_create(app: AppHandle, first_message: String) -> Result<i64, String> {
    let db = app.try_state::<Db>().ok_or("history is turned off")?;
    let conn =
        db.0.lock()
            .map_err(|_| "the database is busy".to_string())?;
    history::create_chat(&conn, &first_message, now_ms())
}

#[tauri::command]
pub fn chat_add_message(app: AppHandle, message: NewMessage) -> Result<i64, String> {
    let db = app.try_state::<Db>().ok_or("history is turned off")?;
    let conn =
        db.0.lock()
            .map_err(|_| "the database is busy".to_string())?;
    history::add_message(&conn, &message, now_ms())
}
