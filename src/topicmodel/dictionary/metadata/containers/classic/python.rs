use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use pyo3::{pyclass, pymethods};
use crate::register_python;
use crate::topicmodel::dictionary::metadata::classic::reference::ClassicMetadataRef;


register_python! {
    enum MetadataPyStateValues;
    struct SolvedMetadata;
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass]
#[derive(Debug, Clone)]
pub enum MetadataPyStateValues {
    InternedVec(Vec<usize>),
    UnstemmedMapping(HashMap<usize, Vec<usize>>)
}

/// A completely memory save copy of some [Metadata]
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SolvedMetadata {
    associated_dictionaries: Option<Vec<String>>,
    subjects: Option<Vec<String>>,
    unstemmed: Option<HashMap<String, Vec<String>>>
}

impl SolvedMetadata {
    pub fn new(associated_dictionaries: Option<Vec<String>>, subjects: Option<Vec<String>>, unstemmed: Option<HashMap<String, Vec<String>>>) -> Self {
        Self { associated_dictionaries, subjects, unstemmed }
    }

    pub fn associated_dictionaries(&self) -> &Option<Vec<String>> {
        &self.associated_dictionaries
    }

    pub fn subjects(&self) -> &Option<Vec<String>> {
        &self.subjects
    }

    pub fn unstemmed(&self) -> &Option<HashMap<String, Vec<String>>> {
        &self.unstemmed
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl SolvedMetadata {
    #[getter]
    #[pyo3(name = "associated_dictionaries")]
    pub fn associated_dictionaries_py(&self) -> Option<Vec<String>> {
        self.associated_dictionaries.clone()
    }

    #[getter]
    #[pyo3(name = "subjects")]
    pub fn subjects_py(&self) -> Option<Vec<String>> {
        self.subjects.clone()
    }

    #[getter]
    #[pyo3(name = "unstemmed")]
    pub fn unstemmed_py(&self) -> Option<HashMap<String, Vec<String>>> {
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
