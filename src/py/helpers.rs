use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;
use derive_more::{From, TryInto};
use pyo3::{FromPyObject, IntoPy, PyAny, PyObject, Python};
use thiserror::Error;
use crate::py::vocabulary::PyVocabulary;
use crate::py::voting::PyVoting;
use crate::topicmodel::dictionary::metadata::MetadataContainerPyStateValues;
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::topic_model::meta::TopicMetaPyStateValue;
use crate::topicmodel::topic_model::TopicModelPyStateValue;
use crate::topicmodel::vocabulary::VocabularyPyStateValue;
use crate::translate::KeepOriginalWord;
use crate::voting::BuildInVoting;
use crate::voting::py::PyVotingModel;

pub trait HasPickleSupport {
    type FieldValue: Clone + Debug;
    type Error: Error;

    fn get_py_state(&self) -> HashMap<String, Self::FieldValue>;

    fn from_py_state(values: &HashMap<String, Self::FieldValue>) -> Result<Self, Self::Error> where Self: Sized;
}

#[derive(FromPyObject, Debug)]
pub enum ListOrInt {
    List(Vec<String>),
    Int(usize)
}

#[derive(FromPyObject, Debug)]
pub enum LanguageHintValue {
    Hint(LanguageHint),
    Value(String)
}

impl Into<LanguageHint> for LanguageHintValue {
    fn into(self) -> LanguageHint {
        match self {
            LanguageHintValue::Hint(value) => {
                value
            }
            LanguageHintValue::Value(value) => {
                value.into()
            }
        }
    }
}


#[derive(FromPyObject, Debug, Clone)]
pub enum PyVocabularyStateValue {
    Hint(String),
    Value(Vec<String>)
}

impl IntoPy<PyObject> for PyVocabularyStateValue {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PyVocabularyStateValue::Hint(value) => {value.into_py(py)}
            PyVocabularyStateValue::Value(value) => {value.into_py(py)}
        }
    }
}

impl From<VocabularyPyStateValue<String>> for PyVocabularyStateValue {
    fn from(value: VocabularyPyStateValue<String>) -> Self {
        match value {
            VocabularyPyStateValue::Hint(value) => {
                PyVocabularyStateValue::Hint(value)
            }
            VocabularyPyStateValue::Value(value) => {
                PyVocabularyStateValue::Value(value)
            }
        }
    }
}


#[derive(FromPyObject, Debug)]
pub enum KeepOriginalWordArg {
    String(String),
    Value(KeepOriginalWord)
}

impl TryInto<KeepOriginalWord> for KeepOriginalWordArg {
    type Error = <KeepOriginalWord as FromStr>::Err;

    fn try_into(self) -> Result<KeepOriginalWord, Self::Error> {
        match self {
            KeepOriginalWordArg::String(value) => {value.parse()}
            KeepOriginalWordArg::Value(value) => {Ok(value)}
        }
    }
}


#[derive(FromPyObject)]
pub enum VotingArg<'a> {
    Voting(PyVoting),
    BuildIn(BuildInVoting),
    Parseable(String),
    PyCallable(PyVotingModel<'a>),
}


#[derive(FromPyObject)]
pub enum StrOrIntCatching<'a> {
    String(String),
    Int(usize),
    #[pyo3(transparent)]
    CatchAll(&'a PyAny)
}



#[derive(Debug, FromPyObject)]
pub enum PyDictionaryStateValue {
    Voc(HashMap<String, <PyVocabulary as HasPickleSupport>::FieldValue>),
    Mapping(Vec<Vec<usize>>),
    Meta(HashMap<String, MetadataContainerPyStateValues>)
}

impl IntoPy<PyObject> for PyDictionaryStateValue {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PyDictionaryStateValue::Voc(value) => {
                value.into_py(py)
            }
            PyDictionaryStateValue::Mapping(value) => {
                value.into_py(py)
            }
            PyDictionaryStateValue::Meta(value) => {
                value.into_py(py)
            }
        }
    }
}


#[derive(Debug, Copy, Clone, FromPyObject, From, TryInto)]
pub enum IntOrFloat {
    Int(usize),
    Float(f64)
}

#[derive(Debug, Clone, Error)]
pub enum IntOrFloatPyStatsError {
    #[error("The value for the field {0} is missing!")]
    ValueMissing(&'static str),
    #[error("Invalid value at {0} for {1:?}!")]
    InvalidValueEncountered(&'static str, IntOrFloat)
}



impl IntoPy<PyObject> for IntOrFloat {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            IntOrFloat::Int(value) => {value.into_py(py)}
            IntOrFloat::Float(value) => {value.into_py(py)}
        }
    }
}

pub(crate) fn get_or_fail<T>(key: &'static str, values: &HashMap<String, IntOrFloat>) -> Result<T, IntOrFloatPyStatsError> where IntOrFloat: TryInto<T> {
    match values.get(key) {
        None => {
            Err(IntOrFloatPyStatsError::ValueMissing(key))
        }
        Some(value) => {
            value.clone().try_into().map_err(|_| IntOrFloatPyStatsError::InvalidValueEncountered(key, value.clone()))
        }
    }
}


#[derive(Debug, Clone, FromPyObject)]
pub enum PyTopicModelStateValue {
    Voc(HashMap<String, <PyVocabulary as HasPickleSupport>::FieldValue>),
    VecVecProbability(Vec<Vec<f64>>),
    VecCount(Vec<u64>),
    VecMeta(Vec<HashMap<String, TopicMetaPyStateValue>>)
}

impl IntoPy<PyObject> for PyTopicModelStateValue {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PyTopicModelStateValue::Voc(value) => {
                value.into_py(py)
            }
            PyTopicModelStateValue::VecVecProbability(value) => {
                value.into_py(py)
            }
            PyTopicModelStateValue::VecCount(value) => {
                value.into_py(py)
            }
            PyTopicModelStateValue::VecMeta(value) => {
                value.into_py(py)
            }
        }
    }
}

impl From<TopicModelPyStateValue<PyVocabulary>> for PyTopicModelStateValue {
    fn from(value: TopicModelPyStateValue<PyVocabulary>) -> Self {
        match value {
            TopicModelPyStateValue::Voc(value) => {
                PyTopicModelStateValue::Voc(value)
            }
            TopicModelPyStateValue::VecVecProbability(value) => {
                PyTopicModelStateValue::VecVecProbability(value)
            }
            TopicModelPyStateValue::VecCount(value) => {
                PyTopicModelStateValue::VecCount(value)
            }
            TopicModelPyStateValue::VecMeta(value) => {
                PyTopicModelStateValue::VecMeta(value)
            }
        }
    }
}