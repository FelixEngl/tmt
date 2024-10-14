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

use pyo3::{Bound, PyResult};
use pyo3::prelude::PyModule;
use crate::py::dictionary::dictionary_module;
use crate::py::tokenizer::tokenizer_module;
use crate::py::topic_model::topic_model_module;
use crate::py::translate::translate_module;
use crate::py::variable_provider::variable_provider_module;
use crate::py::vocabulary::vocabulary_module;
use crate::py::voting::voting_module;

pub mod topic_model;
pub mod vocabulary;
pub mod dictionary;
pub mod translate;
pub mod voting;
mod variable_provider;
pub mod helpers;
mod topic_model_builder;
mod tokenizer;
pub mod enum_mapping;


pub(crate) fn register_modules(m: &Bound<'_, PyModule>) -> PyResult<()>{
    vocabulary_module(m)?;
    dictionary_module(m)?;
    topic_model_module(m)?;
    translate_module(m)?;
    voting_module(m)?;
    variable_provider_module(m)?;
    tokenizer_module(m)?;
    Ok(())
}
