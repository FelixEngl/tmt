use std::sync::Arc;
use evalexpr::{ContextWithMutableVariables, Value};
use nom::error::Error;
use nom::Finish;
use pyo3::{Bound, pyclass,  pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use crate::voting::parser::input::ParserInput;
use crate::voting::parser::{parse, ParseResult};
use crate::voting::registry::VotingRegistry;
use crate::voting::{VotingMethod, VotingResult};
use crate::voting::traits::VotingMethodMarker;

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

#[pymethods]
impl PyVotingRegistry {

    pub fn get_registered(&self, name: &str) -> Option<PyVoting> {
        self.inner.get(name).map(PyVoting::from)
    }

    pub fn register_at(&self, name: &str, voting: &str) -> PyResult<()> {
        let parsed = parse::<Error<_>>(ParserInput::new(voting, &self.inner)).finish();
        match parsed {
            Ok((_, result)) => {
                match result {
                    ParseResult::BuildIn(_) => {
                        return Err(PyValueError::new_err("BuildIn functions can not be registered!".to_string()))
                    }
                    ParseResult::FromRegistry(_) => {
                        return Err(PyValueError::new_err("The name is already registered!".to_string()))
                    }
                    ParseResult::Parsed(parsed) => {
                        self.inner.register(name.to_string(), parsed);
                        Ok(())
                    }
                    ParseResult::ForRegistry(value) => {
                        let func = Arc::new(value.1);
                        self.inner.register_arc(name.to_string(), func.clone());
                        self.inner.register_arc(value.0, func);
                        Ok(())
                    }
                    ParseResult::Limited(_) => {
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
                    ParseResult::BuildIn(_) => {
                        return Err(PyValueError::new_err("BuildIn functions can not be registered!".to_string()))
                    }
                    ParseResult::FromRegistry(_) => {
                        return Err(PyValueError::new_err("The name is already registered!".to_string()))
                    }
                    ParseResult::Parsed(_) => {
                        return Err(PyValueError::new_err("Missing the name for the registration!".to_string()))
                    }
                    ParseResult::ForRegistry(value) => {
                        let func = Arc::new(value.1.clone());
                        self.inner.register_arc(value.0.to_string(), func);
                        Ok(())
                    }
                    ParseResult::Limited(_) => {
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


#[pyclass]
#[derive(Clone, Debug)]
pub struct PyVoting(ParseResult);

#[pymethods]
impl PyVoting {
    #[staticmethod]
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
}

impl VotingMethod for PyVoting {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        self.0.execute(global_context, voters)
    }
}

impl VotingMethodMarker for PyVoting{}

impl<T> From<T> for PyVoting where T: Into<ParseResult> {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub(crate) fn voting_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVotingRegistry>()?;
    m.add_class::<PyVoting>()?;
    Ok(())
}