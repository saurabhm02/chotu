use crate::models::chat::{ChatSummary, NewMessage, StoredMessage};
use rusqlite::{params, Connection};

const MAX_TITLE_CHARS: usize = 40;

/// Temporary chat title: the first message without its command, cut at 40 characters.
pub fn make_title(first_message: &str) -> String {
    // New lines and runs of spaces become single spaces.
    let one_line = strip_command(first_message)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if one_line.is_empty() {
        return "New chat".to_string();
    }
    if one_line.chars().count() > MAX_TITLE_CHARS {
        let cut: String = one_line.chars().take(MAX_TITLE_CHARS).collect();
        return format!("{}…", cut.trim_end());
    }
    one_line
}

/// Drops a leading "/command " unless nothing follows it.
pub fn strip_command(text: &str) -> &str {
    let text = text.trim();
    match text.strip_prefix('/') {
        Some(rest) => match rest.split_once(char::is_whitespace) {
            Some((_, after)) if !after.trim().is_empty() => after.trim(),
            _ => text,
        },
        None => text,
    }
}

/// Starts a chat and returns its id.
pub fn create_chat(conn: &Connection, first_message: &str, now_ms: i64) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO chats (title, created_at, updated_at) VALUES (?1, ?2, ?2)",
        params![make_title(first_message), now_ms],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn add_message(conn: &Connection, msg: &NewMessage, now_ms: i64) -> Result<i64, String> {
    if msg.role != "user" && msg.role != "assistant" {
        return Err(format!("unknown role {:?}", msg.role));
    }

    let sources = if msg.sources.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&msg.sources).map_err(|e| e.to_string())?)
    };

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "
         INSERT INTO messages 
         (chat_id, role, content, quote, sources, model, is_error,elapsed_ms, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            msg.chat_id,
            msg.role,
            msg.content,
            msg.selected,
            sources,
            msg.model,
            msg.is_error,
            msg.elapsed_ms,
            now_ms
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "UPDATE chats SET updated_at = ?1 WHERE id = ?2",
        params![now_ms, msg.chat_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(id)
}

const MAX_NAME_CHARS: usize = 200;

/// Chats newest first. A non-empty `filter` keeps titles containing it, ignoring case.
pub fn list_chats(conn: &Connection, filter: &str) -> Result<Vec<ChatSummary>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, title, updated_at FROM chats
             WHERE ?1 = '' OR instr(lower(title), lower(?1)) > 0
             ORDER BY updated_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = statement
        .query_map([filter.trim()], |row| {
            Ok(ChatSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Messages of one chat, oldest first.
pub fn load_messages(conn: &Connection, chat_id: i64) -> Result<Vec<StoredMessage>, String> {
    let mut statement = conn
        .prepare(
            "SELECT role, content, quote, sources, model, is_error, elapsed_ms, created_at, attachments
             FROM messages WHERE chat_id = ?1 ORDER BY id",
        )
        .map_err(|e| e.to_string())?;

    let rows = statement
        .query_map([chat_id], |row| {
            // Sources and attachments are JSON text. Unreadable text counts as none.
            let sources_json: Option<String> = row.get(3)?;
            let sources = sources_json
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();
            let attachments_json: Option<String> = row.get(8)?;
            let attachments = attachments_json
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();

            Ok(StoredMessage {
                role: row.get(0)?,
                content: row.get(1)?,
                selected: row.get(2)?,
                sources,
                model: row.get(4)?,
                is_error: row.get(5)?,
                elapsed_ms: row.get(6)?,
                attachments,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Renames a chat and returns the saved name (one line, at most `MAX_NAME_CHARS`).
pub fn rename_chat(conn: &Connection, chat_id: i64, title: &str, now_ms: i64) -> Result<String, String> {
    let name = title.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return Err("the name is empty".to_string());
    }
    let name: String = name.chars().take(MAX_NAME_CHARS).collect();

    let changed = conn
        .execute(
            "UPDATE chats SET title = ?2, updated_at = ?3 WHERE id = ?1",
            params![chat_id, name, now_ms],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("that chat does not exist".to_string());
    }
    Ok(name)
}

/// Deletes a chat; its messages go with it (`ON DELETE CASCADE`).
pub fn delete_chat(conn: &Connection, chat_id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM chats WHERE id = ?1", [chat_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Stores the image paths of a message. The message is saved before its screenshot
/// exists, so this updates the row afterwards.
pub fn set_attachments(conn: &Connection, message_id: i64, paths: &[String]) -> Result<(), String> {
    let json = serde_json::to_string(paths).map_err(|e| e.to_string())?;
    let changed = conn
        .execute(
            "UPDATE messages SET attachments = ?2 WHERE id = ?1",
            params![message_id, json],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("that message does not exist".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn message(chat_id: i64, role: &str, content: &str) -> NewMessage {
        NewMessage {
            chat_id,
            role: role.to_string(),
            content: content.to_string(),
            selected: None,
            sources: vec![],
            model: None,
            is_error: false,
            elapsed_ms: None,
        }
    }

    #[test]
    fn check_title_is_the_qhestion_no_cmd() {
        assert_eq!(make_title("/web what is rust"), "what is rust");
        assert_eq!(make_title("  hello   there\nfriend "), "hello there friend");
    }

    #[test]
    fn check_only_cmd_as_the_title() {
        assert_eq!(make_title("/ss"), "/ss");
        assert_eq!(make_title(" "), "New chat");
    }

    #[test]
    fn check_long_title_are_cut_or_not() {
        let title = make_title(&"words".repeat(30));
        assert!(title.ends_with('…'));
        assert!(title.chars().count() <= MAX_TITLE_CHARS + 1);
    }

    #[test]
    fn check_is_chat_gets_a_title_and_an_id() {
        let conn = open_in_memory().unwrap();
        let id = create_chat(&conn, "/web what is rust?", 100).unwrap();
        let (title, created): (String, i64) = conn
            .query_row(
                "SELECT title, created_at FROM chats WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(title, "what is rust?");
        assert_eq!(created, 100);
    }

    #[test]
    fn check_messages_stores_in_chat() {
        let conn = open_in_memory().unwrap();
        let chat = create_chat(&conn, "hi", 100).unwrap();

        let mut answer = message(chat, "assistant", "Rust is a language [1].");
        answer.sources = vec![("Rust".into(), "https://rust-lang.org".into())];
        answer.model = Some("gemma".into());
        answer.is_error = true;
        answer.elapsed_ms = Some(1234);
        add_message(&conn, &answer, 500).unwrap();
        let (sources, model, is_error, elapsed): (String, String, bool, i64) = conn
            .query_row(
                "SELECT sources, model, is_error, elapsed_ms FROM messages",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(sources, r#"[["Rust","https://rust-lang.org"]]"#);
        assert_eq!(model, "gemma");
        assert!(is_error);
        assert_eq!(elapsed, 1234);

        let updated: i64 = conn
            .query_row("SELECT updated_at FROM chats WHERE id = ?1", [chat], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(updated, 500);
    }

    #[test]
    fn check_no_sources_means_nothing_is_created() {
        let conn = open_in_memory().unwrap();
        let chat = create_chat(&conn, "hi", 1).unwrap();
        add_message(&conn, &message(chat, "user", "hi"), 2).unwrap();

        let sources: Option<String> = conn
            .query_row("SELECT sources FROM messages", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sources, None);
    }

    #[test]
    fn check_a_message_for_a_missing_chat_is_refused() {
        let conn = open_in_memory().unwrap();
        assert!(add_message(&conn, &message(999, "user", "x"), 2).is_err());
    }

    #[test]
    fn check_unknown_role_is_refused() {
        let conn = open_in_memory().unwrap();
        let chat = create_chat(&conn, "hi", 1).unwrap();
        assert!(add_message(&conn, &message(chat, "system", "x"), 2).is_err());
    }

    fn chat_with_message(conn: &Connection, first: &str, at: i64) -> i64 {
        let id = create_chat(conn, first, at).unwrap();
        add_message(conn, &message(id, "user", first), at).unwrap();
        id
    }

    fn titles(chats: &[ChatSummary]) -> Vec<&str> {
        chats.iter().map(|chat| chat.title.as_str()).collect()
    }

    #[test]
    fn strip_command_drops_only_a_leading_command() {
        assert_eq!(strip_command("/web what is rust?"), "what is rust?");
        assert_eq!(strip_command("  /ss   read this  "), "read this");
        assert_eq!(strip_command("/ss"), "/ss");
        assert_eq!(strip_command("no command here"), "no command here");
    }

    #[test]
    fn list_chats_is_newest_first_and_filters_by_title() {
        let conn = open_in_memory().unwrap();
        chat_with_message(&conn, "what is rust?", 100);
        chat_with_message(&conn, "best pizza dough", 300);
        chat_with_message(&conn, "rust ownership", 200);

        let all = list_chats(&conn, "").unwrap();
        assert_eq!(titles(&all), ["best pizza dough", "rust ownership", "what is rust?"]);

        let rust = list_chats(&conn, "  RUST ").unwrap();
        assert_eq!(titles(&rust), ["rust ownership", "what is rust?"]);
    }

    #[test]
    fn list_chats_has_no_limit() {
        let conn = open_in_memory().unwrap();
        for i in 0..120 {
            chat_with_message(&conn, &format!("chat {i}"), i);
        }
        assert_eq!(list_chats(&conn, "").unwrap().len(), 120);
    }

    #[test]
    fn list_chats_treats_percent_as_plain_text() {
        let conn = open_in_memory().unwrap();
        chat_with_message(&conn, "100% sure", 1);
        chat_with_message(&conn, "other", 2);
        assert_eq!(list_chats(&conn, "%").unwrap().len(), 1);
    }

    #[test]
    fn load_messages_returns_one_chat_in_order_with_sources_and_attachments() {
        let conn = open_in_memory().unwrap();
        let chat = create_chat(&conn, "hi", 1).unwrap();
        let question = add_message(&conn, &message(chat, "user", "hi"), 2).unwrap();
        set_attachments(&conn, question, &["/data/a.jpg".to_string()]).unwrap();
        let mut answer = message(chat, "assistant", "hello [1]");
        answer.sources = vec![("Rust".into(), "https://rust-lang.org".into())];
        answer.model = Some("gemma".into());
        add_message(&conn, &answer, 3).unwrap();
        let other = create_chat(&conn, "other", 4).unwrap();
        add_message(&conn, &message(other, "user", "x"), 5).unwrap();

        let messages = load_messages(&conn, chat).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].attachments, ["/data/a.jpg"]);
        assert_eq!(messages[1].content, "hello [1]");
        assert_eq!(messages[1].sources.len(), 1);
        assert_eq!(messages[1].model.as_deref(), Some("gemma"));
        assert!(messages[1].attachments.is_empty());
    }

    #[test]
    fn rename_chat_replaces_the_temporary_title() {
        let conn = open_in_memory().unwrap();
        let chat = create_chat(&conn, "/web what is rust?", 1).unwrap();
        assert_eq!(list_chats(&conn, "").unwrap()[0].title, "what is rust?");

        rename_chat(&conn, chat, "Rust Language Basics", 2).unwrap();
        assert_eq!(list_chats(&conn, "").unwrap()[0].title, "Rust Language Basics");
    }

    #[test]
    fn rename_chat_normalizes_the_name_and_bumps_updated_at() {
        let conn = open_in_memory().unwrap();
        let chat = chat_with_message(&conn, "old", 10);
        let saved = rename_chat(&conn, chat, "  My   Rust\nnotes ", 99).unwrap();
        assert_eq!(saved, "My Rust notes");

        let chats = list_chats(&conn, "").unwrap();
        assert_eq!(chats[0].title, "My Rust notes");
        assert_eq!(chats[0].updated_at, 99);
    }

    #[test]
    fn rename_chat_cuts_very_long_names() {
        let conn = open_in_memory().unwrap();
        let chat = chat_with_message(&conn, "old", 1);
        let saved = rename_chat(&conn, chat, &"a".repeat(500), 2).unwrap();
        assert_eq!(saved.chars().count(), MAX_NAME_CHARS);
    }

    #[test]
    fn rename_chat_rejects_an_empty_name_and_an_unknown_chat() {
        let conn = open_in_memory().unwrap();
        let chat = chat_with_message(&conn, "old", 10);
        assert!(rename_chat(&conn, chat, "   ", 1).is_err());
        assert!(rename_chat(&conn, 999, "name", 1).is_err());
    }

    #[test]
    fn delete_chat_removes_its_messages_and_keeps_the_others() {
        let conn = open_in_memory().unwrap();
        let keep = chat_with_message(&conn, "keep", 1);
        let gone = chat_with_message(&conn, "gone", 2);
        delete_chat(&conn, gone).unwrap();

        assert_eq!(list_chats(&conn, "").unwrap().len(), 1);
        assert!(load_messages(&conn, gone).unwrap().is_empty());
        assert_eq!(load_messages(&conn, keep).unwrap().len(), 1);
    }

    #[test]
    fn set_attachments_stores_the_paths_as_json() {
        let conn = open_in_memory().unwrap();
        let chat = create_chat(&conn, "/screen what is this?", 1).unwrap();
        let message_id = add_message(&conn, &message(chat, "user", "/screen what is this?"), 2).unwrap();

        set_attachments(&conn, message_id, &["/data/a.jpg".to_string()]).unwrap();

        let stored: String = conn
            .query_row("SELECT attachments FROM messages WHERE id = ?1", [message_id], |row| row.get(0))
            .unwrap();
        assert_eq!(stored, r#"["/data/a.jpg"]"#);
    }

    #[test]
    fn set_attachments_rejects_an_unknown_message() {
        let conn = open_in_memory().unwrap();
        assert!(set_attachments(&conn, 999, &["/data/a.jpg".to_string()]).is_err());
    }
}
