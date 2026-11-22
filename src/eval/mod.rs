use crate::ast::Expr;
use crate::quantity::Quantity;
use crate::symbolic;
use crate::units::UnitRegistry;
use std::collections::HashMap;

pub mod functions;
pub mod helpers;
pub mod integral;
pub mod quantity;

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

    pub(crate) fn is_purely_symbolic(&self, expr: &Expr) -> bool {
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
}
