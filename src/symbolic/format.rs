use crate::ast::{BinaryOp, Expr, UnaryOp};

pub fn format_symbolic(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => format!("{}", n),
        Expr::Variable(v) => v.clone(),
        Expr::Unit(u) => u.clone(),
        Expr::Unary(UnaryOp::Neg, inner) => format!("-{}", format_symbolic(inner)),
        Expr::Binary(lhs, BinaryOp::Mul, rhs) => {
            // Check for concise coefficient format e.g. 12x
            if let Expr::Number(n) = &**lhs
                && let Expr::Variable(v) | Expr::Unit(v) = &**rhs
            {
                return format!("{}{}", n, v);
            }
            format!("{} * {}", format_symbolic(lhs), format_symbolic(rhs))
        }
        Expr::Binary(lhs, op, rhs) => {
            format!("{} {} {}", format_symbolic(lhs), op, format_symbolic(rhs))
        }
        Expr::Call(name, args) => {
            let formatted_args: Vec<String> = args.iter().map(format_symbolic).collect();
            format!("{}({})", name, formatted_args.join(", "))
        }
        Expr::List(list) => {
            let elements: Vec<String> = list.iter().map(format_symbolic).collect();
            format!("[{}]", elements.join(", "))
        }
        _ => format!("{:?}", expr),
    }
}
