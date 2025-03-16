use anyhow::Result;
use rs_fsrs::Card;
use sqlx::Row;
use sqlx::SqlitePool;

pub mod sqlite_history;

pub async fn get_word(pool: &SqlitePool, word: &str) -> Result<Card> {
    let sqlite_row = sqlx::query("SELECT due, stability, difficulty, elapsed_days, scheduled_days, reps, lapses, state, last_review
    FROM fsrs WHERE word = $1")
        .bind(word)
        .fetch_one(pool)
        .await?;

    let card: Card = Card {
        due: serde_json::from_str(sqlite_row.get(0))?,
        stability: sqlite_row.get(1),
        difficulty: sqlite_row.get(2),
        elapsed_days: sqlite_row.get(3),
        scheduled_days: sqlite_row.get(4),
        reps: sqlite_row.get(5),
        lapses: sqlite_row.get(6),
        state: serde_json::from_str(sqlite_row.get(7))?,
        last_review: serde_json::from_str(sqlite_row.get(8))?,
    };
    Ok(card)
}
