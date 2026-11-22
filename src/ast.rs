use crate::numeric::NumericValue;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(NumericValue),
    Unit(String),
    Variable(String),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
    Convert {
        expr: Box<Expr>,
        target: String,
    },
    ToBase {
        expr: Box<Expr>,
        base: u32,
    },
    Uncertainty {
        nominal: Box<Expr>,
        error: Box<Expr>,
    },
    Equation {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    List(Vec<Expr>),
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
            Self::Pow => "^",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^^",
            Self::Shl => "<<",
            Self::Shr => ">>",
        };
        write!(f, "{}", op)
    }
}
