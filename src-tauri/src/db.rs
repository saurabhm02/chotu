use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use rusqlite::Connection;
use tauri::{App, Manager};

/// The open database, shared by every command.
/// The `Mutex` lets one command use it at a time, so two commands can never write together.

pub struct Db(pub Mutex<Connection>);

const MIGRATIONS: &[&str] = &["
    CREATE TABLE chats(
        id              INTEGER PRIMARY KEY,
        title           TEXT    NOT NULL,
        created_at      INTEGER NOT NULL,
        updated_at      INTEGER NOT NULL
    );

    CREATE TABLE messages(
        id               INTEGER PRIMARY KEY,
        chat_id          INTEGER NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
        role             TEXT NOT NULL,
        content          TEXT NOT NULL, 
        quote            TEXT,
        sources           TEXT,
        model            TEXT,
        is_error         INTEGER NOT NULL DEFAULT 0,
        attachments      TEXT,
        elapsed_ms       INTEGER,                    
        created_at       INTEGER NOT NULL
    );
    CREATE INDEX messages_by_chat ON messages (chat_id, id); 
    CREATE INDEX chats_by_update ON chats (updated_at DESC);"];

pub fn migrate(conn: &mut Connection) -> Result<(), String> {
    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let user_version = user_version as usize;

    if user_version > MIGRATIONS.len() {
        return Err(format!(
            "this database is from a newer TY (layout {user_version}, this TY knows {})",
            MIGRATIONS.len()
        ));
    }

    for (index, sql) in MIGRATIONS.iter().enumerate().skip(user_version) {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute_batch(sql).map_err(|e| e.to_string())?;
        tx.pragma_update(None, "user_version", (index + 1) as i64)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        log::info!("database: ran step {}", index + 1);
    }
    Ok(())
}

fn prepare(mut conn: Connection) -> Result<Connection, String> {
    conn.pragma_update(None, "foreign_keys", true)
        .map_err(|e| e.to_string())?;
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|e| e.to_string())?;

    migrate(&mut conn)?;
    Ok(conn)
}

pub fn open(path: &Path) -> Result<Connection, String> {
    prepare(Connection::open(path).map_err(|e| e.to_string())?)
}

pub fn open_in_memory() -> Result<Connection, String> {
    prepare(Connection::open_in_memory().map_err(|e| e.to_string())?)
}

pub fn init(app: &App) {
    match open_app_database(app) {
        Ok(conn) => {
            app.manage(Db(Mutex::new(conn)));
        }
        Err(e) => log::error!("history is turned off, the database did not open: {e}"),
    }
}

fn open_app_database(app: &App) -> Result<Connection, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join("ty.db");
    let conn = open(&path)?;
    log::info!("database ready: {}", path.display());
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |row| row.get::<_, i64>(0),
        )
        .unwrap()
            == 1
    }

    fn version(conn: &Connection) -> usize {
        let number: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        number as usize
    }

    #[test]
    fn a_new_database_gets_all_tables() {
        let conn = open_in_memory().unwrap();
        assert!(table_exists(&conn, "chats"));
        assert!(table_exists(&conn, "messages"));
        assert_eq!(version(&conn), MIGRATIONS.len());
    }

    #[test]
    fn running_the_migrations_twice_changes_nothing() {
        let mut conn = open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        migrate(&mut conn).unwrap();
        assert_eq!(version(&conn), MIGRATIONS.len());
    }

    #[test]
    fn a_database_from_a_newer_app_is_refused() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        assert!(migrate(&mut conn).unwrap_err().contains("newer"));
    }

    #[test]
    fn deleting_a_chat_deletes_its_messages() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO chats (id, title, created_at, updated_at) VALUES (1, 'Rust', 0, 0)",
            [],
        )
        .unwrap();
        for text in ["hi", "hello"] {
            conn.execute(
                "INSERT INTO messages (chat_id, role, content, created_at) VALUES (1, 'user', ?1, 0)",
                [text],
            )
            .unwrap();
        }

        conn.execute("DELETE FROM chats WHERE id = 1", []).unwrap();

        let left: i64 = conn
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .unwrap();
        assert_eq!(left, 0);
    }

    #[test]
    fn a_message_needs_a_real_chat() {
        let conn = open_in_memory().unwrap();
        let result = conn.execute(
            "INSERT INTO messages (chat_id, role, content, created_at) VALUES (42, 'user', 'x', 0)",
            [],
        );
        assert!(result.is_err());
    }

    #[test]
    fn data_survives_closing_and_reopening_the_file() {
        let path = std::env::temp_dir().join(format!("ty-test-{}.db", uuid::Uuid::new_v4()));

        {
            let conn = open(&path).unwrap();
            conn.execute(
                "INSERT INTO chats (title, created_at, updated_at) VALUES ('Saved', 1, 1)",
                [],
            )
            .unwrap();
        } // the connection closes here

        let conn = open(&path).unwrap();
        let title: String = conn
            .query_row("SELECT title FROM chats", [], |row| row.get(0))
            .unwrap();
        assert_eq!(title, "Saved");
        assert_eq!(version(&conn), MIGRATIONS.len());

        drop(conn);
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }
}
