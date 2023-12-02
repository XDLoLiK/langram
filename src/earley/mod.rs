use anyhow::Context;

use super::*;

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

    fn nth(&self, pos: usize) -> char {
        self.rule.1.chars().nth(pos).unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct EarleyParser {
    grammar: Option<CFGrammar>,
    situations: Vec<HashSet<EarleySituation>>,
}

impl Parser for EarleyParser {
    fn fit(&mut self, grammar: &CFGrammar) -> Result<(), anyhow::Error> {
        self.check_grammar(grammar)
            .with_context(|| "The grammar is not context free.")?;
        self.grammar = Some(grammar.clone());
        Ok(())
    }

    fn predict(&mut self, word: &str) -> bool {
        if self.grammar.is_none() {
            return false;
        }

        self.situations.clear();
        self.situations.resize(word.len() + 1, HashSet::new());
        let start_rule = self.grammar.as_ref().unwrap().get_start_rule();
        self.situations[0].insert(EarleySituation::new(&start_rule, 0, 0));
        self.do_layer(0);

        for (i, letter) in word.char_indices() {
            self.scan(letter, i);
            self.do_layer(i + 1);
        }

        self.situations[word.len()].contains(&EarleySituation::new(
            &start_rule,
            start_rule.1.len(),
            0,
        ))
    }
}

impl EarleyParser {
    pub fn new() -> Self {
        Self {
            grammar: None,
            situations: Vec::new(),
        }
    }

    fn check_grammar(&self, grammar: &CFGrammar) -> Result<(), anyhow::Error> {
        for rule in grammar.rules.iter() {
            if !grammar.non_terminals.contains(&rule.0) {
                bail!("There must be no terminals in the left part of the CF grammar rule");
            }
        }

        Ok(())
    }

    fn predict(&mut self, curr_cnt: usize) {
        let mut new_situations = HashSet::<EarleySituation>::new();

        for situation in self.situations[curr_cnt].iter() {
            let rule_left = if situation.pos >= situation.rule.1.len() {
                continue;
            } else {
                situation.nth(situation.pos)
            };

            if let Some(rules) = self.grammar.as_ref().unwrap().rules.get_vec(&rule_left) {
                for rule_right in rules.iter() {
                    new_situations.insert(EarleySituation::new(
                        &(rule_left, rule_right.clone()),
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
            let rule_curr = if situation.pos >= situation.rule.1.len() {
                continue;
            } else {
                situation.nth(situation.pos)
            };

            if rule_curr == letter {
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
                if prev_situation.nth(prev_situation.pos) == curr_situation.rule.0 {
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

            if self.situations[layer].len() == prev_size {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earley_unit_test_1() {
        let grammar = get_test_grammar();
        let mut parser = EarleyParser::new();
        parser.fit(&grammar).expect("Fit unsuccessful");
        assert_eq!(Parser::predict(&mut parser, "(a+a)"), true);
    }

    #[test]
    fn earley_unit_test_2() {
        let grammar = get_test_grammar();
        let mut parser = EarleyParser::new();
        parser.fit(&grammar).expect("Fit unsuccessful");
        assert_eq!(Parser::predict(&mut parser, "(a+a*a())"), false);
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
