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

use std::ops::{Deref, DerefMut};
use std::path::{PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use pyo3::{pyclass, pyfunction, pymethods, PyRef, PyResult};
use pyo3::exceptions::{PyAssertionError, PyRuntimeError, PyStopIteration, PyValueError};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::py::dictionary::{PyDictionary};
use crate::py::helpers::{LanguageHintValue, ListOrInt};
use crate::register_python;
use crate::toolkit::rw_ext::RWLockUnwrapped;
use crate::topicmodel::create_topic_model_specific_dictionary as create_topic_model_specific_dictionary_impl;
use crate::topicmodel::dictionary::{BasicDictionaryWithVocabulary, DictionaryWithMeta};
use crate::topicmodel::dictionary::direction::LanguageKind;
use crate::topicmodel::dictionary::metadata::ex::MetadataManagerEx;
use crate::topicmodel::language_hint::{LanguageHint};
use crate::topicmodel::vocabulary::{LoadableVocabulary, StoreableVocabulary, BasicVocabulary, Vocabulary, VocabularyMut, SearchableVocabulary};

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PyVocabulary {
    inner: PyVocabularyInner,
}

impl From<Vocabulary<String>> for PyVocabulary {
    fn from(v: Vocabulary<String>) -> Self {
        Self { inner: v.into() }
    }
}

impl PyVocabulary {

    pub fn new_from_value(value: Vocabulary<String>) -> Self {
        Self { inner: value.into() }
    }

    pub fn new_from_dict(
        origin: Arc<RwLock<DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>>>,
        target: LanguageKind,
    ) -> Self {
        Self { inner: PyVocabularyInner::dict_based(origin, target) }
    }

    pub fn to_voc(self) -> Vocabulary<String> {
        self.inner.to_voc()
    }

    pub fn get(&self) -> PyVocabularyRef {
        self.inner.get()
    }

    pub fn get_mut(&self) -> PyVocabularyRefMut {
        self.inner.get_mut()
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyVocabulary {
    #[new]
    #[pyo3(signature = (language=None, size=None))]
    pub fn new(language: Option<LanguageHintValue>, size: Option<ListOrInt>) -> Self {
        let language = language.map(|value| value.into());

        match size {
            None => Self {
                    inner: Vocabulary::empty(language).into()
            },
            Some(value) => {
                match value {
                    ListOrInt::List(values) => Self {
                        inner: Vocabulary::create_from(language, values).into()
                    },
                    ListOrInt::Int(value) => Self {
                        inner: Vocabulary::with_capacity(language, value).into()
                    }
                }
            }
        }
    }

    #[getter]
    #[pyo3(name="language")]
    fn language_hint(&self) -> Option<LanguageHint> {
        self.get().language().cloned()
    }

    #[setter]
    #[pyo3(name="set_language")]
    fn set_language_hint(&mut self, value: Option<LanguageHintValue>) -> PyResult<()>{
        self.get_mut().set_language(value.map(|value| {
            let x: LanguageHint = value.into();
            x
        }));
        Ok(())
    }

    #[doc(hidden)]
    fn __repr__(&self) -> String {
        format!("PyVocabulary({:?})", self.get().deref())
    }

    #[doc(hidden)]
    fn __str__(&self) -> String {
        self.get().to_string()
    }

    #[doc(hidden)]
    fn __len__(&self) -> usize {
        self.get().len()
    }

    #[doc(hidden)]
    fn __contains__(&self, value: &str) -> bool {
        self.get().contains(value)
    }

    #[doc(hidden)]
    fn __iter__(&self) -> PyVocIter {
        PyVocIter::new(&self.inner)
    }

    fn add(&mut self, word: String) -> usize {
        self.get_mut().add_value(word)
    }

    fn word_to_id(&mut self, word: String) -> Option<usize> {
        self.get().get_id(word.as_str())
    }

    pub fn id_to_word(&self, id: usize) -> Option<String> {
        self.get().get_value(id).map(|value| value.to_string())
    }

    /// Save the vocabulary in a standardisized way
    fn save(&self, path: PathBuf) -> PyResult<usize> {
        Ok(self.get().save_to_file(path)?)
    }

    /// Load the vocabulary from a file
    #[staticmethod]
    fn load(path: PathBuf) -> PyResult<PyVocabulary> {
        match Vocabulary::<String>::load_from_file(path) {
            Ok(inner) => {
                Ok(Self{ inner: inner.into() })
            }
            Err(value) => {
                Err(PyValueError::new_err(value.to_string()))
            }
        }
    }

    /// Serializes this to a json
    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    /// Deserializes a json to a vocabulary.
    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
    }
}



#[derive(Debug, Clone)]
enum PyVocabularyInner {
    DictBased {
        origin: Arc<RwLock<DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>>>,
        target: LanguageKind,
        mut_version: Arc<AtomicUsize>
    },
    Raw {
        value: Arc<RwLock<Vocabulary<String>>>,
        mut_version: Arc<AtomicUsize>
    },
}
impl PyVocabularyInner {
    pub fn raw(value: Vocabulary<String>) -> Self {
        Self::Raw {
            value: Arc::new(RwLock::new(value.into())),
            mut_version: Arc::new(AtomicUsize::new(0))
        }
    }

    pub fn dict_based(
        origin: Arc<RwLock<DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>>>,
        target: LanguageKind,
    ) -> Self {
        Self::DictBased {
            origin,
            target,
            mut_version: Arc::new(AtomicUsize::new(0))
        }
    }

    pub fn to_voc(self) -> Vocabulary<String> {
        self.get().clone()
    }
}

impl Default for PyVocabularyInner {
    fn default() -> Self {
        Self::raw(Vocabulary::default())
    }
}
impl Serialize for PyVocabularyInner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        self.get().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for PyVocabularyInner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        Ok(PyVocabularyInner::raw(<Vocabulary<String> as Deserialize<'de>>::deserialize(deserializer)?))
    }
}
impl From<Vocabulary<String>> for PyVocabularyInner {
    fn from(value: Vocabulary<String>) -> Self {
        Self::raw(value)
    }
}
impl PyVocabularyInner {
    pub fn get_mut_version(&self) -> usize {
        match self {
            PyVocabularyInner::DictBased { mut_version, .. } => {
                mut_version.load(Ordering::Acquire)
            }
            PyVocabularyInner::Raw { mut_version, .. } => {
                mut_version.load(Ordering::Acquire)
            }
        }
    }

    pub fn get(&self) -> PyVocabularyRef {
        match self {
            PyVocabularyInner::DictBased {
                origin,
                target,
                mut_version
            } => {
                PyVocabularyRef::DictBased {
                    origin: origin.read_unwrapped(),
                    target: *target,
                    mut_version: mut_version.clone()
                }
            }
            PyVocabularyInner::Raw {
                value,
                mut_version
            } => {
                PyVocabularyRef::Raw {
                    value: value.read_unwrapped(),
                    mut_version: mut_version.clone()
                }
            }
        }
    }

    pub fn get_mut(&self) -> PyVocabularyRefMut {
        match self {
            PyVocabularyInner::DictBased {
                origin,
                target,
                mut_version,
            } => {
                PyVocabularyRefMut::DictBased {
                    origin: origin.write_unwrapped(),
                    target: *target,
                    mut_version: mut_version.clone()
                }
            }
            PyVocabularyInner::Raw {
                value,
                mut_version
            } => {
                PyVocabularyRefMut::Raw {
                    value: value.write_unwrapped(),
                    mut_version: mut_version.clone()
                }
            }
        }
    }
}

pub enum PyVocabularyRef<'a> {
    DictBased {
        origin: RwLockReadGuard<'a, DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>>,
        target: LanguageKind,
        mut_version: Arc<AtomicUsize>,
    },
    Raw {
        value: RwLockReadGuard<'a, Vocabulary<String>>,
        mut_version: Arc<AtomicUsize>
    },
}
impl PyVocabularyRef<'_> {
    pub fn get_mut_version(&self) -> usize {
        match self {
            PyVocabularyRef::DictBased { mut_version, .. } => {
                mut_version.load(Ordering::Acquire)
            }
            PyVocabularyRef::Raw { mut_version, .. } => {
                mut_version.load(Ordering::Acquire)
            }
        }
    }
}
impl<'a> Deref for PyVocabularyRef<'a> {
    type Target = Vocabulary<String>;

    fn deref(&self) -> &Self::Target {
        match self {
            PyVocabularyRef::DictBased {
                origin,
                target,
                ..
            } => {
                match target {
                    LanguageKind::A => {
                        origin.voc_a()
                    }
                    LanguageKind::B => {
                        origin.voc_b()
                    }
                }
            }
            PyVocabularyRef::Raw {
                value,
                ..
            } => {
                value.deref()
            }
        }
    }
}


pub enum PyVocabularyRefMut<'a> {
    DictBased {
        origin: RwLockWriteGuard<'a, DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>>,
        target: LanguageKind,
        mut_version: Arc<AtomicUsize>,
    },
    Raw {
        value: RwLockWriteGuard<'a, Vocabulary<String>>,
        mut_version: Arc<AtomicUsize>
    },
}
impl PyVocabularyRefMut<'_> {
    pub fn get_mut_version(&self) -> usize {
        match self {
            PyVocabularyRefMut::DictBased { mut_version, .. } => {
                mut_version.load(Ordering::Acquire)
            }
            PyVocabularyRefMut::Raw { mut_version, .. } => {
                mut_version.load(Ordering::Acquire)
            }
        }
    }
}
impl<'a> Deref for PyVocabularyRefMut<'a> {
    type Target = Vocabulary<String>;

    fn deref(&self) -> &Self::Target {
        match self {
            PyVocabularyRefMut::DictBased {
                origin,
                target,
                ..
            } => {
                match target {
                    LanguageKind::A => {
                        origin.voc_a()
                    }
                    LanguageKind::B => {
                        origin.voc_b()
                    }
                }
            }
            PyVocabularyRefMut::Raw {
                value,
                ..
            } => {
                value.deref()
            }
        }
    }
}
impl<'a> DerefMut for PyVocabularyRefMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            PyVocabularyRefMut::DictBased {
                ref mut origin,
                target,
                mut_version,
            } => {
                mut_version.fetch_add(1, Ordering::Release);
                match target {
                    LanguageKind::A => {
                        origin.voc_a_mut()
                    }
                    LanguageKind::B => {
                        origin.voc_b_mut()
                    }
                }
            }
            PyVocabularyRefMut::Raw {
                ref mut value,
                mut_version,
            } => {
                mut_version.fetch_add(1, Ordering::Release);
                value.deref_mut()
            }
        }
    }
}



#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyVocIter {
    target: PyVocabularyInner,
    mut_version: usize,
    pos: usize,
}

unsafe impl Send for PyVocIter{}
unsafe impl Sync for PyVocIter{}

impl PyVocIter {
    fn new(voc: &PyVocabularyInner) -> Self {
        Self {
            target: voc.clone(),
            mut_version: voc.get_mut_version(),
            pos: 0,
        }
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyVocIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// May raise an assertion error when the vocabulary changes while iterating.
    fn __next__(&mut self) -> PyResult<String> {
        let read = self.target.get();
        if self.mut_version != read.get_mut_version() {
            Err(PyAssertionError::new_err("The value of the dictionary changed while iterating!"))
        } else {
            if let Some(value) = read.get_value(self.pos) {
                self.pos += 1;
                let value = value.to_string();
                Ok(value)
            } else {
                Err(PyStopIteration::new_err("End of iteration."))
            }
        }
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
pub fn create_topic_model_specific_dictionary(dictionary: &PyDictionary, vocabulary: &PyVocabulary) -> PyDictionary {
    let read = dictionary.get();
    let read_voc = vocabulary.get();
    let result = create_topic_model_specific_dictionary_impl(
        read.deref(),
        read_voc.deref()
    );
    PyDictionary::new(result)
}


register_python! {
    struct PyVocabulary;
    struct PyVocIter;
    fn create_topic_model_specific_dictionary;
}


// #[cfg(test)]
// mod test {
//     use std::path::PathBuf;
//     use crate::py::dictionary::PyDictionary;
//     use crate::topicmodel::dictionary::BasicDictionary;
//
//     #[test]
//     fn load_test(){
//         let loaded = PyDictionary::load("E:\\git\\ptmt\\data\\experiment1\\my_dictionary.dict".parse::<PathBuf>().unwrap()).unwrap();
//         println!("{}", loaded.iter().count())
//     }
// }