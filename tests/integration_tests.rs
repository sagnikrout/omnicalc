use omnicalc::eval::Evaluator;
use omnicalc::lexer::Lexer;
use omnicalc::parser::Parser;

fn eval_expr(input: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    let mut evaluator = Evaluator::new();
    evaluator.eval(&ast)
}

#[test]
fn test_bandwidth_conversion() {
    let res = eval_expr("1 megabit/s * 30 day to gibibyte").unwrap();
    assert!(res.contains("301.7"));
    assert!(res.contains("gibibyte"));
}

#[test]
fn test_currency_conversion() {
    let res = eval_expr("1 EUR to USD").unwrap();
    assert_eq!(res, "1.038 USD");
}

#[test]
fn test_uncertainty_propagation() {
    let res = eval_expr("(2±0.02 J)/(523±5 W) to ms").unwrap();
    assert!(res.contains("3.824"));
    assert!(res.contains("0.052"));
    assert!(res.ends_with("ms"));
}

#[test]
fn test_symbolic_differentiation() {
    let res = eval_expr("diff(6x^2)").unwrap();
    assert_eq!(res, "12x");
}

#[test]
fn test_numerical_integration() {
    let res = eval_expr("integrate(6x^2; 1; 5)").unwrap();
    assert_eq!(res, "248");
}

#[test]
fn test_base_conversion() {
    let res = eval_expr("52 to hex").unwrap();
    assert_eq!(res, "0x34");
}

#[test]
fn test_numbat_dimensional_safety() {
    let res = eval_expr("15 km/h * 30 min to km").unwrap();
    assert_eq!(res, "7.5 km");

    let err = eval_expr("5 m + 3 s").unwrap_err();
    assert!(err.contains("Dimensional mismatch"));
}

#[test]
fn test_fend_ergonomics() {
    let res1 = eval_expr("5'10\" to cm").unwrap();
    assert_eq!(res1, "177.8 cm");

    let res2 = eval_expr("5% of 100").unwrap();
    assert_eq!(res2, "5");

    let res3 = eval_expr("(5 V) / (2 A) to ohm").unwrap();
    assert_eq!(res3, "2.5 ohm");
}

#[test]
fn test_equation_solving_and_factoring() {
    let eq_res = eval_expr("2x + 4 = 10").unwrap();
    assert_eq!(eq_res, "x = 3");

    let factor_res = eval_expr("factor(x^2 - 9)").unwrap();
    assert_eq!(factor_res, "(x - 3)(x + 3)");
}

#[test]
fn test_temperature_affine_conversions() {
    let res1 = eval_expr("100 degC to degF").unwrap();
    assert_eq!(res1, "212 degF");

    let res2 = eval_expr("0 degC to degF").unwrap();
    assert_eq!(res2, "32 degF");

    let res3 = eval_expr("32 degF to degC").unwrap();
    assert_eq!(res3, "0 degC");

    let res4 = eval_expr("-40 degC to degF").unwrap();
    assert_eq!(res4, "-40 degF");

    let res5 = eval_expr("-273.15 degC to K").unwrap();
    assert_eq!(res5, "0 K");
}

#[test]
fn test_complex_multiplication() {
    let res1 = eval_expr("i * i").unwrap();
    assert_eq!(res1, "-1");

    let res2 = eval_expr("i^2").unwrap();
    assert_eq!(res2, "-1");

    let res3 = eval_expr("sqrt(-4)").unwrap();
    assert_eq!(res3, "2i");
}

#[test]
fn test_bitwise_xor_and_precedence() {
    let res1 = eval_expr("0xFF ^^ 0x0F").unwrap();
    assert_eq!(res1, "240");

    let res2 = eval_expr("0xFF xor 0x0F").unwrap();
    assert_eq!(res2, "240");

    let res3 = eval_expr("2^10").unwrap();
    assert_eq!(res3, "1024");
}

#[test]
fn test_unit_implicit_precedence() {
    let res = eval_expr("100 J / 10 s to W").unwrap();
    assert_eq!(res, "10 W");
}

#[test]
fn test_inch_and_conversion_disambiguation() {
    let res1 = eval_expr("12 in to ft").unwrap();
    assert_eq!(res1, "1 ft");

    let res2 = eval_expr("1 in to cm").unwrap();
    assert_eq!(res2, "2.54 cm");

    let res3 = eval_expr("5 km in m").unwrap();
    assert_eq!(res3, "5000 m");
}

#[test]
fn test_edge_case_errors() {
    assert!(eval_expr("10 % 0").is_err());
    assert!(eval_expr("asin(2)").is_err());
    assert!(eval_expr("x^2 = -1").is_err());
    assert!(eval_expr("1 << 64").is_err());
}

#[test]
fn test_affine_temperature_safety() {
    // Subtraction of two absolute temperatures yields difference in Kelvin
    let diff = eval_expr("20 degC - 10 degC").unwrap();
    assert_eq!(diff, "10 K");

    // Adding two absolute temperatures is physically invalid
    let add_err = eval_expr("10 degC + 20 degC").unwrap_err();
    assert!(add_err.contains("Cannot add two affine temperatures"));

    // Adding/subtracting temperature difference to absolute temperature
    let add_diff = eval_expr("20 degC + 5 K").unwrap();
    assert_eq!(add_diff, "25 degC");

    let sub_diff = eval_expr("20 degC - 5 K").unwrap();
    assert_eq!(sub_diff, "15 degC");

    // Subtracting absolute temperature from difference is invalid
    assert!(eval_expr("5 K - 20 degC").is_err());
}

#[test]
fn test_bitwise_integer_and_dimension_validation() {
    // Rejects non-integers
    let err1 = eval_expr("2.5 & 3").unwrap_err();
    assert!(err1.contains("Bitwise operations require integer operands"));

    // Rejects dimensioned quantities
    let err2 = eval_expr("5 m & 3 m").unwrap_err();
    assert!(err2.contains("Bitwise operations require dimensionless operands"));

    // Bitwise NOT rejects floats and dimensioned values
    assert!(eval_expr("~2.5").is_err());
    assert!(eval_expr("~5 m").is_err());
}

#[test]
fn test_malformed_radix_rejection() {
    assert!(eval_expr("0x").is_err());
    assert!(eval_expr("0b").is_err());
    assert!(eval_expr("0o").is_err());
}

#[test]
fn test_evaluator_function_arity_and_dimensions() {
    // Mathematical functions require dimensionless arguments
    let err_sin = eval_expr("sin(5 m)").unwrap_err();
    assert!(err_sin.contains("Domain error"));

    let err_sqrt = eval_expr("sqrt(5 m)").unwrap_err();
    assert!(err_sqrt.contains("Dimensional error"));

    // Excess arguments rejected
    assert!(eval_expr("sin(1, 2)").is_err());
    assert!(eval_expr("cos(1, 2)").is_err());
    assert!(eval_expr("sqrt(4, 2)").is_err());

    // Empty AST calls rejected without panicking
    assert!(eval_expr("diff()").is_err());
    assert!(eval_expr("factor()").is_err());
}

#[test]
fn test_equation_solver_degeneracy_and_variables() {
    // Identity equation
    let id_res = eval_expr("x = x").unwrap();
    assert!(id_res.contains("Identity"));

    // Contradiction equation
    let contra_err = eval_expr("x = x + 5").unwrap_err();
    assert!(contra_err.contains("Contradiction"));

    // Variable auto-detection for non-x variables
    let y_res = eval_expr("2y + 4 = 10").unwrap();
    assert_eq!(y_res, "y = 3");
}

#[test]
fn test_dimensional_power_safety() {
    // Fractional exponent on dimensioned quantity rejected
    let err_frac = eval_expr("(5 m)^2.5").unwrap_err();
    assert!(err_frac.contains("Non-integer dimensional exponent"));

    // Exponent overflow rejected
    let err_over = eval_expr("(5 m)^130").unwrap_err();
    assert!(err_over.contains("Dimensional exponent overflow"));
}
