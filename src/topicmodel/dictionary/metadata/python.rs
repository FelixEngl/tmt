use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use pyo3::{pyclass, pymethods, FromPyObject, IntoPy, PyObject, Python};
use crate::topicmodel::dictionary::metadata::container::MetadataContainer;
use crate::topicmodel::dictionary::metadata::{Metadata, MetadataRef};
use crate::toolkit::typesafe_interner::DefaultDictionaryOrigin;
use crate::topicmodel::vocabulary::{SearchableVocabulary, VocabularyMut};

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



pub struct MetadataMutRef<'a> {
    pub(in crate::topicmodel::dictionary) meta: &'a mut Metadata,
    // always outlifes meta
    metadata_ref: *mut MetadataContainer
}

impl<'a> MetadataMutRef<'a> {
    pub(in crate::topicmodel::dictionary) fn new(dict_ref: *mut MetadataContainer, meta: &'a mut Metadata) -> Self {
        Self { meta, metadata_ref: dict_ref }
    }

    pub fn push_associated_dictionary(&mut self, dictionary: impl AsRef<str>) {
        let interned = unsafe{&mut *self.metadata_ref }.get_dictionary_interner_mut().get_or_intern(dictionary);
        unsafe {
            self.meta.add_associated_dictionary(interned);
        }
    }

    pub fn get_or_push_associated_dictionary(&mut self, dictionary: impl AsRef<str>) -> DefaultDictionaryOrigin {
        let interned = unsafe{&mut *self.metadata_ref }.get_dictionary_interner_mut().get_or_intern(dictionary);
        if self.meta.has_associated_dictionary(interned) {
            return interned
        }
        unsafe{self.meta.add_associated_dictionary(interned)};
        interned
    }

    pub fn push_subject(&mut self, tag: impl AsRef<str>) {
        let interned = unsafe{&mut *self.metadata_ref }.get_tag_interner_mut().get_or_intern(tag);
        unsafe {
            self.meta.add_subject(interned);
        }
    }

    pub fn push_unstemmed(&mut self, word: impl AsRef<str>)  {
        let interned = unsafe{&mut *self.metadata_ref }.get_unstemmed_voc_mut().add(word.as_ref());
        self.meta.add_unstemmed(interned);
    }


    pub fn get_or_push_unstemmed(&mut self, word: impl AsRef<str>) -> usize {
        let reference = unsafe{&mut *self.metadata_ref }.get_unstemmed_voc_mut();
        let word = word.as_ref();
        match reference.get_id(word) {
            None => {
                let interned = reference.add(word.to_string());
                self.meta.add_unstemmed(interned);
                interned
            }
            Some(value) => {
                value
            }
        }
    }

    pub fn push_unstemmed_with_origin(&mut self, word: impl AsRef<str>, origin: impl AsRef<str>) {
        let word = self.get_or_push_unstemmed(word);
        let origin = self.get_or_push_associated_dictionary(origin);
        unsafe { self.meta.add_unstemmed_origin(word, origin) }
    }

    pub fn push_unstemmed_with_origins(&mut self, word: impl AsRef<str>, origins: &[impl AsRef<str>]) {
        let word = self.get_or_push_unstemmed(word);
        let origins = origins.iter().map(|value| self.get_or_push_associated_dictionary(value)).collect_vec();
        unsafe { self.meta.add_all_unstemmed_origins(word, &origins) }
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

impl<'a> From<MetadataRef<'a>> for SolvedMetadata {
    fn from(value: MetadataRef<'a>) -> Self {
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
