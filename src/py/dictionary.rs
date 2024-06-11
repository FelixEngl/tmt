use std::borrow::Borrow;
use std::fs::File;
use std::hash::Hash;
use itertools::Itertools;
use pyo3::{Bound, pyclass, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use serde::{Deserialize, Serialize};
use crate::py::vocabulary::PyVocabulary;
use crate::topicmodel::dictionary::{Dictionary, DictionaryImpl, DictionaryMut, DictionaryWithVoc};
use crate::topicmodel::dictionary::direction::{AToB, BToA, Direction, Invariant, Translation};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::VocabularyImpl;

#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PyDictionary {
    inner: DictionaryImpl<String, PyVocabulary>
}

#[pymethods]
impl PyDictionary {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: Default::default()
        }
    }

    pub fn voc_a(&self) -> PyVocabulary {
        self.inner.voc_a().clone()
    }

    pub fn voc_b(&self) -> PyVocabulary {
        self.inner.voc_b().clone()
    }

    pub fn add_word_pair(&mut self, word_a: String, word_b: String) {
        self.inner.insert_value::<Invariant>(word_a, word_b)
    }

    pub fn get_translation_a_to_b(&self, word: &str) -> Option<Vec<String>> {
        self.inner
            .translate_value_to_values::<AToB, _>(word)
            .map(|value|
                value
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect_vec()
            )
    }

    pub fn get_translation_b_to_a(&self, word: &str) -> Option<Vec<String>> {
        self.inner
            .translate_value_to_values::<BToA, _>(word)
            .map(|value|
                value
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect_vec()
            )
    }

    pub fn __repr__(&self) -> String {
        format!("PyDictionary({:?})", self.inner)
    }

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    pub fn save(&self, path: &str) -> PyResult<()> {
        let mut writer = File::options().write(true).create_new(true).open(path)?;
        match serde_json::to_writer(&mut writer, &self) {
            Ok(_) => {Ok(())}
            Err(err) => {
                return Err(PyValueError::new_err(err.to_string()))
            }
        }
    }

    #[staticmethod]
    pub fn load(path: &str) -> PyResult<Self> {
        let mut reader = File::options().read(true).create_new(true).open(path)?;
        match serde_json::from_reader(&mut reader) {
            Ok(result) => {Ok(result)}
            Err(err) => {
                return Err(PyValueError::new_err(err.to_string()))
            }
        }
    }
}

impl Dictionary<String, PyVocabulary> for PyDictionary {
    delegate::delegate! {
        to self.inner {
            fn voc_a(&self) -> &PyVocabulary;

            fn voc_b(&self) -> &PyVocabulary;

            fn map_a_to_b(&self) -> &Vec<Vec<usize>>;

            fn map_b_to_a(&self) -> &Vec<Vec<usize>>;
        }
    }
}

impl DictionaryWithVoc<String, PyVocabulary> for PyDictionary {
    #[inline(always)]
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        self.inner.can_translate_id::<D>(id)
    }

    #[inline(always)]
    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        self.inner.translate_id_to_ids::<D>(word_id)
    }

    #[inline(always)]
    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<String>> where PyVocabulary: 'a {
        self.inner.id_to_word::<D>(id)
    }

    #[inline(always)]
    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<String>)> where PyVocabulary: 'a {
        self.inner.ids_to_id_entry::<D>(ids)
    }

    #[inline(always)]
    fn ids_to_values<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<&'a HashRef<String>> where PyVocabulary: 'a {
        self.inner.ids_to_values::<D>(ids)
    }

    #[inline(always)]
    fn translate_id<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<(usize, &'a HashRef<String>)>> where PyVocabulary: 'a {
        self.inner.translate_id::<D>(word_id)
    }

    #[inline(always)]
    fn translate_id_to_values<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<&'a HashRef<String>>> where PyVocabulary: 'a {
        self.inner.translate_id_to_values::<D>(word_id)
    }
}

impl DictionaryMut<String, PyVocabulary> for PyDictionary {
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<String>, word_b: HashRef<String>) {
        self.inner.insert_hash_ref::<D>(word_a, word_b)
    }

    fn insert_value<D: Direction>(&mut self, word_a: String, word_b: String) {
        self.inner.insert_value::<D>(word_a, word_b)
    }

    fn insert<D: Direction>(&mut self, word_a: impl Into<String>, word_b: impl Into<String>) {
        self.inner.insert::<D>(word_a, word_b)
    }

    fn translate_value<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<(usize, &'a HashRef<String>)>> where String: Borrow<Q>, Q: Hash + Eq, PyVocabulary: 'a {
        self.inner.translate_value::<D, _>(word)
    }

    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>> where String: Borrow<Q>, Q: Hash + Eq {
        self.inner.translate_value_to_ids::<D, _>(word)
    }

    fn word_to_id<D: Translation, Q: ?Sized>(&self, id: &Q) -> Option<usize> where String: Borrow<Q>, Q: Hash + Eq {
        self.inner.word_to_id::<D, _>(id)
    }

    fn can_translate_word<D: Translation, Q: ?Sized>(&self, word: &Q) -> bool where String: Borrow<Q>, Q: Hash + Eq {
        self.inner.can_translate_word::<D, _>(word)
    }

    fn translate_value_to_values<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<&'a HashRef<String>>> where String: Borrow<Q>, Q: Hash + Eq, PyVocabulary: 'a {
        self.inner.translate_value_to_values::<D, _>(word)
    }
}

impl From<DictionaryImpl<String, VocabularyImpl<String>>> for PyDictionary {
    fn from(value: DictionaryImpl<String, VocabularyImpl<String>>) -> Self {
        Self { inner: value.map(|value| value.clone()) }
    }
}

impl From<DictionaryImpl<String, PyVocabulary>> for PyDictionary {
    #[inline(always)]
    fn from(inner: DictionaryImpl<String, PyVocabulary>) -> Self {
        Self { inner }
    }
}

pub(crate) fn dictionary_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDictionary>()?;
    Ok(())
}