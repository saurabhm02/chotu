use crate::models::chat::NewMessage;
use rusqlite::{params, Connection};

const MAX_TITLE_CHARS: usize = 40;

/// A short chat title made from the first message.
/// "/web what is rust?" becomes "what is rust?"; long text is cut at 40 characters.
pub fn make_title(first_message: &str) -> String {
    let text = first_message.trim();

    // Drop a leading "/command " when something follows it.
    let text = match text.strip_prefix('/') {
        Some(rest) => match rest.split_once(char::is_whitespace) {
            Some((_, after)) if !after.trim().is_empty() => after.trim(),
            _ => text,
        },
        None => text,
    };

    // New lines and runs of spaces become single spaces.
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if one_line.is_empty() {
        return "New chat".to_string();
    }
    if one_line.chars().count() > MAX_TITLE_CHARS {
        let cut: String = one_line.chars().take(MAX_TITLE_CHARS).collect();
        return format!("{}…", cut.trim_end());
    }
    one_line
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
            msg.quote,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn message(chat_id: i64, role: &str, content: &str) -> NewMessage {
        NewMessage {
            chat_id,
            role: role.to_string(),
            content: content.to_string(),
            quote: None,
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
}
