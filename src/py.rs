use pyo3::{Bound, PyResult};
use pyo3::prelude::PyModule;
use crate::py::dictionary::{dictionary_module};
use crate::py::topic_model::{topic_model_module};
use crate::py::translate::{translate_module};
use crate::py::vocabulary::vocabulary_module;
use crate::py::voting::voting_module;

pub mod topic_model;
pub mod vocabulary;
pub mod dictionary;
pub mod translate;
pub mod voting;


pub(crate) fn register_modules(m: &Bound<'_, PyModule>) -> PyResult<()>{
    vocabulary_module(m)?;
    dictionary_module(m)?;
    topic_model_module(m)?;
    translate_module(m)?;
    voting_module(m)
}