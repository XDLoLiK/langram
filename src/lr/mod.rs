use super::{CFGrammar, CFRule, Parser};

#[derive(Debug, Default, Clone, Copy)]
pub struct LRParser {}

impl Parser for LRParser {
    fn fit(&mut self, _grammar: &CFGrammar) {
        todo!()
    }

    fn predict(&mut self, _word: &str) -> bool {
        todo!()
    }
}
