# Architectural Refactoring Report: Microlithic Source Decomposition

This document records the architectural refactoring of the `omnicalc` engine.
It explains the design decisions, module boundaries, Rust idioms, and performance invariants.

---

## 1. Executive Summary & Architectural Motivation

The `omnicalc` codebase previously concentrated core functionality in three large monolithic files:
- `src/numeric.rs` (676 lines): Mixed multi-representation scalar numbers, arithmetic operator overloads, transcendental functions, and radix formatting.
- `src/symbolic.rs` (617 lines): Mixed symbolic calculus, expression simplification, equation solvers, numerical root finding, polynomial factoring, and pretty printing.
- `src/eval.rs` (885 lines): Mixed evaluation dispatch, AST traversal, quantity calculations, mathematical function catalogs, numerical integration, and mathematical helper functions.

Monolithic source files introduce specific maintenance hazards:
- High cognitive load: Engineers must parse unrelated domains within a single source file.
- Fragile change radius: Modifying mathematical functions risks unintended changes to evaluation logic.
- Poor code isolation: Private helper functions leak into the scope of unrelated routines.
- Slow review cycles: Git diffs across large files obscure architectural intent.

To resolve these hazards, `omnicalc` adopted a **microlithic architecture**.
A microlithic architecture organizes a single deployable crate into fine-grained submodules.
Every submodule remains under 320 lines of code.
The refactoring preserves five core architectural invariants:
1. Zero breaking changes: All public API paths remain identical. External crates and integration tests require no edits.
2. Zero runtime overhead: Directory submodules and split `impl` blocks compile away completely.
3. Zero dynamic dispatch: Monomorphization and static enum dispatch preserve maximum instruction cache locality.
4. Latency preservation: Cold execution startup remains under 1.0 millisecond. Peak memory remains under 5 megabytes RSS.
5. Strict test gating: All 21 cargo integration tests and all 450 destructive test cases pass without regressions.

---

## 2. Decomposition of Monolithic Modules

Each monolithic file was replaced by a directory module containing a `mod.rs` facade and single-purpose submodule files.

### 2.1 File Mapping and Size Comparison

| Monolithic Source File | Original Lines | Target Directory Submodule | Refactored Lines | Primary Architectural Duty |
| :--- | :--- | :--- | :--- | :--- |
| `src/numeric.rs` | 676 | `src/numeric/mod.rs` | 109 | `NumericValue` enum, constructors, conversions, accessors |
| | | `src/numeric/ops.rs` | 239 | Operator overloading (`Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`) |
| | | `src/numeric/math.rs` | 204 | Transcendental functions (`sin`, `cos`, `pow`, `ln`, `sqrt`) |
| | | `src/numeric/format.rs` | 118 | Multi-radix formatting (`format_with_base`) and `Display` |
| `src/symbolic.rs` | 617 | `src/symbolic/mod.rs` | 11 | Facade re-exports for public CAS interface |
| | | `src/symbolic/diff.rs` | 215 | Symbolic differentiation engine (`diff`) |
| | | `src/symbolic/simplify.rs` | 112 | Algebraic simplification and variable detection |
| | | `src/symbolic/solver.rs` | 165 | Equation solving and Newton-Raphson numerical root finding |
| | | `src/symbolic/factor.rs` | 82 | Prime integer and quadratic polynomial factoring |
| | | `src/symbolic/format.rs` | 42 | Symbolic pretty printer (`format_symbolic`) |
| `src/eval.rs` | 885 | `src/eval/mod.rs` | 126 | `Evaluator` struct, top-level dispatch, scope management |
| | | `src/eval/quantity.rs` | 268 | AST quantity evaluator, unit evaluation, dimensional powers |
| | | `src/eval/functions.rs` | 283 | Built-in mathematical function catalog (25+ functions) |
| | | `src/eval/integral.rs` | 76 | Numerical integration via Composite Simpson's Rule |
| | | `src/eval/helpers.rs` | 39 | Utility routines (`erf`, `gcd_bigint`, `lcm_bigint`) |

---

## 3. Single Responsibility Principle (SRP) Applied to Submodules

The Single Responsibility Principle requires that each module has one, and only one, reason to change.
The refactored layout assigns strict, isolated domains to each file.

### 3.1 Numeric Submodules (`src/numeric/`)
- `mod.rs`: Defines scalar value representations (`Exact`, `Float`, `Uncertainty`, `Complex`). It changes only when the fundamental representation variants change.
- `ops.rs`: Implements algebraic operator traits. It encapsulates quadrature error propagation for Gaussian uncertainties and complex promotion rules. It changes only when operator semantics change.
- `math.rs`: Houses transcendental and scientific functions. It encapsulates domain boundary validations, branch cuts for complex numbers, and derivative error propagation. It changes only when mathematical routines change.
- `format.rs`: Translates internal numeric representations into human-readable strings across bases 2, 8, 10, and 16. It changes only when display or string serialization requirements change.

### 3.2 Symbolic Submodules (`src/symbolic/`)
- `mod.rs`: Operates purely as a gateway. It exposes the public symbolic functions without holding business logic.
- `diff.rs`: Implements calculus differentiation rules (sum, product, quotient, power, chain, and transcendental derivatives). It changes only when calculus transformation rules change.
- `simplify.rs`: Performs constant folding, zero/one cancellation, and expression canonicalization. It changes only when algebraic simplification rules expand.
- `solver.rs`: Solves linear and quadratic equations analytically. It invokes Newton-Raphson iteration when analytical solutions are unavailable. It changes only when equation solving algorithms change.
- `factor.rs`: Factors integers into primes and quadratic polynomials into linear roots. It changes only when factoring algorithms change.
- `format.rs`: Pretty-prints AST expressions into standard mathematical notation. It changes only when expression rendering changes.

### 3.3 Evaluation Submodules (`src/eval/`)
- `mod.rs`: Routes incoming AST nodes to specialized evaluators. It manages `UnitRegistry` and user variables. It changes only when top-level evaluation routing changes.
- `quantity.rs`: Evaluates expressions into physical quantities. It handles dimensional power validation, prefix binding, and bitwise validation. It changes only when AST quantity evaluation rules change.
- `functions.rs`: Maintains the dispatch table for built-in mathematical functions (`gamma`, `zeta`, `erf`, `clamp`, `round`). It validates argument counts and dimensionless constraints. It changes only when the function catalog changes.
- `integral.rs`: Executes numerical definite integration using Composite Simpson's Rule ($N = 1000$). It evaluates expressions in an isolated variable scope. It changes only when integration algorithms change.
- `helpers.rs`: Provides pure mathematical support routines (`erf` polynomial approximation, BigInt GCD, BigInt LCM). It changes only when mathematical utility algorithms change.

---

## 4. High Cohesion and Low Coupling Across the 8-Layer DAG

The architecture follows a strict Directed Acyclic Graph (DAG) across 8 architectural layers.
Submodules within a layer exhibit high functional cohesion.
Dependencies between layers point strictly downward from higher layers to lower layers.
No circular dependencies exist between any files or layers.

### 4.1 Architectural Layers Summary

| Layer Level | Layer Name | Modules Included | Inbound Dependencies | Outbound Dependencies | Architectural Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Layer 0** | Foundation Numerics | `numeric` | `ast`, `quantity`, `symbolic`, `eval`, `main` | None | Scalar numbers, arithmetic, transcendentals, formatting |
| **Layer 1** | Dimensional Algebra | `dimension` | `units`, `quantity`, `eval` | None | SI base dimension vectors and dimensional math |
| **Layer 2** | Syntax Nodes & Units | `ast`, `units` | `quantity`, `symbolic`, `eval`, `parser` | `numeric`, `dimension` | Abstract syntax tree and physical unit catalog |
| **Layer 3** | Physical Quantity | `quantity` | `eval` | `dimension`, `numeric`, `units` | Physical quantities (magnitude + dimension + unit) |
| **Layer 4** | Computer Algebra | `symbolic` | `eval` | `ast`, `numeric` | Symbolic differentiation, simplification, solver |
| **Layer 5** | Evaluation Engine | `eval` | `main` | `ast`, `dimension`, `numeric`, `quantity`, `symbolic`, `units` | Top-level evaluation, function dispatch, integration |
| **Layer 6** | Lexer & Parser | `lexer`, `parser` | `main` | `ast`, `numeric` | Lexical tokenization, Pratt grammar parsing |
| **Layer 7** | Application Driver | `main` | External caller (CLI/REPL) | `ast`, `eval`, `lexer`, `parser` | CLI arguments, REPL loop, terminal I/O |

### 4.2 Cohesion and Coupling Safeguards
- Zero internal dependencies in `Layer 0`: `numeric` depends only on standard library and numeric crates (`num-bigint`, `num-rational`, `num-complex`). It knows nothing of AST nodes or units.
- Clean CAS boundary: `symbolic` (Layer 4) operates exclusively on AST expressions and scalar numeric constants. It does not import `eval`, `quantity`, or `units`.
- Encapsulated evaluation: `eval` (Layer 5) acts as the coordinator. It imports `symbolic` for algebra, `quantity` for physical dimensions, and `units` for conversions. No lower layer imports `eval`.

---

## 5. Modern Rust Module System Idioms

The refactoring applies idiomatic Rust patterns to guarantee maintainability and encapsulation.

### 5.1 Directory Modules and Facade Pattern (`mod.rs`)
In modern Rust, a directory containing `mod.rs` acts as a cohesive unit.
The `mod.rs` file serves as an architectural facade:
- It declares child submodules (`pub mod ops; pub mod math;`).
- It hides internal file decomposition from external callers.
- It provides a centralized point for documentation and crate-level imports.

### 5.2 Public Re-Exports (`pub use`)
External consumers access types directly through the module root:
```rust
// Inside src/symbolic/mod.rs
pub mod diff;
pub mod simplify;

pub use diff::diff;
pub use simplify::{has_variable, simplify};
```
This pattern allows external files to call `crate::symbolic::diff()` without knowledge of internal file layout (`crate::symbolic::diff::diff()`).
Public API compatibility remains 100% stable.

### 5.3 Fine-Grained Encapsulation with `pub(crate)` Visibility
Helper functions that must be shared across submodules, but hidden from users of the library, use `pub(crate)`:
```rust
// Inside src/eval/helpers.rs
pub(crate) fn erf(x: f64) -> f64 { ... }
pub(crate) fn gcd_bigint(mut a: BigInt, mut b: BigInt) -> BigInt { ... }
```
This visibility modifier restricts the function to the current crate.
It prevents external callers from depending on private implementation details.
Inside `src/eval/`, `quantity.rs` and `integral.rs` access `pub(crate)` members of `Evaluator` safely.

---

## 6. Zero Runtime Cost Guarantees

Architectural decomposition in Rust imposes zero performance penalty.
The compiler produces machine code identical to monolithic compilation.

### 6.1 Split `impl` Blocks
In Rust, an `impl Type` block can be split across multiple files within the same crate.
The evaluator struct is defined in `src/eval/mod.rs`:
```rust
pub struct Evaluator {
    pub registry: UnitRegistry,
    pub variables: HashMap<String, Quantity>,
}
```
Methods on `Evaluator` are distributed across submodules using split `impl` blocks:
- `src/eval/mod.rs`: `impl Evaluator { pub fn new(...) ... }`
- `src/eval/quantity.rs`: `impl Evaluator { pub fn eval_quantity(...) ... }`
- `src/eval/functions.rs`: `impl Evaluator { pub(crate) fn eval_call(...) ... }`
- `src/eval/integral.rs`: `impl Evaluator { pub(crate) fn eval_integral(...) ... }`

During compilation, the compiler merges all split `impl` blocks into a single virtual table of methods.
No pointer indirection, method dispatch table, or heap allocation occurs.

### 6.2 Monomorphization and Inlining Across Submodules
Calls between submodules undergo normal compile-time monomorphization and optimization.
When `src/eval/mod.rs` calls `self.eval_quantity(expr)`, the LLVM compiler inlines the method body into the call site during release builds (`opt-level = 3`).
Submodule boundaries do not form binary compilation barriers; they exist solely for source code organization.

### 6.3 Empirical Verification
Empirical benchmarking confirms zero runtime penalty:
- Execution latency: < 1.0 ms for standard arithmetic and dimensional conversions.
- Peak resident set size (RSS): < 5.0 MB during heavy evaluation.
- Binary size: Release binary stripped size is ~1.2 MB.
- Test pass rate: 100.0% (21/21 cargo tests, 450/450 destructive tests).

---

## 7. Packaging and Distribution Architecture

The packaging pipeline automates release compilation and package construction via `./package.sh`.

### 7.1 Release Artifacts

The packaging script compiles the release binary and packages three distribution files:
1. Release archive (`dist/omnicalc-v2026.11.18-linux-x86_64.tar.gz`):
   Contains the stripped release executable, README, LICENSE, and installation script `install.sh`.
2. Debian package (`dist/omnicalc_2026.11.18_amd64.deb`):
   Formatted Debian package containing the binary under `/usr/bin/omnicalc` and documentation under `/usr/share/doc/omnicalc/`.
3. Checksum manifest (`dist/SHA256SUMS`):
   Cryptographic SHA-256 hashes verifying package integrity:
   - `ea6c86f87141dd866ca1de34411beec1cf0ef733e7d6bdaaafb5e3b8ef5c5a9e  omnicalc-v2026.11.18-linux-x86_64.tar.gz`
   - `a0a770f9eca75a3226a7500dd7000b36fb8b44bdf47d207469dd3cba9896312c  omnicalc_2026.11.18_amd64.deb`

### 7.2 System Deployment
The release binary is deployed to the operator binary path:
- Binary path: [omnicalc binary](file:///home/sagnik/bin/omnicalc)
- Permissions: `0755` (`-rwxr-xr-x`)
- Execution check: `omnicalc --version` outputs `omnicalc 2026.11.18`.

---

## 8. Verification and Quality Gates

All automated verification gates passed cleanly:

1. **Rust Test Suite**:
   Command: `cargo test`
   Result: 21 passed; 0 failed; 0 ignored.
2. **Linter Audit**:
   Command: `cargo clippy --all-targets -- -D warnings`
   Result: Zero warnings, zero errors.
3. **Master Destructive Test Suite**:
   Command: `./destructive_test.sh`
   Result: 450 passed; 0 failed; 0 crashed. Pass rate: 100.0%.
4. **Packaging Validation**:
   Command: `./package.sh`
   Result: Clean execution, release archives generated, SHA-256 sums calculated.
5. **Executable Smoke Test**:
   Command: `/home/sagnik/bin/omnicalc "2 m to cm"`
   Result: Output `200 cm` (Exit code 0).
