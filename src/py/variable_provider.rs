use evalexpr::{ContextWithMutableVariables, Value};
use pyo3::{Bound, FromPyObject, PyAny, pyclass, PyErr, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use crate::external_variable_provider::{VariableProvider, VariableProviderError, VariableProviderOut, VariableProviderResult};
use crate::py::dictionary::PyDictionary;
use crate::py::topic_model::PyTopicModel;
use crate::topicmodel::dictionary::{DictionaryMut};
use crate::topicmodel::dictionary::direction::AToB;
use crate::topicmodel::topic_model::{BasicTopicModel};
use crate::topicmodel::vocabulary::Vocabulary;


#[derive(FromPyObject)]
pub enum ProviderValue<'a> {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
    #[pyo3(transparent)]
    CatchAll(&'a PyAny)
}

impl<'a> TryFrom<ProviderValue<'a>> for Value {
    type Error = PyErr;

    fn try_from(value: ProviderValue<'a>) -> Result<Self, Self::Error> {
        match value {
            ProviderValue::Float(value) => {Ok(value.into())}
            ProviderValue::Int(value) => {Ok(value.into())}
            ProviderValue::Bool(value) => {Ok(value.into())}
            ProviderValue::String(value) => {Ok(value.into())}
            ProviderValue::CatchAll(_) => {Err(PyValueError::new_err("Not a bool, int, float or string!".to_string()))}
        }
    }
}

impl From<VariableProviderError> for PyErr {
    fn from(value: VariableProviderError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyVariableProvider {
    inner: VariableProvider
}

impl PyVariableProvider {
    pub(crate) fn new(topic_count: usize, word_count_a: usize, word_count_b: usize) -> Self {
        Self {
            inner: VariableProvider::new(topic_count, word_count_a, word_count_b)
        }
    }
}


#[pymethods]
impl PyVariableProvider {

    #[new]
    pub fn new_with(model: &PyTopicModel, dictionary: &PyDictionary) -> Self {
        Self::new(
            model.topic_count(),
            dictionary.voc_a().len(),
            dictionary.voc_b().len()
        )
    }

    #[staticmethod]
    pub fn builder(model: &PyTopicModel, dictionary: PyDictionary) -> PyVariableProviderBuilder {
        let new = Self::new_with(model, &dictionary);
        PyVariableProviderBuilder::new(dictionary, new)
    }

    pub fn add_global<'a>(&self, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        Ok(self.inner.add_global(key, Value::try_from(value)?)?)
    }
    pub fn add_for_topic<'a>(&self, topic_id: usize, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        Ok(self.inner.add_for_topic(topic_id, key, Value::try_from(value)?)?)
    }
    pub fn add_for_word_a<'a>(&self, word_id: usize, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        Ok(self.inner.add_for_word_a(word_id, key, Value::try_from(value)?)?)
    }
    pub fn add_for_word_b<'a>(&self, word_id: usize, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        Ok(self.inner.add_for_word_b(word_id, key, Value::try_from(value)?)?)
    }
    pub fn add_for_word_in_topic_a<'a>(&self, topic_id: usize, word_id: usize, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        Ok(self.inner.add_for_word_in_topic_a(topic_id, word_id, key, Value::try_from(value)?)?)
    }

    pub fn add_for_word_in_topic_b<'a>(&self, topic_id: usize, word_id: usize, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        Ok(self.inner.add_for_word_in_topic_b(topic_id, word_id, key, Value::try_from(value)?)?)
    }
}

impl VariableProviderOut for PyVariableProvider {
    delegate::delegate! {
        to self.inner {
            fn provide_global(&self, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_topic(&self, topic_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_word_a(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_word_b(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyVariableProviderBuilder {
    dict: PyDictionary,
    inner: PyVariableProvider
}


impl PyVariableProviderBuilder {
    pub(crate) fn new(dict: PyDictionary, provider: PyVariableProvider) -> Self {
        Self {
            dict,
            inner: provider
        }
    }
}

#[derive(FromPyObject)]
pub enum StrOrInt<'a> {
    String(String),
    Int(usize),
    #[pyo3(transparent)]
    CatchAll(&'a PyAny)
}


#[pymethods]
impl PyVariableProviderBuilder {

    pub fn add_global<'a>(&self, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        self.inner.add_global(key, value)
    }

    pub fn add_for_topic<'a>(&self, topic_id: usize, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        self.inner.add_for_topic(topic_id, key, value)
    }

    pub fn add_for_word_a<'a>(&self, word_id: StrOrInt<'a>, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        match word_id {
            StrOrInt::String(s) => {
                if let Some(trans) = self.dict.word_to_id::<AToB, _>(&s) {
                    self.inner.add_for_word_a(trans, key, value)
                } else {
                    Err(PyValueError::new_err("Value was not found!".to_string()))
                }
            }
            StrOrInt::Int(i) => {
                self.inner.add_for_word_a(i, key, value)
            }
            StrOrInt::CatchAll(_) => {
                Err(PyValueError::new_err("Value not a int or str!".to_string()))
            }
        }
    }
    pub fn add_for_word_b<'a>(&self, word_id: StrOrInt<'a>, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        match word_id {
            StrOrInt::String(s) => {
                if let Some(trans) = self.dict.word_to_id::<AToB, _>(&s) {
                    self.inner.add_for_word_b(trans, key, value)
                } else {
                    Err(PyValueError::new_err("Value was not found!".to_string()))
                }
            }
            StrOrInt::Int(i) => {
                self.inner.add_for_word_b(i, key, value)
            }
            StrOrInt::CatchAll(_) => {
                Err(PyValueError::new_err("Value not a int or str!".to_string()))
            }
        }
    }
    pub fn add_for_word_in_topic_a<'a>(&self, topic_id: usize, word_id: StrOrInt<'a>, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        match word_id {
            StrOrInt::String(s) => {
                if let Some(trans) = self.dict.word_to_id::<AToB, _>(&s) {
                    self.inner.add_for_word_in_topic_a(topic_id, trans, key, value)
                } else {
                    Err(PyValueError::new_err("Value was not found!".to_string()))
                }
            }
            StrOrInt::Int(i) => {
                self.inner.add_for_word_in_topic_a(topic_id, i, key, value)
            }
            StrOrInt::CatchAll(_) => {
                Err(PyValueError::new_err("Value not a int or str!".to_string()))
            }
        }
    }
    pub fn add_for_word_in_topic_b<'a>(&self, topic_id: usize, word_id: StrOrInt<'a>, key: &str, value: ProviderValue<'a>) -> PyResult<()> {
        match word_id {
            StrOrInt::String(s) => {
                if let Some(trans) = self.dict.word_to_id::<AToB, _>(&s) {
                    self.inner.add_for_word_in_topic_b(topic_id, trans, key, value)
                } else {
                    Err(PyValueError::new_err("Value was not found!".to_string()))
                }
            }
            StrOrInt::Int(i) => {
                self.inner.add_for_word_in_topic_b(topic_id, i, key, value)
            }
            StrOrInt::CatchAll(_) => {
                Err(PyValueError::new_err("Value not a int or str!".to_string()))
            }
        }
    }


    pub fn build(&self) -> PyVariableProvider {
        self.inner.clone()
    }
}

pub(crate) fn variable_provider_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVariableProvider>()?;
    m.add_class::<PyVariableProviderBuilder>()?;
    Ok(())
}