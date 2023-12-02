pub use std::collections::HashSet;
pub use std::str::FromStr;

pub use anyhow::bail;
use anyhow::Ok;
pub use multimap::MultiMap;

#[cfg(feature = "earley")]
pub mod earley;

#[cfg(feature = "lr")]
pub mod lr;

pub const START_RULE: char = '\u{1}';
pub const END_TERMINAL: char = '\u{2}';
pub const EPS_TERMINAL: char = '\u{3}';

pub type CFRule = (char, String);

#[derive(Debug, Default, Clone)]
pub struct CFGrammar {
    /// Terminal symbols.
    terminals: HashSet<char>,
    /// Non-terminal symbols.
    non_terminals: HashSet<char>,
    /// List of rules.
    rules: MultiMap<char, String>,
    /// Start non-terminal.
    start: char,
}

fn check_lines(lines: &Vec<&str>) -> Result<(), anyhow::Error> {
    if lines.len() < 4 {
        bail!("Invalid input string format.");
    } else {
        Ok(())
    }
}

fn check_parts(parts: &Vec<&str>) -> Result<(), anyhow::Error> {
    if parts.len() != 2 {
        bail!("There must be exactly one delimiter in the rule.");
    } else {
        Ok(())
    }
}

fn check_key(key: &str) -> Result<char, anyhow::Error> {
    if key.len() != 1 {
        bail!("There must be exactly one non-terminal in the left part of the CF grammar.");
    } else {
        Ok(key.chars().next().unwrap())
    }
}

fn check_start(start: &str) -> Result<char, anyhow::Error> {
    if start.len() != 1 {
        bail!("There must be exactly one start rule.");
    } else {
        Ok(start.chars().next().unwrap())
    }
}

impl FromStr for CFGrammar {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<_> = s.lines().collect();
        check_lines(&lines)?;
        let non_terminals: HashSet<_> = lines[0].chars().collect();
        let terminals: HashSet<_> = lines[1].chars().collect();
        let mut rules = MultiMap::new();

        for i in 2..(lines.len() - 1) {
            let parts: Vec<_> = lines[i].split("->").map(|s: &str| s.trim()).collect();
            check_parts(&parts)?;
            let key = check_key(parts[0])?;

            if !non_terminals.contains(&key) {
                bail!(
                    "Only non-terminals can be present in the left part of the CF grammar {key}."
                );
            }

            let value = parts[1].to_string();
            rules.insert(key, value);
        }

        let start = check_start(lines.last().unwrap())?;
        Ok(Self::new(&terminals, &non_terminals, &rules, start))
    }
}

impl CFGrammar {
    pub fn new(
        terminals: &HashSet<char>,
        non_terminals: &HashSet<char>,
        rules: &MultiMap<char, String>,
        start: char,
    ) -> Self {
        let mut grammar = Self {
            terminals: terminals.clone(),
            non_terminals: non_terminals.clone(),
            rules: rules.clone(),
            start: START_RULE,
        };
        grammar.terminals.insert(END_TERMINAL);
        grammar.terminals.insert(EPS_TERMINAL);
        grammar.non_terminals.insert(START_RULE);
        grammar.rules.insert(START_RULE, start.to_string());
        grammar
    }

    pub fn is_terminal(&self, symbol: char) -> bool {
        self.terminals.contains(&symbol)
    }

    pub fn is_non_terminal(&self, symbol: char) -> bool {
        self.non_terminals.contains(&symbol)
    }

    fn get_start_rule(&self) -> CFRule {
        let rules = self
            .rules
            .get_vec(&self.start)
            .expect("There are no start rules.");
        assert!(rules.len() == 1, "There must be exactly one start rule.");
        (self.start, rules.first().unwrap().clone())
    }
}

pub trait Parser {
    /// Grammar preprocessing.
    fn fit(&mut self, grammar: &CFGrammar) -> Result<(), anyhow::Error>;

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
            CFGrammar::from_str("STFN\na+*()\nS->N\nN->T+N\nN->T\nT->F*T\nT->F\nF->(N)\nF->a\nS")
                .unwrap();
        assert_eq!(test_grammar.non_terminals, parsed_grammar.non_terminals);
        assert_eq!(test_grammar.terminals, parsed_grammar.terminals);
        assert_eq!(test_grammar.rules, parsed_grammar.rules);
        assert_eq!(test_grammar.start, parsed_grammar.start);
    }

    #[test]
    fn grammar_should_fail_1() {
        let grammar = CFGrammar::from_str("S\nab\nS->a\nb->a\nS");
        assert_eq!(grammar.is_err(), true);
    }

    #[test]
    fn grammar_should_fail_2() {
        let grammar = CFGrammar::from_str("S\nab\nSS->a\nb->a\nS");
        assert_eq!(grammar.is_err(), true);
    }

    #[test]
    fn grammar_should_fail_3() {
        let grammar = CFGrammar::from_str("S\nab\nS");
        assert_eq!(grammar.is_err(), true);
    }

    #[test]
    fn grammar_should_fail_4() {
        let grammar = CFGrammar::from_str("S\nab\nS->a\nS->b\nSS");
        assert_eq!(grammar.is_err(), true);
    }

    #[test]
    fn grammar_should_fail_5() {
        let grammar = CFGrammar::from_str("ST\nab\nS->T->a\nS");
        assert_eq!(grammar.is_err(), true);
    }

    fn get_test_grammar() -> CFGrammar {
        let terminals = HashSet::from(['a', '+', '*', '(', ')']);
        let non_terminals = HashSet::from(['S', 'T', 'F', 'N']);
        let mut rules = MultiMap::new();
        rules.insert('S', "N".to_string());
        rules.insert('N', "T+N".to_string());
        rules.insert('N', "T".to_string());
        rules.insert('T', "F*T".to_string());
        rules.insert('T', "F".to_string());
        rules.insert('F', "(N)".to_string());
        rules.insert('F', "a".to_string());
        let start = 'S';
        CFGrammar::new(&terminals, &non_terminals, &rules, start)
    }
}
