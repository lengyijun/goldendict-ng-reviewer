use anyhow::Result;
use rs_fsrs::Rating;

pub trait SpacedRepetition: Sized {
    /// find next reviewable word
    async fn next_to_review(&mut self) -> Result<String>;

    async fn update(&mut self, question: &str, rating: Rating) -> Result<()>;

    async fn delete(&self, question: &str) -> Result<()>;
}
