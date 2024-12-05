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

use std::collections::{HashMap};
use std::sync::Arc;
use evalexpr::{Context, ContextWithMutableVariables, EvalexprError, EvalexprResult, Value};
use itertools::Itertools;
use nom::error::Error;
use nom::Finish;
use pyo3::{Bound, FromPyObject, PyAny, pyclass, pymethods, PyResult, IntoPyObject, Python, PyErr, IntoPyObjectExt};
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::{PyAnyMethods};
use pyo3::types::PyFunction;
use ldatranslate_toolkit::{impl_py_stub, impl_py_type_def, register_python};
use crate::traits::{RootVotingMethodMarker, VotingMethodMarker};
use crate::{VotingExpressionError, VotingMethod, VotingMethodContext, VotingResult};
use crate::constants::TMTNumericTypes;
use crate::parser::{parse, InterpretedVoting};
use crate::parser::input::ParserInput;
use crate::registry::VotingRegistry;

/// A voting model based on a python method.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyVotingModel<'a> {
    model: Bound<'a, PyFunction>
}

impl<'a> FromPyObject<'a> for PyVotingModel<'a> {
    fn extract_bound(ob: &Bound<'a, PyAny>) -> PyResult<Self> {
        Ok(Self { model: ob.downcast()?.clone() })
    }
}

impl_py_stub!(PyVotingModel<'_>  {
        output: {
            let mut input_typ = PyContextWithMutableVariables::type_output();
            let output_typ = PyExprValue::type_output();
            let name = format!(
                "typing.Callable[[{inp}, list[{inp}]], {out}]",
                inp=input_typ.name,
                out=output_typ.name
            );
            input_typ.import.insert("typing".into());
            TypeInfo {
                name,
                import: input_typ.import
            }
        }
    }
);

unsafe impl Send for PyVotingModel<'_> {}
unsafe impl Sync for PyVotingModel<'_> {}
impl RootVotingMethodMarker for PyVotingModel<'_> {}
impl VotingMethodMarker for PyVotingModel<'_> {}
impl<'a> VotingMethod for PyVotingModel<'a> {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: VotingMethodContext, B: VotingMethodContext {
        unsafe {
            let global_context = PyContextWithMutableVariables::new(global_context);
            let voters = voters.iter_mut().map(|value| PyContextWithMutableVariables::new(value)).collect_vec();
            let result = self.model.call1((global_context, voters)).map_err(VotingExpressionError::PythonError)?;
            let py_expr_value: PyExprValue = result.extract().map_err(VotingExpressionError::PythonError)?;
            Ok(py_expr_value.into())
        }
    }
}

/// The value that can be returned by the [PyVotingModel]
#[derive(Clone, Debug)]
pub enum PyExprValue {
    /// A string value.
    String(String),
    /// A float value.
    Float(f64),
    /// An integer value.
    Int(i64),
    /// A boolean value.
    Boolean(bool),
    /// A tuple value.
    Tuple(Vec<PyExprValue>),
    /// An empty value.
    Empty,
}


impl_py_type_def! {
    PyExprValueSingle; {
        output: {
            builder()
            .add_name("str | float | int | bool | None | list[PyExprValueSingle]")
            .build_output()
        }
        input: {
            builder()
            .add_name("str | float | int | bool | None | typing.Sequence[PyExprValueSingle]")
            .add_import("typing")
            .build_input()
        }
    }
}


impl_py_stub!(PyExprValue: PyExprValueSingle, Vec<PyExprValueSingle>);

impl<'a> FromPyObject<'a> for PyExprValue {

    fn extract_bound(ob: &Bound<'a, PyAny>) -> PyResult<Self> {
        if ob.is_none() {
            Ok(PyExprValue::Empty)
        } else {
            if let Ok(value) = ob.extract::<String>() {
                return Ok(PyExprValue::String(value))
            }
            if let Ok(value) = ob.extract::<i64>() {
                return Ok(PyExprValue::Int(value))
            }
            if let Ok(value) = ob.extract::<f64>() {
                return Ok(PyExprValue::Float(value))
            }
            if let Ok(value) = ob.extract::<bool>() {
                return Ok(PyExprValue::Boolean(value))
            }
            if let Ok(value) = ob.extract::<Vec<PyExprValue>>() {
                return Ok(PyExprValue::Tuple(value))
            }
            Err(PyValueError::new_err(format!("The value {ob} is not supported!")))
        }
    }
}

impl<'py> IntoPyObject<'py> for PyExprValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            PyExprValue::String(value) => {
                value.into_bound_py_any(py)
            }
            PyExprValue::Float(value) => {
                value.into_bound_py_any(py)
            }
            PyExprValue::Int(value) => {
                value.into_bound_py_any(py)
            }
            PyExprValue::Boolean(value) => {
                value.into_bound_py_any(py)
            }
            PyExprValue::Tuple(value) => {
                value.into_bound_py_any(py)
            }
            PyExprValue::Empty => {
                py.None().into_bound_py_any(py)
            }
        }
    }
}


impl From<Value> for PyExprValue {
    fn from(value: Value) -> Self {
        match value {
            Value::String(value) => {PyExprValue::String(value)}
            Value::Float(value) => {PyExprValue::Float(value)}
            Value::Int(value) => {PyExprValue::Int(value)}
            Value::Boolean(value) => {PyExprValue::Boolean(value)}
            Value::Tuple(value) => {PyExprValue::Tuple(value.into_iter().map(|value| value.into()).collect())}
            Value::Empty => {PyExprValue::Empty}
        }
    }
}

impl Into<Value> for PyExprValue {
    fn into(self) -> Value {
        match self {
            PyExprValue::String(value) => {Value::String(value)}
            PyExprValue::Float(value) => {Value::Float(value)}
            PyExprValue::Int(value) => {Value::Int(value)}
            PyExprValue::Boolean(value) => {Value::Boolean(value)}
            PyExprValue::Tuple(value) => {Value::Tuple(value.into_iter().map(|value| value.into()).collect())}
            PyExprValue::Empty => {Value::Empty}
        }
    }
}


/// This is an unsafe reference to a VotingMethodContext.
/// If a python user saves them outside of the method, there will be a memory error.
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Copy, Clone, Debug)]
pub struct PyContextWithMutableVariables {
    inner: *mut dyn VotingMethodContext
}

unsafe impl Send for PyContextWithMutableVariables {}
unsafe impl Sync for PyContextWithMutableVariables {}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContextWithMutableVariables {
    pub fn __getitem__(&self, item: &str) -> PyResult<PyExprValue> {
        if let Some(found) = unsafe{&*self.inner}.get_value(item) {
            Ok(found.clone().into())
        } else {
            Err(PyKeyError::new_err(format!("No value found for {item}!")))
        }
    }

    pub fn __setitem__(&mut self, key: String, value: PyExprValue) -> PyResult<()> {
        unsafe{&mut *self.inner}.set_value(key, value.into()).map_err(|err| {
            match err {
                EvalexprError::ContextNotMutable => {
                    PyValueError::new_err("Context not mutable!")
                }
                EvalexprError::CustomMessage(message) => {
                    PyValueError::new_err(format!("Custom: {message}"))
                }
                otherwise => PyValueError::new_err(otherwise.to_string())
            }
        })
    }

    pub fn __contains__(&self, item: &str) -> bool {
        unsafe{&*self.inner}.get_value(item).is_some()
    }

    pub fn get_all_values(&self) -> HashMap<String, PyExprValue> {
        unsafe{&*self.inner}
            .variable_map()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }
}


impl PyContextWithMutableVariables {
    unsafe fn new<'a>(value: &'a mut dyn VotingMethodContext ) -> Self {
        // Transmute does not change the real lifeline!
        let value: &'static mut dyn VotingMethodContext = std::mem::transmute::<&'a mut dyn VotingMethodContext, &'static mut dyn VotingMethodContext>(value);
        Self {
            inner: value
        }
    }
}

impl Context for PyContextWithMutableVariables {
    type NumericTypes = TMTNumericTypes;
    delegate::delegate! {
        to unsafe{&*self.inner} {
            fn get_value(&self, identifier: &str) -> Option<&Value>;
            fn call_function(&self, identifier: &str, argument: &Value) -> EvalexprResult<Value>;
            fn are_builtin_functions_disabled(&self) -> bool;
        }
    }

    delegate::delegate! {
        to unsafe{&mut *self.inner} {
            fn set_builtin_functions_disabled(&mut self, disabled: bool) -> EvalexprResult<()>;
        }
    }
}

impl ContextWithMutableVariables for PyContextWithMutableVariables {
    delegate::delegate! {
        to unsafe{&mut *self.inner} {
            fn set_value(&mut self, _identifier: String, _value: Value) -> EvalexprResult<()>;
        }
    }
}

impl VotingMethodContext for PyContextWithMutableVariables {
    fn variable_map(&self) -> HashMap<String, Value> {
        unsafe{&*self.inner}.variable_map()
    }
}


register_python! {
    struct PyContextWithMutableVariables;
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
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<TMTNumericTypes>>
    where
        A : VotingMethodContext,
        B : VotingMethodContext
    {
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
}


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
                        Err(PyValueError::new_err("BuildIn functions can not be registered!".to_string()))
                    }
                    InterpretedVoting::FromRegistry(_) => {
                        Err(PyValueError::new_err("The name is already registered!".to_string()))
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
                        Err(PyValueError::new_err("You can not register a limited method!".to_string()))
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





register_python! {
    struct PyVotingRegistry;
}