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

### Milestone 2: Calculus & Linear Algebra Engine (v0.2.0 - Planned)
* [ ] **Numerical Definite Integration**:
  * Implement adaptive 15-point Gauss-Kronrod quadrature (`GK15`).
  * Add syntax: `integrate(f(x), x, a, b)` and `int(sin(x), 0, pi)`.
  * Support singular endpoint handling with adaptive Simpson fallback.
* [ ] **Spatial Vector Algebra**:
  * Add vector syntax: `[1, 2, 3] + [4, 5, 6]`.
  * Implement vector functions: `dot(u, v)`, `cross(u, v)`, `norm(u)`.
* [ ] **Matrix Engine**:
  * Add matrix syntax: `[[1, 2], [3, 4]]`.
  * Implement matrix multiplication and transposition.
  * Implement matrix determinant (`det(M)`) and inverse (`inv(M)`).
  * Use stack-allocated arrays for dimensions $\le 4 \times 4$ to avoid heap overhead.
* [ ] **Multi-Variable Linear Systems Solver**:
  * Add syntax: `solve([2x + y = 7, x - y = 2], [x, y])`.
  * Implement Gaussian elimination with partial pivoting over exact rationals.

---

### Milestone 3: CLI Ergonomics & Shell Polish (v0.3.0 - Planned)
* [ ] **Native Shell Tab-Completions**:
  * Add `--completions <shell>` argument for `bash`, `zsh`, `fish`, and `powershell`.
  * Integrate `clap_complete` to auto-complete commands, options, 200+ unit names, and math functions.
  * Write automated installer script for user shell directories.
* [ ] **UNIX Manual Page**:
  * Generate standard roff man page at `/home/sagnik/code/omnicalc/dist/man/omnicalc.1`.
  * Document all syntax, precedence rules, physical constants, and exit codes.
* [ ] **Opt-in Live Currency Updater**:
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
