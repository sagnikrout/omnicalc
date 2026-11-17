use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::dimension::Dimension;
use crate::numeric::NumericValue;
use crate::quantity::Quantity;
use crate::symbolic;
use crate::units::UnitRegistry;
use std::collections::HashMap;

pub struct Evaluator {
    pub registry: UnitRegistry,
    pub variables: HashMap<String, Quantity>,
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
                symbolic::solve_equation(lhs, rhs, "x")
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
            Expr::Call(name, args) if name == "integrate" => {
                self.eval_integral(args)
            }
            _ => {
                // If it contains unknown variables and isn't purely numerical, simplify symbolically
                if self.is_purely_symbolic(expr) {
                    let simp = symbolic::simplify(expr);
                    return Ok(symbolic::format_symbolic(&simp));
                }

                let q = self.eval_quantity(expr)?;
                Ok(q.format_plain())
            }
        }
    }

    pub fn eval_raw(&mut self, expr: &Expr) -> Result<String, String> {
        let q = self.eval_quantity(expr)?;
        Ok(q.format_raw())
    }

    fn is_purely_symbolic(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Unit(name) => {
                // If name is NOT a unit in registry, it's a symbolic variable
                self.registry.lookup(name).is_none() && !self.variables.contains_key(name)
            }
            Expr::Variable(_) => true,
            Expr::Binary(lhs, _, rhs) => self.is_purely_symbolic(lhs) || self.is_purely_symbolic(rhs),
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
                if nom_q.dimension != err_q.dimension && !err_q.dimension.is_dimensionless() {
                    return Err(format!(
                        "Uncertainty dimension mismatch: nominal ({}) vs error ({})",
                        nom_q.dimension, err_q.dimension
                    ));
                }
                let val = NumericValue::from_uncertainty(nom_q.value.to_f64(), err_q.value.to_f64());
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
                let int_val = q.value.to_f64() as i64;
                Ok(Quantity::dimensionless(NumericValue::from_i64(!int_val)))
            }
            Expr::Binary(lhs, op, rhs) => {
                if *op == BinaryOp::Mul {
                    if let Expr::Unit(name) = &**rhs {
                        if let Some(def) = self.registry.lookup(name) {
                            if def.offset != 0.0 {
                                let q_lhs = self.eval_quantity(lhs)?;
                                if q_lhs.dimension.is_dimensionless() {
                                    return Quantity::from_unit(q_lhs.value, name, &self.registry);
                                }
                            }
                        }
                    }
                    if let Expr::Unit(name) = &**lhs {
                        if let Some(def) = self.registry.lookup(name) {
                            if def.offset != 0.0 {
                                let q_rhs = self.eval_quantity(rhs)?;
                                if q_rhs.dimension.is_dimensionless() {
                                    return Quantity::from_unit(q_rhs.value, name, &self.registry);
                                }
                            }
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
                        let new_dim = q_lhs.dimension.pow(exp_f as i8);
                        let new_val = q_lhs.value.pow(&q_rhs.value);
                        Ok(Quantity {
                            value: new_val,
                            dimension: new_dim,
                            unit_str: None,
                        })
                    }
                    BinaryOp::BitAnd => {
                        let a = q_lhs.value.to_f64() as i64;
                        let b = q_rhs.value.to_f64() as i64;
                        Ok(Quantity::dimensionless(NumericValue::from_i64(a & b)))
                    }
                    BinaryOp::BitOr => {
                        let a = q_lhs.value.to_f64() as i64;
                        let b = q_rhs.value.to_f64() as i64;
                        Ok(Quantity::dimensionless(NumericValue::from_i64(a | b)))
                    }
                    BinaryOp::BitXor => {
                        let a = q_lhs.value.to_f64() as i64;
                        let b = q_rhs.value.to_f64() as i64;
                        Ok(Quantity::dimensionless(NumericValue::from_i64(a ^ b)))
                    }
                    BinaryOp::Shl => {
                        let a = q_lhs.value.to_f64() as i64;
                        let b = q_rhs.value.to_f64() as u32;
                        if b >= 64 {
                            return Err(format!("Bit shift overflow: shift amount {} exceeds maximum 63", b));
                        }
                        Ok(Quantity::dimensionless(NumericValue::from_i64(a << b)))
                    }
                    BinaryOp::Shr => {
                        let a = q_lhs.value.to_f64() as i64;
                        let b = q_rhs.value.to_f64() as u32;
                        if b >= 64 {
                            return Err(format!("Bit shift overflow: shift amount {} exceeds maximum 63", b));
                        }
                        Ok(Quantity::dimensionless(NumericValue::from_i64(a >> b)))
                    }
                }
            }
            Expr::Call(name, args) => self.eval_call(name, args),
            Expr::Convert { expr, target } => {
                let q = self.eval_quantity(expr)?;
                q.to_unit(target, &self.registry)
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
                let new_dim = Dimension {
                    length: first.dimension.length / 2,
                    mass: first.dimension.mass / 2,
                    time: first.dimension.time / 2,
                    current: first.dimension.current / 2,
                    temperature: first.dimension.temperature / 2,
                    amount: first.dimension.amount / 2,
                    luminosity: first.dimension.luminosity / 2,
                    digital: first.dimension.digital / 2,
                    currency: first.dimension.currency / 2,
                };
                Ok(Quantity {
                    value: first.value.sqrt(),
                    dimension: new_dim,
                    unit_str: None,
                })
            }
            "cbrt" => Ok(Quantity::dimensionless(NumericValue::from_f64(first.value.to_f64().cbrt()))),
            "sin" => Ok(Quantity::dimensionless(first.value.sin())),
            "cos" => Ok(Quantity::dimensionless(first.value.cos())),
            "tan" => Ok(Quantity::dimensionless(first.value.tan())),
            "asin" => {
                let x = first.value.to_f64();
                if !(-1.0..=1.0).contains(&x) {
                    return Err(format!("Domain error: argument to asin ({}) must be between -1 and 1", x));
                }
                Ok(Quantity::dimensionless(first.value.asin()))
            }
            "acos" => {
                let x = first.value.to_f64();
                if !(-1.0..=1.0).contains(&x) {
                    return Err(format!("Domain error: argument to acos ({}) must be between -1 and 1", x));
                }
                Ok(Quantity::dimensionless(first.value.acos()))
            }
            "atan" => Ok(Quantity::dimensionless(first.value.atan())),
            "exp" => Ok(Quantity::dimensionless(first.value.exp())),
            "ln" => Ok(Quantity::dimensionless(first.value.ln())),
            "log" | "log10" => Ok(Quantity::dimensionless(first.value.log10())),
            "abs" => Ok(Quantity {
                value: first.value.abs(),
                dimension: first.dimension,
                unit_str: first.unit_str.clone(),
            }),
            "normdist" => {
                // normdist(x, mean, std) or standard normdist(x)
                let x = first.value.to_f64();
                let (mean, std) = if evaluated_args.len() >= 3 {
                    (evaluated_args[1].value.to_f64(), evaluated_args[2].value.to_f64())
                } else {
                    (0.0, 1.0)
                };
                let z = (x - mean) / (std * std::f64::consts::SQRT_2);
                let cdf = 0.5 * (1.0 + erf(z));
                Ok(Quantity::dimensionless(NumericValue::from_f64(cdf)))
            }
            "round" => {
                let v = first.value.to_f64().round();
                Ok(Quantity::dimensionless(NumericValue::from_f64(v)))
            }
            "floor" => {
                let v = first.value.to_f64().floor();
                Ok(Quantity::dimensionless(NumericValue::from_f64(v)))
            }
            "ceil" => {
                let v = first.value.to_f64().ceil();
                Ok(Quantity::dimensionless(NumericValue::from_f64(v)))
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
            return Err("integrate requires 3 or 4 arguments: (expr; [var;] lower; upper)".to_string());
        };

        let a = self.eval_quantity(lower_e)?.value.to_f64();
        let b = self.eval_quantity(upper_e)?.value.to_f64();

        // Numerical integration via Composite Simpson's Rule with N = 1000 intervals
        let n = 1000;
        let h = (b - a) / (n as f64);

        let mut sum = self.eval_expr_at_var(expr, &var, a)?
            + self.eval_expr_at_var(expr, &var, b)?;

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
            Ok(format!("{:.6}", integral).trim_end_matches('0').trim_end_matches('.').to_string())
        }
    }

    fn eval_expr_at_var(&mut self, expr: &Expr, var: &str, val: f64) -> Result<f64, String> {
        self.variables.insert(var.to_string(), Quantity::dimensionless(NumericValue::from_f64(val)));
        let res = self.eval_quantity(expr)?;
        self.variables.remove(var);
        Ok(res.value.to_f64())
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
