use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use pyo3::{pyclass, pymethods, FromPyObject, IntoPy, PyObject, Python};
use crate::topicmodel::dictionary::metadata::classic::reference::ClassicMetadataRef;

#[derive(Debug, FromPyObject, Clone)]
pub enum MetadataPyStateValues {
    InternedVec(Vec<usize>),
    UnstemmedMapping(HashMap<usize, Vec<usize>>)
}

impl IntoPy<PyObject> for MetadataPyStateValues {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            MetadataPyStateValues::InternedVec(value) => {
                value.into_py(py)
            }
            MetadataPyStateValues::UnstemmedMapping(value) => {
                value.into_py(py)
            }
        }
    }
}

/// A completely memory save copy of some [Metadata]
#[derive(Debug, Clone, Eq, PartialEq)]
#[pyclass]
pub struct SolvedMetadata {
    associated_dictionaries: Option<Vec<String>>,
    subjects: Option<Vec<String>>,
    unstemmed: Option<HashMap<String, Vec<String>>>
}

impl SolvedMetadata {
    pub fn new(associated_dictionaries: Option<Vec<String>>, subjects: Option<Vec<String>>, unstemmed: Option<HashMap<String, Vec<String>>>) -> Self {
        Self { associated_dictionaries, subjects, unstemmed }
    }
}

#[pymethods]
impl SolvedMetadata {
    #[getter]
    pub fn associated_dictionaries(&self) -> Option<Vec<String>> {
        self.associated_dictionaries.clone()
    }

    #[getter]
    pub fn subjects(&self) -> Option<Vec<String>> {
        self.subjects.clone()
    }

    #[getter]
    pub fn unstemmed(&self) -> Option<HashMap<String, Vec<String>>> {
        self.unstemmed.clone()
    }

    pub fn __repr__(&self) -> String {
        self.to_string()
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

impl Display for SolvedMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Meta{{")?;
        match &self.associated_dictionaries {
            None => {
                write!(f, "associated_dictionaries=[], ")
            }
            Some(value) => {
                write!(f, "associated_dictionaries=[{}], ", value.join(", "))
            }
        }?;
        match &self.subjects {
            None => {
                write!(f, "subjects=[]")
            }
            Some(value) => {
                write!(f, "subjects=[{}]", value.join(", "))
            }
        }?;
        match &self.unstemmed {
            None => {
                write!(f, ", unstemmed=[]")
            }
            Some(value) => {
                write!(
                    f,
                    ", unstemmed=[{}]",
                    value.iter().map(|(k, v)| {
                        format!("({k}, {{{}}})", v.iter().join(", "))
                    }).join(", "))
            }
        }?;
        write!(f, "}}")
    }
}

impl<'a> From<ClassicMetadataRef<'a>> for SolvedMetadata {
    fn from(value: ClassicMetadataRef<'a>) -> Self {
        let associated_dictionaries: Option<Vec<String>> = value.associated_dictionaries().map(|value| value.iter().map(|value| value.to_string()).collect());
        let subjects: Option<Vec<String>> = value.subjects().map(|value| value.iter().map(|value| value.to_string()).collect());
        let unstemmed: Option<HashMap<String, Vec<String>>> = value.unstemmed().map(|value| value.iter().map(|(a, b)| (a.to_string(), b.iter().map(|v|v.to_string()).collect_vec())).collect());
        SolvedMetadata::new(
            associated_dictionaries,
            subjects,
            unstemmed
        )
    }
}
