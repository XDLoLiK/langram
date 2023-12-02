use langram::lr::LR1Parser;
use langram::{CFGrammar, FromStr, Parser};

fn main() {
    let grammar =
        CFGrammar::from_str("S\na\nS->Sa\nS->a\nS->\nS").expect("Failed to parse the grammar.");
    let mut parser = LR1Parser::new();
    parser.fit(&grammar).expect("Failed to fit the parser");
    println!("{}", parser.predict("cdd"));
}
