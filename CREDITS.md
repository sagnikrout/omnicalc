# Credits, Vision & Acknowledgments

**Project:** `cal` / `omnicalc`  
**Repository:** [https://github.com/sagnikrout/cal](https://github.com/sagnikrout/cal)  
**Version:** CalVer `2026.11.18`  

---

## 1. The Vision & Origin

`cal` was conceived years ago by **Sagnik Rout** ([@sagnikrout](https://github.com/sagnikrout)).

It started with a persistent ambition and relentless optimism: to build the definitive command-line scientific calculator. Most existing tools forced an uncomfortable tradeoff: either instant startup with limited math, or full computer algebra with multi-megabyte C++ dependencies and 50ms startup latency. 

Sagnik envisioned a unified engine that provides both: sub-millisecond execution, exact arbitrary precision, dimensional physics, and robust computer algebra, without compromise. The initial dream took root at [https://github.com/sagnikrout/cal](https://github.com/sagnikrout/cal) and culminated in this production engine.

---

## 2. Co-Engineering & Synthesis

This engine was co-engineered by **Sagnik Rout** in partnership with **Gemini 3.8 Flash High & Antigravity** ([Google DeepMind](https://deepmind.google/)).

Operating in the **Charlie Rutledge** engineering mode, the collaboration focused on:
* Skeptical interrogation of every parser assumption and precedence rule.
* Zero hand-waving and zero tolerance for unhandled edge-case panics.
* Construction of the 442-test destructive boundary battery and empirical benchmarking against the entire ecosystem.

---

## 3. Standing on the Shoulders of Giants

This project exists because of decades of work by open-source maintainers, compiler authors, and mathematical software pioneers. We express our deepest gratitude to the creators and communities of:

### Qalculate! (`libqalculate`)
* **Author:** Niklas Knutsson and the Qalculate team.
* **Contribution:** The gold standard for open-source calculator engines, arbitrary-precision numerical systems, dimensional units, and interval arithmetic.

### Numbat & Insect
* **Author:** David Peter ([@sharkdp](https://github.com/sharkdp)) and contributors.
* **Contribution:** Pioneered rigorous static physical dimension typing, unit inference, and clean CLI syntax for physical sciences.

### fend
* **Author:** Evan Jones ([@printfn](https://github.com/printfn)) and contributors.
* **Contribution:** Pioneered zero-latency exact rational AST evaluation, clean standard library design, and lightweight Rust calculation ergonomics.

### kalker
* **Author:** PaddiM8 ([@PaddiM8](https://github.com/PaddiM8)) and contributors.
* **Contribution:** Elegant mathematical notation, arbitrary-precision complex powers, matrices, and calculus parsing.

### calc (`apcalc`)
* **Authors:** David I. Bell, Landon Curt Noll, and contributors.
* **Contribution:** The original arbitrary-precision C calculator, inspiring CLI mathematical scripting since the early 1990s.

---

## 4. Mathematical Foundations

We honor the mathematicians whose formal theories and algorithms form the backbone of this engine:

| Mathematician | Epoch | Core Algorithmic & Theoretical Contribution |
| :--- | :--- | :--- |
| **Leonhard Euler** | 1707–1783 | Complex exponentiation, Euler's identity ($e^{i\pi} + 1 = 0$), Gamma function |
| **Isaac Newton & Joseph Raphson** | 1643–1727 | Iterative numerical root finding ($x_{n+1} = x_n - \frac{f(x_n)}{f'(x_n)}$) |
| **Carl Friedrich Gauss** | 1777–1855 | Gaussian distributions, error propagation, modular arithmetic |
| **Alexander Kronrod** | 1921–1986 | Adaptive quadrature rules for numerical integration |
| **Thomas Simpson** | 1710–1761 | Parabolic numerical quadrature for definite integrals |
| **Bernhard Riemann** | 1826–1866 | Complex analysis, Riemann surfaces, and branch cut definitions |
| **Vaughan Pratt** | 1944–Present | Top-Down Operator Precedence parsing (Pratt Parser algorithm) |
| **BIPM** | 1875–Present | Bureau International des Poids et Mesures (SI physical dimensions) |

---

## 5. The Open Source Community

Thanks to the worldwide Rust community, the crates.io ecosystem (`num-bigint`, `num-rational`, `num-complex`, `clap`, `rustyline`), and all developers who believe that command-line tools should be fast, correct, and beautiful.
