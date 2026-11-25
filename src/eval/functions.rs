use super::Evaluator;
use super::helpers::{erf, gcd_bigint, lcm_bigint};
use crate::ast::Expr;
use crate::dimension::Dimension;
use crate::numeric::NumericValue;
use crate::quantity::Quantity;
use num_bigint::BigInt;
use num_traits::One;

fn check_single_dimensionless<'a>(
    name: &str,
    args: &'a [Quantity],
) -> Result<&'a Quantity, String> {
    if args.len() != 1 {
        return Err(format!(
            "Function '{}' expects exactly 1 argument (got {})",
            name,
            args.len()
        ));
    }
    if !args[0].dimension.is_dimensionless() {
        return Err(format!(
            "Domain error: argument to '{}' must be dimensionless (got [{}])",
            name, args[0].dimension
        ));
    }
    Ok(&args[0])
}

impl Evaluator {
    pub(crate) fn eval_call(&mut self, name: &str, args: &[Expr]) -> Result<Quantity, String> {
        if args.is_empty() {
            return Err(format!("Function '{}' requires arguments", name));
        }

        let evaluated_args: Vec<Quantity> = args
            .iter()
            .map(|a| self.eval_quantity(a))
            .collect::<Result<Vec<_>, _>>()?;

        let first = &evaluated_args[0];

        match name {
            "sqrt" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'sqrt' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_divisible_by(2) {
                    return Err(format!(
                        "Dimensional error: square root of dimension [{}] is not divisible by 2",
                        first.dimension
                    ));
                }
                let new_dim = first.dimension.div_scalar(2);
                let unit_str = if first.dimension.is_dimensionless() {
                    first.unit_str.clone()
                } else if new_dim == Dimension::LENGTH && first.unit_str.as_deref() == Some("m^2") {
                    Some("m".to_string())
                } else {
                    None
                };
                Ok(Quantity {
                    value: first.value.sqrt(),
                    dimension: new_dim,
                    unit_str,
                })
            }
            "cbrt" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'cbrt' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_divisible_by(3) {
                    return Err(format!(
                        "Dimensional error: cube root of dimension [{}] is not divisible by 3",
                        first.dimension
                    ));
                }
                let new_dim = first.dimension.div_scalar(3);
                let unit_str = if first.dimension.is_dimensionless() {
                    first.unit_str.clone()
                } else if new_dim == Dimension::LENGTH && first.unit_str.as_deref() == Some("m^3") {
                    Some("m".to_string())
                } else {
                    None
                };
                Ok(Quantity {
                    value: first.value.cbrt(),
                    dimension: new_dim,
                    unit_str,
                })
            }
            "sin" => {
                let q = check_single_dimensionless("sin", &evaluated_args)?;
                Ok(Quantity::dimensionless(q.value.sin()))
            }
            "cos" => {
                let q = check_single_dimensionless("cos", &evaluated_args)?;
                Ok(Quantity::dimensionless(q.value.cos()))
            }
            "tan" => {
                let q = check_single_dimensionless("tan", &evaluated_args)?;
                Ok(Quantity::dimensionless(q.value.tan()))
            }
            "asin" => {
                let q = check_single_dimensionless("asin", &evaluated_args)?;
                let x = q.value.to_f64();
                if !(-1.0..=1.0).contains(&x) {
                    return Err(format!(
                        "Domain error: argument to asin ({}) must be between -1 and 1",
                        x
                    ));
                }
                Ok(Quantity::dimensionless(q.value.asin()))
            }
            "acos" => {
                let q = check_single_dimensionless("acos", &evaluated_args)?;
                let x = q.value.to_f64();
                if !(-1.0..=1.0).contains(&x) {
                    return Err(format!(
                        "Domain error: argument to acos ({}) must be between -1 and 1",
                        x
                    ));
                }
                Ok(Quantity::dimensionless(q.value.acos()))
            }
            "atan" => {
                let q = check_single_dimensionless("atan", &evaluated_args)?;
                Ok(Quantity::dimensionless(q.value.atan()))
            }
            "exp" => {
                let q = check_single_dimensionless("exp", &evaluated_args)?;
                Ok(Quantity::dimensionless(q.value.exp()))
            }
            "ln" => {
                let q = check_single_dimensionless("ln", &evaluated_args)?;
                Ok(Quantity::dimensionless(q.value.ln()))
            }
            "log" | "log10" => {
                let q = check_single_dimensionless(name, &evaluated_args)?;
                Ok(Quantity::dimensionless(q.value.log10()))
            }
            "log2" => {
                let q = check_single_dimensionless("log2", &evaluated_args)?;
                Ok(Quantity::dimensionless(q.value.log2()))
            }
            "abs" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'abs' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                Ok(Quantity {
                    value: first.value.abs(),
                    dimension: first.dimension,
                    unit_str: first.unit_str.clone(),
                })
            }
            "fact" | "factorial" => {
                let q = check_single_dimensionless(name, &evaluated_args)?;
                let n_f = q.value.to_f64();
                if n_f < 0.0 || n_f.fract() != 0.0 || n_f > 1000.0 {
                    return Err("Factorial requires non-negative integer <= 1000".to_string());
                }
                let mut res = BigInt::one();
                let n = n_f as u64;
                for i in 2..=n {
                    res *= BigInt::from(i);
                }
                Ok(Quantity::dimensionless(NumericValue::from_bigint(res)))
            }
            "gcd" => {
                if evaluated_args.len() < 2 {
                    return Err("gcd requires at least 2 arguments".to_string());
                }
                for arg in &evaluated_args {
                    if !arg.dimension.is_dimensionless() {
                        return Err(format!(
                            "gcd arguments must be dimensionless (got [{}])",
                            arg.dimension
                        ));
                    }
                }
                let mut res = evaluated_args[0].value.to_exact_bigint().ok_or_else(|| {
                    format!(
                        "gcd requires integer arguments (got {})",
                        evaluated_args[0].value
                    )
                })?;
                for arg in &evaluated_args[1..] {
                    let b = arg.value.to_exact_bigint().ok_or_else(|| {
                        format!("gcd requires integer arguments (got {})", arg.value)
                    })?;
                    res = gcd_bigint(res, b);
                }
                Ok(Quantity::dimensionless(NumericValue::from_bigint(res)))
            }
            "lcm" => {
                if evaluated_args.len() < 2 {
                    return Err("lcm requires at least 2 arguments".to_string());
                }
                for arg in &evaluated_args {
                    if !arg.dimension.is_dimensionless() {
                        return Err(format!(
                            "lcm arguments must be dimensionless (got [{}])",
                            arg.dimension
                        ));
                    }
                }
                let mut res = evaluated_args[0].value.to_exact_bigint().ok_or_else(|| {
                    format!(
                        "lcm requires integer arguments (got {})",
                        evaluated_args[0].value
                    )
                })?;
                for arg in &evaluated_args[1..] {
                    let b = arg.value.to_exact_bigint().ok_or_else(|| {
                        format!("lcm requires integer arguments (got {})", arg.value)
                    })?;
                    res = lcm_bigint(res, b);
                }
                Ok(Quantity::dimensionless(NumericValue::from_bigint(res)))
            }
            "normdist" => {
                if evaluated_args.len() != 1 && evaluated_args.len() != 3 {
                    return Err(format!(
                        "normdist requires 1 or 3 arguments: normdist(x[, mean, std]) (got {})",
                        evaluated_args.len()
                    ));
                }
                for arg in &evaluated_args {
                    if !arg.dimension.is_dimensionless() {
                        return Err(format!(
                            "normdist arguments must be dimensionless (got [{}])",
                            arg.dimension
                        ));
                    }
                }
                let x = first.value.to_f64();
                let (mean, std) = if evaluated_args.len() >= 3 {
                    (
                        evaluated_args[1].value.to_f64(),
                        evaluated_args[2].value.to_f64(),
                    )
                } else {
                    (0.0, 1.0)
                };
                if std <= 0.0 {
                    return Err(format!("Standard deviation must be positive (got {})", std));
                }
                let z = (x - mean) / (std * std::f64::consts::SQRT_2);
                let cdf = 0.5 * (1.0 + erf(z));
                Ok(Quantity::dimensionless(NumericValue::from_f64(cdf)))
            }
            "round" | "floor" | "ceil" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function '{}' expects exactly 1 argument (got {})",
                        name,
                        evaluated_args.len()
                    ));
                }
                super::helpers::round_quantity(first, name, &self.registry)
            }
            _ => Err(format!("Unknown function: '{}'", name)),
        }
    }
}
