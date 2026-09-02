use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod julia_set_rs {
    use pyo3::prelude::*;
    use pyo3::types::{PyComplex, PyList};

    /// Pull the re/im parts out of a list of Python complex numbers so the
    /// hot loop below works on plain f64 pairs.
    fn extract_re_im(list: &Bound<'_, PyList>) -> PyResult<Vec<(f64, f64)>> {
        let mut out: Vec<(f64, f64)> = Vec::with_capacity(list.len());

        for item in list.iter() {
            let c: &Bound<'_, PyComplex> = item.cast::<PyComplex>().map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err("expected a list of complex numbers")
            })?;
            out.push((c.real(), c.imag()));
        }

        Ok(out)
    }

    /// Compute the Julia-set escape time for each point in a batch.
    ///
    /// For every complex starting value `z` in `zs`, paired with the complex
    /// constant `c` at the same index in `cs`, iterates the Julia update rule
    /// `z <- z^2 + c` until the point escapes the radius-2 circle around the
    /// origin (`|z|^2 >= 4`) or `maxiter` iterations have been performed.
    /// Points that never escape are reported as `maxiter`.
    ///
    /// # Arguments
    ///
    /// * `maxiter` - Maximum number of iterations per point.
    /// * `zs` - List of complex starting values.
    /// * `cs` - List of complex constants, one per starting value in `zs`.
    ///
    /// # Returns
    ///
    /// A list of `int` with one escape-time count per input point, in input
    /// order.
    ///
    /// # Raises
    ///
    /// * `TypeError` if `zs` or `cs` contains a non-complex item.
    /// * `ValueError` if `zs` and `cs` have different lengths.
    ///
    /// # Examples
    ///
    /// ```python
    /// >>> julia_set_rs.calculate_z_serial(20, [0j], [1 + 0j])
    /// [2]
    /// >>> julia_set_rs.calculate_z_serial(20, [0j], [0j])
    /// [20]
    /// ```
    #[pyfunction]
    fn calculate_z_serial(
        maxiter: u32,
        zs: &Bound<'_, PyList>,
        cs: &Bound<'_, PyList>,
    ) -> PyResult<Vec<u32>> {
        if zs.len() != cs.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "zs and cs must have the same length ({} != {})",
                zs.len(),
                cs.len()
            )));
        }

        let zs: Vec<(f64, f64)> = extract_re_im(zs)?;
        let cs: Vec<(f64, f64)> = extract_re_im(cs)?;

        let mut output: Vec<u32> = vec![0; zs.len()];

        for (idx, z) in zs.iter().enumerate() {
            let mut n: u32 = 0;
            let (cre, cimg): (f64, f64) = cs[idx];
            let (mut zre, mut zim): (f64, f64) = *z;

            while n < maxiter && zre * zre + zim * zim < 4.0 {
                // (re + i*im)^2 + c
                let new_re: f64 = zre * zre - zim * zim + cre;
                zim = 2.0 * zre * zim + cimg;
                zre = new_re;
                n += 1;
            }

            output[idx] = n;
        }

        Ok(output)
    }
}
