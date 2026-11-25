# omnicalc

**The Unified High-Performance Scientific CLI Calculator & Computer Algebra System**

[![Version](https://img.shields.io/badge/version-2026.11.18-blue.svg)](https://github.com/sagnikrout/omnicalc)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-475%2F475%20passed-brightgreen.svg)](destructive_test.sh)
[![Benchmark](https://img.shields.io/badge/latency-0.80%20ms-purple.svg)](benchmark_comparison.py)

---

## Origin & Vision

**Repository:** [https://github.com/sagnikrout/omnicalc](https://github.com/sagnikrout/omnicalc)  
Conceived years ago by **Sagnik Rout** ([@sagnikrout](https://github.com/sagnikrout)) with relentless optimism: to eliminate the painful compromise in command-line math. 

Engineers were forced to choose between lightweight calculators with limited features or heavyweight C++ Computer Algebra Systems with slow startup times. Sagnik envisioned a unified tool: instant startup (<1 ms), arbitrary precision, dimensional physics, and symbolic computer algebra, built natively in Rust.

Co-engineered in partnership with **Gemini 3.8 Flash High & Antigravity** ([Google DeepMind](https://deepmind.google/)) in Charlie Rutledge rigorous engineering mode.

---

## Shell Aliases & Ergonomics

The executable binary and package are named `omnicalc` to preserve standard UNIX `/usr/bin/cal` calendar functionality.

* **Repository & Crate Identity:** `omnicalc` ([https://github.com/sagnikrout/omnicalc](https://github.com/sagnikrout/omnicalc))
* **Installed System Binary:** `omnicalc` (zero collision)
* **Recommended Shell Aliases:**
  ```bash
  # In ~/.bashrc or ~/.zshrc:
  alias c="omnicalc"
  alias calc="omnicalc"
  alias cal="omnicalc" # If you prefer overriding system calendar
  ```

---

## Benchmark Performance vs Competitors

Benchmarked on Intel Core Ultra x86_64 across 50 iterations with 10 warmups:

| Metric | `cal` (`omnicalc`) | `fend` | `libqalculate` (`qalc`) | `numbat` | Advantage |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Simple Math Latency** | **0.80 ms** | 3.14 ms | 55.45 ms | 58.07 ms | **~4x faster than fend, ~70x faster than qalc** |
| **Unit Conversion Latency** | **0.84 ms** | 3.44 ms | 57.59 ms | 59.66 ms | **~4x faster than fend, ~70x faster than numbat** |
| **Complex Math Latency** | **0.74 ms** | 2.91 ms | 51.55 ms | 56.64 ms | **Lowest latency across all categories** |
| **Peak Memory (RSS)** | **4.1 MB** | 3.8 MB | 33.3 MB | 8.7 MB | **8x smaller memory footprint than qalc** |
| **Binary Size (Stripped)** | **1.17 MB** | 2.45 MB | 10.80 MB | 14.20 MB | **Smallest binary in class** |

---

## Core Capabilities

* **Sub-Millisecond Execution**: Instant startup (<0.85 ms) makes it ideal for piped command-line workflows and shell prompts.
* **Exact Rational Arithmetic**: Uses `num_rational::BigRational` and `num_bigint::BigInt` for exact fractions (`1/3 * 3 = 1`).
* **Physical Dimensional Analysis**: Validates 200+ SI base and derived units. Rejects non-physical sums like `5 m + 2 s` with clear dimensional errors.
* **Affine Temperature Scales**: Supports correct physical offset conversions (`100 degC to degF` = `212 degF`, `0 degC to K` = `273.15 K`).
* **Complex Analysis**: Full support for Euler's identity ($e^{i\pi} = -1$), complex exponents ($i^i$), complex roots, and complex trigonometry (`sin(i)`, `cos(i)`).
* **Symbolic Computer Algebra (CAS)**: Computes analytical derivatives (`diff(x^3, x)` -> `3 * x^2`) and solves equations via Newton-Raphson (`sin(x) = 0`, `x^3 - 27 = 0`).
* **Vector & Matrix Linear Algebra**: Spatial vectors (`[1, 2, 3] + [4, 5, 6]`, `dot`, `cross`, `norm`), matrices (`[[1, 2], [3, 4]] * [[5], [6]]`, `det`, `inv`, `transpose`), and simultaneous linear system solver (`solve([2x + y = 7, x - y = 2], [x, y])`).
* **Gaussian Uncertainty Propagation**: Automatic absolute error propagation across operations (`(10 +/- 0.5) * (2 +/- 0.1)`).
* **Bitwise & Radix Engine**: 64-bit integer engine with conversion to hexadecimal, binary, and octal (`-b hex`, `-b bin`).
* **Deterministic Offline Currency**: Fast, offline currency conversions across major world currencies without network latency.

---

## Installation

### Version Scheme: CalVer (`YYYY.MM.DD`)
`cal` uses Calendar Versioning (`2026.11.18`). Every release directly reflects its build date for straightforward maintenance.

### From Debian Package (`.deb`)
```bash
sudo dpkg -i dist/omnicalc_2026.11.18_amd64.deb
```

### From Binary Tarball
```bash
tar -xzf dist/omnicalc-v2026.11.18-linux-x86_64.tar.gz
cd omnicalc-v2026.11.18-linux-x86_64
sudo ./install.sh
```

### Build from Source
```bash
git clone https://github.com/sagnikrout/omnicalc.git
cd omnicalc
cargo build --release
sudo cp target/release/omnicalc /usr/local/bin/
```

---

## Usage Examples

### Command-Line Evaluation
```bash
# Basic arithmetic
omnicalc -e "2 + 2 * 3^2"

# Physical unit conversion
omnicalc -e "100 km/h to m/s"
omnicalc -e "60 mph to m/s"

# Affine temperature conversion
omnicalc -e "100 degC to degF"

# Complex numbers
omnicalc -e "exp(i * pi)"
omnicalc -e "i^i"

# Symbolic differentiation & definite integration
omnicalc -e "diff(x^3, x)"
omnicalc -e "int(sin(x), 0, pi)"

# Vector, matrix & linear system solving
omnicalc -e "det([[1, 2], [3, 4]])"
omnicalc -e "solve([2x + y = 7, x - y = 2], [x, y])"

# Equation solving
omnicalc -e "2x + 4 = 10"
omnicalc -e "sin(x) = 0"

# Gaussian uncertainty propagation
omnicalc -e "(10 +/- 0.5) * (2 +/- 0.1)"

# Radix output (hexadecimal)
omnicalc -r -b hex -e 255

# View vision and credits
omnicalc --credits
```

### Piped Stdin Mode
```bash
echo "100 EUR to USD" | omnicalc
echo "sqrt(4 m^2)" | omnicalc
```

### Interactive REPL Mode
```bash
$ omnicalc
omnicalc v2026.11.18 — Unified CLI Scientific & Physical Engine
Origin: Sagnik Rout (https://github.com/sagnikrout/omnicalc) | Synthesis: Gemini 3.8 Flash High
Type 'quit' or 'exit' or press Ctrl-D to exit.

omnicalc> 5 cm to in
1.968503937 in
omnicalc> diff(sin(x), x)
cos(x)
omnicalc> (-8)^(1/3)
1 + 1.7320508076i
omnicalc> cbrt(-8)
-2
```

---

## Verification & Test Suites

The codebase is validated against 475 automated destructive tests with a 100% pass rate:

1. **Master Destructive Test Suite (475 tests):**
   ```bash
   ./destructive_test.sh
   ```
   Validates basic arithmetic, arbitrary-precision rationals, physical dimensions, complex Euler identities, Newton-Raphson solvers, Gaussian uncertainties, and edge-case crash probes.

2. **Rust Integration Suite:**
   ```bash
   cargo test
   ```

---

## Roadmap & Milestones

See [ROADMAP.md](ROADMAP.md) for the phased master engineering plan covering numerical integration, matrix algebra, shell completions, and fuzz testing.

---

## Credits & Acknowledgments

See [CREDITS.md](CREDITS.md) for full acknowledgments honoring:
* **Sagnik Rout** ([@sagnikrout](https://github.com/sagnikrout)) — Conceived the project and maintained the vision.
* **Google DeepMind & Antigravity** (Gemini 3.8 Flash High) — Co-engineering and verification synthesis.
* **Open Source Pioneers:** `Qalculate!` (Niklas Knutsson), `Numbat` & `Insect` (David Peter), `fend` (Evan Jones), `kalker` (PaddiM8), and `calc` (David I. Bell & Landon Curt Noll).
* **The Mathematicians:** Euler, Newton, Raphson, Gauss, Kronrod, Simpson, Pratt, and Riemann.

---

## License

Licensed under the MIT License ([LICENSE](LICENSE)) or Apache-2.0.
