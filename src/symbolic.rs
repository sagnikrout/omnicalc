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
            simplify(&Expr::Binary(Box::new(term1), BinaryOp::Add, Box::new(term2)))
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
                    Box::new(Expr::Binary(lhs.clone(), BinaryOp::Pow, Box::new(simplify(&p_minus_1)))),
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
                    let cos_sq = Expr::Binary(Box::new(cos_u), BinaryOp::Pow, Box::new(Expr::Number(NumericValue::from_i64(2))));
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
                _ => Expr::Call(format!("diff_{}", name), vec![u.clone(), du]),
            }
        }
        _ => Expr::Number(NumericValue::from_i64(0)),
    }
}

pub fn has_variable(expr: &Expr, var: &str) -> bool {
    match expr {
        Expr::Variable(v) | Expr::Unit(v) => v == var,
        Expr::Number(_) => false,
        Expr::Unary(_, inner) => has_variable(inner, var),
        Expr::Binary(lhs, _, rhs) => has_variable(lhs, var) || has_variable(rhs, var),
        Expr::Call(_, args) => args.iter().any(|a| has_variable(a, var)),
        Expr::Convert { expr, .. } | Expr::ToBase { expr, .. } => has_variable(expr, var),
        Expr::Uncertainty { nominal, error } => has_variable(nominal, var) || has_variable(error, var),
        Expr::Equation { lhs, rhs } => has_variable(lhs, var) || has_variable(rhs, var),
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
                (Expr::Number(a), BinaryOp::Div, Expr::Number(b)) if !b.is_zero() => Expr::Number(a / b),
                (Expr::Number(a), BinaryOp::Pow, Expr::Number(b)) => Expr::Number(a.pow(&b)),

                // Algebraic identities: Addition
                (Expr::Number(n), BinaryOp::Add, other) if n.is_zero() => other,
                (other, BinaryOp::Add, Expr::Number(n)) if n.is_zero() => other,

                // Algebraic identities: Subtraction
                (other, BinaryOp::Sub, Expr::Number(n)) if n.is_zero() => other,
                (l, BinaryOp::Sub, r) if l == r => Expr::Number(NumericValue::from_i64(0)),

                // Algebraic identities: Multiplication
                (Expr::Number(n), BinaryOp::Mul, _) if n.is_zero() => Expr::Number(NumericValue::from_i64(0)),
                (_, BinaryOp::Mul, Expr::Number(n)) if n.is_zero() => Expr::Number(NumericValue::from_i64(0)),
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
                        Expr::Binary(Box::new(Expr::Number(c1)), BinaryOp::Mul, Box::new(Expr::Binary(box_l, BinaryOp::Mul, box_r)))
                    }
                }

                // Power identities
                (_, BinaryOp::Pow, Expr::Number(n)) if n.is_zero() => Expr::Number(NumericValue::from_i64(1)),
                (other, BinaryOp::Pow, Expr::Number(n)) if n.is_one() => other,

                (l, op, r) => Expr::Binary(Box::new(l), op, Box::new(r)),
            }
        }
        _ => expr.clone(),
    }
}

pub fn format_symbolic(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => format!("{}", n),
        Expr::Variable(v) => v.clone(),
        Expr::Unit(u) => u.clone(),
        Expr::Unary(UnaryOp::Neg, inner) => format!("-{}", format_symbolic(inner)),
        Expr::Binary(lhs, BinaryOp::Mul, rhs) => {
            // Check for concise coefficient format e.g. 12x
            if let Expr::Number(n) = &**lhs {
                if let Expr::Variable(v) | Expr::Unit(v) = &**rhs {
                    return format!("{}{}", n, v);
                }
            }
            format!("{} * {}", format_symbolic(lhs), format_symbolic(rhs))
        }
        Expr::Binary(lhs, op, rhs) => {
            format!("{} {} {}", format_symbolic(lhs), op, format_symbolic(rhs))
        }
        Expr::Call(name, args) => {
            let formatted_args: Vec<String> = args.iter().map(format_symbolic).collect();
            format!("{}({})", name, formatted_args.join(", "))
        }
        _ => format!("{:?}", expr),
    }
}

/// Solves an equation for a variable.
/// Handles linear forms ax + b = c and quadratic forms ax^2 + bx + c = 0.
pub fn solve_equation(lhs: &Expr, rhs: &Expr, var: &str) -> Result<String, String> {
    // Equation: lhs - rhs = 0
    let diff_expr = simplify(&Expr::Binary(
        Box::new(lhs.clone()),
        BinaryOp::Sub,
        Box::new(rhs.clone()),
    ));

    // Linear check: diff(diff(diff_expr, var), var) == 0
    let d1 = simplify(&diff(&diff_expr, var));
    let d2 = simplify(&diff(&d1, var));

    if let Expr::Number(n) = &d2 {
        if n.is_zero() {
            // Linear equation: a*x + b = 0  => x = -b / a
            // a = d1
            if let Expr::Number(a) = d1 {
                if a.is_zero() {
                    return Err("Equation has no unique solution (degenerate).".to_string());
                }
                // Evaluate diff_expr at var = 0 to get b
                let b = eval_with_var(&diff_expr, var, 0.0);
                let x_val = -b / a.to_f64();
                return Ok(format!("{} = {}", var, x_val));
            }
        } else {
            // Exact quadratic equation: ax^2 + bx + c = 0
            let a = n.to_f64() / 2.0;
            let b = eval_with_var(&d1, var, 0.0);
            let c = eval_with_var(&diff_expr, var, 0.0);
            let disc = b * b - 4.0 * a * c;
            if disc < 0.0 {
                return Err("No real solution (discriminant < 0)".to_string());
            } else if disc.abs() < 1e-12 {
                let r = -b / (2.0 * a);
                return Ok(format!("{} = {}", var, r));
            } else {
                let r1 = (-b + disc.sqrt()) / (2.0 * a);
                let r2 = (-b - disc.sqrt()) / (2.0 * a);
                return Ok(format!("{} = {}, {}", var, r1, r2));
            }
        }
    }

    // Try numerical root
    let root = find_numerical_root(&diff_expr, var, 1.0)?;
    Ok(format!("{} ≈ {:.6}", var, root))
}

fn eval_with_var(expr: &Expr, var: &str, val: f64) -> f64 {
    match expr {
        Expr::Number(n) => n.to_f64(),
        Expr::Variable(v) | Expr::Unit(v) if v == var => val,
        Expr::Unary(UnaryOp::Neg, inner) => -eval_with_var(inner, var, val),
        Expr::Binary(lhs, BinaryOp::Add, rhs) => eval_with_var(lhs, var, val) + eval_with_var(rhs, var, val),
        Expr::Binary(lhs, BinaryOp::Sub, rhs) => eval_with_var(lhs, var, val) - eval_with_var(rhs, var, val),
        Expr::Binary(lhs, BinaryOp::Mul, rhs) => eval_with_var(lhs, var, val) * eval_with_var(rhs, var, val),
        Expr::Binary(lhs, BinaryOp::Div, rhs) => eval_with_var(lhs, var, val) / eval_with_var(rhs, var, val),
        Expr::Binary(lhs, BinaryOp::Pow, rhs) => eval_with_var(lhs, var, val).powf(eval_with_var(rhs, var, val)),
        Expr::Call(name, args) => {
            let eval_args: Vec<f64> = args.iter().map(|a| eval_with_var(a, var, val)).collect();
            match name.as_str() {
                "sin" if eval_args.len() == 1 => eval_args[0].sin(),
                "cos" if eval_args.len() == 1 => eval_args[0].cos(),
                "tan" if eval_args.len() == 1 => eval_args[0].tan(),
                "asin" if eval_args.len() == 1 => eval_args[0].asin(),
                "acos" if eval_args.len() == 1 => eval_args[0].acos(),
                "atan" if eval_args.len() == 1 => eval_args[0].atan(),
                "exp" if eval_args.len() == 1 => eval_args[0].exp(),
                "ln" if eval_args.len() == 1 => eval_args[0].ln(),
                "log" | "log10" if eval_args.len() == 1 => eval_args[0].log10(),
                "sqrt" if eval_args.len() == 1 => eval_args[0].sqrt(),
                "cbrt" if eval_args.len() == 1 => eval_args[0].cbrt(),
                "abs" if eval_args.len() == 1 => eval_args[0].abs(),
                _ => 0.0,
            }
        }
        _ => 0.0,
    }
}

pub fn find_numerical_root(expr: &Expr, var: &str, init_guess: f64) -> Result<f64, String> {
    let mut x = init_guess;
    let d_expr = diff(expr, var);

    for _ in 0..100 {
        let fx = eval_with_var(expr, var, x);
        if fx.abs() < 1e-12 {
            return Ok(if x.abs() < 1e-10 { 0.0 } else { x });
        }
        let dfx = eval_with_var(&d_expr, var, x);
        if dfx.abs() < 1e-15 {
            x += 0.1; // perturbation
            continue;
        }
        let next_x = x - fx / dfx;
        if (next_x - x).abs() < 1e-12 {
            let final_check = eval_with_var(expr, var, next_x);
            if final_check.abs() < 1e-5 {
                return Ok(if next_x.abs() < 1e-10 { 0.0 } else { next_x });
            }
        }
        x = next_x;
    }
    let final_fx = eval_with_var(expr, var, x);
    if final_fx.abs() < 1e-5 {
        Ok(if x.abs() < 1e-10 { 0.0 } else { x })
    } else {
        Err("No real solution found".to_string())
    }
}

/// Factors simple polynomials or expressions (e.g. x^2 - 9 -> (x - 3)(x + 3))
pub fn factor_polynomial(expr: &Expr, var: &str) -> String {
    // If it's a constant integer, compute prime factorization
    if let Expr::Number(n) = expr {
        let val_f = n.to_f64();
        if val_f > 1.0 && val_f.fract() == 0.0 && val_f < 1e12 {
            let mut num = val_f as u64;
            let mut factors: Vec<(u64, u32)> = Vec::new();
            let mut d = 2;
            while d * d <= num {
                if num % d == 0 {
                    let mut count = 0;
                    while num % d == 0 {
                        count += 1;
                        num /= d;
                    }
                    factors.push((d, count));
                }
                d += if d == 2 { 1 } else { 2 };
            }
            if num > 1 {
                factors.push((num, 1));
            }
            if factors.len() == 1 && factors[0].1 == 1 {
                return format!("{}", factors[0].0);
            }
            let factor_strs: Vec<String> = factors
                .iter()
                .map(|(p, c)| if *c == 1 { format!("{}", p) } else { format!("{}^{}", p, c) })
                .collect();
            return factor_strs.join(" * ");
        }
    }

    // If it's quadratic: ax^2 + bx + c
    let d1 = simplify(&diff(expr, var));
    let d2 = simplify(&diff(&d1, var));
    if let Expr::Number(a2) = d2 {
        let a = a2.to_f64() / 2.0;
        if a != 0.0 {
            // Find b = d1(0)
            let b = eval_with_var(&d1, var, 0.0);
            let c = eval_with_var(expr, var, 0.0);
            let disc = b * b - 4.0 * a * c;
            if disc >= 0.0 {
                let r1 = (-b + disc.sqrt()) / (2.0 * a);
                let r2 = (-b - disc.sqrt()) / (2.0 * a);
                let fmt_r = |r: f64| -> String {
                    if r > 0.0 {
                        format!("({} - {})", var, r)
                    } else if r < 0.0 {
                        format!("({} + {})", var, -r)
                    } else {
                        var.to_string()
                    }
                };
                if (r1 - r2).abs() < 1e-10 {
                    return format!("{}^2", fmt_r(r1));
                } else if a == 1.0 {
                    return format!("{}{}", fmt_r(r1), fmt_r(r2));
                } else {
                    return format!("{}{}{}", a, fmt_r(r1), fmt_r(r2));
                }
            }
        }
    }
    format_symbolic(expr)
}
