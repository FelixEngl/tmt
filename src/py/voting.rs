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

use std::sync::Arc;
use evalexpr::{Value};
use nom::error::Error;
use nom::Finish;
use pyo3::{pyclass,  pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use crate::register_python;
use crate::voting::parser::input::ParserInput;
use crate::voting::parser::{parse, InterpretedVoting};
use crate::voting::registry::VotingRegistry;
use crate::voting::{VotingMethod, VotingMethodContext, VotingResult};
use crate::voting::py::{PyContextWithMutableVariables, PyExprValue};
use crate::voting::traits::VotingMethodMarker;

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyVotingRegistry {
    inner: VotingRegistry
}

impl PyVotingRegistry {
    pub fn registry(&self) -> &VotingRegistry {
        &self.inner
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyVotingRegistry {

    #[new]
    pub fn new() -> Self {
        Self {
            inner: VotingRegistry::new()
        }
    }

    pub fn get_registered(&self, name: &str) -> Option<PyVoting> {
        self.inner.get(name).map(PyVoting::from)
    }

    pub fn register_at(&self, name: &str, voting: &str) -> PyResult<()> {
        let parsed = parse::<Error<_>>(ParserInput::new(voting, &self.inner)).finish();
        match parsed {
            Ok((_, result)) => {
                match result {
                    InterpretedVoting::BuildIn(_) => {
                        return Err(PyValueError::new_err("BuildIn functions can not be registered!".to_string()))
                    }
                    InterpretedVoting::FromRegistry(_) => {
                        return Err(PyValueError::new_err("The name is already registered!".to_string()))
                    }
                    InterpretedVoting::Parsed(parsed) => {
                        self.inner.register(name.to_string(), parsed);
                        Ok(())
                    }
                    InterpretedVoting::ForRegistry(value) => {
                        let func = Arc::new(value.1);
                        self.inner.register_arc(name.to_string(), func.clone());
                        self.inner.register_arc(value.0, func);
                        Ok(())
                    }
                    InterpretedVoting::Limited(_) => {
                        return Err(PyValueError::new_err("You can not register a limited method!".to_string()))
                    }
                }
            }
            Err(err) => {
                Err(PyValueError::new_err(err.to_string()))
            }
        }
    }

    pub fn register(&self, voting: &str) -> PyResult<()> {
        let parsed = parse::<Error<_>>(ParserInput::new(voting, &self.inner)).finish();
        match parsed {
            Ok((_, result)) => {
                match &result {
                    InterpretedVoting::BuildIn(_) => {
                        return Err(PyValueError::new_err("BuildIn functions can not be registered!".to_string()))
                    }
                    InterpretedVoting::FromRegistry(_) => {
                        return Err(PyValueError::new_err("The name is already registered!".to_string()))
                    }
                    InterpretedVoting::Parsed(_) => {
                        return Err(PyValueError::new_err("Missing the name for the registration!".to_string()))
                    }
                    InterpretedVoting::ForRegistry(value) => {
                        let func = Arc::new(value.1.clone());
                        self.inner.register_arc(value.0.to_string(), func);
                        Ok(())
                    }
                    InterpretedVoting::Limited(_) => {
                        return Err(PyValueError::new_err("You can not register a limited method!".to_string()))
                    }
                }
            }
            Err(err) => {
                Err(PyValueError::new_err(err.to_string()))
            }
        }
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyVoting(InterpretedVoting);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyVoting {
    #[staticmethod]
    #[pyo3(signature = (value, registry=None))]
    pub fn parse(value: String, registry: Option<PyVotingRegistry>) -> PyResult<Self> {
        match parse::<Error<_>>(ParserInput::new(&value, registry.unwrap_or_default().registry())).finish() {
            Ok((_, parse_result)) => {
                Ok(Self(parse_result))
            }
            Err(err) => {
                Err(PyValueError::new_err(err.to_string()))
            }
        }
    }

    //noinspection DuplicatedCode
    pub fn __call__(&self, mut global_context: PyContextWithMutableVariables, mut voters: Vec<PyContextWithMutableVariables>) -> PyResult<(PyExprValue, Vec<PyContextWithMutableVariables>)>{
        let used_voters= voters.as_mut_slice();
        match self.execute(&mut global_context, used_voters) {
            Ok(value) => {
                Ok((value.into(), used_voters.iter().cloned().collect()))
            }
            Err(err) => {
                Err(PyValueError::new_err(err.to_string()))
            }
        }
    }
}

impl VotingMethod for PyVoting {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: VotingMethodContext, B: VotingMethodContext {
        self.0.execute(global_context, voters)
    }
}

impl VotingMethodMarker for PyVoting{}

impl<T> From<T> for PyVoting where T: Into<InterpretedVoting> {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}


register_python! {
    struct PyVoting;
    struct PyVotingRegistry;
}