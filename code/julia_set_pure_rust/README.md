# julia_set_pure_rust

A pure-Rust implementation of the same Julia-set escape-time benchmark used by its sibling project [julia_set_rs](../julia_set_rs) — with no Python involved at all.

It runs the workload both serially and in parallel with [rayon](https://github.com/rayon-rs/rayon), checks each result against the checksum the Python scripts assert on, and verifies the two variants produce identical output. Together with [julia_pure.py](../julia_set_rs/julia_pure.py) and [julia_with_rs.py](../julia_set_rs/julia_with_rs.py), it forms a three-way comparison: pure Python vs. a Python extension in Rust vs. a standalone Rust binary.

## Prerequisites

- A Rust toolchain (install via [rustup](https://rustup.rs/))

## Usage

```bash
cargo run --release                # "both" (default): serial + parallel + cross-check
cargo run --release -- serial      # serial baseline only
cargo run --release -- parallel    # rayon variant only
```

> [!IMPORTANT]
> Build with `--release`. The dev profile is easily 10–20× slower and useless for benchmarking.

An unrecognized mode argument is rejected with a usage hint and exit code 2.

Example output:

```text
Length of x: 1000
Total elements: 1000000
serial kernel took 0.0719 seconds
serial checksum OK: 33219980
parallel kernel took 0.0091 seconds
parallel checksum OK: 33219980
serial and parallel outputs identical
```

## What it computes

The same workload as the Python scripts: a 1000×1000 grid (1,000,000 points) over the square \([-1.8, 1.8]^2\) with `c = -0.62772 - 0.42193j` and `maxiter = 300`. For each point it counts how many iterations of `z ← z² + c` pass before the point escapes the radius-2 circle. The sum of all escape counts must equal `33_219_980` — the very checksum [julia_pure.py](../julia_set_rs/julia_pure.py) and [julia_with_rs.py](../julia_set_rs/julia_with_rs.py) assert on — which is what makes the three implementations comparable.

## Benchmark

Example run on one dev machine (release builds, same machine for all rows):

| Implementation                                                                 | Time    |
| ------------------------------------------------------------------------------ | ------- |
| Pure Python ([julia_pure.py](../julia_set_rs/julia_pure.py))                   | ~2.79 s |
| Python + Rust extension ([julia_with_rs.py](../julia_set_rs/julia_with_rs.py)) | ~0.14 s |
| Pure Rust, serial                                                              | ~0.07 s |
| Pure Rust, rayon parallel                                                      | ~0.01 s |

Compared with pure Python that is roughly **40×** for the serial binary and **300×** for the parallel one. Note the pure-Rust serial run is also ~2× faster than the PyO3 extension: the extension pays for crossing the Python/Rust boundary (converting lists of `complex` into `(f64, f64)` pairs), while this binary keeps everything in native memory. Exact numbers vary by hardware.

## How it works

The whole crate is [src/main.rs](src/main.rs). A few details worth knowing:

- **Bit-identical grid.** The coordinate lists are built with the same cumulative `+= step` loop order as the Python version, so the `f64` values match Python's exactly and the escape-count checksum agrees — not just approximately.
- **No `sqrt` in the hot loop.** The escape test uses the squared modulus `re² + im² < 4` instead of `abs(z) < 2`.
- **Row-level parallelism.** Rows are independent, so the flat output buffer is split into row-sized chunks and filled with `par_chunks_mut` — one rayon task per row, no locking.

## Project layout

```
julia_set_pure_rust/
├── src/main.rs     # Serial + rayon-parallel benchmark, checksum verification
├── Cargo.toml      # Manifest (rayon dependency)
└── .gitignore
```

## Development

```bash
cargo fmt --check
cargo clippy
```

Both are expected to pass clean.
