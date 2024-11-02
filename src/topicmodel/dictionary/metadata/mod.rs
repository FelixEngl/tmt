//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

pub(super) mod dictionary;
pub mod containers;
pub mod domain_matrix;

pub use containers::*;

use pyo3::{Bound, PyResult};
use pyo3::prelude::{PyModule, PyModuleMethods};
use crate::topicmodel::dictionary::metadata::loaded::register_loaded;

pub(crate) fn register_py_metadata(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<classic::python::SolvedMetadata>()?;
    register_loaded(m)?;
    Ok(())
}

