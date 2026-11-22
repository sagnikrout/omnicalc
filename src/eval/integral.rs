use super::Evaluator;
use crate::ast::Expr;
use crate::numeric::NumericValue;
use crate::quantity::Quantity;

impl Evaluator {
    pub(crate) fn eval_integral(&mut self, args: &[Expr]) -> Result<String, String> {
        // integrate(expr; lower; upper) or integrate(expr, lower, upper)
        // or integrate(expr; var; lower; upper) or integrate(expr, var, lower, upper)
        let (expr, var, lower_e, upper_e) = if args.len() == 4 {
            let v_str = match &args[1] {
                Expr::Unit(s) | Expr::Variable(s) => s.clone(),
                _ => "x".to_string(),
            };
            (&args[0], v_str, &args[2], &args[3])
        } else if args.len() == 3 {
            let mut detected_vars = Vec::new();
            crate::symbolic::collect_variables(&args[0], &mut detected_vars);
            let v_str = if detected_vars.contains(&"x".to_string()) {
                "x".to_string()
            } else if let Some(first) = detected_vars.into_iter().next() {
                first
            } else {
                "x".to_string()
            };
            (&args[0], v_str, &args[1], &args[2])
        } else {
            return Err(
                "Integration requires 3 or 4 arguments: (expr, [var,] lower, upper) or (expr; [var;] lower; upper)".to_string(),
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

    pub(crate) fn eval_expr_at_var(
        &mut self,
        expr: &Expr,
        var: &str,
        val: f64,
    ) -> Result<f64, String> {
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
