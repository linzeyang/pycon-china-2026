//! Usage:
//!   julia_set_pure_rust [serial | parallel | both]
//!
//! "both" (the default) runs the serial baseline and the rayon parallel
//! variant, checks each against the Python script's known checksum, and
//! verifies the two outputs are identical.

use std::env;
use std::time::Instant;

use rayon::prelude::*;

// Area of complex space to investigate.
const X1: f64 = -1.8;
const X2: f64 = 1.8;
const Y1: f64 = -1.8;
const Y2: f64 = 1.8;
const C_REAL: f64 = -0.62772;
const C_IMAG: f64 = -0.42193;

const DESIRED_WIDTH: f64 = 1_000.0;
const MAXITER: u32 = 300;

// Python script's self-check: sum of all escape counts.
const EXPECTED_SUM: u64 = 33_219_980;

/// Build the coordinate lists exactly like the Python version does: same
/// cumulative `+= step` loop order, so the f64 values are bit-identical to
/// Python's and the escape-count checksum matches exactly.
fn build_coords() -> (Vec<f64>, Vec<f64>) {
    let x_step: f64 = (X2 - X1) / DESIRED_WIDTH;
    let y_step: f64 = (Y1 - Y2) / DESIRED_WIDTH;

    let mut y: Vec<f64> = Vec::new();
    let mut ycoord: f64 = Y2;
    while ycoord > Y1 {
        y.push(ycoord);
        ycoord += y_step;
    }

    let mut x: Vec<f64> = Vec::new();
    let mut xcoord: f64 = X1;
    while xcoord < X2 {
        x.push(xcoord);
        xcoord += x_step;
    }

    (x, y)
}

/// Escape iteration count for one starting point, using the squared modulus
/// `re^2 + im^2 < 4` instead of `abs(z) < 2` to avoid a sqrt per iteration.
#[inline]
fn escape_count(mut zre: f64, mut zim: f64) -> u32 {
    let mut n: u32 = 0;

    while n < MAXITER && zre * zre + zim * zim < 4.0 {
        // (re + i*im)^2 + c
        let new_re = zre * zre - zim * zim + C_REAL;
        zim = 2.0 * zre * zim + C_IMAG;
        zre = new_re;
        n += 1;
    }

    n
}

/// Serial baseline: row-major over y then x, like the Python loop.
fn julia_serial(x: &[f64], y: &[f64]) -> Vec<u32> {
    let mut output: Vec<u32> = Vec::with_capacity(x.len() * y.len());

    for &ycoord in y {
        for &xcoord in x {
            output.push(escape_count(xcoord, ycoord));
        }
    }

    output
}

/// Parallel variant: rows are independent, so split the flat output into
/// row-sized chunks and compute them with rayon.
fn julia_parallel(x: &[f64], y: &[f64]) -> Vec<u32> {
    let mut output: Vec<u32> = vec![0u32; x.len() * y.len()];

    output
        .par_chunks_mut(x.len())
        .enumerate()
        .for_each(|(row, chunk)| {
            let ycoord: f64 = y[row];
            for (col, cell) in chunk.iter_mut().enumerate() {
                *cell = escape_count(x[col], ycoord);
            }
        });

    output
}

fn run_and_report(label: &str, output: Vec<u32>, elapsed: f64) -> Vec<u32> {
    let sum: u64 = output.iter().map(|&v| u64::from(v)).sum();
    println!("{label} kernel took {elapsed:.4} seconds");
    assert_eq!(sum, EXPECTED_SUM, "{label} checksum mismatch: got {sum}");
    println!("{label} checksum OK: {sum}");
    output
}

fn main() {
    let mode: String = env::args().nth(1).unwrap_or_else(|| "both".into());
    if !matches!(mode.as_str(), "serial" | "parallel" | "both") {
        eprintln!("error: unknown mode {mode:?}; expected \"serial\", \"parallel\", or \"both\"");
        std::process::exit(2);
    }

    let (x, y) = build_coords();
    println!("Length of x: {}", x.len());
    println!("Total elements: {}", x.len() * y.len());

    let serial: Option<Vec<u32>> = match mode.as_str() {
        "parallel" => None,
        _ => {
            let t: Instant = Instant::now();
            let out: Vec<u32> = julia_serial(&x, &y);
            Some(run_and_report("serial", out, t.elapsed().as_secs_f64()))
        }
    };

    if mode != "serial" {
        let t: Instant = Instant::now();
        let out: Vec<u32> = julia_parallel(&x, &y);
        let out: Vec<u32> = run_and_report("parallel", out, t.elapsed().as_secs_f64());

        if let Some(s) = serial {
            assert_eq!(s, out, "serial and parallel outputs differ");
            println!("serial and parallel outputs identical");
        }
    }
}
