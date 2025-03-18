use anyhow::Result;
use goldendict_ng_helper::fsrs::sqlite_history::SQLiteHistory;
use std::env::args;

#[tokio::main]
async fn main() -> Result<()> {
    let w = args().nth(1).unwrap_or("--help".to_owned());
    if w == "--help" {
        println!("used in goldendict-ng, program dic");
        println!("add all arguments to sqlite");
        println!("https://github.com/lengyijun/goldendict-ng-reviewer");
        return Ok(());
    }

    let sqlite_history = SQLiteHistory::default().await;
    for w in args().skip(1) {
        sqlite_history.delete(&w).await?;
    }
    Ok(())
}
