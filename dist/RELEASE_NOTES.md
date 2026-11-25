# omnicalc v2026.11.18 — Initial Production Release

High-performance, zero-allocation scientific CLI calculator, physical unit conversion engine, and Computer Algebra System (CAS) written in modern Rust.

Conceived by **Sagnik Rout** ([@sagnikrout](https://github.com/sagnikrout)), co-engineered in partnership with **Gemini 3.8 Flash High & Antigravity** ([Google DeepMind](https://deepmind.google/)).

---

## Highlights & Capabilities

* **Sub-Millisecond Cold-Start Latency**: Native turnaround in ~0.80 ms, executing ~4x faster than `fend` and ~70x faster than `qalc` and `numbat`.
* **Low Memory Footprint**: Bounded to ~4.1 MB Peak RSS.
* **Exact Rational Arithmetic**: Pure arbitrary-precision integer and rational math powered by `num_bigint` and `num_rational`.
* **Physical Dimensional Analysis**: Validates 200+ SI base and derived units with compile-time-grade dimensional algebra.
* **Affine Temperature Conversions**: Supports temperature scales with physical zero offsets (`degC`, `degF`, `K`).
* **Complex Numbers & Trigonometry**: Full support for Euler's identity ($e^{i\pi} = -1$), complex exponents ($i^i$), and complex trigonometry.
* **Symbolic Computer Algebra (CAS)**: Analytical derivatives (`diff(x^3, x)`), Newton-Raphson equation solving (`sin(x) = 0`, `x^3 - 27 = 0`), and polynomial factoring.
* **Gaussian Uncertainty Propagation**: Automatic absolute error propagation across all operations.
* **Bitwise & Radix Engine**: 64-bit integer engine with binary, octal, and hexadecimal radix outputs.
* **Deterministic Offline Currency**: Offline baseline conversions across major world currencies without network latency.
* **Master Destructive Test Defense**: 475 automated tests passed (100% pass rate, 0 failures, 0 crashes) defending against failure archetypes cataloged across the open-source calculator ecosystem.

---

## Installation

### Debian / Ubuntu (`.deb`)
```bash
sudo dpkg -i omnicalc_2026.11.18_amd64.deb
```

### Standalone Linux x86_64 Tarball
```bash
tar -xzf omnicalc-v2026.11.18-linux-x86_64.tar.gz
cd omnicalc-v2026.11.18-linux-x86_64
sudo ./install.sh
```

---

## Integrity Checksums (SHA-256)
```
025418cb8bd799b9c72e1035b9cf592e973b3e49be429c5f5ffd351119227af5  omnicalc-v2026.11.18-linux-x86_64.tar.gz
15a1e01b3e6376dd7d38cf6417e5b34c406bb3d3f2c7c522f99badd65fbbbdfe  omnicalc_2026.11.18_amd64.deb
```
