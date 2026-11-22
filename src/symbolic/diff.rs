use super::simplify::{has_variable, simplify};
use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::numeric::NumericValue;

pub fn diff(expr: &Expr, var: &str) -> Expr {
    match expr {
        Expr::Number(_) => Expr::Number(NumericValue::from_i64(0)),
        Expr::Variable(v) | Expr::Unit(v) => {
            if v == var {
                Expr::Number(NumericValue::from_i64(1))
            } else {
                Expr::Number(NumericValue::from_i64(0))
            }
        }
        Expr::Unary(UnaryOp::Neg, inner) => {
            simplify(&Expr::Unary(UnaryOp::Neg, Box::new(diff(inner, var))))
        }
        Expr::Binary(lhs, BinaryOp::Add, rhs) => {
            let dl = diff(lhs, var);
            let dr = diff(rhs, var);
            simplify(&Expr::Binary(Box::new(dl), BinaryOp::Add, Box::new(dr)))
        }
        Expr::Binary(lhs, BinaryOp::Sub, rhs) => {
            let dl = diff(lhs, var);
            let dr = diff(rhs, var);
            simplify(&Expr::Binary(Box::new(dl), BinaryOp::Sub, Box::new(dr)))
        }
        Expr::Binary(lhs, BinaryOp::Mul, rhs) => {
            // Product rule: d(u*v) = u'*v + u*v'
            let dl = diff(lhs, var);
            let dr = diff(rhs, var);
            let term1 = Expr::Binary(Box::new(dl), BinaryOp::Mul, rhs.clone());
            let term2 = Expr::Binary(lhs.clone(), BinaryOp::Mul, Box::new(dr));
            simplify(&Expr::Binary(
                Box::new(term1),
                BinaryOp::Add,
                Box::new(term2),
            ))
        }
        Expr::Binary(lhs, BinaryOp::Div, rhs) => {
            // Quotient rule: d(u/v) = (u'*v - u*v') / v^2
            let dl = diff(lhs, var);
            let dr = diff(rhs, var);
            let num = Expr::Binary(
                Box::new(Expr::Binary(Box::new(dl), BinaryOp::Mul, rhs.clone())),
                BinaryOp::Sub,
                Box::new(Expr::Binary(lhs.clone(), BinaryOp::Mul, Box::new(dr))),
            );
            let denom = Expr::Binary(
                rhs.clone(),
                BinaryOp::Pow,
                Box::new(Expr::Number(NumericValue::from_i64(2))),
            );
            simplify(&Expr::Binary(Box::new(num), BinaryOp::Div, Box::new(denom)))
        }
        Expr::Binary(lhs, BinaryOp::Pow, rhs) => {
            // Power rule for u^n where n is constant
            if !has_variable(rhs, var) {
                let dl = diff(lhs, var);
                let p_minus_1 = Expr::Binary(
                    rhs.clone(),
                    BinaryOp::Sub,
                    Box::new(Expr::Number(NumericValue::from_i64(1))),
                );
                let n_u_p = Expr::Binary(
                    rhs.clone(),
                    BinaryOp::Mul,
                    Box::new(Expr::Binary(
                        lhs.clone(),
                        BinaryOp::Pow,
                        Box::new(simplify(&p_minus_1)),
                    )),
                );
                simplify(&Expr::Binary(Box::new(n_u_p), BinaryOp::Mul, Box::new(dl)))
            } else {
                // Generalized: d(u^v) = u^v * (v'*ln(u) + v*u'/u)
                let term = Expr::Binary(
                    Box::new(expr.clone()),
                    BinaryOp::Mul,
                    Box::new(Expr::Binary(
                        Box::new(Expr::Binary(
                            Box::new(diff(rhs, var)),
                            BinaryOp::Mul,
                            Box::new(Expr::Call("ln".to_string(), vec![*lhs.clone()])),
                        )),
                        BinaryOp::Add,
                        Box::new(Expr::Binary(
                            rhs.clone(),
                            BinaryOp::Mul,
                            Box::new(Expr::Binary(
                                Box::new(diff(lhs, var)),
                                BinaryOp::Div,
                                lhs.clone(),
                            )),
                        )),
                    )),
                );
                simplify(&term)
            }
        }
        Expr::Call(name, args) if name == "diff" => {
            let inner_expr = &args[0];
            let v = if args.len() > 1 {
                match &args[1] {
                    Expr::Unit(s) | Expr::Variable(s) => s.as_str(),
                    _ => var,
                }
            } else {
                var
            };
            let d1 = diff(inner_expr, v);
            simplify(&diff(&d1, var))
        }
        Expr::Call(name, args) if args.len() == 1 => {
            let u = &args[0];
            let du = diff(u, var);
            match name.as_str() {
                "sin" => {
                    let d = Expr::Binary(
                        Box::new(Expr::Call("cos".to_string(), vec![u.clone()])),
                        BinaryOp::Mul,
                        Box::new(du),
                    );
                    simplify(&d)
                }
                "cos" => {
                    let neg_sin = Expr::Unary(
                        UnaryOp::Neg,
                        Box::new(Expr::Call("sin".to_string(), vec![u.clone()])),
                    );
                    let d = Expr::Binary(Box::new(neg_sin), BinaryOp::Mul, Box::new(du));
                    simplify(&d)
                }
                "tan" => {
                    let cos_u = Expr::Call("cos".to_string(), vec![u.clone()]);
                    let cos_sq = Expr::Binary(
                        Box::new(cos_u),
                        BinaryOp::Pow,
                        Box::new(Expr::Number(NumericValue::from_i64(2))),
                    );
                    let d = Expr::Binary(Box::new(du), BinaryOp::Div, Box::new(cos_sq));
                    simplify(&d)
                }
                "sqrt" => {
                    let denom = Expr::Binary(
                        Box::new(Expr::Number(NumericValue::from_i64(2))),
                        BinaryOp::Mul,
                        Box::new(Expr::Call("sqrt".to_string(), vec![u.clone()])),
                    );
                    let d = Expr::Binary(Box::new(du), BinaryOp::Div, Box::new(denom));
                    simplify(&d)
                }
                "exp" => {
                    let d = Expr::Binary(Box::new(expr.clone()), BinaryOp::Mul, Box::new(du));
                    simplify(&d)
                }
                "ln" => {
                    let d = Expr::Binary(Box::new(du), BinaryOp::Div, Box::new(u.clone()));
                    simplify(&d)
                }
                "asin" => {
                    let u_sq = Expr::Binary(
                        Box::new(u.clone()),
                        BinaryOp::Pow,
                        Box::new(Expr::Number(NumericValue::from_i64(2))),
                    );
                    let one_minus = Expr::Binary(
                        Box::new(Expr::Number(NumericValue::from_i64(1))),
                        BinaryOp::Sub,
                        Box::new(u_sq),
                    );
                    let denom = Expr::Call("sqrt".to_string(), vec![one_minus]);
                    let d = Expr::Binary(Box::new(du), BinaryOp::Div, Box::new(denom));
                    simplify(&d)
                }
                "acos" => {
                    let u_sq = Expr::Binary(
                        Box::new(u.clone()),
                        BinaryOp::Pow,
                        Box::new(Expr::Number(NumericValue::from_i64(2))),
                    );
                    let one_minus = Expr::Binary(
                        Box::new(Expr::Number(NumericValue::from_i64(1))),
                        BinaryOp::Sub,
                        Box::new(u_sq),
                    );
                    let denom = Expr::Call("sqrt".to_string(), vec![one_minus]);
                    let neg_du = Expr::Unary(UnaryOp::Neg, Box::new(du));
                    let d = Expr::Binary(Box::new(neg_du), BinaryOp::Div, Box::new(denom));
                    simplify(&d)
                }
                "atan" => {
                    let u_sq = Expr::Binary(
                        Box::new(u.clone()),
                        BinaryOp::Pow,
                        Box::new(Expr::Number(NumericValue::from_i64(2))),
                    );
                    let one_plus = Expr::Binary(
                        Box::new(Expr::Number(NumericValue::from_i64(1))),
                        BinaryOp::Add,
                        Box::new(u_sq),
                    );
                    let d = Expr::Binary(Box::new(du), BinaryOp::Div, Box::new(one_plus));
                    simplify(&d)
                }
                _ => Expr::Call(format!("diff_{}", name), vec![u.clone(), du]),
            }
        }
        _ => Expr::Number(NumericValue::from_i64(0)),
    }
}
