# Repository audit (2026-11-22)

## Fixes applied on `audit/fix-evaluator-safety` (Merged via PR #1)

- `package.sh` no longer recursively deletes the existing `dist` directory or copies a build into the developer-specific `/home/sagnik/bin` path. It checks its Linux/x86_64 prerequisites, stages artifacts in a temporary directory, and only publishes the named package artifacts after successful builds.
- The generated installer now installs relative to its own location and handles a writable, initially non-existent install prefix correctly.
- `benchmark_comparison.py` no longer depends on paths under `/home/sagnik`; it discovers installed tools on `PATH` or accepts explicit `*_BIN` overrides, skips unavailable calculators, and reports command failures instead of silently treating them as valid measurements. RSS is optional when GNU `time` is unavailable.

## Evaluator Hardening and Correctness Fixes Applied

All remaining correctness and evaluator safety vectors identified in the audit have now been addressed with comprehensive regression test coverage:

1. `src/eval.rs`:
   - Enforced strict arity and dimensionless guards on `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `exp`, `ln`, `log`, `log10`, `log2`, `fact`, `sqrt`, `cbrt`, `round`, `floor`, and `ceil`. Extra or invalid arguments are rejected with descriptive errors.
   - Fixed panic risk on empty call ASTs: `diff` and `factor` explicitly validate `!args.is_empty()` before inspecting parameters.
   - Fixed variable environment leakage: `eval_integral` / `eval_expr_at_var` saves and restores prior variable bindings unconditionally, guaranteeing state cleanup even when evaluation returns an error.
2. `src/eval.rs` / `src/numeric.rs`:
   - Implemented `NumericValue::to_exact_bigint(&self)` to reject non-integer floats and prevent lossy float truncations.
   - Enforced dimensionless integer validation on bitwise operators (`&`, `|`, `^`, `<<`, `>>`, `~`) and integer utility functions (`gcd`, `lcm`).
   - Hardened `BinaryOp::Pow`: rejects non-integer exponents on dimensioned quantities; guards against 8-bit dimension exponent overflow; enforces the 100,000 exponent limit across all numeric representations.
3. `src/quantity.rs`:
   - Affine temperature arithmetic: rejecting addition of two absolute affine temperatures (`degC + degC`). Subtracting two absolute temperatures (`20 degC - 10 degC`) yields a temperature difference in Kelvin (`10 K`). Adding/subtracting temperature differences (`20 degC + 5 K`) preserves affine scale (`25 degC`).
4. `src/symbolic.rs`:
   - Variable auto-detection: `detect_equation_variable` dynamically extracts the target variable from equation ASTs instead of assuming `"x"`.
   - Linear equation degeneracy: cleanly distinguishes identity equations ($0 = 0$, all real numbers satisfy) from contradictory equations ($0 = 5$, no solution).
   - Numerical root safety: `eval_with_var` returns `Result<f64, String>`, propagating domain errors and rejecting unsupported AST nodes rather than silently returning `0.0`.
5. `src/lexer.rs`:
   - Malformed radix prefixes (`0x`, `0b`, `0o`) followed by no valid digits are rejected at lexer time with descriptive error messages.
6. `src/units.rs`:
   - Explicitly documented that offline baseline currency rates reflect reference values (dated November 2026) for dimensional estimation, not live financial market data.
7. `destructive_test.sh`:
   - Portable binary discovery: dynamically checks `OMNICALC_BIN`, `./target/release/omnicalc`, `./target/debug/omnicalc`, `PATH`, and `$HOME/bin/omnicalc`.
   - Expanded test suite with 8 new audit verification tests (`TC-AUDIT-01` through `TC-AUDIT-08`).
8. CI Automation:
   - Added GitHub Actions workflow (`.github/workflows/ci.yml`) executing `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --verbose`, release build, script verification, and the master destructive test suite.

## Verification

Run the verification battery:
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --verbose
./destructive_test.sh
bash -n package.sh
python3 -m py_compile benchmark_comparison.py
```
