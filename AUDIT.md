# Repository audit (2026-11-22)

## Fixes applied on `audit/fix-evaluator-safety`

- `package.sh` no longer recursively deletes the existing `dist` directory or copies a build into the developer-specific `/home/sagnik/bin` path. It checks its Linux/x86_64 prerequisites, stages artifacts in a temporary directory, and only publishes the named package artifacts after successful builds.
- The generated installer now installs relative to its own location and handles a writable, initially non-existent install prefix correctly.
- `benchmark_comparison.py` no longer depends on paths under `/home/sagnik`; it discovers installed tools on `PATH` or accepts explicit `*_BIN` overrides, skips unavailable calculators, and reports command failures instead of silently treating them as valid measurements. RSS is optional when GNU `time` is unavailable.

## Important correctness issues still needing targeted fixes

The source review found additional calculator issues that should be resolved separately and covered by focused regression tests:

1. `src/eval.rs`: `sqrt`, trigonometric, logarithmic, rounding, and statistics functions generally accept invalid dimensions and ignore extra arguments. `diff` and `factor` access `args[0]` without checking arity, so direct construction of an empty call AST can panic. `eval_integral` changes the variable environment and fails to restore an existing binding when evaluation returns an error.
2. `src/eval.rs` / `src/numeric.rs`: non-integer dimensional exponents are silently truncated when converting the exponent to `i8`; large exponents can overflow dimension exponents. The evaluator's exponent cap is not consistently enforced for every numeric representation. `NumericValue::to_bigint` truncates fractional values and saturates floating conversions, making bitwise operations surprising rather than rejecting non-integers.
3. `src/quantity.rs`: arithmetic on affine temperatures needs explicit semantics (for example, adding two Celsius temperatures should not use ordinary offset-unit addition). Conversions and unit-bearing operations also need broader tests for uncertainty and complex values.
4. `src/symbolic.rs`: equation solving assumes the default variable is `x`, approximates polynomial coefficients through `f64`, and does not reliably distinguish no-solution / infinitely-many-solution equations. The numerical root solver can return misleading results for unsupported expression nodes.
5. `src/lexer.rs` / `src/parser.rs`: malformed radix literals can tokenize as empty values, and number separators are accepted in invalid positions. Parser recursion limits should be stress-tested against deeply nested function calls and unary operators.
6. `src/units.rs`: the registry contains hard-coded currency rates described as a 2026 baseline; these are not live rates and should be clearly labeled with a dated source or removed from claims of accurate conversion. Unit aliases and derived-unit equivalences need systematic tests.
7. `destructive_test.sh`: the advertised test count is not derived from executed cases; its `eval`-based generator and hard-coded default binary path make it less portable than the Rust test suite. Prefer Rust integration/unit tests for reproducible CI coverage.
8. README badges and performance/test claims are static and benchmark scripts rely on a particular environment. Treat them as unverified until produced by CI on a documented machine.

## Verification

The GitHub file-editing environment does not provide a local checkout or a Rust build runner, so `cargo test`, formatting, shell checks, and benchmark execution were not run for these changes. Run `cargo fmt --check`, `cargo test`, `bash -n package.sh`, and `python3 -m py_compile benchmark_comparison.py` after checking out this branch.
