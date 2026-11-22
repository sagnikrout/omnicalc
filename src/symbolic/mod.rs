pub mod diff;
pub mod factor;
pub mod format;
pub mod simplify;
pub mod solver;

pub use diff::diff;
pub use factor::factor_polynomial;
pub use format::format_symbolic;
pub use simplify::{has_variable, simplify};
pub use solver::{detect_equation_variable, find_numerical_root, solve_equation};
