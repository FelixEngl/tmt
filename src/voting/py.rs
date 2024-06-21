use evalexpr::{ContextWithMutableVariables, EvalexprError, FloatType, IntType, Value};
use itertools::Itertools;
use pyo3::{Bound, FromPyObject, IntoPy, PyAny, pyclass, pymethods, PyObject, PyResult, Python};
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::types::PyFunction;
use crate::voting::traits::{RootVotingMethodMarker, VotingMethodMarker};
use crate::voting::{VotingExpressionError, VotingMethod, VotingResult};

#[derive(Debug, Clone, FromPyObject)]
#[repr(transparent)]
pub struct PyVotingModel<'a> {
    model: &'a PyAny
}

impl<'a> PyVotingModel<'a> {
    pub fn new(model: &'a PyFunction) -> Self {
        Self{model}
    }
}

unsafe impl Send for PyVotingModel<'_> {}
unsafe impl Sync for PyVotingModel<'_> {}
impl RootVotingMethodMarker for PyVotingModel<'_> {}
impl VotingMethodMarker for PyVotingModel<'_> {}
impl<'a> VotingMethod for PyVotingModel<'a> {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        unsafe {
            let global_context = PyContextWithMutableVariables::new(global_context);
            let voters = voters.iter_mut().map(|value| PyContextWithMutableVariables::new(value)).collect_vec();
            let result = self.model.call1((global_context, voters)).map_err(VotingExpressionError::PythonError)?;
            let py_expr_value: PyExprValue = result.extract().map_err(VotingExpressionError::PythonError)?;
            Ok(py_expr_value.into())
        }
    }
}

#[derive(Clone, Debug)]
pub enum PyExprValue {
    /// A string value.
    String(String),
    /// A float value.
    Float(FloatType),
    /// An integer value.
    Int(IntType),
    /// A boolean value.
    Boolean(bool),
    /// A tuple value.
    Tuple(Vec<PyExprValue>),
    /// An empty value.
    Empty,
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

impl<'a> FromPyObject<'a> for PyExprValue {
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        if ob.is_none() {
            Ok(PyExprValue::Empty)
        } else {
            if let Ok(value) = ob.extract::<String>() {
                return Ok(PyExprValue::String(value))
            }
            if let Ok(value) = ob.extract::<IntType>() {
                return Ok(PyExprValue::Int(value))
            }
            if let Ok(value) = ob.extract::<FloatType>() {
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

impl IntoPy<PyObject> for PyExprValue {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PyExprValue::String(value) => {
                value.into_py(py)
            }
            PyExprValue::Float(value) => {
                value.into_py(py)
            }
            PyExprValue::Int(value) => {
                value.into_py(py)
            }
            PyExprValue::Boolean(value) => {
                value.into_py(py)
            }
            PyExprValue::Tuple(value) => {
                value.into_py(py)
            }
            PyExprValue::Empty => {
                py.None()
            }
        }
    }
}


#[pyclass]
#[derive(Copy, Clone, Debug)]
struct PyContextWithMutableVariables {
    inner: *mut dyn ContextWithMutableVariables
}

unsafe impl Send for PyContextWithMutableVariables {}
unsafe impl Sync for PyContextWithMutableVariables {}

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
}


impl PyContextWithMutableVariables {
    unsafe fn new<'a>(value: &'a mut dyn ContextWithMutableVariables) -> Self {
        // Transmute does not change the real lifeline!
        let value: &'static mut dyn ContextWithMutableVariables = std::mem::transmute::<&'a mut dyn ContextWithMutableVariables, &'static mut dyn ContextWithMutableVariables>(value);
        Self {
            inner: value
        }
    }
}

pub(crate) fn register_py_voting_filters(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyContextWithMutableVariables>()?;
    Ok(())
}