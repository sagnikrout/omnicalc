use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::numeric::NumericValue;

pub fn has_variable(expr: &Expr, var: &str) -> bool {
    match expr {
        Expr::Variable(v) | Expr::Unit(v) => v == var,
        Expr::Number(_) => false,
        Expr::Unary(_, inner) => has_variable(inner, var),
        Expr::Binary(lhs, _, rhs) => has_variable(lhs, var) || has_variable(rhs, var),
        Expr::Call(_, args) => args.iter().any(|a| has_variable(a, var)),
        Expr::Convert { expr, .. } | Expr::ToBase { expr, .. } => has_variable(expr, var),
        Expr::Uncertainty { nominal, error } => {
            has_variable(nominal, var) || has_variable(error, var)
        }
        Expr::Equation { lhs, rhs } => has_variable(lhs, var) || has_variable(rhs, var),
        Expr::List(list) => list.iter().any(|e| has_variable(e, var)),
    }
}

pub fn simplify(expr: &Expr) -> Expr {
    match expr {
        Expr::Unary(UnaryOp::Neg, inner) => {
            let simp_inner = simplify(inner);
            if let Expr::Number(n) = simp_inner {
                Expr::Number(-n)
            } else if let Expr::Unary(UnaryOp::Neg, double_inner) = simp_inner {
                *double_inner
            } else {
                Expr::Unary(UnaryOp::Neg, Box::new(simp_inner))
            }
        }
        Expr::Binary(lhs, op, rhs) => {
            let l = simplify(lhs);
            let r = simplify(rhs);

            match (l, *op, r) {
                // Constant folding
                (Expr::Number(a), BinaryOp::Add, Expr::Number(b)) => Expr::Number(a + b),
                (Expr::Number(a), BinaryOp::Sub, Expr::Number(b)) => Expr::Number(a - b),
                (Expr::Number(a), BinaryOp::Mul, Expr::Number(b)) => Expr::Number(a * b),
                (Expr::Number(a), BinaryOp::Div, Expr::Number(b)) if !b.is_zero() => {
                    Expr::Number(a / b)
                }
                (Expr::Number(a), BinaryOp::Pow, Expr::Number(b)) => Expr::Number(a.pow(&b)),

                // Algebraic identities: Addition
                (Expr::Number(n), BinaryOp::Add, other) if n.is_zero() => other,
                (other, BinaryOp::Add, Expr::Number(n)) if n.is_zero() => other,

                // Algebraic identities: Subtraction
                (other, BinaryOp::Sub, Expr::Number(n)) if n.is_zero() => other,
                (l, BinaryOp::Sub, r) if l == r => Expr::Number(NumericValue::from_i64(0)),

                // Algebraic identities: Multiplication
                (Expr::Number(n), BinaryOp::Mul, _) if n.is_zero() => {
                    Expr::Number(NumericValue::from_i64(0))
                }
                (_, BinaryOp::Mul, Expr::Number(n)) if n.is_zero() => {
                    Expr::Number(NumericValue::from_i64(0))
                }
                (Expr::Number(n), BinaryOp::Mul, other) if n.is_one() => other,
                (other, BinaryOp::Mul, Expr::Number(n)) if n.is_one() => other,

                // Combine constant coefficients: e.g. 6 * (2 * x) = 12 * x
                (Expr::Number(c1), BinaryOp::Mul, Expr::Binary(box_l, BinaryOp::Mul, box_r)) => {
                    if let Expr::Number(c2) = *box_l {
                        simplify(&Expr::Binary(
                            Box::new(Expr::Number(c1 * c2)),
                            BinaryOp::Mul,
                            box_r,
                        ))
                    } else {
                        Expr::Binary(
                            Box::new(Expr::Number(c1)),
                            BinaryOp::Mul,
                            Box::new(Expr::Binary(box_l, BinaryOp::Mul, box_r)),
                        )
                    }
                }

                // Power identities
                (_, BinaryOp::Pow, Expr::Number(n)) if n.is_zero() => {
                    Expr::Number(NumericValue::from_i64(1))
                }
                (other, BinaryOp::Pow, Expr::Number(n)) if n.is_one() => other,

                (l, op, r) => Expr::Binary(Box::new(l), op, Box::new(r)),
            }
        }
        Expr::List(list) => Expr::List(list.iter().map(simplify).collect()),
        _ => expr.clone(),
    }
}
