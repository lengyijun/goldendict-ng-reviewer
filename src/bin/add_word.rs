use anyhow::Result;
use goldendict_ng_helper::fsrs::{get_word, sqlite_history::SQLiteHistory};
use rs_fsrs::Card;
use std::env::args;

#[tokio::main]
async fn main() -> Result<()> {
    let w = args().nth(1).unwrap_or("--help".to_owned());
    if w == "--help" {
        println!("used in goldendict-ng, program dic");
        println!("add all arguments to sqlite");
        println!("https://github.com/lengyijun/goldendict-ng-helper");
        return Ok(());
    }
    let mut sqlite_history = SQLiteHistory::default().await;
    for w in args().skip(1) {
        let card = get_word(&sqlite_history.conn, &w)
            .await
            .unwrap_or(Card::new());
        sqlite_history.insert_or_replace(&w, card).await?;
    }
    Ok(())
}
