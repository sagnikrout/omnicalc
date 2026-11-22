use super::diff::diff;
use super::simplify::simplify;
use crate::ast::{BinaryOp, Expr, UnaryOp};

pub fn detect_equation_variable(lhs: &Expr, rhs: &Expr) -> String {
    let mut vars = Vec::new();
    collect_variables(lhs, &mut vars);
    collect_variables(rhs, &mut vars);
    if vars.contains(&"x".to_string()) {
        "x".to_string()
    } else if let Some(first) = vars.into_iter().next() {
        first
    } else {
        "x".to_string()
    }
}

fn collect_variables(expr: &Expr, vars: &mut Vec<String>) {
    match expr {
        Expr::Variable(v) => {
            if !vars.contains(v) {
                vars.push(v.clone());
            }
        }
        Expr::Unit(u) => {
            if !matches!(u.as_str(), "pi" | "tau" | "phi" | "e" | "c" | "i") && !vars.contains(u) {
                vars.push(u.clone());
            }
        }
        Expr::Unary(_, inner) => collect_variables(inner, vars),
        Expr::Binary(lhs, _, rhs) => {
            collect_variables(lhs, vars);
            collect_variables(rhs, vars);
        }
        Expr::Call(_, args) => {
            for a in args {
                collect_variables(a, vars);
            }
        }
        Expr::Convert { expr, .. } | Expr::ToBase { expr, .. } => collect_variables(expr, vars),
        Expr::Uncertainty { nominal, error } => {
            collect_variables(nominal, vars);
            collect_variables(error, vars);
        }
        Expr::Equation { lhs, rhs } => {
            collect_variables(lhs, vars);
            collect_variables(rhs, vars);
        }
        _ => {}
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
                    let b = eval_with_var(&diff_expr, var, 0.0)?;
                    if b.abs() < 1e-12 {
                        return Ok(
                            "Identity: all real numbers satisfy this equation (0 = 0)".to_string()
                        );
                    } else {
                        return Err("Contradiction: equation has no solution".to_string());
                    }
                }
                // Evaluate diff_expr at var = 0 to get b
                let b = eval_with_var(&diff_expr, var, 0.0)?;
                let x_val = -b / a.to_f64();
                return Ok(format!("{} = {}", var, x_val));
            }
        } else {
            // Exact quadratic equation: ax^2 + bx + c = 0
            let a = n.to_f64() / 2.0;
            let b = eval_with_var(&d1, var, 0.0)?;
            let c = eval_with_var(&diff_expr, var, 0.0)?;
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

pub(crate) fn eval_with_var(expr: &Expr, var: &str, val: f64) -> Result<f64, String> {
    match expr {
        Expr::Number(n) => Ok(n.to_f64()),
        Expr::Variable(v) | Expr::Unit(v) if v == var => Ok(val),
        Expr::Variable(v) | Expr::Unit(v) => Err(format!("Unknown variable '{}' in evaluation", v)),
        Expr::Unary(UnaryOp::Neg, inner) => Ok(-eval_with_var(inner, var, val)?),
        Expr::Binary(lhs, BinaryOp::Add, rhs) => {
            Ok(eval_with_var(lhs, var, val)? + eval_with_var(rhs, var, val)?)
        }
        Expr::Binary(lhs, BinaryOp::Sub, rhs) => {
            Ok(eval_with_var(lhs, var, val)? - eval_with_var(rhs, var, val)?)
        }
        Expr::Binary(lhs, BinaryOp::Mul, rhs) => {
            Ok(eval_with_var(lhs, var, val)? * eval_with_var(rhs, var, val)?)
        }
        Expr::Binary(lhs, BinaryOp::Div, rhs) => {
            let d = eval_with_var(rhs, var, val)?;
            if d.abs() < 1e-15 {
                return Err("Division by zero in function evaluation".to_string());
            }
            Ok(eval_with_var(lhs, var, val)? / d)
        }
        Expr::Binary(lhs, BinaryOp::Pow, rhs) => {
            Ok(eval_with_var(lhs, var, val)?.powf(eval_with_var(rhs, var, val)?))
        }
        Expr::Call(name, args) => {
            let mut eval_args = Vec::with_capacity(args.len());
            for a in args {
                eval_args.push(eval_with_var(a, var, val)?);
            }
            match name.as_str() {
                "sin" if eval_args.len() == 1 => Ok(eval_args[0].sin()),
                "cos" if eval_args.len() == 1 => Ok(eval_args[0].cos()),
                "tan" if eval_args.len() == 1 => Ok(eval_args[0].tan()),
                "asin" if eval_args.len() == 1 => {
                    let x = eval_args[0];
                    if !(-1.0..=1.0).contains(&x) {
                        return Err(format!("Domain error in asin({})", x));
                    }
                    Ok(x.asin())
                }
                "acos" if eval_args.len() == 1 => {
                    let x = eval_args[0];
                    if !(-1.0..=1.0).contains(&x) {
                        return Err(format!("Domain error in acos({})", x));
                    }
                    Ok(x.acos())
                }
                "atan" if eval_args.len() == 1 => Ok(eval_args[0].atan()),
                "exp" if eval_args.len() == 1 => Ok(eval_args[0].exp()),
                "ln" if eval_args.len() == 1 => {
                    if eval_args[0] <= 0.0 {
                        return Err(format!("Domain error in ln({})", eval_args[0]));
                    }
                    Ok(eval_args[0].ln())
                }
                "log" | "log10" if eval_args.len() == 1 => {
                    if eval_args[0] <= 0.0 {
                        return Err(format!("Domain error in log10({})", eval_args[0]));
                    }
                    Ok(eval_args[0].log10())
                }
                "sqrt" if eval_args.len() == 1 => {
                    if eval_args[0] < 0.0 {
                        return Err(format!("Domain error in sqrt({})", eval_args[0]));
                    }
                    Ok(eval_args[0].sqrt())
                }
                "cbrt" if eval_args.len() == 1 => Ok(eval_args[0].cbrt()),
                "abs" if eval_args.len() == 1 => Ok(eval_args[0].abs()),
                _ => Err(format!("Unsupported function '{}' in root finder", name)),
            }
        }
        _ => Err(format!(
            "Unsupported expression node '{:?}' in equation evaluation",
            expr
        )),
    }
}

pub fn find_numerical_root(expr: &Expr, var: &str, init_guess: f64) -> Result<f64, String> {
    let mut x = init_guess;
    let d_expr = diff(expr, var);

    for _ in 0..100 {
        let fx = eval_with_var(expr, var, x)?;
        if fx.abs() < 1e-12 {
            return Ok(if x.abs() < 1e-10 { 0.0 } else { x });
        }
        let dfx = eval_with_var(&d_expr, var, x)?;
        if dfx.abs() < 1e-15 {
            x += 0.1; // perturbation
            continue;
        }
        let next_x = x - fx / dfx;
        if (next_x - x).abs() < 1e-12 {
            let final_check = eval_with_var(expr, var, next_x)?;
            if final_check.abs() < 1e-5 {
                return Ok(if next_x.abs() < 1e-10 { 0.0 } else { next_x });
            }
        }
        x = next_x;
    }
    let final_fx = eval_with_var(expr, var, x)?;
    if final_fx.abs() < 1e-5 {
        Ok(if x.abs() < 1e-10 { 0.0 } else { x })
    } else {
        Err("No real solution found".to_string())
    }
}
