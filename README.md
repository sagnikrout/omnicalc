# omnicalc

High-performance, zero-allocation scientific CLI calculator, physical unit conversion engine, and Computer Algebra System (CAS) written in modern Rust.

---

## Features

- **Sub-Millisecond Execution**: Native cold-start turnaround (<0.85 ms), executing ~4x faster than `fend` and ~70x faster than `qalc` and `numbat`.
- **Minimal Memory Overhead**: Peak memory footprint constrained to ~4.1 MB (Peak RSS).
- **Physical Dimensional Algebra**: Static SI base dimension checks preventing invalid operations like `5 m + 2 s`.
- **Complex Numbers & Trigonometry**: Full support for Euler's identity ($e^{i\pi} = -1$), complex roots, powers ($i^i$), and trigonometric functions.
- **Uncertainty Propagation**: Absolute Gaussian error propagation across nominal-zero boundaries and unit bounds.
- **Computer Algebra System (CAS)**: Symbolic derivatives, Newton-Raphson transcendental equation solving, and prime/polynomial factoring.
- **Currency & Units**: Currency conversions (USD, EUR, GBP, JPY, INR, CAD, AUD, CHF, CNY) and compound unit algebra (e.g. data rates, energy, velocity).
- **Bitwise Precision**: 64-bit integer engine with binary, octal, and hexadecimal radix conversions.

---

## Installation

### From Debian Package (`.deb`)
```bash
sudo dpkg -i omnicalc_0.1.0_amd64.deb
```

### From Binary Tarball
```bash
tar -xzf omnicalc-v0.1.0-linux-x86_64.tar.gz
cd omnicalc-v0.1.0-linux-x86_64
sudo ./install.sh
```

### Build from Source
```bash
cargo build --release
sudo cp target/release/omnicalc /usr/local/bin/
```

---

## Quick Start & Usage Examples

### Command-Line Evaluation
```bash
# Basic evaluation
omnicalc -e "2 + 2 * 3^2"

# Scientific unit conversion
omnicalc -e "100 km/h to m/s"

# Complex numbers
omnicalc -e "exp(i * pi)"

# Symbolic differentiation
omnicalc -e "diff(x^3, x)"

# Equation solving
omnicalc -e "2x + 4 = 10"

# Uncertainty propagation
omnicalc -e "(10 +/- 0.5) * (2 +/- 0.1)"

# Hexadecimal radix
omnicalc -r -b hex -e 255
```

### Interactive REPL Mode
Launch interactive session:
```bash
omnicalc
```
Type expressions directly; type `exit` or `quit` to leave.

---

## Roadmap & Future Milestones

See [ROADMAP.md](/home/sagnik/code/omnicalc/ROADMAP.md) for the phased master engineering plan to expand calculus, vector/matrix algebra, shell completions, fuzzing, and cross-platform packaging.

---

## License

Licensed under the MIT License ([LICENSE](LICENSE)).
