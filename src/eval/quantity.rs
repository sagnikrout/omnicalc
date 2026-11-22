use super::Evaluator;
use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::dimension::Dimension;
use crate::numeric::NumericValue;
use crate::quantity::Quantity;
use num_bigint::BigInt;
use num_traits::Signed;

impl Evaluator {
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
            Expr::List(_) => Err("Cannot evaluate list as scalar quantity".to_string()),
        }
    }
}
