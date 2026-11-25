#![no_main]

use libfuzzer_sys::fuzz_target;
use omnicalc::eval::Evaluator;
use omnicalc::lexer::Lexer;
use omnicalc::parser::Parser;
use omnicalc::units::UnitRegistry;
use std::collections::HashMap;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let mut lexer = Lexer::new(input);
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(tokens);
            if let Ok(ast) = parser.parse() {
                let mut evaluator = Evaluator {
                    registry: UnitRegistry::new_offline(),
                    variables: HashMap::new(),
                };
                let _ = evaluator.eval(&ast);
            }
        }
    }
});
