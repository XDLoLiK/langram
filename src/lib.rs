#![allow(dead_code)]

use multimap::MultiMap;

use std::collections::HashSet;
use std::str::FromStr;
use std::string::ParseError;

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

impl FromStr for CFGrammar {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<_> = s.lines().collect();
        assert!(
            lines.len() >= 4,
            "There must be at least four lines in the string (at least one for each field)."
        );
        let terminals: HashSet<_> = lines[0].chars().collect();
        let non_terminals: HashSet<_> = lines[1].chars().collect();
        let mut rules = MultiMap::new();

        for i in 2..(lines.len() - 1) {
            let parts: Vec<_> = lines[i].split("->").map(|s: &str| s.trim()).collect();
            assert!(
                parts.len() == 2,
                "There must be only one delimiter in the rule."
            );
            let key = parts[0];
            assert!(
                key.len() == 1,
                "There must be exactly one non-terminal in the left part of the CF grammar {key}."
            );
            let key = key.chars().next().unwrap();
            assert!(
                non_terminals.contains(&key),
                "Only non-terminals can be present in the left part of the CF grammar"
            );
            let value = parts[1];
            rules.insert(key, value.to_string());
        }

        let start = lines.last().unwrap();
        assert!(start.len() == 1, "There must be exactly one start rule.");
        let start = start.chars().next().unwrap();
        Ok(Self {
            terminals,
            non_terminals,
            rules,
            start,
        })
    }
}

impl CFGrammar {
    fn new(terminals: &[char], non_terminals: &[char], rules: &[CFRule], start: char) -> Self {
        Self {
            terminals: HashSet::from_iter(terminals.iter().copied()),
            non_terminals: HashSet::from_iter(non_terminals.iter().copied()),
            rules: MultiMap::from_iter(rules.iter().cloned()),
            start,
        }
    }

    fn start_rule(&self) -> CFRule {
        let mut start_rules = self.rules.get_vec(&self.start).cloned().unwrap_or_default();
        assert!(
            start_rules.len() == 1,
            "There must be exactly one start rule."
        );
        (self.start, start_rules.remove(0))
    }
}

trait Parser {
    /// Grammar preprocessing.
    fn fit(&mut self, grammar: &CFGrammar);

    /// Check if the word is in the language.
    fn predict(&mut self, word: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_unit_test_1() {
        let test_grammar = get_test_grammar();
        let parsed_grammar =
            CFGrammar::from_str("a+*()\nSTFN\nS->N\nN->T+N\nN->T\nT->F*T\nT->F\nF->(N)\nF->a\nS")
                .unwrap();
        assert_eq!(test_grammar.terminals, parsed_grammar.terminals);
        assert_eq!(test_grammar.non_terminals, parsed_grammar.non_terminals);
        assert_eq!(test_grammar.rules, parsed_grammar.rules);
        assert_eq!(test_grammar.start, parsed_grammar.start);
    }

    fn get_test_grammar() -> CFGrammar {
        let terminals = ['a', '+', '*', '(', ')'];
        let non_terminals = ['S', 'T', 'F', 'N'];
        let rules = [
            ('S', "N".to_string()),
            ('N', "T+N".to_string()),
            ('N', "T".to_string()),
            ('T', "F*T".to_string()),
            ('T', "F".to_string()),
            ('F', "(N)".to_string()),
            ('F', "a".to_string()),
        ];
        let start = 'S';
        CFGrammar::new(&terminals, &non_terminals, &rules, start)
    }
}
