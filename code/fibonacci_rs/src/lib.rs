use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod fibonacci_rs {
    use pyo3::prelude::*;

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
