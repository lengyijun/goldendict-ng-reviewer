use anyhow::Result;
use goldendict_ng_helper::fsrs::{get_card, get_word_ignore_case, sqlite_history::SQLiteHistory};
use rs_fsrs::Card;
use std::env::args;

#[tokio::main]
async fn main() -> Result<()> {
    let word = args().nth(1).unwrap_or("--help".to_owned());
    if word == "--help" {
        println!("used in goldendict-ng, program dic");
        println!("add all arguments to sqlite");
        println!("https://github.com/lengyijun/goldendict-ng-reviewer");
        return Ok(());
    }

    let mut sqlite_history = SQLiteHistory::default().await;
    for word in args().skip(1) {
        match get_word_ignore_case(&sqlite_history.conn, &word).await {
            Ok(word) => {
                let card = get_card(&sqlite_history.conn, &word).await.unwrap();
                sqlite_history.insert_or_replace(&word, card).await?;
            }
            Err(_) => {
                sqlite_history.insert_or_replace(&word, Card::new()).await?;
            }
        }
    }
    Ok(())
}
