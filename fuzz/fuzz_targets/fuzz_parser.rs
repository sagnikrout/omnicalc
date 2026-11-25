#![no_main]

use libfuzzer_sys::fuzz_target;
use omnicalc::lexer::Lexer;
use omnicalc::parser::Parser;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let mut lexer = Lexer::new(&input);
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        let _ = parser.parse();
    }
});
