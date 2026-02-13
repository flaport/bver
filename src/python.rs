use pyo3::prelude::*;

#[pyfunction]
fn cli(args: Vec<String>) {
    crate::run_from_args(args);
}

#[pymodule]
fn _bver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
