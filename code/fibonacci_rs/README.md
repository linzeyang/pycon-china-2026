# fibonacci_rs

A Python extension module written in Rust with [PyO3](https://github.com/PyO3/pyo3), packaged with [maturin](https://github.com/PyO3/maturin).

It exposes a single function, `fibonacci(n)`, which computes the n-th Fibonacci number using an iterative O(n) algorithm on native `u64` integers — a minimal example of extending CPython with Rust.

## Prerequisites

- Python 3.8+
- A Rust toolchain (install via [rustup](https://rustup.rs/))
- [maturin](https://github.com/PyO3/maturin) (`uv tool install maturin`)

## Getting started

Create and activate a virtual environment, then build and install the extension into it:

```bash
uv venv
source .venv/bin/activate

maturin develop --release
```

`maturin develop` compiles the Rust crate and installs the resulting extension module into the active environment, so you can iterate quickly during development.

## Usage

```python
>>> import fibonacci_rs
>>> fibonacci_rs.fibonacci(0)
0
>>> fibonacci_rs.fibonacci(1)
1
>>> fibonacci_rs.fibonacci(10)
55
>>> fibonacci_rs.fibonacci(50)
12586269025
```

> [!NOTE]
> Results fit in a `u64` up to `n = 93`. For larger inputs, the implementation uses `wrapping_add`, so it wraps around instead of panicking on overflow.

## How it works

The whole extension lives in [src/lib.rs](src/lib.rs):

```rust
#[pymodule]
mod fibonacci_rs {
    /// Calculate nth Fibonacci number using iteration
    #[pyfunction]
    fn fibonacci(n: u64) -> u64 {
        let mut a: u64 = 0;
        let mut b: u64 = 1;

        for _ in 0..n {
            let temp: u64 = b;
            b = a.wrapping_add(b);
            a = temp;
        }

        a
    }
}
```

- `#[pymodule]` declares the Python-visible module.
- `#[pyfunction]` exposes the Rust function to Python; PyO3 converts arguments and return values automatically.

## Project layout

```
fibonacci_rs/
├── .github/workflows/CI.yml   # Wheel builds & publishing (maturin-generated)
├── src/lib.rs                 # PyO3 module and fibonacci implementation
├── Cargo.toml                 # Rust manifest (pyo3 dependency, cdylib crate)
├── pyproject.toml             # Python packaging metadata (maturin backend)
└── README.md
```

## CI/CD

The GitHub Actions workflow (generated with `maturin generate-ci github`) builds wheels for:

- Linux (`x86_64`, `x86`, `aarch64`, `armv7`, `s390x`, `ppc64le` — manylinux and musllinux)
- Windows (`x64`, `x86`, `aarch64`)
- macOS (`x86_64`, `aarch64`)

It also builds an sdist. Pushing a tag triggers a release: the wheels are attested and published to PyPI with `uv publish` using the `PYPI_API_TOKEN` secret.
