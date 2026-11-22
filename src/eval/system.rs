#![allow(clippy::needless_range_loop)]

use super::Evaluator;
use super::la::format_scalar;
use crate::ast::Expr;
use crate::symbolic;

impl Evaluator {
    pub(crate) fn eval_solve(&mut self, args: &[Expr]) -> Result<String, String> {
        if args.is_empty() {
            return Err(
                "solve requires at least 1 argument: solve(eq[, var]) or solve([eqs], [vars])"
                    .to_string(),
            );
        }
        match args.len() {
            1 => match &args[0] {
                Expr::List(eqs) => {
                    let mut detected = Vec::new();
                    for eq in eqs {
                        symbolic::collect_variables(eq, &mut detected);
                    }
                    detected.sort();
                    detected.dedup();
                    solve_linear_system(eqs, &detected)
                }
                Expr::Equation { lhs, rhs } => {
                    let var = symbolic::detect_equation_variable(lhs, rhs);
                    symbolic::solve_equation(lhs, rhs, &var)
                }
                _ => Err("solve expects an equation or a list of equations".to_string()),
            },
            2 => match (&args[0], &args[1]) {
                (Expr::List(eqs), Expr::List(var_exprs)) => {
                    let mut vars = Vec::new();
                    for v in var_exprs {
                        match v {
                            Expr::Variable(s) | Expr::Unit(s) => vars.push(s.clone()),
                            _ => return Err(format!("Expected variable name, found {:?}", v)),
                        }
                    }
                    solve_linear_system(eqs, &vars)
                }
                (Expr::Equation { lhs, rhs }, Expr::Variable(v))
                | (Expr::Equation { lhs, rhs }, Expr::Unit(v)) => {
                    symbolic::solve_equation(lhs, rhs, v)
                }
                _ => Err(
                    "Invalid arguments to solve: expected solve([eqs], [vars]) or solve(eq, var)"
                        .to_string(),
                ),
            },
            _ => Err("solve expects 1 or 2 arguments".to_string()),
        }
    }
}

pub fn solve_linear_system(equations: &[Expr], vars: &[String]) -> Result<String, String> {
    let n = vars.len();
    let m = equations.len();
    if n == 0 {
        return Err("No variables specified for linear system".to_string());
    }
    if m == 0 {
        return Err("No equations provided to solve".to_string());
    }
    if m < n {
        return Err(format!(
            "Underdetermined system: {} equations for {} variables",
            m, n
        ));
    }

    let mut a = vec![vec![0.0; n]; m];
    let mut b = vec![0.0; m];

    let zeros = vec![0.0; n];
    for (i, eq) in equations.iter().enumerate() {
        let c0 = eval_expr_vars(eq, vars, &zeros)?;
        b[i] = -c0;
        for j in 0..n {
            let mut ej = vec![0.0; n];
            ej[j] = 1.0;
            let val_ej = eval_expr_vars(eq, vars, &ej)?;
            a[i][j] = val_ej - c0;
        }
        let ones = vec![1.0; n];
        let val_ones = eval_expr_vars(eq, vars, &ones)?;
        let expected: f64 = a[i].iter().sum::<f64>() + c0;
        if (val_ones - expected).abs() > 1e-5 {
            return Err(format!("Equation {} is non-linear", i + 1));
        }
    }

    let mut aug = vec![vec![0.0; n + 1]; m];
    for i in 0..m {
        for j in 0..n {
            aug[i][j] = a[i][j];
        }
        aug[i][n] = b[i];
    }

    for i in 0..n {
        let mut max_row = i;
        let mut max_val = aug[i][i].abs();
        for k in i + 1..m {
            if aug[k][i].abs() > max_val {
                max_val = aug[k][i].abs();
                max_row = k;
            }
        }
        if max_val < 1e-12 {
            for k in i..m {
                if aug[k][n].abs() > 1e-12 {
                    return Err("Inconsistent system: no solution exists".to_string());
                }
            }
            return Err("System is degenerate (infinitely many solutions or singular)".to_string());
        }
        if max_row != i {
            aug.swap(i, max_row);
        }
        let pivot = aug[i][i];
        for j in i..=n {
            aug[i][j] /= pivot;
        }
        for k in 0..m {
            if k != i {
                let factor = aug[k][i];
                for j in i..=n {
                    aug[k][j] -= factor * aug[i][j];
                }
            }
        }
    }

    for i in n..m {
        if aug[i][n].abs() > 1e-10 {
            return Err("Inconsistent system: no solution exists".to_string());
        }
    }

    let mut solutions = Vec::new();
    for (i, var) in vars.iter().enumerate() {
        let val = aug[i][n];
        solutions.push(format!("{} = {}", var, format_scalar(val)));
    }
    Ok(format!("[{}]", solutions.join(", ")))
}

fn eval_expr_vars(expr: &Expr, vars: &[String], vals: &[f64]) -> Result<f64, String> {
    match expr {
        Expr::Number(n) => Ok(n.to_f64()),
        Expr::Variable(v) | Expr::Unit(v) => {
            if let Some(idx) = vars.iter().position(|name| name == v) {
                Ok(vals[idx])
            } else {
                Err(format!("Unknown variable '{}' in linear system", v))
            }
        }
        Expr::Unary(crate::ast::UnaryOp::Neg, inner) => Ok(-eval_expr_vars(inner, vars, vals)?),
        Expr::Binary(lhs, crate::ast::BinaryOp::Add, rhs) => {
            Ok(eval_expr_vars(lhs, vars, vals)? + eval_expr_vars(rhs, vars, vals)?)
        }
        Expr::Binary(lhs, crate::ast::BinaryOp::Sub, rhs) => {
            Ok(eval_expr_vars(lhs, vars, vals)? - eval_expr_vars(rhs, vars, vals)?)
        }
        Expr::Binary(lhs, crate::ast::BinaryOp::Mul, rhs) => {
            Ok(eval_expr_vars(lhs, vars, vals)? * eval_expr_vars(rhs, vars, vals)?)
        }
        Expr::Binary(lhs, crate::ast::BinaryOp::Div, rhs) => {
            let d = eval_expr_vars(rhs, vars, vals)?;
            if d.abs() < 1e-12 {
                return Err("Division by zero in linear system".to_string());
            }
            Ok(eval_expr_vars(lhs, vars, vals)? / d)
        }
        Expr::Equation { lhs, rhs } => {
            Ok(eval_expr_vars(lhs, vars, vals)? - eval_expr_vars(rhs, vars, vals)?)
        }
        _ => Err(format!(
            "Unsupported expression in linear system: {:?}",
            expr
        )),
    }
}
