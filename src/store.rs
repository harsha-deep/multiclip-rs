use rusqlite::{params, Connection};
use std::path::PathBuf;

const MAX_ENTRIES: i64 = 200;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open() -> Self {
        let path = db_path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).expect("failed to create data dir");
        }

        let conn = Connection::open(path).expect("failed to open history db");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                content    TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .expect("failed to create history table");

        Store { conn }
    }

    /// Inserts new content, or bumps it to the top if it already exists.
    pub fn push(&self, content: &str) {
        let now = now_secs();
        self.conn
            .execute(
                "INSERT INTO history (content, created_at) VALUES (?1, ?2)
                 ON CONFLICT(content) DO UPDATE SET created_at = excluded.created_at",
                params![content, now],
            )
            .expect("failed to insert history entry");

        self.conn
            .execute(
                "DELETE FROM history WHERE id NOT IN (
                    SELECT id FROM history ORDER BY created_at DESC LIMIT ?1
                )",
                params![MAX_ENTRIES],
            )
            .expect("failed to trim history");
    }

    pub fn recent(&self) -> Vec<String> {
        let mut stmt = self
            .conn
            .prepare("SELECT content FROM history ORDER BY created_at DESC")
            .expect("failed to prepare query");

        stmt.query_map([], |row| row.get::<_, String>(0))
            .expect("failed to query history")
            .filter_map(Result::ok)
            .collect()
    }
}

fn db_path() -> PathBuf {
    dirs::data_dir()
        .expect("could not determine data dir")
        .join("multiclip-rs")
        .join("history.db")
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs() as i64
}
