use super::{CFGrammar, CFRule, Parser};
use std::collections::HashSet;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct EarleySituation {
    rule: CFRule,
    pos: usize,
    prev_cnt: usize,
}

impl EarleySituation {
    fn new(rule: &CFRule, pos: usize, prev_cnt: usize) -> Self {
        Self {
            rule: rule.clone(),
            pos,
            prev_cnt,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct EarleyParser {
    grammar: Option<CFGrammar>,
    situations: Vec<HashSet<EarleySituation>>,
}

impl Parser for EarleyParser {
    fn fit(&mut self, grammar: &CFGrammar) {
        assert!(self.check_grammar(grammar));
        self.grammar = Some(grammar.clone());
    }

    fn predict(&mut self, word: &str) -> bool {
        self.situations.clear();
        self.situations.resize(word.len() + 1, HashSet::new());
        let start_rule = self.grammar.as_ref().unwrap().start_rule();
        self.situations[0].insert(EarleySituation::new(&start_rule, 0, 0));
        self.do_layer(0);

        for i in 1..=word.len() {
            self.scan(word.chars().nth(i - 1).unwrap(), i - 1);
            self.do_layer(i);
        }

        self.situations[word.len()].contains(&EarleySituation::new(&start_rule, 1, 0))
    }
}

impl EarleyParser {
    pub fn new() -> Self {
        Self {
            grammar: None,
            situations: Vec::new(),
        }
    }

    fn check_grammar(&self, grammar: &CFGrammar) -> bool {
        for rule in grammar.rules.iter() {
            if !grammar.non_terminals.contains(&rule.0) {
                return false;
            }
        }

        true
    }

    fn predict(&mut self, curr_cnt: usize) {
        let mut new_situations = HashSet::<EarleySituation>::new();

        for situation in self.situations[curr_cnt].iter() {
            if situation.pos >= situation.rule.1.len() {
                continue;
            }

            let curr_rule = situation.rule.1.chars().nth(situation.pos).unwrap();

            if let Some(rules) = self.grammar.as_ref().unwrap().rules.get_vec(&curr_rule) {
                for rule in rules.iter() {
                    new_situations.insert(EarleySituation::new(
                        &(curr_rule, rule.clone()),
                        0,
                        curr_cnt,
                    ));
                }
            }
        }

        self.situations[curr_cnt].extend(new_situations);
    }

    fn scan(&mut self, letter: char, curr_cnt: usize) {
        let mut new_situations = HashSet::<EarleySituation>::new();

        for situation in self.situations[curr_cnt].iter() {
            if situation.pos >= situation.rule.1.len() {
                continue;
            }

            if situation.rule.1.chars().nth(situation.pos).unwrap() == letter {
                new_situations.insert(EarleySituation::new(
                    &situation.rule,
                    situation.pos + 1,
                    situation.prev_cnt,
                ));
            }
        }

        self.situations[curr_cnt + 1].extend(new_situations);
    }

    fn complete(&mut self, curr_cnt: usize) {
        let mut new_situations = HashSet::<EarleySituation>::new();

        for curr_situation in self.situations[curr_cnt].iter() {
            if curr_situation.pos != curr_situation.rule.1.len() {
                continue;
            }

            for prev_situation in self.situations[curr_situation.prev_cnt].iter() {
                if prev_situation
                    .rule
                    .1
                    .chars()
                    .nth(prev_situation.pos)
                    .unwrap()
                    == curr_situation.rule.0
                {
                    new_situations.insert(EarleySituation::new(
                        &prev_situation.rule,
                        prev_situation.pos + 1,
                        prev_situation.prev_cnt,
                    ));
                }
            }
        }

        self.situations[curr_cnt].extend(new_situations);
    }

    fn do_layer(&mut self, layer: usize) {
        loop {
            let prev_size = self.situations[layer].len();
            self.predict(layer);
            self.complete(layer);
            let curr_size = self.situations[layer].len();

            if curr_size == prev_size {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use multimap::MultiMap;

    #[test]
    fn earley_unit_test_1() {
        let grammar = get_test_grammar();
        let mut parser = EarleyParser::new();
        parser.fit(&grammar);
        assert_eq!(Parser::predict(&mut parser, "(a+a)"), true);
    }

    fn earley_unit_test_2() {
        let grammar = get_test_grammar();
        let mut parser = EarleyParser::new();
        parser.fit(&grammar);
        assert_eq!(Parser::predict(&mut parser, "(a+a*a())"), false);
    }

    fn get_test_grammar() -> CFGrammar {
        let terminals = ['a', '+', '*', '(', ')'];
        let non_terminals = ['S', 'T', 'F', 'N'];
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
