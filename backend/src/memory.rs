use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Memory {
    pub id: i64,
    pub content: String,
    pub created_at: String,
}

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    info!("Database initialized");
    Ok(())
}

pub async fn add_memory(pool: &SqlitePool, content: &str) -> Result<i64, sqlx::Error> {
    let created_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let id = sqlx::query("INSERT INTO memories (content, created_at) VALUES (?, ?)")
        .bind(content)
        .bind(created_at)
        .execute(pool)
        .await?
        .last_insert_rowid();

    info!("Memory added with ID: {}", id);
    Ok(id)
}

pub async fn get_recent_memories(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<Memory>, sqlx::Error> {
    let memories = sqlx::query_as::<_, Memory>(
        "SELECT id, content, created_at FROM memories ORDER BY id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(memories)
}

pub async fn delete_memory(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM memories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    info!("Memory with ID {} deleted", id);
    Ok(())
}
