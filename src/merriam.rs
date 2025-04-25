use anyhow::Result;
use sqlx::Row;
use std::env;

use crate::fsrs::sqlite_history::{conn, SQLiteHistory};

impl SQLiteHistory {
    pub async fn extend_by_merriam(&mut self, word: &str) -> Result<()> {
        let mut p = env::current_exe().unwrap();
        p.pop();
        p.push("merriam.db");

        let merriam = conn(p.to_str().unwrap()).await?;
        let related_word: String =
            sqlx::query("SELECT related_words FROM merriam where word = $1;")
                .bind(word)
                .fetch_one(&merriam)
                .await?
                .get(0);
        self.queue
            .extend(related_word.split(',').map(ToOwned::to_owned));
        Ok(())
    }
}
