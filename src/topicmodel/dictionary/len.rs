use std::fmt::{Debug, Display, Formatter};
use pyo3::{pyclass, pymethods};
use crate::register_python;

register_python!(struct Len;);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(frozen, eq, hash)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Len {
    #[pyo3(get)]
    pub voc_a: usize,
    #[pyo3(get)]
    pub voc_b: usize,
    #[pyo3(get)]
    pub map_a_to_b: usize,
    #[pyo3(get)]
    pub map_b_to_a: usize,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Len {
    fn __str__(&self) -> String {
        self.to_string()
    }

    pub fn diff(&self, other: &Len) -> Self {
        Self {
            voc_a: self.voc_a.abs_diff(other.voc_a),
            voc_b: self.voc_b.abs_diff(other.voc_b),
            map_a_to_b: self.map_a_to_b.abs_diff(other.map_a_to_b),
            map_b_to_a: self.map_b_to_a.abs_diff(other.map_b_to_a),
        }
    }
}

impl Display for Len {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
