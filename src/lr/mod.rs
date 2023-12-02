use std::collections::{BTreeSet, HashMap, VecDeque};

use super::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct LR1Situation {
    rule: CFRule,
    pos: usize,
    lookahead: char,
}

impl LR1Situation {
    fn new(rule: &CFRule, pos: usize, lookahead: char) -> Self {
        Self {
            rule: rule.clone(),
            pos,
            lookahead,
        }
    }

    fn nth(&self, pos: usize) -> char {
        self.rule.1.chars().nth(pos).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LR1Action {
    NoAction,
    Shift(usize),
    Reduce(usize, char),
    Accept,
}

impl Default for LR1Action {
    fn default() -> Self {
        LR1Action::NoAction
    }
}

#[derive(Debug, Default, Clone)]
pub struct LR1Parser {
    transitions: HashMap<usize, HashMap<char, LR1Action>>,
    start: usize,
}

impl Parser for LR1Parser {
    fn fit(&mut self, grammar: &CFGrammar) -> Result<(), anyhow::Error> {
        let states = Self::get_states(grammar);
        let mut mapping = HashMap::new();

        for (i, state) in states.iter().enumerate() {
            mapping.insert(state, i);
        }

        for state in states.iter() {
            let state_mapped = mapping.get(state).unwrap();

            for situation in state.iter() {
                if situation.pos < situation.rule.1.len() {
                    if grammar.is_terminal(situation.nth(situation.pos)) {
                        let letter = situation.nth(situation.pos);
                        let goto = Self::goto(grammar, state, letter);
                        let goto_mapped = mapping.get(&goto).unwrap();
                        self.add_transition(
                            *state_mapped,
                            letter,
                            &LR1Action::Shift(*goto_mapped),
                        )?;
                    }

                    if situation.rule.0 == START_RULE {
                        self.start = *state_mapped;
                    }

                    continue;
                }

                if situation.rule.0 == START_RULE && situation.lookahead == END_TERMINAL {
                    let letter = situation.lookahead;
                    self.add_transition(*state_mapped, letter, &LR1Action::Accept)?;
                } else {
                    let letter = situation.lookahead;
                    let size = situation.rule.1.len();
                    let symbol = situation.rule.0;
                    self.add_transition(*state_mapped, letter, &LR1Action::Reduce(size, symbol))?;
                }
            }

            for letter in grammar.non_terminals.iter() {
                let goto = Self::goto(grammar, state, *letter);

                if !goto.is_empty() {
                    let goto_mapped = mapping.get(&goto).unwrap();
                    self.add_transition(*state_mapped, *letter, &LR1Action::Shift(*goto_mapped))?;
                }
            }
        }

        Ok(())
    }

    fn predict(&mut self, word: &str) -> bool {
        let mut queue = VecDeque::from([self.start]);
        let mut stack = VecDeque::from_iter(word.chars().rev());
        stack.push_front(END_TERMINAL);

        while !(queue.is_empty() || stack.is_empty()) {
            let action = self
                .transitions
                .entry(*queue.back().unwrap())
                .or_default()
                .entry(*stack.back().unwrap())
                .or_default();

            match *action {
                LR1Action::Shift(state) => {
                    queue.push_back(state);
                    stack.pop_back();
                }
                LR1Action::Reduce(count, symbol) => {
                    queue.drain((queue.len() - count)..);
                    stack.push_back(symbol);
                }
                LR1Action::Accept => {
                    return true;
                }
                LR1Action::NoAction => {
                    return false;
                }
            }
        }

        false
    }
}

impl LR1Parser {
    pub fn new() -> Self {
        Self {
            start: 0,
            transitions: HashMap::new(),
        }
    }

    fn closure(grammar: &CFGrammar, state: &BTreeSet<LR1Situation>) -> BTreeSet<LR1Situation> {
        let mut new_state = state.clone();
        let mut prev_diff = new_state.clone();

        loop {
            let mut curr_diff = BTreeSet::new();

            for situation in prev_diff.iter() {
                let rule_left = if situation.pos >= situation.rule.1.len() {
                    continue;
                } else {
                    situation.nth(situation.pos)
                };

                if let Some(rules) = grammar.rules.get_vec(&rule_left) {
                    for rule_right in rules.iter() {
                        let mut lookup: String =
                            situation.rule.1.chars().skip(situation.pos + 1).collect();
                        lookup.push(situation.lookahead);
                        let first = Self::get_first(grammar, &lookup);

                        for symbol in first.iter() {
                            let new_situation =
                                LR1Situation::new(&(rule_left, rule_right.clone()), 0, *symbol);

                            if new_state.insert(new_situation.clone()) {
                                curr_diff.insert(new_situation);
                            }
                        }
                    }
                }
            }

            if curr_diff.len() == 0 {
                break;
            } else {
                prev_diff = curr_diff;
            }
        }

        new_state
    }

    fn goto(
        grammar: &CFGrammar,
        state: &BTreeSet<LR1Situation>,
        symbol: char,
    ) -> BTreeSet<LR1Situation> {
        let new_state = state
            .iter()
            .filter_map(|situation| {
                if situation.pos >= situation.rule.1.len() || situation.nth(situation.pos) != symbol
                {
                    None
                } else {
                    Some(LR1Situation::new(
                        &situation.rule,
                        situation.pos + 1,
                        situation.lookahead,
                    ))
                }
            })
            .collect();
        Self::closure(grammar, &new_state)
    }

    fn get_first(grammar: &CFGrammar, lookup: &str) -> HashSet<char> {
        let mut visited = HashSet::new();
        Self::dfs(grammar, &mut visited, lookup)
    }

    fn dfs(grammar: &CFGrammar, visited: &mut HashSet<CFRule>, lookup: &str) -> HashSet<char> {
        for symbol in lookup.chars() {
            if grammar.terminals.contains(&symbol) {
                return HashSet::from([symbol]);
            }

            let rule_left = symbol;
            let mut found = HashSet::new();

            if let Some(rules) = grammar.rules.get_vec(&rule_left) {
                for rule_right in rules.iter() {
                    if visited.insert((rule_left, rule_right.clone())) {
                        found.extend(&Self::dfs(grammar, visited, &rule_right));
                    }
                }
            }

            if !found.is_empty() {
                return found;
            }
        }

        HashSet::new()
    }

    fn get_states(grammar: &CFGrammar) -> BTreeSet<BTreeSet<LR1Situation>> {
        let mut states = BTreeSet::new();
        states.insert(Self::closure(
            grammar,
            &BTreeSet::<_>::from([LR1Situation::new(
                &grammar.get_start_rule(),
                0,
                END_TERMINAL,
            )]),
        ));
        let mut prev_diff = states.clone();
        let mut all_symbols = Vec::from_iter(grammar.terminals.iter().copied());
        all_symbols.extend(grammar.non_terminals.iter());

        loop {
            let mut curr_diff = BTreeSet::new();

            for state in prev_diff.iter() {
                for symbol in all_symbols.iter() {
                    let goto = Self::goto(grammar, state, *symbol);

                    if !goto.is_empty() && states.insert(goto.clone()) {
                        curr_diff.insert(goto);
                    }
                }
            }

            if curr_diff.len() == 0 {
                break;
            } else {
                prev_diff = curr_diff;
            }
        }

        states
    }

    fn add_transition(
        &mut self,
        state: usize,
        letter: char,
        action: &LR1Action,
    ) -> Result<(), anyhow::Error> {
        let curr_entry = self
            .transitions
            .entry(state)
            .or_default()
            .entry(letter)
            .or_default();

        if *curr_entry == LR1Action::NoAction || *curr_entry == *action {
            *curr_entry = *action;
            Ok(())
        } else {
            bail!("The given grammar is not LR(1).")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lr1_unit_test_1() {
        let grammar = get_test_grammar();
        let mut parser = LR1Parser::new();
        parser.fit(&grammar).expect("Fit unsuccessful");
        assert_eq!(Parser::predict(&mut parser, "cdd"), true);
    }

    #[test]
    fn lr1_unit_test_2() {
        let grammar = get_test_grammar();
        let mut parser = LR1Parser::new();
        parser.fit(&grammar).expect("Fit unsuccessful");
        assert_eq!(Parser::predict(&mut parser, "ddd"), false);
    }

    #[test]
    fn lr1_should_fail_1() {
        let grammar =
            CFGrammar::from_str("S\na\nS->Sa\nS->a\nS->\nS").expect("Failed to parse the grammar.");
        let mut parser = LR1Parser::new();
        assert_eq!(parser.fit(&grammar).is_err(), true);
    }

    fn get_test_grammar() -> CFGrammar {
        let terminals = HashSet::from(['c', 'd']);
        let non_terminals = HashSet::from(['S', 'C']);
        let mut rules = MultiMap::new();
        rules.insert('S', "CC".to_string());
        rules.insert('C', "cC".to_string());
        rules.insert('C', "d".to_string());
        let start = 'S';
        CFGrammar::new(&terminals, &non_terminals, &rules, start)
    }
}
