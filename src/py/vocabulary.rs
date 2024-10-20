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

use std::borrow::{Borrow};
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::io::Write;
use std::ops::{Deref, DerefMut, Range};
use std::path::{PathBuf};
use std::slice::Iter;
use std::vec::IntoIter;
use pyo3::{Bound, pyclass, pyfunction, pymethods, PyRef, PyResult, wrap_pyfunction};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::{PyModule, PyModuleMethods};
use serde::{Deserialize, Serialize};
use crate::py::dictionary::{PyDictionary};
use crate::py::helpers::{LanguageHintValue, ListOrInt};
use crate::topicmodel::create_topic_model_specific_dictionary as create_topic_model_specific_dictionary_impl;
use crate::topicmodel::language_hint::{LanguageHint, register_py_language_hint};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{LoadableVocabulary, MappableVocabulary, StoreableVocabulary, BasicVocabulary, Vocabulary, VocabularyMut, SearchableVocabulary};


#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PyVocabulary {
    inner: Vocabulary<String>
}

impl PyVocabulary {
    pub fn into_inner(self) -> Vocabulary<String> {
        self.inner
    }
}

#[pymethods]
impl PyVocabulary {
    #[new]
    pub fn new(language: Option<LanguageHintValue>, size: Option<ListOrInt>) -> Self {
        let language = language.map(|value| value.into());

        match size {
            None => {
                Self {
                    inner: Vocabulary::new(language)
                }
            }
            Some(value) => {
                match value {
                    ListOrInt::List(values) => Self {
                            inner: Vocabulary::create_from(language, values)
                    },
                    ListOrInt::Int(value) => Self {
                        inner: Vocabulary::with_capacity(language, value)
                    }
                }
            }
        }
    }

    #[getter]
    #[pyo3(name="language")]
    fn language_hint(&self) -> Option<LanguageHint> {
        self.language().cloned()
    }

    #[setter]
    #[pyo3(name="set_language")]
    fn set_language_hint(&mut self, value: Option<LanguageHintValue>) -> PyResult<()>{
        self.set_language(value.map(|value| {
            let x: LanguageHint = value.into();
            x
        }));
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!("PyVocabulary({:?})", self.inner)
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __contains__(&self, value: &str) -> bool {
        self.inner.contains(value)
    }

    fn __iter__(&self) -> PyVocIter {
        PyVocIter::new(self.clone())
    }

    fn add(&mut self, word: String) -> usize {
        self.inner.add_value(word)
    }

    fn word_to_id(&mut self, word: String) -> Option<usize> {
        self.inner.get_id(word.as_str())
    }

    pub fn id_to_word(&self, id: usize) -> Option<&String> {
        self.inner.get_value(id).map(|value| value.as_ref())
    }

    fn save(&self, path: PathBuf) -> PyResult<usize> {
        Ok(self.inner.save_to_file(path)?)
    }

    #[staticmethod]
    fn load(path: PathBuf) -> PyResult<PyVocabulary> {
        match Vocabulary::<String>::load_from_file(path) {
            Ok(inner) => {
                Ok(Self{ inner })
            }
            Err(value) => {
                Err(PyValueError::new_err(value.to_string()))
            }
        }
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
    }
}

impl Deref for PyVocabulary {
    type Target = Vocabulary<String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PyVocabulary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}


impl BasicVocabulary<String> for PyVocabulary {
    delegate::delegate! {
        to self.inner {
            fn language(&self) -> Option<&LanguageHint>;

            fn set_language(&mut self, new: Option<impl Into<LanguageHint>>) -> Option<LanguageHint>;

            /// The number of entries in the vocabulary
            fn len(&self) -> usize;

            /// Clear the whole thing
            fn clear(&mut self);

            /// Get the ids
            fn ids(&self) -> Range<usize>;

            /// Iterate over the words
            fn iter(&self) -> Iter<HashRef<String>>;

            fn get_id_entry(&self, id: usize) -> Option<(usize, &HashRef<String>)>;

            /// Get the HashRef for a specific `id` or none
            fn get_value(&self, id: usize) -> Option<&HashRef<String>>;

            /// Check if the `id` is contained in this
            fn contains_id(&self, id: usize) -> bool;
        }
    }

    fn create(language: Option<LanguageHint>) -> Self where Self: Sized {
        Self {
            inner: Vocabulary::create(language)
        }
    }

    fn create_from(language: Option<LanguageHint>, voc: Vec<String>) -> Self where Self: Sized, String: Eq + Hash {
        Self {
            inner: Vocabulary::create_from(language, voc)
        }
    }
}

impl MappableVocabulary<String> for PyVocabulary {
    fn map<Q: Eq + Hash, V, F>(self, mapping: F) -> V where F: Fn(&String) -> Q, V: BasicVocabulary<Q> {
        self.inner.map(mapping)
    }
}

impl AsRef<Vec<HashRef<String>>> for PyVocabulary {
    fn as_ref(&self) -> &Vec<HashRef<String>> {
        self.inner.as_ref()
    }
}

impl SearchableVocabulary<String> for PyVocabulary {

    delegate::delegate! {
        to self.inner {
            /// Retrieves the id for `value`
            fn get_id<Q: ?Sized>(&self, value: &Q) -> Option<usize>
                where
                    String: Borrow<Q>,
                    Q: Hash + Eq;

            /// Retrieves the id for `value`
            fn get_hash_ref<Q: ?Sized>(&self, value: &Q) -> Option<&HashRef<String>>
                where
                    String: Borrow<Q>,
                    Q: Hash + Eq;

            /// Retrieves the complete entry for `value` in the vocabulary, if it exists
            fn get_entry_id<Q: ?Sized>(&self, value: &Q) -> Option<(&HashRef<String>, &usize)>
                where
                    String: Borrow<Q>,
                    Q: Hash + Eq;

            fn contains<Q: ?Sized>(&self, value: &Q) -> bool
                where
                    String: Borrow<Q>,
                    Q: Hash + Eq;
        }
    }

    fn filter_by_id<F: Fn(usize) -> bool>(&self, filter: F) -> Self where Self: Sized {
        self.inner.filter_by_id(filter).into()
    }

    fn filter_by_value<'a, F: Fn(&'a HashRef<String>) -> bool>(&'a self, filter: F) -> Self where Self: Sized, String: 'a {
        self.inner.filter_by_value(filter).into()
    }
}

impl VocabularyMut<String> for PyVocabulary {
    delegate::delegate! {
        to self.inner {
            /// Adds the `value` to the vocabulary and returns the associated id
            fn add_hash_ref(&mut self, value: HashRef<String>) -> usize;

            fn add_value(&mut self, value: String) -> usize;

            /// Adds any `value` that can be converted into `T`
            fn add<V: Into<String>>(&mut self, value: V) -> usize;
        }
    }
}
impl StoreableVocabulary<String> for PyVocabulary {
    fn save_to_output(&self, writer: &mut impl Write) -> std::io::Result<usize> {
        self.inner.save_to_output(writer)
    }
}

impl LoadableVocabulary<String, Infallible> for PyVocabulary {}

impl Display for PyVocabulary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl<T> From<Vec<T>> for PyVocabulary where T: Into<String> {
    fn from(value: Vec<T>) -> Self {
        Self { inner: Vocabulary::from(value.into_iter().map(|value| value.into()).collect::<Vec<_>>()) }
    }
}

impl<T> From<(Option<LanguageHint>, Vec<T>)> for PyVocabulary where T: Into<String> {
    fn from((hint, value): (Option<LanguageHint>, Vec<T>)) -> Self {
        Self { inner: Vocabulary::from((hint, value.into_iter().map(|value| value.into()).collect::<Vec<_>>())) }
    }
}

impl From<Vocabulary<String>> for PyVocabulary {
    #[inline(always)]
    fn from(inner: Vocabulary<String>) -> Self {
        Self { inner }
    }
}

impl From<Option<LanguageHint>> for  PyVocabulary {
    fn from(value: Option<LanguageHint>) -> Self {
        Self { inner: value.into() }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyVocIter {
    iter: IntoIter<HashRef<String>>
}

unsafe impl Send for PyVocIter{}
unsafe impl Sync for PyVocIter{}

impl PyVocIter {
    pub fn new(voc: PyVocabulary) -> Self {
        Self { iter: voc.inner.into_iter() }
    }
}

#[pymethods]
impl PyVocIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<String> {
        Some(self.iter.next()?.to_string())
    }
}

#[pyfunction]
pub fn create_topic_model_specific_dictionary(dictionary: &PyDictionary, vocabulary: &PyVocabulary) -> PyDictionary {
    create_topic_model_specific_dictionary_impl(dictionary, vocabulary)
}

pub(crate) fn vocabulary_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVocabulary>()?;
    m.add_class::<PyVocIter>()?;
    register_py_language_hint(m)?;
    m.add_function(wrap_pyfunction!(create_topic_model_specific_dictionary, m)?)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use crate::py::dictionary::PyDictionary;
    use crate::topicmodel::dictionary::BasicDictionary;

    #[test]
    fn load_test(){
        let loaded = PyDictionary::load("E:\\git\\ptmt\\data\\experiment1\\my_dictionary.dict".parse::<PathBuf>().unwrap()).unwrap();
        println!("{}", loaded.iter().count())
    }
}