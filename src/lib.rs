#[cfg(feature = "earley")]
pub mod earley;

#[cfg(feature = "lr")]
pub mod lr;

pub struct Grammar {}

trait Parser {
    /// Grammar preprocessing.
    fn fit(grammar: &Grammar);
 
    /// Check if the word is in the language.
    fn predict(word: &str) -> bool;
}
