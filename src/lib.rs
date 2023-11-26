#![allow(dead_code)]

use multimap::MultiMap;
use std::collections::HashSet;

#[cfg(feature = "earley")]
pub mod earley;

#[cfg(feature = "lr")]
pub mod lr;

pub type CFRule = (char, String);

#[derive(Debug, Default, Clone)]
pub struct CFGrammar {
    terminals: HashSet<char>,
    non_terminals: HashSet<char>,
    rules: MultiMap<char, String>,
    start: char,
}

impl CFGrammar {
    fn new(
        terminals: &[char],
        non_terminals: &[char],
        rules: &MultiMap<char, String>,
        start: char,
    ) -> Self {
        Self {
            terminals: HashSet::from_iter(terminals.iter().copied()),
            non_terminals: HashSet::from_iter(non_terminals.iter().copied()),
            rules: rules.clone(),
            start,
        }
    }

    fn start_rule(&self) -> CFRule {
        let start_rules = self.rules.get_vec(&self.start).unwrap();
        assert!(start_rules.len() == 1);
        (self.start, start_rules[0].clone())
    }
}

trait Parser {
    /// Grammar preprocessing.
    fn fit(&mut self, grammar: &CFGrammar);

    /// Check if the word is in the language.
    fn predict(&mut self, word: &str) -> bool;
}
