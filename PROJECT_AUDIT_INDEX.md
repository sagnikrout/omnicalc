# `omnicalc` Project Index & Baseline Audit Ledger

**Baseline Date:** 2026-11-25  
**Prior Audit Baseline Commit:** `ce7ea73` (`master`)  
**Milestone 3 Scope:** CLI Ergonomics & Shell Polish (`--completions`, `dist/man/omnicalc.1`, `--update-rates` ECB cache) across all 49 repository files (excluding `target/` and `.git/`).

---

## 1. Audit Status Legend & Summary Counts

| Status Label | Definition | File Count |
| :--- | :--- | :--- |
| **AUDITED** | Thoroughly audited and verified against hardening invariants, `cargo fmt`, `clippy -D warnings`, 28 integration tests, and 475 destructive tests. | 44 |
| **AUDITED (STATIC BASE)** | Static manifest, lockfile, license, or credits. Verified read-only. | 5 |
| **Total** | **All tracked repository files (excluding this index)** | **49** |

---

## 2. Master File Inventory (All 49 Repository Files)

### 2.1 Rust Application & Library Source (`src/` — 29 Files)

| File | Layer / Duty | Lines | SHA-256 (12) | Audit Status |
| :--- | :--- | :--- | :--- | :--- |
| [src/lib.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/lib.rs) | Crate module exports | 11 | `2e49d496c87e` | **AUDITED** |
| [src/main.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/main.rs) | Layer 7: CLI, completions, rates, REPL | 272 | `885bad253ba0` | **AUDITED** |
| [src/completions.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/completions.rs) | Layer 7: `clap_complete` + unit/fn words | 76 | `d1535dd378e1` | **AUDITED** |
| [src/rates.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/rates.rs) | Layer 2: Opt-in ECB XML fetch & cache | 178 | `6b82bc81a4eb` | **AUDITED** |
| [src/ast.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/ast.rs) | Layer 2: AST expressions and ops | 69 | `4806dc4262ac` | **AUDITED** |
| [src/lexer.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/lexer.rs) | Layer 6: Tokenizer & literals | 443 | `e93454c8de52` | **AUDITED** |
| [src/parser.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/parser.rs) | Layer 6: Pratt parser & list syntax | 495 | `74c70def18fb` | **AUDITED** |
| [src/dimension.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/dimension.rs) | Layer 1: 9-axis SI dimensions + checked ops | 283 | `8b4de38a4472` | **AUDITED** |
| [src/quantity.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/quantity.rs) | Layer 3: Quantities & affine temps | 286 | `09ce4768e0c0` | **AUDITED** |
| [src/units.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/units.rs) | Layer 2: Unit & currency catalog | 568 | `9333891c0a3b` | **AUDITED** |
| [src/numeric/mod.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/numeric/mod.rs) | Layer 0: `NumericValue` core & casts | 109 | `384adf42175b` | **AUDITED** |
| [src/numeric/ops.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/numeric/ops.rs) | Layer 0: Scalar & error ops | 240 | `02272bf790d5` | **AUDITED** |
| [src/numeric/math.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/numeric/math.rs) | Layer 0: Powers & transcendentals | 227 | `62e7e194fbed` | **AUDITED** |
| [src/numeric/format.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/numeric/format.rs) | Layer 0: Multi-radix & Display fmt | 127 | `fed36ee9b3bb` | **AUDITED** |
| [src/symbolic/mod.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/symbolic/mod.rs) | Layer 4: CAS facade re-exports | 13 | `32a84300fb3e` | **AUDITED** |
| [src/symbolic/diff.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/symbolic/diff.rs) | Layer 4: Symbolic differentiation | 214 | `8cbe0e73b0b1` | **AUDITED** |
| [src/symbolic/factor.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/symbolic/factor.rs) | Layer 4: Prime & quadratic factors | 80 | `562e83e9cb65` | **AUDITED** |
| [src/symbolic/format.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/symbolic/format.rs) | Layer 4: Symbolic AST formatter | 31 | `4b03ae0d121b` | **AUDITED** |
| [src/symbolic/simplify.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/symbolic/simplify.rs) | Layer 4: Algebraic simplification | 93 | `111245e84d45` | **AUDITED** |
| [src/symbolic/solver.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/symbolic/solver.rs) | Layer 4: Single-var & Newton root | 226 | `380d42695a45` | **AUDITED** |
| [src/eval/mod.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/mod.rs) | Layer 5: Top-level eval & call validation | 180 | `0457427defb7` | **AUDITED** |
| [src/eval/quantity.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/quantity.rs) | Layer 5: Scalar quantity AST eval | 279 | `dd31d1a4d04a` | **AUDITED** |
| [src/eval/functions.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/functions.rs) | Layer 5: Math function catalog | 274 | `be89ea8db9a2` | **AUDITED** |
| [src/eval/integral.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/integral.rs) | Layer 5: Simpson integration | 108 | `146c121c154c` | **AUDITED** |
| [src/eval/helpers.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/helpers.rs) | Layer 5: `erf`, `gcd`/`lcm`, unit rounding | 65 | `e610afbc0873` | **AUDITED** |
| [src/eval/la.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/la.rs) | Layer 5: Linear algebra AST eval | 251 | `66be58a0995f` | **AUDITED** |
| [src/eval/matrix.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/matrix.rs) | Layer 5: Matrix LU & Gauss-Jordan | 271 | `fa3a1e8c79b7` | **AUDITED** |
| [src/eval/system.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/system.rs) | Layer 5: Linear systems solver | 215 | `7e6d684e39b1` | **AUDITED** |
| [src/eval/vector.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/vector.rs) | Layer 5: `dot`, `cross`, `norm` | 34 | `c4be3e37d04c` | **AUDITED** |

### 2.2 Tests, CI, Build, Docs & Release Artifacts (20 Files)

| File | Duty | Lines / Size | SHA-256 (12) | Audit Status |
| :--- | :--- | :--- | :--- | :--- |
| [tests/integration_tests.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/tests/integration_tests.rs) | 28 Rust integration tests | 439 lines | `141ed40e75a3` | **AUDITED** |
| [destructive_test.sh](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/destructive_test.sh) | 475-case boundary & crash suite | 1061 lines | `8c464391f4e2` | **AUDITED** |
| [install_completions.sh](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/scripts/install_completions.sh) | User shell completion installer | 42 lines | `674036cbf7f4` | **AUDITED** |
| [benchmark_comparison.py](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/benchmark_comparison.py) | Portable CLI benchmark harness | 108 lines | `9773ce3a4a98` | **AUDITED** |
| [package.sh](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/package.sh) | Linux x86_64 `.tar.gz`/`.deb` builder | 103 lines | `e586d63c382b` | **AUDITED** |
| [.github/workflows/ci.yml](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/.github/workflows/ci.yml) | GitHub Actions CI workflow | 56 lines | `a223039324ef` | **AUDITED** |
| [.gitignore](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/.gitignore) | Git ignore patterns (`/target`, logs) | 5 lines | `add69fae8010` | **AUDITED** |
| [AUDIT.md](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/AUDIT.md) | 2026-11-22 hardening log | 47 lines | `a31be7c2a843` | **AUDITED** |
| [ARCHITECTURAL_REFACTORING_REPORT.md](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/docs/ARCHITECTURAL_REFACTORING_REPORT.md) | Microlithic decomposition report | 230 lines | `f0001afc883d` | **AUDITED** |
| [Cargo.toml](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/Cargo.toml) | Crate manifest & release profile | 27 lines | `d28806c2e119` | **AUDITED (STATIC BASE)** |
| [Cargo.lock](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/Cargo.lock) | Pinned dependency lockfile | 390 lines | `a0fd6a5e4c59` | **AUDITED (STATIC BASE)** |
| [LICENSE](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/LICENSE) | MIT license text | 21 lines | `98a4a0073f78` | **AUDITED (STATIC BASE)** |
| [CREDITS.md](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/CREDITS.md) | Authors, pioneers & math credits | 75 lines | `ccccb5866229` | **AUDITED (STATIC BASE)** |
| [dist/RELEASE_NOTES.md](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/dist/RELEASE_NOTES.md) | Release notes (synced checksums) | 45 lines | `0a353f2f9380` | **AUDITED (STATIC BASE)** |
| [dist/man/omnicalc.1](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/dist/man/omnicalc.1) | UNIX roff manual page | 113 lines | `e8a513cd52ad` | **AUDITED** |
| [README.md](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/README.md) | User documentation & benchmarks | 195 lines | `a4245e639a80` | **AUDITED** |
| [ROADMAP.md](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/ROADMAP.md) | Phased engineering milestones | 114 lines | `53eb86d74939` | **AUDITED** |
| [dist/SHA256SUMS](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/dist/SHA256SUMS) | Checksums for release packages | 2 lines | `bc07cb2a00fe` | **AUDITED** |
| [omnicalc-v2026.11.18-linux-x86_64.tar.gz](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/dist/omnicalc-v2026.11.18-linux-x86_64.tar.gz) | Stripped tarball package | 686 KB | `025418cb8bd7` | **AUDITED** |
| [omnicalc_2026.11.18_amd64.deb](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/dist/omnicalc_2026.11.18_amd64.deb) | Debian binary package | 586 KB | `15a1e01b3e63` | **AUDITED** |

---

## 3. Hardening Changes Locked into This Baseline

1. **Nested CAS Panic Guard:** `validate_symbolic_calls` in [src/eval/mod.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/mod.rs) and safe `args.first()` handling in [src/symbolic/diff.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/symbolic/diff.rs) eliminate index-out-of-bounds panics on nested `diff()` or `factor()`.
2. **Dimension Overflow Guards:** `Dimension::checked_add` and `Dimension::checked_sub` in [src/dimension.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/dimension.rs) prevent 8-bit exponent overflow panics during multiplication and division in [src/eval/quantity.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/quantity.rs).
3. **Unit-Aware Rounding:** `round_quantity` in [src/eval/helpers.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/helpers.rs) converts quantities to their active display unit before `round`, `floor`, or `ceil` (`round(10.6 km)` -> `11 km`, `round(20.6 degC)` -> `21 degC`).
4. **Nonlinearity Bypass Fix:** `solve_linear_system` in [src/eval/system.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/system.rs) probes two non-lattice vectors (`p1`, `p2`) and supports `Pow` in `eval_expr_vars`, rejecting polynomials like `x * (x - 1)` cleanly.
5. **Complex, NaN & Empty Matrix Guards:** [src/eval/la.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/la.rs), [src/eval/matrix.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/matrix.rs), and [src/eval/integral.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/integral.rs) reject complex numbers, `[[]]`, `transpose([])`, and non-finite values instead of truncating or emitting `NaN`.
6. **Overflow-Safe Vector Norm:** `eval_norm` in [src/eval/vector.rs](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/src/eval/vector.rs) scales by max absolute magnitude so large vectors (`norm([3e200, 4e200])`) do not overflow.
7. **Workspace & Doc Sync:** Removed empty `crash_repro.log` and aligned test counts and checksums across [README.md](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/README.md), [destructive_test.sh](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/destructive_test.sh), [dist/RELEASE_NOTES.md](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/dist/RELEASE_NOTES.md), and [dist/SHA256SUMS](file://wsl.localhost/Ubuntu/home/sagnik/code/omnicalc/dist/SHA256SUMS).

---

## 4. Verification Baseline Results

* `cargo fmt --check`: 0 diffs.
* `cargo clippy --all-targets -- -D warnings`: 0 warnings, 0 errors.
* `cargo test`: 28 passed, 0 failed.
* `./destructive_test.sh`: 475 passed, 0 failed, 0 crashed (100.0%).
* Microlithic invariant: all submodules under `src/eval/`, `src/numeric/`, and `src/symbolic/` are strictly $\le 279$ lines (`src/rates.rs`: 178 lines, `src/completions.rs`: 76 lines).
