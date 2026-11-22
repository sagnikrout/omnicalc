use super::diff::diff;
use super::format::format_symbolic;
use super::simplify::simplify;
use super::solver::eval_with_var;
use crate::ast::Expr;

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
                if num.is_multiple_of(d) {
                    let mut count = 0;
                    while num.is_multiple_of(d) {
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
                .map(|(p, c)| {
                    if *c == 1 {
                        format!("{}", p)
                    } else {
                        format!("{}^{}", p, c)
                    }
                })
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
            if let (Ok(b), Ok(c)) = (eval_with_var(&d1, var, 0.0), eval_with_var(expr, var, 0.0)) {
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
    }
    format_symbolic(expr)
}
