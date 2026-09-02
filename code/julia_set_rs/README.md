# julia_set_rs

A Julia-set escape-time calculator implemented as a Python extension module in Rust with [PyO3](https://github.com/PyO3/pyo3), packaged with [maturin](https://github.com/PyO3/maturin).

It exposes a single function, `calculate_z_serial(maxiter, zs, cs)`, which computes — for a batch of complex points — how many iterations of the Julia update rule `z ← z² + c` it takes for each point to escape. A pure-Python implementation ([julia_pure.py](julia_pure.py)) is included as a baseline, so you can profile and compare the two side by side.

## Prerequisites

- Python 3.8+
- A Rust toolchain (install via [rustup](https://rustup.rs/))
- [maturin](https://github.com/PyO3/maturin) (`uv tool install maturin`)
- Python dependencies ([requirements.txt](requirements.txt)): `line_profiler` for profiling the demo scripts

## Getting started

Create and activate a virtual environment, then build and install the extension into it:

```bash
uv venv
source .venv/bin/activate

uv pip install -r requirements.txt
maturin develop --release
```

`maturin develop` compiles the Rust crate and installs the resulting extension module into the active environment, so you can iterate quickly during development.

## Usage

```python
>>> import julia_set_rs
>>> julia_set_rs.calculate_z_serial(20, [0j], [1 + 0j])
[2]
>>> julia_set_rs.calculate_z_serial(20, [0j], [0j])  # never escapes -> maxiter
[20]
```

Errors: a `TypeError` is raised if `zs`/`cs` contain non-complex items, and a `ValueError` if the two lists differ in length. See `help(julia_set_rs.calculate_z_serial)` for the full docstring.

## Benchmark

[julia_pure.py](julia_pure.py) and [julia_with_rs.py](julia_with_rs.py) build the same workload — a 1000×1000 grid (1,000,000 points) over the square \([-1.8, 1.8]^2\) with `c = -0.62772 - 0.42193j` and `maxiter = 300` — and time the escape-time computation. Both verify the result with `assert sum(output) == 33_219_980`.

```bash
python julia_pure.py     # pure-Python baseline
python julia_with_rs.py  # Rust extension
```

Example run on one dev machine (release build):

| Implementation | Time    |
| -------------- | ------- |
| Pure Python    | ~2.79 s |
| `julia_set_rs` | ~0.14 s |

That is roughly a **19× speedup**; exact numbers vary by hardware.

## Profiling

Both demo scripts decorate their driver function with `@profile` from [line_profiler](https://github.com/pyutils/line_profiler). To get per-line timings:

```bash
kernprof -l -v julia_pure.py
kernprof -l -v julia_with_rs.py
```

This writes a `julia_with_rs.py.lprof` (or `julia_pure.py.lprof`) profile file and prints a line-by-line report — handy for seeing where the pure-Python version spends its time versus the Rust one.

## How it works

The extension lives in [src/lib.rs](src/lib.rs). Two details make the hot loop fast:

- Input `complex` values are unpacked once into plain `(f64, f64)` pairs, so the iteration loop runs entirely on native doubles with no Python object traffic.
- The escape test uses `zre² + zim² < 4.0` instead of `abs(z) < 2`, avoiding a square root per iteration (mathematically equivalent).

```rust
while n < maxiter && zre * zre + zim * zim < 4.0 {
    // (re + i*im)^2 + c
    let new_re: f64 = zre * zre - zim * zim + cre;
    zim = 2.0 * zre * zim + cimg;
    zre = new_re;
    n += 1;
}
```

## Project layout

```
julia_set_rs/
├── .github/workflows/CI.yml    # Wheel builds & publishing (maturin-generated)
├── src/lib.rs                  # PyO3 module and calculate_z_serial implementation
├── julia_pure.py               # Pure-Python baseline benchmark
├── julia_with_rs.py            # Benchmark using the Rust extension
├── requirements.txt            # Python dependencies (line_profiler)
├── Cargo.toml                  # Rust manifest (pyo3 dependency, cdylib crate)
├── pyproject.toml              # Python packaging metadata (maturin backend)
└── README.md
```

## CI/CD

The GitHub Actions workflow (generated with `maturin generate-ci github`) builds wheels for:

- Linux (`x86_64`, `x86`, `aarch64`, `armv7`, `s390x`, `ppc64le` — manylinux and musllinux)
- Windows (`x64`, `x86`, `aarch64`)
- macOS (`x86_64`, `aarch64`)

It also builds an sdist. Pushing a tag triggers a release: the wheels are attested and published to PyPI with `uv publish` using the `PYPI_API_TOKEN` secret.
