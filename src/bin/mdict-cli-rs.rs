#![feature(async_closure)]

use anyhow::Result;
use chrono::prelude::*;
use env_logger::Target;
use mdict_cli_rs::db_path;
use mdict_cli_rs::dictionary_dir;
use mdict_cli_rs::fsrs::sqlite_history::add_history;
use mdict_cli_rs::load_dict;
use mdict_cli_rs::log_dir;
use mdict_cli_rs::query;
use shadow_rs::shadow;
use std::fs::OpenOptions;
use std::{
    env::{self},
    process::Command,
};

shadow!(build);

#[tokio::main]
async fn main() -> Result<()> {
    let word = env::args().nth(1).unwrap();
    match &*word {
        "--help" => {
            println!("https://github.com/lengyijun/mdict-cli-rs/");
            Ok(())
        }
        "--version" => {
            println!("{}", build::VERSION); //print version const
            Ok(())
        }
        "--list-dict" | "--list-dicts" => {
            let v = load_dict();
            if v.is_empty() {
                println!("no dictionary found in {:?}", dictionary_dir());
                return Ok(());
            }
            for dict in v {
                println!("{:?}", dict.path());
            }
            Ok(())
        }
        "--show-path" => {
            println!("dictionary dir            {:?}", dictionary_dir());
            println!("history database          {:?}", db_path());
            println!("log dir                   {:?}", log_dir());
            Ok(())
        }
        "anki" => {
            let local: DateTime<Local> = Local::now();
            let log_path = log_dir().join(format!("log.{}", local.to_rfc3339()));

            let log_file = Box::new(
                OpenOptions::new()
                    .read(true)
                    .create(true)
                    .append(true)
                    .open(&log_path)?,
            );
            println!("log file: {:?}", log_path);

            env_logger::Builder::from_default_env()
                .target(Target::Pipe(log_file))
                .filter_level(log::LevelFilter::Info) // Set the minimum log level
                .init();
            mdict_cli_rs::anki::anki().await?;
            Ok(())
        }
        _ => {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Info) // Set the minimum log level
                .init();

            let temp_dir = tempfile::Builder::new().prefix(&word).tempdir()?;
            let index_html = query(&word, temp_dir.path())?;
            add_history(&word).await?;
            let _ = Command::new("carbonyl").arg(index_html).status()?;
            Ok(())
        }
    }
}
