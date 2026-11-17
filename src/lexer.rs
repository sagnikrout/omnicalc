#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(String),
    Hex(String),
    Bin(String),
    Oct(String),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PercentPostfix,
    Caret,
    BitXor,
    Ampersand,
    Pipe,
    Tilde,
    Shl,
    Shr,
    Equal,
    In,
    PlusMinus, // ± or +/-
    Arrow,     // -> or to
    Of,        // of (e.g. 5% of 100)
    Prime,     // ' (derivative)
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    FeetInches(f64, f64), // 5'10"
}

pub struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            // Check for ± (Unicode)
            if ch == '±' {
                self.advance();
                tokens.push(Token::PlusMinus);
                continue;
            }

            // Check for ° (degree symbol: °, °C, °F)
            if ch == '°' {
                self.advance();
                if self.peek() == Some('C') {
                    self.advance();
                    tokens.push(Token::Ident("degC".to_string()));
                } else if self.peek() == Some('F') {
                    self.advance();
                    tokens.push(Token::Ident("degF".to_string()));
                } else {
                    tokens.push(Token::Ident("deg".to_string()));
                }
                continue;
            }

            // Check for +/-
            if ch == '+' && self.peek_next() == Some('/') {
                self.advance(); // '+'
                self.advance(); // '/'
                if self.peek() == Some('-') {
                    self.advance();
                    tokens.push(Token::PlusMinus);
                    continue;
                } else {
                    tokens.push(Token::Plus);
                    tokens.push(Token::Slash);
                    continue;
                }
            }

            // Check for ->
            if ch == '-' && self.peek_next() == Some('>') {
                self.advance();
                self.advance();
                tokens.push(Token::Arrow);
                continue;
            }

            // Check for << and >>
            if ch == '<' && self.peek_next() == Some('<') {
                self.advance();
                self.advance();
                tokens.push(Token::Shl);
                continue;
            }
            if ch == '>' && self.peek_next() == Some('>') {
                self.advance();
                self.advance();
                tokens.push(Token::Shr);
                continue;
            }

            // Check for ** (power)
            if ch == '*' && self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                tokens.push(Token::Caret);
                continue;
            }

            // Check for ^^ (xor)
            if ch == '^' && self.peek_next() == Some('^') {
                self.advance();
                self.advance();
                tokens.push(Token::BitXor);
                continue;
            }

            // Feet-inches notation and numbers (including leading decimal like .5)
            if ch.is_ascii_digit() || (ch == '.' && self.peek_next().is_some_and(|n| n.is_ascii_digit())) {
                // Peek ahead to see if it's feet-inches format like 5'10", 5', 10"
                if let Some(token) = self.try_lex_feet_inches() {
                    tokens.push(token);
                    continue;
                }

                // Check hex (0x...), binary (0b...), octal (0o...)
                if ch == '0' && (self.peek_next() == Some('x') || self.peek_next() == Some('X')) {
                    self.advance(); // '0'
                    self.advance(); // 'x'
                    let mut hex_str = String::new();
                    while let Some(c) = self.peek() {
                        if c.is_ascii_hexdigit() || c == '_' {
                            if c != '_' {
                                hex_str.push(c);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Hex(hex_str));
                    continue;
                }

                if ch == '0' && (self.peek_next() == Some('b') || self.peek_next() == Some('B')) {
                    self.advance(); // '0'
                    self.advance(); // 'b'
                    let mut bin_str = String::new();
                    while let Some(c) = self.peek() {
                        if c == '0' || c == '1' || c == '_' {
                            if c != '_' {
                                bin_str.push(c);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Bin(bin_str));
                    continue;
                }

                if ch == '0' && (self.peek_next() == Some('o') || self.peek_next() == Some('O')) {
                    self.advance(); // '0'
                    self.advance(); // 'o'
                    let mut oct_str = String::new();
                    while let Some(c) = self.peek() {
                        if ('0'..='7').contains(&c) || c == '_' {
                            if c != '_' {
                                oct_str.push(c);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Oct(oct_str));
                    continue;
                }

                // Standard number (int or float)
                let mut num_str = String::new();
                let mut seen_dot = false;
                let mut seen_exp = false;

                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() || c == '_' {
                        if c != '_' {
                            num_str.push(c);
                        }
                        self.advance();
                    } else if c == '.' && !seen_dot && !seen_exp {
                        seen_dot = true;
                        num_str.push(c);
                        self.advance();
                    } else if (c == 'e' || c == 'E') && !seen_exp {
                        let is_valid_exp = match self.peek_next() {
                            Some(d) if d.is_ascii_digit() => true,
                            Some('+' | '-') => {
                                self.chars.get(self.pos + 2).is_some_and(|d| d.is_ascii_digit())
                            }
                            _ => false,
                        };
                        if is_valid_exp {
                            seen_exp = true;
                            num_str.push(c);
                            self.advance();
                            if let Some(next_c) = self.peek()
                                && (next_c == '+' || next_c == '-') {
                                    num_str.push(next_c);
                                    self.advance();
                                }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num_str));

                // Check for immediate percent postfix with no whitespace (e.g. 50%, 10%)
                if self.peek() == Some('%') {
                    self.advance();
                    tokens.push(Token::PercentPostfix);
                }
                continue;
            }

            // Identifiers / Units / Keywords
            if ch.is_alphabetic() || ch == '_' || ch == 'π' || ch == 'τ' || ch == 'φ' {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '°' || c == 'π' || c == 'τ' || c == 'φ' {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                match ident.as_str() {
                    "to" | "as" => tokens.push(Token::Arrow),
                    "in" => tokens.push(Token::In),
                    "of" => tokens.push(Token::Of),
                    "xor" => tokens.push(Token::BitXor),
                    "pi" | "π" => tokens.push(Token::Ident("pi".to_string())),
                    "tau" | "τ" => tokens.push(Token::Ident("tau".to_string())),
                    "phi" | "φ" => tokens.push(Token::Ident("phi".to_string())),
                    _ => tokens.push(Token::Ident(ident)),
                }
                continue;
            }

            // Single-character symbols
            match ch {
                '+' => { self.advance(); tokens.push(Token::Plus); }
                '-' => { self.advance(); tokens.push(Token::Minus); }
                '*' => { self.advance(); tokens.push(Token::Star); }
                '/' => { self.advance(); tokens.push(Token::Slash); }
                '%' => { self.advance(); tokens.push(Token::Percent); }
                '^' => { self.advance(); tokens.push(Token::Caret); }
                '&' => { self.advance(); tokens.push(Token::Ampersand); }
                '|' => { self.advance(); tokens.push(Token::Pipe); }
                '~' => { self.advance(); tokens.push(Token::Tilde); }
                '=' => { self.advance(); tokens.push(Token::Equal); }
                '\'' => { self.advance(); tokens.push(Token::Prime); }
                '(' => { self.advance(); tokens.push(Token::LParen); }
                ')' => { self.advance(); tokens.push(Token::RParen); }
                '[' => { self.advance(); tokens.push(Token::LBracket); }
                ']' => { self.advance(); tokens.push(Token::RBracket); }
                ',' => { self.advance(); tokens.push(Token::Comma); }
                ';' => { self.advance(); tokens.push(Token::Semicolon); }
                _ => {
                    return Err(format!("Unexpected character: '{}' at position {}", ch, self.pos));
                }
            }
        }

        Ok(tokens)
    }

    fn try_lex_feet_inches(&mut self) -> Option<Token> {
        let saved_pos = self.pos;
        let mut first_num = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                first_num.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if first_num.is_empty() {
            self.pos = saved_pos;
            return None;
        }

        // Check for standalone inches: e.g. 10"
        if self.peek() == Some('"') {
            self.advance();
            if let Ok(inch) = first_num.parse::<f64>() {
                return Some(Token::FeetInches(0.0, inch));
            }
        }

        // Check for feet (with optional inches): e.g. 5'10" or 5'
        if self.peek() == Some('\'') {
            self.advance(); // consume '\''
            let mut inches_str = String::new();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '.' {
                    inches_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if self.peek() == Some('"') {
                self.advance(); // consume '"'
            }
            if let Ok(ft) = first_num.parse::<f64>() {
                if !inches_str.is_empty() {
                    if let Ok(inch) = inches_str.parse::<f64>() {
                        return Some(Token::FeetInches(ft, inch));
                    }
                } else {
                    return Some(Token::FeetInches(ft, 0.0));
                }
            }
        }

        self.pos = saved_pos;
        None
    }
}
