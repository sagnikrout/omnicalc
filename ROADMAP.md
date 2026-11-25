# Master Engineering Roadmap & Milestones

**Project:** `omnicalc`  
**Repository:** `/home/sagnik/code/omnicalc`  
**Target Goal:** The Definitive Scientific CLI Calculator  

---

## 1. Architectural Invariants

Every milestone must maintain these performance and safety thresholds:

| Invariant Metric | Maximum Threshold | Verification Method |
| :--- | :--- | :--- |
| **Startup Latency** | $\le 1.2\text{ ms}$ | 50-run benchmark loop against `/home/sagnik/bin/omnicalc` |
| **Peak Memory RSS** | $\le 5.0\text{ MB}$ | Linux `/usr/bin/time -v` resident set size inspection |
| **Binary Size** | $\le 2.0\text{ MB}$ | Stripped release binary inspection |
| **Offline Safety** | 100% Offline by Default | Zero network calls during standard evaluation |
| **Panic Freedom** | 0 Unhandled Panics | 100% pass on destructive boundary batteries |

---

## 2. Milestone Breakdown

### Milestone 1: Core Foundation & Boundary Hardening (v0.1.0 - Complete)
* [x] Pure-Rust lexer, Pratt parser, and AST engine.
* [x] Exact rational arithmetic (`num_rational::BigRational`) and arbitrary integers (`num_bigint::BigInt`).
* [x] SI dimensional analysis with 200+ physical units.
* [x] Affine temperature conversion (`degC`, `degF`, `K`).
* [x] Deterministic offline currency conversion (USD, EUR, GBP, JPY, INR, CAD, AUD, CHF, CNY).
* [x] Symbolic single-variable differentiation (`diff(x^3, x)`).
* [x] Single-variable algebraic and transcendental equation solver (`sin(x) = 0`, `x^3 - 27 = 0`).
* [x] Absolute Gaussian uncertainty propagation (`(10 +/- 0.5) * (2 +/- 0.1)`).
* [x] Bitwise operations and radix formatting (`hex`, `bin`, `oct`).
* [x] Master regression defense (85 catalog tests, 172 v2 destructive tests, 270 v1 destructive tests).
* [x] Linux x86_64 standalone tarball and Debian `.deb` packaging.

---

### Milestone 1.5: Evaluator Correctness, Hardening & Microlithic Architecture (Complete — Commits `99f0651`, `9a0b10d`, `a35075e`)
* [x] **Evaluator/API arity and dimension validation**: Strict arity and dimensionless guards on all 18 math functions; empty argument validation on `diff()` and `factor()`; unconditional scope save/restore in `eval_integral`.
* [x] **Integer and exponent safety**: Implemented `to_exact_bigint(&self)` to reject non-integers; bitwise ops require dimensionless integers; dimensional powers reject non-integers and 8-bit overflow; 100k exponent cap enforced.
* [x] **Temperature and uncertainty semantics**: Affine temperature addition (`degC + degC`) rejected; subtraction yields Kelvin differences (`20 degC - 10 degC = 10 K`); delta additions (`20 degC + 5 K = 25 degC`) preserve affine scale; Gaussian quadrature error propagation.
* [x] **Equation-solving degeneracy and root robustness**: Disambiguates $0=0$ (`Identity`) from $0=5$ (`Contradiction`); dynamic variable auto-detection (`2y + 4 = 10` -> `y = 3`); root solver propagates errors via `Result<f64, String>`.
* [x] **Lexer/parser malformed-input handling**: Lexer rejects empty radix literals (`0x`, `0b`, `0o`); parser enforces `MAX_DEPTH = 512` recursion limit.
* [x] **Unit/currency documentation and tests**: Prominently documented offline baseline currency exchange rates (November 2026); added tests for base, derived, and compound units.
* [x] **Portable destructive-test and CI setup**: Dynamic binary resolution in `destructive_test.sh`; added GitHub Actions CI pipeline (`.github/workflows/ci.yml`).
* [x] **Accurate benchmark/test claims**: Derived test counts directly from execution; aligned README badge with the 450/450 test suite.
* [x] **Rust formatting, tests, clippy, shell/Python checks, and verification commits**: `cargo fmt` clean; `cargo clippy --all-targets -- -D warnings` 0 warnings; 21/21 cargo tests pass; 450/450 destructive tests pass; clean script syntax checks.
* [x] **Microlithic modular source architecture**: Decomposed monolithic files (`eval.rs`, `symbolic.rs`, `numeric.rs`) into clean submodules under `src/eval/`, `src/symbolic/`, and `src/numeric/`, with all files strictly under 283 lines.

---

### Milestone 2: Calculus & Linear Algebra Engine (v0.2.0 - Complete)
* [x] **Numerical Definite Integration**:
  * Added `int(...)` function alias alongside `integrate(...)`.
  * Supported standard comma-separated syntax (`int(sin(x), 0, pi)`) alongside semicolon syntax.
  * Preserved composite Simpson rule with dynamic variable detection for multi-parameter integration.
* [x] **Spatial Vector Algebra**:
  * Added bracket list syntax: `[1, 2, 3] + [4, 5, 6]` -> `[5, 7, 9]`.
  * Added scalar multiplication and negation: `2 * [1, 2, 3]` -> `[2, 4, 6]`, `-[1, 2, 3]`.
  * Implemented vector functions: `dot(u, v)`, `cross(u, v)`, `norm(u)`.
* [x] **Matrix Engine**:
  * Added nested bracket matrix syntax: `[[1, 2], [3, 4]]`.
  * Implemented matrix addition, subtraction, and multiplication: `[[1, 2], [3, 4]] * [[5], [6]]` -> `[[17], [39]]`.
  * Implemented matrix determinant (`det(M)`), inverse (`inv(M)`), and transposition (`transpose(M)`).
  * Enforced square-matrix checks and singular matrix detection without panics.
* [x] **Multi-Variable Linear Systems Solver**:
  * Added syntax: `solve([2x + y = 7, x - y = 2], [x, y])` -> `[x = 3, y = 1]`.
  * Supported auto-variable detection: `solve([2x + y = 7, x - y = 2])`.
  * Implemented Gaussian elimination with partial pivoting.
  * Disambiguates inconsistent systems (`Inconsistent system: no solution exists`) from degenerate systems.

---

### Milestone 3: CLI Ergonomics & Shell Polish (v0.3.0 - Complete)
* [x] **Native Shell Tab-Completions**:
  * Add `--completions <shell>` argument for `bash`, `zsh`, `fish`, and `powershell`.
  * Integrate `clap_complete` to auto-complete commands, options, 200+ unit names, and math functions.
  * Write automated installer script for user shell directories ([scripts/install_completions.sh](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/scripts/install_completions.sh)).
* [x] **UNIX Manual Page**:
  * Generate standard roff man page at [dist/man/omnicalc.1](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/dist/man/omnicalc.1).
  * Document all syntax, precedence rules, physical constants, and exit codes.
* [x] **Opt-in Live Currency Updater**:
  * Add `--update-rates` command.
  * Fetch daily European Central Bank (ECB) XML feed on explicit operator request.
  * Cache rates to `~/.cache/omnicalc/rates.json`.
  * Retain 100% offline default execution when cache is absent.

---

### Milestone 4: Continuous Fuzz Testing & Hardening (v0.4.0 - Planned)
* [ ] **Continuous Fuzz Harness**:
  * Setup `cargo-fuzz` with LLVM `libFuzzer` in `/home/sagnik/code/omnicalc/fuzz`.
  * Target 1: `fuzz_parser` (arbitrary UTF-8 and binary byte streams).
  * Target 2: `fuzz_eval` (random AST expressions with extreme numeric values).
  * Target 3: `fuzz_units` (random dimensional reductions).
* [ ] **Sanitizer Verification**:
  * Run AddressSanitizer (`ASan`) and UndefinedBehaviorSanitizer (`UBSan`).
  * Verify zero memory violations across $10^7$ fuzzing iterations.

---

### Milestone 5: Multi-Architecture Packaging Matrix (v0.5.0 - Planned)
* [ ] **Cross-Compilation Matrix**:
  * `x86_64-unknown-linux-gnu` (Linux glibc $\ge 2.31$).
  * `x86_64-unknown-linux-musl` (Linux static binary).
  * `aarch64-unknown-linux-gnu` (Linux ARM64 / Raspberry Pi / Graviton).
  * `x86_64-pc-windows-msvc` (Windows 10/11 x86_64).
  * `aarch64-apple-darwin` (macOS Apple Silicon M1-M4).
* [ ] **Automated Release Script**:
  * Script path: `/home/sagnik/code/omnicalc/build_all_targets.sh`.
  * Build tarballs, `.deb`, `.rpm`, and `.zip` archives.
  * Compute and verify SHA-256 checksums in `/home/sagnik/code/omnicalc/dist/SHA256SUMS`.
