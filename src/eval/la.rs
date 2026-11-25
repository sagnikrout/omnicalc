use super::Evaluator;
use super::matrix::{eval_det, eval_inv, eval_transpose, la_add, la_mul, la_sub};
use super::vector::{eval_cross, eval_dot, eval_norm};
use crate::ast::Expr;
use crate::numeric::NumericValue;
use crate::quantity::Quantity;

fn extract_real_scalar(q: &Quantity) -> Result<f64, String> {
    if !q.dimension.is_dimensionless() {
        return Err(format!(
            "Vectors and matrices require dimensionless values (got [{}])",
            q.dimension
        ));
    }
    if matches!(q.value, NumericValue::Complex(_)) {
        return Err(
            "Vectors and matrices currently only support real numbers (got complex value)"
                .to_string(),
        );
    }
    let val = q.value.to_f64();
    if !val.is_finite() {
        return Err("Vectors and matrices require finite real numbers".to_string());
    }
    Ok(val)
}

#[derive(Debug, Clone, PartialEq)]
pub enum LaValue {
    Scalar(f64),
    Vector(Vec<f64>),
    Matrix(Vec<Vec<f64>>),
}

pub fn format_scalar(f: f64) -> String {
    if f.abs() < 1e-12 {
        "0".to_string()
    } else if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{:.0}", f)
    } else {
        let s = format!("{:.6}", f);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

impl LaValue {
    pub fn format(&self) -> String {
        match self {
            LaValue::Scalar(s) => format_scalar(*s),
            LaValue::Vector(v) => {
                let elems: Vec<String> = v.iter().map(|x| format_scalar(*x)).collect();
                format!("[{}]", elems.join(", "))
            }
            LaValue::Matrix(m) => {
                let rows: Vec<String> = m
                    .iter()
                    .map(|row| {
                        let elems: Vec<String> = row.iter().map(|x| format_scalar(*x)).collect();
                        format!("[{}]", elems.join(", "))
                    })
                    .collect();
                format!("[{}]", rows.join(", "))
            }
        }
    }
}

pub fn is_la_expr(expr: &Expr) -> bool {
    match expr {
        Expr::List(_) => true,
        Expr::Call(name, _) => {
            matches!(
                name.as_str(),
                "dot" | "cross" | "norm" | "det" | "inv" | "transpose"
            )
        }
        Expr::Unary(_, inner) => is_la_expr(inner),
        Expr::Binary(lhs, _, rhs) => is_la_expr(lhs) || is_la_expr(rhs),
        _ => false,
    }
}

impl Evaluator {
    pub fn eval_la(&mut self, expr: &Expr) -> Result<LaValue, String> {
        match expr {
            Expr::List(elements) => {
                if elements.is_empty() {
                    return Ok(LaValue::Vector(Vec::new()));
                }
                let vals: Vec<LaValue> = elements
                    .iter()
                    .map(|e| self.eval_la(e))
                    .collect::<Result<Vec<_>, _>>()?;

                let all_vectors = vals.iter().all(|v| matches!(v, LaValue::Vector(_)));
                if all_vectors {
                    let mut rows = Vec::new();
                    let expected_cols = if let LaValue::Vector(v) = &vals[0] {
                        v.len()
                    } else {
                        0
                    };
                    if expected_cols == 0 {
                        return Err("Empty matrix rows ([[]]) are not valid".to_string());
                    }
                    for v in vals {
                        if let LaValue::Vector(row) = v {
                            if row.len() != expected_cols {
                                return Err(format!(
                                    "Matrix row length mismatch: expected {} elements, got {}",
                                    expected_cols,
                                    row.len()
                                ));
                            }
                            rows.push(row);
                        }
                    }
                    Ok(LaValue::Matrix(rows))
                } else {
                    let mut vec = Vec::new();
                    for v in vals {
                        match v {
                            LaValue::Scalar(s) => vec.push(s),
                            _ => {
                                return Err(
                                    "Nested lists beyond 2D matrices are not supported".to_string()
                                );
                            }
                        }
                    }
                    Ok(LaValue::Vector(vec))
                }
            }
            Expr::Call(name, args) => match name.as_str() {
                "dot" => {
                    if args.len() != 2 {
                        return Err(format!("dot expects 2 arguments (got {})", args.len()));
                    }
                    let u = self.eval_la_vector(&args[0])?;
                    let v = self.eval_la_vector(&args[1])?;
                    let val = eval_dot(&u, &v)?;
                    Ok(LaValue::Scalar(val))
                }
                "cross" => {
                    if args.len() != 2 {
                        return Err(format!("cross expects 2 arguments (got {})", args.len()));
                    }
                    let u = self.eval_la_vector(&args[0])?;
                    let v = self.eval_la_vector(&args[1])?;
                    let val = eval_cross(&u, &v)?;
                    Ok(LaValue::Vector(val))
                }
                "norm" => {
                    if args.len() != 1 {
                        return Err(format!("norm expects 1 argument (got {})", args.len()));
                    }
                    match self.eval_la(&args[0])? {
                        LaValue::Vector(v) => {
                            let val = eval_norm(&v)?;
                            Ok(LaValue::Scalar(val))
                        }
                        LaValue::Scalar(s) => Ok(LaValue::Scalar(s.abs())),
                        LaValue::Matrix(_) => {
                            Err("norm is only defined for vectors and scalars".to_string())
                        }
                    }
                }
                "det" => {
                    if args.len() != 1 {
                        return Err(format!("det expects 1 argument (got {})", args.len()));
                    }
                    let m = self.eval_la_matrix(&args[0])?;
                    let val = eval_det(&m)?;
                    Ok(LaValue::Scalar(val))
                }
                "inv" => {
                    if args.len() != 1 {
                        return Err(format!("inv expects 1 argument (got {})", args.len()));
                    }
                    let m = self.eval_la_matrix(&args[0])?;
                    let inv_m = eval_inv(&m)?;
                    Ok(LaValue::Matrix(inv_m))
                }
                "transpose" => {
                    if args.len() != 1 {
                        return Err(format!("transpose expects 1 argument (got {})", args.len()));
                    }
                    match self.eval_la(&args[0])? {
                        LaValue::Matrix(m) => Ok(LaValue::Matrix(eval_transpose(&m))),
                        LaValue::Vector(v) => {
                            if v.is_empty() {
                                return Err("Cannot transpose an empty vector".to_string());
                            }
                            Ok(LaValue::Matrix(vec![v]))
                        }
                        LaValue::Scalar(s) => Ok(LaValue::Scalar(s)),
                    }
                }
                _ => {
                    let q = self.eval_quantity(expr)?;
                    Ok(LaValue::Scalar(extract_real_scalar(&q)?))
                }
            },
            Expr::Unary(crate::ast::UnaryOp::Neg, inner) => match self.eval_la(inner)? {
                LaValue::Scalar(s) => Ok(LaValue::Scalar(-s)),
                LaValue::Vector(v) => Ok(LaValue::Vector(v.into_iter().map(|x| -x).collect())),
                LaValue::Matrix(m) => Ok(LaValue::Matrix(
                    m.into_iter()
                        .map(|row| row.into_iter().map(|x| -x).collect())
                        .collect(),
                )),
            },
            Expr::Unary(crate::ast::UnaryOp::Not, _) => {
                Err("Bitwise NOT is not defined for vectors and matrices".to_string())
            }
            Expr::Binary(lhs, op, rhs) => {
                let left = self.eval_la(lhs)?;
                let right = self.eval_la(rhs)?;
                match op {
                    crate::ast::BinaryOp::Add => la_add(left, right),
                    crate::ast::BinaryOp::Sub => la_sub(left, right),
                    crate::ast::BinaryOp::Mul => la_mul(left, right),
                    _ => Err(format!(
                        "Operation '{}' is not supported between linear algebra values",
                        op
                    )),
                }
            }
            _ => {
                let q = self.eval_quantity(expr)?;
                Ok(LaValue::Scalar(extract_real_scalar(&q)?))
            }
        }
    }

    pub(crate) fn eval_la_vector(&mut self, expr: &Expr) -> Result<Vec<f64>, String> {
        match self.eval_la(expr)? {
            LaValue::Vector(v) => Ok(v),
            LaValue::Scalar(_) => Err("Expected vector, got scalar".to_string()),
            LaValue::Matrix(_) => Err("Expected vector, got matrix".to_string()),
        }
    }

    pub(crate) fn eval_la_matrix(&mut self, expr: &Expr) -> Result<Vec<Vec<f64>>, String> {
        match self.eval_la(expr)? {
            LaValue::Matrix(m) => Ok(m),
            LaValue::Scalar(_) => Err("Expected matrix, got scalar".to_string()),
            LaValue::Vector(_) => Err("Expected matrix, got vector".to_string()),
        }
    }
}
