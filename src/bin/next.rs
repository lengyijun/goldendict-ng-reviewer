use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use goldendict_ng_helper::fsrs::sqlite_history::SQLiteHistory;
use shadow_rs::shadow;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::collections::HashSet;
use std::env;
use walkdir::WalkDir;

shadow!(build);

#[derive(Parser, Debug)]
struct Args {
    suffix: Vec<String>,

    #[arg(long, default_value_t = false)]
    random: bool,

    #[arg(long, default_value_t = false)]
    help: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    if args.help {
        println!("used with goldendict-ng");
        println!("https://github.com/lengyijun/goldendict-ng-helper");
        println!("{}", build::VERSION); //print version const
        return Ok(());
    }

    let mut deck = SQLiteHistory::default().await;
    let w = if args.random {
        match deck.next_to_review().await {
            Ok(s) => s,
            _ => {
                eprintln!("all reviewed");
                return Err(anyhow!("not found"));
            }
        }
    } else {
        match foo(&mut deck.conn).await {
            Some(x) => x,
            None => {
                eprintln!("all reviewed");
                return Err(anyhow!("not found"));
            }
        }
    };

    let w = w.to_lowercase();
    let w = if let Some(suffix) = args.suffix.first() {
        format!("{w}.{suffix}")
    } else {
        w
    };
    println!("{w}");
    eprintln!("{w}");

    Ok(())
}

fn prefixes_in_dir() -> HashSet<char> {
    let mut res = HashSet::new();
    let walk_dir = WalkDir::new(env::current_dir().unwrap()).max_depth(1);
    for x in walk_dir {
        let Ok(x) = x else { continue };
        let Some(x) = x.file_name().to_str() else {
            continue;
        };
        let Some(c) = x.chars().next() else { continue };
        let Some(c) = c.to_lowercase().next() else {
            continue;
        };
        res.insert(c);
    }
    res
}

async fn foo(conn: &SqlitePool) -> Option<String> {
    let existed_prefixes = prefixes_in_dir();

    sqlx::query("SELECT word FROM fsrs WHERE word REGEXP '^[A-Za-z]+$' ORDER BY RANDOM();")
        .fetch_all(conn)
        .await
        .ok()?
        .into_iter()
        .map(|sqlite_row| {
            let x: String = sqlite_row.get(0);
            x
        })
        .filter(|word| !existed_prefixes.contains(&word.chars().next().unwrap()))
        .next()
}
