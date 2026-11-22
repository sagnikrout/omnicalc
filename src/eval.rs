use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::dimension::Dimension;
use crate::numeric::NumericValue;
use crate::quantity::Quantity;
use crate::symbolic;
use crate::units::UnitRegistry;
use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};
use std::collections::HashMap;

pub struct Evaluator {
    pub registry: UnitRegistry,
    pub variables: HashMap<String, Quantity>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            registry: UnitRegistry::new(),
            variables: HashMap::new(),
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Equation { lhs, rhs } => {
                let var = symbolic::detect_equation_variable(lhs, rhs);
                symbolic::solve_equation(lhs, rhs, &var)
            }
            Expr::ToBase { expr, base } => {
                let q = self.eval_quantity(expr)?;
                Ok(q.value.format_with_base(*base, false))
            }
            Expr::Convert { expr, target } => {
                let q = self.eval_quantity(expr)?;
                let converted = q.to_unit(target, &self.registry)?;
                Ok(converted.format_plain())
            }
            Expr::Call(name, args) if name == "diff" => {
                if args.is_empty() {
                    return Err(
                        "Function 'diff' requires at least 1 argument: diff(expr[, var])"
                            .to_string(),
                    );
                }
                let var = if args.len() > 1 {
                    if let Expr::Unit(v) = &args[1] {
                        v.as_str()
                    } else if let Expr::Variable(v) = &args[1] {
                        v.as_str()
                    } else {
                        "x"
                    }
                } else {
                    "x"
                };
                let derivative = symbolic::diff(&args[0], var);
                let simplified = symbolic::simplify(&derivative);
                Ok(symbolic::format_symbolic(&simplified))
            }
            Expr::Call(name, args) if name == "factor" => {
                if args.is_empty() {
                    return Err(
                        "Function 'factor' requires at least 1 argument: factor(expr[, var])"
                            .to_string(),
                    );
                }
                let var = if args.len() > 1 {
                    if let Expr::Unit(v) = &args[1] {
                        v.as_str()
                    } else if let Expr::Variable(v) = &args[1] {
                        v.as_str()
                    } else {
                        "x"
                    }
                } else {
                    "x"
                };
                Ok(symbolic::factor_polynomial(&args[0], var))
            }
            Expr::Call(name, args) if name == "integrate" => self.eval_integral(args),
            _ => {
                // If it contains unknown variables and isn't purely numerical, simplify symbolically
                if self.is_purely_symbolic(expr) {
                    let simp = symbolic::simplify(expr);
                    return Ok(symbolic::format_symbolic(&simp));
                }

                let q = self.eval_quantity(expr)?;
                if let Some(ref u) = q.unit_str
                    && let Ok(converted) = q.to_unit(u, &self.registry)
                {
                    return Ok(converted.format_plain());
                }
                Ok(q.format_with_registry(&self.registry))
            }
        }
    }

    pub fn eval_raw(&mut self, expr: &Expr, base: u32) -> Result<String, String> {
        let q = self.eval_quantity(expr)?;
        Ok(q.value.format_with_base(base, true))
    }

    fn is_purely_symbolic(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Unit(name) => {
                // If name is NOT a unit in registry, it's a symbolic variable
                self.registry.lookup(name).is_none() && !self.variables.contains_key(name)
            }
            Expr::Variable(_) => true,
            Expr::Binary(lhs, _, rhs) => {
                self.is_purely_symbolic(lhs) || self.is_purely_symbolic(rhs)
            }
            Expr::Unary(_, inner) => self.is_purely_symbolic(inner),
            _ => false,
        }
    }

    pub fn eval_quantity(&mut self, expr: &Expr) -> Result<Quantity, String> {
        match expr {
            Expr::Number(n) => Ok(Quantity::dimensionless(n.clone())),
            Expr::Unit(name) => {
                if let Some(var_q) = self.variables.get(name) {
                    return Ok(var_q.clone());
                }
                Quantity::from_unit(NumericValue::from_i64(1), name, &self.registry)
            }
            Expr::Variable(name) => {
                if let Some(var_q) = self.variables.get(name) {
                    Ok(var_q.clone())
                } else {
                    Err(format!("Undefined variable: '{}'", name))
                }
            }
            Expr::Uncertainty { nominal, error } => {
                let nom_q = self.eval_quantity(nominal)?;
                let err_q = self.eval_quantity(error)?;
                if nom_q.dimension != err_q.dimension {
                    if nom_q.dimension.is_dimensionless() && !err_q.dimension.is_dimensionless() {
                        let val = NumericValue::from_uncertainty(
                            nom_q.value.to_f64(),
                            err_q.value.to_f64(),
                        );
                        return Ok(Quantity {
                            value: val,
                            dimension: err_q.dimension,
                            unit_str: err_q.unit_str,
                        });
                    } else if !err_q.dimension.is_dimensionless() {
                        return Err(format!(
                            "Uncertainty dimension mismatch: nominal ({}) vs error ({})",
                            nom_q.dimension, err_q.dimension
                        ));
                    }
                }
                let val =
                    NumericValue::from_uncertainty(nom_q.value.to_f64(), err_q.value.to_f64());
                Ok(Quantity {
                    value: val,
                    dimension: nom_q.dimension,
                    unit_str: nom_q.unit_str,
                })
            }
            Expr::Unary(UnaryOp::Neg, inner) => {
                let q = self.eval_quantity(inner)?;
                Ok(-q)
            }
            Expr::Unary(UnaryOp::Not, inner) => {
                let q = self.eval_quantity(inner)?;
                if !q.dimension.is_dimensionless() {
                    return Err("Bitwise NOT requires dimensionless operand".to_string());
                }
                let bi = q.value.to_exact_bigint().ok_or_else(|| {
                    format!("Bitwise NOT requires integer operand (got {})", q.value)
                })?;
                Ok(Quantity::dimensionless(NumericValue::from_bigint(!bi)))
            }
            Expr::Binary(lhs, op, rhs) => {
                if *op == BinaryOp::Mul {
                    if let Expr::Unit(name) = &**rhs
                        && let Some(_def) = self.registry.lookup(name)
                    {
                        let q_lhs = self.eval_quantity(lhs)?;
                        if q_lhs.dimension.is_dimensionless() {
                            return Quantity::from_unit(q_lhs.value, name, &self.registry);
                        }
                    }
                    if let Expr::Unit(name) = &**lhs
                        && let Some(_def) = self.registry.lookup(name)
                    {
                        let q_rhs = self.eval_quantity(rhs)?;
                        if q_rhs.dimension.is_dimensionless() {
                            return Quantity::from_unit(q_rhs.value, name, &self.registry);
                        }
                    }
                }

                let q_lhs = self.eval_quantity(lhs)?;
                let q_rhs = self.eval_quantity(rhs)?;

                match op {
                    BinaryOp::Add => q_lhs + q_rhs,
                    BinaryOp::Sub => q_lhs - q_rhs,
                    BinaryOp::Mul => Ok(q_lhs * q_rhs),
                    BinaryOp::Div => {
                        if q_rhs.value.is_zero() {
                            return Err("Division by zero".to_string());
                        }
                        Ok(q_lhs / q_rhs)
                    }
                    BinaryOp::Rem => {
                        if q_rhs.value.is_zero() {
                            return Err("Division by zero in modulus".to_string());
                        }
                        let res = q_lhs.value % q_rhs.value;
                        Ok(Quantity::dimensionless(res))
                    }
                    BinaryOp::Pow => {
                        if !q_rhs.dimension.is_dimensionless() {
                            return Err("Exponent must be dimensionless".to_string());
                        }
                        let exp_f = q_rhs.value.to_f64();
                        if q_lhs.value.is_zero() && exp_f < 0.0 {
                            return Err("Division by zero in power".to_string());
                        }
                        if !q_lhs.value.is_one() && !q_lhs.value.is_zero() {
                            if exp_f.abs() > 100_000.0 || exp_f.is_infinite() || exp_f.is_nan() {
                                return Err(format!(
                                    "Exponent too large: exponent {} exceeds 100,000",
                                    exp_f
                                ));
                            }
                            if let Some(bi) = q_rhs.value.to_exact_bigint()
                                && bi.abs() > BigInt::from(100_000)
                            {
                                return Err(format!(
                                    "Exponent too large: exponent {} exceeds 100,000",
                                    bi
                                ));
                            }
                        }
                        let new_dim = if !q_lhs.dimension.is_dimensionless() {
                            if exp_f.fract() != 0.0 {
                                return Err(format!(
                                    "Non-integer dimensional exponent {} is invalid without root",
                                    exp_f
                                ));
                            }
                            if exp_f > 127.0 || exp_f < -128.0 {
                                return Err(format!(
                                    "Dimensional exponent overflow: {} exceeds 8-bit dimension range",
                                    exp_f
                                ));
                            }
                            let exp_i8 = exp_f as i8;
                            let dims = [
                                q_lhs.dimension.length,
                                q_lhs.dimension.mass,
                                q_lhs.dimension.time,
                                q_lhs.dimension.current,
                                q_lhs.dimension.temperature,
                                q_lhs.dimension.amount,
                                q_lhs.dimension.luminosity,
                                q_lhs.dimension.digital,
                                q_lhs.dimension.currency,
                            ];
                            for d in dims {
                                if d.checked_mul(exp_i8).is_none() {
                                    return Err("Dimensional exponent overflow in dimension power"
                                        .to_string());
                                }
                            }
                            q_lhs.dimension.pow(exp_i8)
                        } else {
                            Dimension::DIMENSIONLESS
                        };
                        let new_val = q_lhs.value.pow(&q_rhs.value);
                        Ok(Quantity {
                            value: new_val,
                            dimension: new_dim,
                            unit_str: None,
                        })
                    }
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                        if !q_lhs.dimension.is_dimensionless()
                            || !q_rhs.dimension.is_dimensionless()
                        {
                            return Err(
                                "Bitwise operations require dimensionless operands".to_string()
                            );
                        }
                        let a = q_lhs.value.to_exact_bigint().ok_or_else(|| {
                            format!(
                                "Bitwise operations require integer operands (got {})",
                                q_lhs.value
                            )
                        })?;
                        let b = q_rhs.value.to_exact_bigint().ok_or_else(|| {
                            format!(
                                "Bitwise operations require integer operands (got {})",
                                q_rhs.value
                            )
                        })?;
                        let res = match op {
                            BinaryOp::BitAnd => a & b,
                            BinaryOp::BitOr => a | b,
                            BinaryOp::BitXor => a ^ b,
                            _ => unreachable!(),
                        };
                        Ok(Quantity::dimensionless(NumericValue::from_bigint(res)))
                    }
                    BinaryOp::Shl | BinaryOp::Shr => {
                        if !q_lhs.dimension.is_dimensionless()
                            || !q_rhs.dimension.is_dimensionless()
                        {
                            return Err(
                                "Bit shift operations require dimensionless operands".to_string()
                            );
                        }
                        let a = q_lhs.value.to_exact_bigint().ok_or_else(|| {
                            format!(
                                "Bitwise operations require integer operands (got {})",
                                q_lhs.value
                            )
                        })?;
                        let b = q_rhs.value.to_exact_bigint().ok_or_else(|| {
                            format!(
                                "Bitwise operations require integer operands (got {})",
                                q_rhs.value
                            )
                        })?;
                        if b.is_negative() {
                            return Err(format!("Negative shift amount {} is invalid", b));
                        }
                        if b >= BigInt::from(64) {
                            return Err(format!(
                                "Bit shift overflow: shift amount {} exceeds maximum 63",
                                b
                            ));
                        }
                        let shift: usize = num_traits::ToPrimitive::to_usize(&b).unwrap();
                        let res = match op {
                            BinaryOp::Shl => a << shift,
                            BinaryOp::Shr => a >> shift,
                            _ => unreachable!(),
                        };
                        Ok(Quantity::dimensionless(NumericValue::from_bigint(res)))
                    }
                }
            }
            Expr::Call(name, args) => self.eval_call(name, args),
            Expr::Convert { expr, target } => {
                let q = self.eval_quantity(expr)?;
                let target_def = self
                    .registry
                    .lookup(target)
                    .ok_or_else(|| format!("Unknown target unit: '{}'", target))?;
                if q.dimension != target_def.dimension {
                    return Err(format!(
                        "Dimensional mismatch: cannot convert {} ({}) to '{}' ({})",
                        q.format_plain(),
                        q.dimension,
                        target,
                        target_def.dimension
                    ));
                }
                Ok(Quantity {
                    value: q.value,
                    dimension: target_def.dimension,
                    unit_str: Some(target_def.symbol),
                })
            }
            Expr::ToBase { expr, .. } => self.eval_quantity(expr),
            Expr::Equation { .. } => Err("Equations cannot be evaluated as quantities".to_string()),
        }
    }

    fn eval_call(&mut self, name: &str, args: &[Expr]) -> Result<Quantity, String> {
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
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'sin' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'sin' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                Ok(Quantity::dimensionless(first.value.sin()))
            }
            "cos" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'cos' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'cos' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                Ok(Quantity::dimensionless(first.value.cos()))
            }
            "tan" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'tan' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'tan' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                Ok(Quantity::dimensionless(first.value.tan()))
            }
            "asin" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'asin' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'asin' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                let x = first.value.to_f64();
                if !(-1.0..=1.0).contains(&x) {
                    return Err(format!(
                        "Domain error: argument to asin ({}) must be between -1 and 1",
                        x
                    ));
                }
                Ok(Quantity::dimensionless(first.value.asin()))
            }
            "acos" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'acos' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'acos' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                let x = first.value.to_f64();
                if !(-1.0..=1.0).contains(&x) {
                    return Err(format!(
                        "Domain error: argument to acos ({}) must be between -1 and 1",
                        x
                    ));
                }
                Ok(Quantity::dimensionless(first.value.acos()))
            }
            "atan" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'atan' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'atan' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                Ok(Quantity::dimensionless(first.value.atan()))
            }
            "exp" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'exp' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'exp' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                Ok(Quantity::dimensionless(first.value.exp()))
            }
            "ln" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'ln' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'ln' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                Ok(Quantity::dimensionless(first.value.ln()))
            }
            "log" | "log10" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function '{}' expects exactly 1 argument (got {})",
                        name,
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to '{}' must be dimensionless (got [{}])",
                        name, first.dimension
                    ));
                }
                Ok(Quantity::dimensionless(first.value.log10()))
            }
            "log2" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'log2' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to 'log2' must be dimensionless (got [{}])",
                        first.dimension
                    ));
                }
                Ok(Quantity::dimensionless(first.value.log2()))
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
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function '{}' expects exactly 1 argument (got {})",
                        name,
                        evaluated_args.len()
                    ));
                }
                if !first.dimension.is_dimensionless() {
                    return Err(format!(
                        "Domain error: argument to '{}' must be dimensionless (got [{}])",
                        name, first.dimension
                    ));
                }
                let n_f = first.value.to_f64();
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
            "round" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'round' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                let v = first.value.to_f64().round();
                Ok(Quantity {
                    value: NumericValue::from_f64(v),
                    dimension: first.dimension,
                    unit_str: first.unit_str.clone(),
                })
            }
            "floor" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'floor' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                let v = first.value.to_f64().floor();
                Ok(Quantity {
                    value: NumericValue::from_f64(v),
                    dimension: first.dimension,
                    unit_str: first.unit_str.clone(),
                })
            }
            "ceil" => {
                if evaluated_args.len() != 1 {
                    return Err(format!(
                        "Function 'ceil' expects exactly 1 argument (got {})",
                        evaluated_args.len()
                    ));
                }
                let v = first.value.to_f64().ceil();
                Ok(Quantity {
                    value: NumericValue::from_f64(v),
                    dimension: first.dimension,
                    unit_str: first.unit_str.clone(),
                })
            }
            _ => Err(format!("Unknown function: '{}'", name)),
        }
    }

    fn eval_integral(&mut self, args: &[Expr]) -> Result<String, String> {
        // integrate(expr; lower; upper) or integrate(expr; var; lower; upper)
        let (expr, var, lower_e, upper_e) = if args.len() == 4 {
            let v_str = match &args[1] {
                Expr::Unit(s) | Expr::Variable(s) => s.clone(),
                _ => "x".to_string(),
            };
            (&args[0], v_str, &args[2], &args[3])
        } else if args.len() == 3 {
            (&args[0], "x".to_string(), &args[1], &args[2])
        } else {
            return Err(
                "integrate requires 3 or 4 arguments: (expr; [var;] lower; upper)".to_string(),
            );
        };

        let a = self.eval_quantity(lower_e)?.value.to_f64();
        let b = self.eval_quantity(upper_e)?.value.to_f64();

        // Numerical integration via Composite Simpson's Rule with N = 1000 intervals
        let n = 1000;
        let h = (b - a) / (n as f64);

        let mut sum =
            self.eval_expr_at_var(expr, &var, a)? + self.eval_expr_at_var(expr, &var, b)?;

        for i in 1..n {
            let x_i = a + (i as f64) * h;
            let val = self.eval_expr_at_var(expr, &var, x_i)?;
            if i % 2 == 0 {
                sum += 2.0 * val;
            } else {
                sum += 4.0 * val;
            }
        }

        let integral = sum * (h / 3.0);
        if integral.fract().abs() < 1e-10 {
            Ok(format!("{:.0}", integral))
        } else {
            Ok(format!("{:.6}", integral)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string())
        }
    }

    fn eval_expr_at_var(&mut self, expr: &Expr, var: &str, val: f64) -> Result<f64, String> {
        let prev = self.variables.insert(
            var.to_string(),
            Quantity::dimensionless(NumericValue::from_f64(val)),
        );
        let res = self.eval_quantity(expr);
        match prev {
            Some(old) => {
                self.variables.insert(var.to_string(), old);
            }
            None => {
                self.variables.remove(var);
            }
        }
        let q = res?;
        Ok(q.value.to_f64())
    }
}

/// Standard error function approximation
fn erf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation formula 7.1.26
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = x.abs();

    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();

    sign * y
}

fn gcd_bigint(mut a: BigInt, mut b: BigInt) -> BigInt {
    while !b.is_zero() {
        let t = b.clone();
        b = a % b;
        a = t;
    }
    a.abs()
}

fn lcm_bigint(a: BigInt, b: BigInt) -> BigInt {
    if a.is_zero() || b.is_zero() {
        BigInt::zero()
    } else {
        let g = gcd_bigint(a.clone(), b.clone());
        ((a / g) * b).abs()
    }
}
