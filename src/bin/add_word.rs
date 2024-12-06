use anyhow::Result;
use mdict_cli_rs::fsrs::sqlite_history::add_history;
use std::env::args;

#[tokio::main]
async fn main() -> Result<()> {
    let w = args().nth(1).unwrap_or("--help".to_owned());
    if w == "--help" {
        println!("used in goldendict-ng, program dic");
        println!("add first argument to sqlite");
        println!("https://github.com/lengyijun/mdict-cli-rs/tree/cursive");
        return Ok(());
    }
    add_history(&w).await?;
    Ok(())
}
