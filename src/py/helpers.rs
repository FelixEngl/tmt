use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use std::intrinsics::transmute;
use std::str::FromStr;
use std::sync::Arc;
use derive_more::{From, TryInto};
use itertools::Itertools;
use pyo3::{FromPyObject, IntoPy, PyAny, PyObject, Python};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::py::voting::PyVoting;
use crate::topicmodel::language_hint::LanguageHint;
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

#[derive(FromPyObject, Debug, Eq, Hash)]
pub enum LanguageHintValue {
    Hint(LanguageHint),
    Value(String)
}

impl PartialEq for LanguageHintValue {
    fn eq(&self, other: &Self) -> bool {
        let a = match self {
            LanguageHintValue::Hint(value_a) => {
                value_a.as_str()
            }
            LanguageHintValue::Value(value_a) => {
                value_a.as_str()
            }
        };

        let b = match other {
            LanguageHintValue::Hint(value_a) => {
                value_a.as_str()
            }
            LanguageHintValue::Value(value_a) => {
                value_a.as_str()
            }
        };
        a == b
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SerializableSpecialVec")]
#[serde(into = "SerializableSpecialVec")]
pub(crate) struct SpecialVec {
    inner: Arc<Vec<String>>,
    references: Arc<Vec<*const str>>
}

unsafe impl Send for SpecialVec{}
unsafe impl Sync for SpecialVec{}

impl SpecialVec {
    pub fn new(inner: Vec<String>) -> Self {
        let references = inner.iter().map(|value| value.as_str() as *const str).collect_vec();
        Self {
            inner: Arc::new(inner),
            references: Arc::new(references)
        }
    }

    fn new_from_arc(inner: Arc<Vec<String>>) -> Self {
        let references = inner.iter().map(|value| value.as_str() as *const str).collect_vec();
        Self {
            inner,
            references: Arc::new(references)
        }
    }

    pub fn as_slice(&self) -> &[&str] {
        // A &str is basically a *const str but with a safe livetime.
        unsafe {transmute(self.references.as_slice())}
    }

    pub fn inner(&self) -> &Arc<Vec<String>> {
        &self.inner
    }
}

impl AsRef<[String]> for SpecialVec {
    fn as_ref(&self) -> &[String] {
        self.inner.as_slice()
    }
}

impl From<SerializableSpecialVec> for SpecialVec {
    fn from(value: SerializableSpecialVec) -> Self {
        Self::new_from_arc(value.inner)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct SerializableSpecialVec {
    inner: Arc<Vec<String>>
}

impl From<SpecialVec> for SerializableSpecialVec {
    fn from(value: SpecialVec) -> Self {
        Self { inner: value.inner }
    }
}



#[cfg(test)]
mod special_vec_test {
    use super::SpecialVec;

    #[test]
    fn can_be_used_safely(){
        let v = SpecialVec::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let r = v.as_slice();

        println!("{:?}", v.as_ref());
        println!("{:?}", r);
    }
}




#[derive(Debug, Clone, FromPyObject)]
pub enum StringSetOrList {
    List(Vec<String>),
    Set(HashSet<String>),
}

impl StringSetOrList {
    pub fn to_vec(self) -> Vec<String> {
        match self {
            StringSetOrList::List(value) => {value}
            StringSetOrList::Set(value) => {value.into_iter().collect_vec()}
        }
    }

    pub fn to_hash_set(self) -> HashSet<String> {
        match self {
            StringSetOrList::List(value) => {value.into_iter().collect()}
            StringSetOrList::Set(value) => {value}
        }
    }
}