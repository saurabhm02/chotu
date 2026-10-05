//! Tauri commands for saved chats and message attachments.
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::db::Db;
use crate::models::chat::{ChatSummary, NewMessage, StoredMessage};
use crate::services::llm::AiClient;
use crate::services::{attachments, history};

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn with_db<T>(
    app: &AppHandle,
    work: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let db = app.try_state::<Db>().ok_or("history is turned off")?;
    let conn =
        db.0.lock()
            .map_err(|_| "the database is busy".to_string())?;
    work(&conn)
}

#[tauri::command]
pub fn chat_create(app: AppHandle, first_message: String) -> Result<i64, String> {
    with_db(&app, |conn| {
        history::create_chat(conn, &first_message, now_ms())
    })
}

#[tauri::command]
pub fn chat_add_message(app: AppHandle, message: NewMessage) -> Result<i64, String> {
    with_db(&app, |conn| history::add_message(conn, &message, now_ms()))
}

#[tauri::command]
pub fn chat_list(app: AppHandle, filter: String) -> Result<Vec<ChatSummary>, String> {
    with_db(&app, |conn| history::list_chats(conn, &filter))
}

#[tauri::command]
pub fn chat_messages(app: AppHandle, chat_id: i64) -> Result<Vec<StoredMessage>, String> {
    with_db(&app, |conn| history::load_messages(conn, chat_id))
}

#[tauri::command]
pub fn chat_rename(app: AppHandle, chat_id: i64, title: String) -> Result<String, String> {
    with_db(&app, |conn| {
        history::rename_chat(conn, chat_id, &title, now_ms())
    })
}

#[tauri::command]
pub fn chat_delete(app: AppHandle, chat_id: i64) -> Result<(), String> {
    with_db(&app, |conn| history::delete_chat(conn, chat_id))
}

#[tauri::command]
pub async fn chat_generate_title(
    app: AppHandle,
    chat_id: i64,
    first_message: String,
    answer: String,
) -> Result<String, String> {
    let question: String = history::strip_command(&first_message)
        .chars()
        .take(300)
        .collect();
    let answer: String = answer.chars().take(600).collect();

    let title = AiClient::shared()
        .generate_title(&question, &answer)
        .await?;

    with_db(&app, |conn| {
        history::rename_chat(conn, chat_id, &title, now_ms())
    })
}

#[tauri::command]
pub async fn store_attachments(
    app: AppHandle,
    message_id: i64,
    screenshots: Vec<String>,
) -> Result<Vec<String>, String> {
    let storage_dir = attachments::storage_dir(&app)?;
    let screenshots_dir = attachments::screenshots_dir(&app)?;

    let stored = tokio::task::spawn_blocking(move || {
        attachments::store_screenshots(&storage_dir, &screenshots_dir, message_id, &screenshots)
    })
    .await
    .map_err(|e| format!("image task failed: {e}"))??;

    with_db(&app, |conn| {
        history::set_attachments(conn, message_id, &stored)
    })?;
    Ok(stored)
}

#[tauri::command]
pub fn attachment_data_url(app: AppHandle, path: String) -> Result<String, String> {
    attachments::read_data_url(&attachments::storage_dir(&app)?, &path)
}
