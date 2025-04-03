use anyhow::Result;
use std::cmp::Ordering;
use std::env;
use std::sync::LazyLock;
use word2vec::wordvectors::WordVector;

use crate::fsrs::sqlite_history::SQLiteHistory;

const THREHOLD: f32 = 0.5;

static MODEL: LazyLock<WordVector> = LazyLock::new(|| {
    let bin_path = env::var("BIN_PATH").expect("Please set BIN_PATH");
    let model = word2vec::wordvectors::WordVector::load_from_binary(&bin_path)
        .expect("Unable to load word vector model");
    model
});

impl SQLiteHistory {
    async fn rank_similar_words(&self, word: &str) -> Result<Vec<(f32, String)>> {
        let word = MODEL.get_vector(word).unwrap();

        let mut v = Vec::new();

        for x in self.all_words_need_review().await? {
            let Some(arr1) = MODEL.get_vector(&x) else {
                continue;
            };
            let product = word2vec::utils::dot_product(arr1, word);
            if product > THREHOLD {
                v.push((product, x));
            }
        }
        v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
        Ok(v)
    }

    pub async fn extend_by_word2vec(&mut self, word: &str) -> Result<()> {
        let v = self.rank_similar_words(word).await?;
        self.queue.extend(v.into_iter().map(|(_, w)| w));
        Ok(())
    }
}
