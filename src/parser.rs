use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::lexer::Token;
use crate::numeric::NumericValue;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Num;
use std::str::FromStr;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
}

const MAX_DEPTH: usize = 512;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            depth: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if let Some(tok) = self.advance() {
            if tok == expected {
                Ok(())
            } else {
                Err(format!("Expected {:?}, found {:?}", expected, tok))
            }
        } else {
            Err(format!("Expected {:?}, found end of input", expected))
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        if self.tokens.is_empty() {
            return Err("Empty input".to_string());
        }

        let expr = self.parse_conversion_or_equation()?;
        if self.pos < self.tokens.len() {
            return Err(format!(
                "Unexpected trailing token: {:?}",
                self.tokens[self.pos]
            ));
        }
        Ok(expr)
    }

    fn parse_conversion_or_equation(&mut self) -> Result<Expr, String> {
        if self.depth > MAX_DEPTH {
            return Err("Recursion depth limit exceeded".to_string());
        }
        self.depth += 1;
        let res = self.parse_conversion_or_equation_inner();
        self.depth -= 1;
        res
    }

    fn parse_conversion_or_equation_inner(&mut self) -> Result<Expr, String> {
        let expr = self.parse_binary(0)?;

        // Check for '=' (equation)
        if self.peek() == Some(&Token::Equal) {
            self.advance();
            let rhs = self.parse_conversion_or_equation()?;
            return Ok(Expr::Equation {
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            });
        }

        // Check for '->' or 'to' or 'as' or 'in' (when used as conversion), allowing chained conversions
        let mut cur_expr = expr;
        loop {
            let is_arrow = self.peek() == Some(&Token::Arrow);
            let is_in_conversion = if self.peek() == Some(&Token::In) {
                matches!(
                    self.tokens.get(self.pos + 1),
                    Some(Token::Ident(_)) | Some(Token::In)
                )
            } else {
                false
            };

            if !(is_arrow || is_in_conversion) {
                break;
            }

            self.advance();
            let mut target_str = String::new();
            while let Some(tok) = self.peek() {
                match tok {
                    Token::Ident(s) => {
                        target_str.push_str(s);
                        self.advance();
                    }
                    Token::In => {
                        target_str.push_str("in");
                        self.advance();
                    }
                    Token::Slash => {
                        target_str.push('/');
                        self.advance();
                    }
                    Token::Star => {
                        target_str.push('*');
                        self.advance();
                    }
                    Token::Caret => {
                        target_str.push('^');
                        self.advance();
                    }
                    Token::Number(s) => {
                        target_str.push_str(s);
                        self.advance();
                    }
                    _ => break,
                }
            }

            if target_str.is_empty() {
                return Err("Expected target unit or base after 'to'/'->'".to_string());
            }

            cur_expr = match target_str.to_lowercase().as_str() {
                "hex" => Expr::ToBase {
                    expr: Box::new(cur_expr),
                    base: 16,
                },
                "bin" => Expr::ToBase {
                    expr: Box::new(cur_expr),
                    base: 2,
                },
                "oct" => Expr::ToBase {
                    expr: Box::new(cur_expr),
                    base: 8,
                },
                "dec" => Expr::ToBase {
                    expr: Box::new(cur_expr),
                    base: 10,
                },
                "factors" | "factor" => Expr::Call("factor".to_string(), vec![cur_expr]),
                _ => Expr::Convert {
                    expr: Box::new(cur_expr),
                    target: target_str,
                },
            };
        }

        Ok(cur_expr)
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, String> {
        if self.depth > MAX_DEPTH {
            return Err("Recursion depth limit exceeded".to_string());
        }
        self.depth += 1;
        let res = self.parse_binary_inner(min_prec);
        self.depth -= 1;
        res
    }

    fn parse_binary_inner(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_primary()?;

        loop {
            // Check for immediate postfix percentage
            if self.peek() == Some(&Token::PercentPostfix) {
                self.advance();
                lhs = Expr::Binary(
                    Box::new(lhs),
                    BinaryOp::Div,
                    Box::new(Expr::Number(NumericValue::from_i64(100))),
                );
                continue;
            }

            // Check for derivative prime: e.g. f' or (x^2)'
            if self.peek() == Some(&Token::Prime) {
                self.advance();
                lhs = Expr::Call("diff".to_string(), vec![lhs]);
                continue;
            }

            // Check for ± / +/- (uncertainty)
            if self.peek() == Some(&Token::PlusMinus) {
                let prec = 4; // Lower precedence than addition/multiplication
                if prec < min_prec {
                    break;
                }
                self.advance();
                let err_expr = self.parse_binary(prec)?;
                lhs = Expr::Uncertainty {
                    nominal: Box::new(lhs),
                    error: Box::new(err_expr),
                };
                continue;
            }

            // Check for 'of' (e.g. 5% of 100)
            if self.peek() == Some(&Token::Of) {
                self.advance();
                let rhs = self.parse_binary(7)?;
                lhs = Expr::Binary(Box::new(lhs), BinaryOp::Mul, Box::new(rhs));
                continue;
            }

            // Check for implicit multiplication:
            // e.g., 2x, 2(3+4), (x+1)(x-1), 5 m
            if self.is_implicit_mul_start() {
                let prec = 9; // higher precedence than * and / (7/8) so '100 J / 10 s' binds (10 s)
                if prec < min_prec {
                    break;
                }
                let rhs = self.parse_binary(prec)?;
                lhs = Expr::Binary(Box::new(lhs), BinaryOp::Mul, Box::new(rhs));
                continue;
            }

            let (op, l_prec, r_prec) = match self.peek() {
                Some(Token::Pipe) => (BinaryOp::BitOr, 1, 2),
                Some(Token::BitXor) => (BinaryOp::BitXor, 2, 3),
                Some(Token::Ampersand) => (BinaryOp::BitAnd, 3, 4),
                Some(Token::Shl) => (BinaryOp::Shl, 4, 5),
                Some(Token::Shr) => (BinaryOp::Shr, 4, 5),
                Some(Token::Plus) => (BinaryOp::Add, 5, 6),
                Some(Token::Minus) => (BinaryOp::Sub, 5, 6),
                Some(Token::Star) => (BinaryOp::Mul, 7, 8),
                Some(Token::Slash) => (BinaryOp::Div, 7, 8),
                Some(Token::Percent) => (BinaryOp::Rem, 7, 8),
                Some(Token::Caret) => (BinaryOp::Pow, 10, 9), // Right-associative
                _ => break,
            };

            if l_prec < min_prec {
                break;
            }

            self.advance();
            let rhs = self.parse_binary(r_prec)?;
            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
        }

        Ok(lhs)
    }

    fn is_implicit_mul_start(&self) -> bool {
        match self.peek() {
            Some(Token::Ident(_))
            | Some(Token::LParen)
            | Some(Token::Number(_))
            | Some(Token::FeetInches(_, _)) => true,
            Some(Token::In) => {
                // If followed by Arrow, RParen, or EOF, 'in' is the unit inch!
                matches!(
                    self.tokens.get(self.pos + 1),
                    None | Some(Token::Arrow) | Some(Token::RParen)
                )
            }
            _ => false,
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        if self.depth > MAX_DEPTH {
            return Err("Recursion depth limit exceeded".to_string());
        }
        self.depth += 1;
        let res = self.parse_primary_inner();
        self.depth -= 1;
        res
    }

    fn parse_primary_inner(&mut self) -> Result<Expr, String> {
        let tok = self
            .advance()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        match tok {
            Token::In => Ok(Expr::Unit("in".to_string())),
            Token::PercentPostfix => {
                Err("Unexpected percentage symbol without preceding value".to_string())
            }
            Token::Number(s) => {
                let mut expr = if s.contains('.') || s.contains('e') || s.contains('E') {
                    let f = s.parse::<f64>().map_err(|e| e.to_string())?;
                    Expr::Number(NumericValue::from_f64(f))
                } else {
                    let b = BigInt::from_str(&s).map_err(|e| e.to_string())?;
                    Expr::Number(NumericValue::Exact(BigRational::from_integer(b)))
                };

                // Check for immediate percent postfix
                if self.peek() == Some(&Token::PercentPostfix) {
                    self.advance();
                    expr = Expr::Binary(
                        Box::new(expr),
                        BinaryOp::Div,
                        Box::new(Expr::Number(NumericValue::from_i64(100))),
                    );
                }

                Ok(expr)
            }
            Token::Hex(s) => {
                let b =
                    BigInt::from_str_radix(&s, 16).map_err(|e| format!("Invalid hex: {}", e))?;
                Ok(Expr::Number(NumericValue::Exact(
                    BigRational::from_integer(b),
                )))
            }
            Token::Bin(s) => {
                let b =
                    BigInt::from_str_radix(&s, 2).map_err(|e| format!("Invalid binary: {}", e))?;
                Ok(Expr::Number(NumericValue::Exact(
                    BigRational::from_integer(b),
                )))
            }
            Token::Oct(s) => {
                let b =
                    BigInt::from_str_radix(&s, 8).map_err(|e| format!("Invalid octal: {}", e))?;
                Ok(Expr::Number(NumericValue::Exact(
                    BigRational::from_integer(b),
                )))
            }
            Token::FeetInches(ft, inch) => {
                // Standalone feet (ft > 0, inch == 0)
                if ft > 0.0 && inch == 0.0 {
                    return Ok(Expr::Binary(
                        Box::new(Expr::Number(NumericValue::from_f64(ft))),
                        BinaryOp::Mul,
                        Box::new(Expr::Unit("ft".to_string())),
                    ));
                }
                // Standalone inches (ft == 0, inch > 0)
                if ft == 0.0 && inch > 0.0 {
                    return Ok(Expr::Binary(
                        Box::new(Expr::Number(NumericValue::from_f64(inch))),
                        BinaryOp::Mul,
                        Box::new(Expr::Unit("in".to_string())),
                    ));
                }
                // 5'10" = 5 ft + 10 in
                let ft_expr = Expr::Binary(
                    Box::new(Expr::Number(NumericValue::from_f64(ft))),
                    BinaryOp::Mul,
                    Box::new(Expr::Unit("ft".to_string())),
                );
                let in_expr = Expr::Binary(
                    Box::new(Expr::Number(NumericValue::from_f64(inch))),
                    BinaryOp::Mul,
                    Box::new(Expr::Unit("in".to_string())),
                );
                Ok(Expr::Binary(
                    Box::new(ft_expr),
                    BinaryOp::Add,
                    Box::new(in_expr),
                ))
            }
            Token::Ident(name) => {
                // Check if it's a function call: name(...)
                if self.peek() == Some(&Token::LParen) {
                    self.advance(); // '('
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        loop {
                            args.push(self.parse_conversion_or_equation()?);
                            if self.peek() == Some(&Token::Comma)
                                || self.peek() == Some(&Token::Semicolon)
                            {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(Token::RParen)?;
                    return Ok(Expr::Call(name, args));
                }

                // Constants
                match name.as_str() {
                    "pi" => Ok(Expr::Number(NumericValue::from_f64(std::f64::consts::PI))),
                    "tau" => Ok(Expr::Number(NumericValue::from_f64(std::f64::consts::TAU))),
                    "phi" => Ok(Expr::Number(NumericValue::from_f64(
                        (1.0 + 5.0f64.sqrt()) / 2.0,
                    ))),
                    "e" => Ok(Expr::Number(NumericValue::from_f64(std::f64::consts::E))),
                    "c" => {
                        // Speed of light in vacuum = 299792458 m/s
                        Ok(Expr::Binary(
                            Box::new(Expr::Number(NumericValue::from_f64(299792458.0))),
                            BinaryOp::Mul,
                            Box::new(Expr::Binary(
                                Box::new(Expr::Unit("m".to_string())),
                                BinaryOp::Div,
                                Box::new(Expr::Unit("s".to_string())),
                            )),
                        ))
                    }
                    "i" => Ok(Expr::Number(NumericValue::Complex(
                        num_complex::Complex64::new(0.0, 1.0),
                    ))),
                    _ => {
                        // Check if it's known unit or variable
                        Ok(Expr::Unit(name))
                    }
                }
            }
            Token::Minus => {
                if let Some(Token::Number(s)) = self.peek() {
                    if self.tokens.get(self.pos + 1) == Some(&Token::Caret) {
                        let inner = self.parse_binary(9)?;
                        Ok(Expr::Unary(UnaryOp::Neg, Box::new(inner)))
                    } else {
                        let s_neg = format!("-{}", s);
                        self.advance();
                        let mut expr =
                            if s_neg.contains('.') || s_neg.contains('e') || s_neg.contains('E') {
                                let f = s_neg.parse::<f64>().map_err(|e| e.to_string())?;
                                Expr::Number(NumericValue::from_f64(f))
                            } else {
                                let b = BigInt::from_str(&s_neg).map_err(|e| e.to_string())?;
                                Expr::Number(NumericValue::Exact(BigRational::from_integer(b)))
                            };

                        // Check for immediate percent postfix
                        if self.peek() == Some(&Token::PercentPostfix) {
                            self.advance();
                            expr = Expr::Binary(
                                Box::new(expr),
                                BinaryOp::Div,
                                Box::new(Expr::Number(NumericValue::from_i64(100))),
                            );
                        }

                        Ok(expr)
                    }
                } else {
                    let inner = self.parse_binary(9)?;
                    Ok(Expr::Unary(UnaryOp::Neg, Box::new(inner)))
                }
            }
            Token::Plus => self.parse_binary(9),
            Token::Tilde => {
                let inner = self.parse_binary(9)?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(inner)))
            }
            Token::LParen => {
                let inner = self.parse_conversion_or_equation()?;
                self.expect(Token::RParen)?;
                Ok(inner)
            }
            Token::LBracket => {
                let mut elements = Vec::new();
                if self.peek() != Some(&Token::RBracket) {
                    loop {
                        elements.push(self.parse_conversion_or_equation()?);
                        if self.peek() == Some(&Token::Comma)
                            || self.peek() == Some(&Token::Semicolon)
                        {
                            self.advance();
                            if self.peek() == Some(&Token::RBracket) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::List(elements))
            }
            other => Err(format!("Unexpected token: {:?}", other)),
        }
    }
}
