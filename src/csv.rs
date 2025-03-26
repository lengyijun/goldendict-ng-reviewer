use crate::dictionary_dir;
use crate::fsrs::sqlite_history::SQLiteHistory;
use anyhow::Result;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;

/// https://github.com/skywind3000/ECDICT/blob/master/ecdict.csv
fn ecdict_path() -> PathBuf {
    dictionary_dir().join("ecdict.csv")
}

#[derive(Debug, serde::Deserialize)]
pub struct Record {
    pub word: String,
    pub bnc: u32,
    pub frq: u32,
}

fn get_records() -> Result<Vec<Record>> {
    let file = File::open(ecdict_path())?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut v = Vec::new();
    v.reserve(770612);
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: Record = result?;
        v.push(record);
    }
    Ok(v)
}

impl SQLiteHistory {
    pub fn init_records(&mut self) -> Result<()> {
        self.records = get_records()?;
        Ok(())
    }

    pub fn binary_search(&self, word: &str) -> Option<&Record> {
        let word = word.to_lowercase();

        let i = self
            .records
            .binary_search_by(|record| record.word.to_lowercase().deref().cmp(&word))
            .ok()?;
        Some(&self.records[i])
    }

    pub fn qualify(&self, record: &Record) -> bool {
        (record.bnc != 0 && record.bnc <= self.freq) || (record.frq != 0 && record.frq <= self.freq)
    }
}
