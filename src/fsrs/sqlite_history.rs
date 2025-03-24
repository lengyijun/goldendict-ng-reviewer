//! <https://github.com/kkawakam/rustyline/blob/master/src/sqlite_history.rs>
//! History impl. based on SQLite

use crate::db_path;
use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use rs_fsrs::Card;
use rs_fsrs::Parameters;
use rs_fsrs::Rating;
use rs_fsrs::FSRS;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
use sqlx::Sqlite;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::get_card;

/// History stored in an SQLite database.
#[derive(Clone)]
pub struct SQLiteHistory {
    ignore_dups: bool,
    pub conn: SqlitePool, /* we need to keep a connection opened at least for in memory
                           * database and also for cached statement(s) */
    /// used in anki mode: avoid re-review
    /// 0 means no new entry added
    pub session_id: i32,
    /// used in review
    /// search next word to review from `row_id`
    pub row_id: i32,
    pub fsrs: FSRS,
    pub history: Vec<String>,
    pub queue: VecDeque<String>,
}

/*
https://sqlite.org/autoinc.html
If no ROWID is specified on the insert, or if the specified ROWID has a value of NULL, then an appropriate ROWID is created automatically.
The usual algorithm is to give the newly created row a ROWID that is one larger than the largest ROWID in the table prior to the insert.
If the table is initially empty, then a ROWID of 1 is used.
If the largest ROWID is equal to the largest possible integer (9223372036854775807) then the database engine starts picking positive candidate ROWIDs
at random until it finds one that is not previously used.
https://sqlite.org/lang_vacuum.html
The VACUUM command may change the ROWIDs of entries in any tables that do not have an explicit INTEGER PRIMARY KEY.
 */

impl SQLiteHistory {
    pub async fn default() -> Self {
        Self::new(db_path()).await.unwrap()
    }

    async fn new(path: PathBuf) -> Result<Self> {
        if !Sqlite::database_exists(path.to_str().unwrap()).await? {
            Sqlite::create_database(path.to_str().unwrap()).await?;
        }
        let conn = conn(&path).await?;
        let mut sh = Self {
            // not strictly consecutive...
            ignore_dups: true,
            conn,
            session_id: 0,
            row_id: -1,
            fsrs: FSRS::new(Parameters::default()),
            history: Vec::new(),
            queue: VecDeque::new(),
        };
        sh.check_schema().await?;
        sh.create_session().await?;
        Ok(sh)
    }

    async fn check_schema(&mut self) -> Result<()> {
        let user_version = &sqlx::query("pragma user_version;")
            .fetch_all(&self.conn)
            .await?[0];
        let user_version: i32 = user_version.get(0);

        if user_version <= 0 {
            sqlx::raw_sql(
                "
BEGIN EXCLUSIVE;
PRAGMA auto_vacuum = INCREMENTAL;
CREATE TABLE session (
    id INTEGER PRIMARY KEY NOT NULL,
    timestamp REAL NOT NULL DEFAULT (julianday('now'))
) STRICT; -- user, host, pid
CREATE TABLE fsrs (
    --entry TEXT NOT NULL,
    word TEXT PRIMARY KEY,
    due TEXT NOT NULL,
    stability REAL NOT NULL,
    difficulty REAL NOT NULL,
    elapsed_days INTEGER NOT NULL,
    scheduled_days INTEGER NOT NULL,
    reps INTEGER NOT NULL,
    lapses INTEGER NOT NULL,
    state TEXT NOT NULL,
    last_review TEXT NOT NULL,
    session_id INTEGER NOT NULL,
    -- card TEXT NOT NULL,
    -- timestamp REAL NOT NULL DEFAULT (julianday('now')),
    FOREIGN KEY (session_id) REFERENCES session(id) ON DELETE CASCADE
) STRICT;
CREATE VIRTUAL TABLE fts USING fts4(content=fsrs, word);
CREATE TRIGGER history_bu BEFORE UPDATE ON fsrs BEGIN
    DELETE FROM fts WHERE docid=old.rowid;
END;
CREATE TRIGGER history_bd BEFORE DELETE ON fsrs BEGIN
    DELETE FROM fts WHERE docid=old.rowid;
END;
CREATE TRIGGER history_au AFTER UPDATE ON fsrs BEGIN
    INSERT INTO fts (docid, word) VALUES (new.rowid, new.word);
END;
CREATE TRIGGER history_ai AFTER INSERT ON fsrs BEGIN
    INSERT INTO fts (docid, word) VALUES(new.rowid, new.word);
END;
PRAGMA user_version = 1;
COMMIT;
                 ",
            )
            .execute(&self.conn)
            .await?;
        }
        sqlx::query("pragma foreign_keys = 1;")
            .execute(&self.conn)
            .await?;
        if self.ignore_dups || user_version > 0 {
            self.set_ignore_dups().await?;
        }
        Ok(())
    }

    async fn set_ignore_dups(&mut self) -> Result<()> {
        if self.ignore_dups {
            // TODO Validate: ignore dups only in the same session_id ?
            sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS ignore_dups ON fsrs(word, session_id);")
                .execute(&self.conn)
                .await?;
            Ok(())
        } else {
            sqlx::query("DROP INDEX IF EXISTS ignore_dups;")
                .execute(&self.conn)
                .await?;
            Ok(())
        }
    }

    async fn create_session(&mut self) -> Result<()> {
        if self.session_id == 0 {
            self.check_schema().await?;
            self.session_id = sqlx::query("INSERT INTO session (id) VALUES (NULL) RETURNING id;")
                .fetch_one(&self.conn)
                .await?
                .get::<i32, _>(0);
        }
        Ok(())
    }

    pub async fn insert_or_replace(&mut self, word: &str, card: Card) -> Result<()> {
        // ignore SQLITE_CONSTRAINT_UNIQUE

        let _sqlite_query_result = sqlx::query("INSERT OR REPLACE INTO fsrs (session_id, word, due, stability, difficulty, elapsed_days, scheduled_days, reps, lapses, state, last_review) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING rowid;")
        .bind(self.session_id)
        .bind(word)
        .bind(serde_json::to_string(&card.due)?)
        .bind(card.stability)
        .bind(card.difficulty)
        .bind(card.elapsed_days)
        .bind(card.scheduled_days)
        .bind(card.reps)
        .bind(card.lapses)
        .bind(serde_json::to_string(&card.state)?)
        .bind(serde_json::to_string(&card.last_review)?)
        .execute(&self.conn).await?;

        Ok(())
    }

    async fn leven(&mut self, word: &str) -> Result<()> {
        let similar_words =
            sqlx::query("SELECT word FROM fsrs where word != $1 AND session_id != $2;")
                .bind(word)
                .bind(&self.session_id)
                .fetch_all(&self.conn)
                .await?
                .into_iter()
                .map(|sqlite_row| sqlite_row.get(0))
                .filter(|a: &String| strsim::levenshtein(a, word) <= 2);

        self.queue.extend(similar_words);
        Ok(())
    }

    async fn next_to_review_db(&mut self) -> Result<String> {
        let word: String = match sqlx::query("SELECT rowid, word FROM fsrs WHERE timediff('now', substr(due, 2, length(due) - 2)) LIKE '+%' AND session_id < $1 AND rowid > $2 ORDER BY RANDOM() LIMIT 1;")
                .bind(self.session_id)
                .bind(self.row_id)
                .fetch_one(&self.conn)
                .await {
                    Ok(row) => {
                        self.row_id = row.get(0);
                        row.get(1)
                    }
                    Err(_) => {
                        // search from start
                        let row = sqlx::query("SELECT rowid, word FROM fsrs WHERE timediff('now', substr(due, 2, length(due) - 2)) LIKE '+%' AND session_id < $1 ORDER BY RANDOM() LIMIT 1;")
                            .bind(self.session_id)
                            .fetch_one(&self.conn)
                            .await?;
                        self.row_id = row.get(0);
                        row.get(1)
                    }
                };
        sqlx::query("UPDATE fsrs SET session_id = $2 WHERE word = $1")
            .bind(&word)
            .bind(self.session_id)
            .execute(&self.conn)
            .await?;
        Ok(word)
    }

    async fn next_to_review_inner(&mut self) -> Result<String> {
        while let Some(word) = self.queue.pop_front() {
            if !self.history.contains(&word) {
                return Ok(word);
            }
        }
        self.next_to_review_db().await
    }

    pub async fn next_to_review(&mut self) -> Result<String> {
        let word = self.next_to_review_inner().await?;
        self.history.push(word.clone());
        let _ = self.leven(&word).await;
        Ok(word)
    }

    pub async fn update(&mut self, question: &str, rating: Rating) -> Result<()> {
        let old_card = get_card(&self.conn, question)
            .await
            .context("get old card fail")?;
        let scheduling_info = self.fsrs.next(old_card, Utc::now(), rating);
        self.insert_or_replace(question, scheduling_info.card).await
    }

    pub async fn delete(&self, question: &str) -> Result<()> {
        sqlx::query("DELETE FROM fsrs WHERE word = $1")
            .bind(question)
            .execute(&self.conn)
            .await?;
        Ok(())
    }

    // return Ok(()): should review
    // return Err(_): not exists in history or should not review
    pub async fn should_review(&self, question: &str) -> Result<()> {
        let _row = sqlx::query("SELECT word FROM fsrs WHERE word = $1 AND timediff('now', substr(due, 2, length(due) - 2)) LIKE '+%' LIMIT 1;")
                            .bind(question)
                            .fetch_one(&self.conn)
                            .await?;
        Ok(())
    }

    pub async fn phrase(&self) -> Result<Vec<String>> {
        let phrases: Vec<String> = sqlx::query("SELECT word FROM fsrs WHERE timediff('now', substr(due, 2, length(due) - 2)) LIKE '+%' AND word LIKE '% %' ORDER BY RANDOM();")
                .fetch_all(&self.conn)
                .await?
                .into_iter()
                .map(|sqlite_row| sqlite_row.get(0))
                .collect();
        Ok(phrases)
    }
}

pub async fn conn(path: &Path) -> sqlx::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(path.to_str().unwrap())?.with_regexp();
    Ok(SqlitePoolOptions::new().connect_with(options).await?)
}
