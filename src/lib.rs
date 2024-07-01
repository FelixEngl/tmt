use pyo3::{Bound, pymodule, PyResult};
use pyo3::prelude::PyModule;
use crate::py::register_modules;

mod topicmodel;
mod translate;
mod voting;
mod toolkit;
mod variable_names;
pub mod py;
mod external_variable_provider;
pub mod aligned_data;
mod tokenizer;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
pub fn ldatranslate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_modules(m)
}