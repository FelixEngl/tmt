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

use pyo3::{Bound, pymodule, PyResult};
use pyo3::prelude::PyModule;
use crate::py::register_modules;

pub mod topicmodel;
pub mod translate;
pub mod voting;
pub mod toolkit;
pub mod variable_names;
pub mod py;
mod external_variable_provider;
pub mod aligned_data;
pub mod tokenizer;

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
pub fn ldatranslate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_modules(m)
}