use std::borrow::Borrow;
use std::fs::File;
use std::hash::Hash;
use itertools::Itertools;
use pyo3::{Bound, pyclass, pymethods, PyRef, PyResult};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use serde::{Deserialize, Serialize};
use crate::py::vocabulary::PyVocabulary;
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithVocabulary, Dictionary, DictionaryFilterable, DictionaryMut, DictionaryWithMeta, DictionaryWithMetaIterator, DictionaryWithVocabulary, DictIter};
use crate::topicmodel::dictionary::direction::{AToB, BToA, Direction, DirectionKind, DirectionTuple, Invariant, LanguageKind, Translation};
use crate::topicmodel::dictionary::metadata::SolvedMetadata;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{VocabularyImpl};


#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PyDictionary {
    inner: DictionaryWithMeta<String, PyVocabulary>,
}

#[pymethods]
impl PyDictionary {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn voc_a(&self) -> PyVocabulary {
        self.inner.voc_a().clone()
    }

    pub fn voc_b(&self) -> PyVocabulary {
        self.inner.voc_b().clone()
    }

    pub fn add_word_pair(&mut self, word_a: String, word_b: String) -> (usize, usize, DirectionKind) {
        self.inner.insert_value::<Invariant>(word_a, word_b).to_tuple()
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

    fn __iter__(&self) -> PyDictIter {
        PyDictIter::new(self.clone())
    }
}

#[pyclass]
pub struct PyDictIter {
    inner: DictionaryWithMetaIterator<DictionaryWithMeta<String, PyVocabulary>, String, PyVocabulary>,
}

unsafe impl Send for PyDictIter{}
unsafe impl Sync for PyDictIter{}

impl PyDictIter {
    pub fn new(inner: PyDictionary) -> Self {
        Self { inner: inner.inner.into_iter() }
    }

    pub fn into_inner(self) -> PyDictionary {
        PyDictionary {
            inner: self.inner.into_inner()
        }
    }
}

#[pymethods]
impl PyDictIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<((usize, String, Option<SolvedMetadata>), (usize, String, Option<SolvedMetadata>), DirectionKind)> {
        let DirectionTuple{
            a: (a, word_a, meta_a),
            b: (b, word_b, meta_b),
            direction
        } = self.inner.next()?;

        Some((
            (a, word_a.to_string(), meta_a),
            (b, word_b.to_string(), meta_b),
            direction
        ))
    }
}

impl BasicDictionary for PyDictionary {
    delegate::delegate! {
        to self.inner {
            fn map_a_to_b(&self) -> &Vec<Vec<usize>>;

            fn map_b_to_a(&self) -> &Vec<Vec<usize>>;

            fn iter(&self) -> DictIter where Self: Sized;
        }
    }

    #[inline(always)]
    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        self.inner.translate_id_to_ids::<D>(word_id)
    }
}

impl BasicDictionaryWithVocabulary<String, PyVocabulary> for PyDictionary {
    delegate::delegate! {
        to self.inner {
            fn voc_a(&self) -> &PyVocabulary;

            fn voc_b(&self) -> &PyVocabulary;
        }
    }
}

impl DictionaryWithVocabulary<String, PyVocabulary> for PyDictionary {

    #[inline(always)]
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        self.inner.can_translate_id::<D>(id)
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

impl DictionaryFilterable<String, PyVocabulary> for PyDictionary {
    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized {
        Self {
            inner: self.inner.filter_by_ids(filter_a, filter_b)
        }
    }

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<String>) -> bool, Fb: Fn(&'a HashRef<String>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, String: 'a {
        Self {
            inner: self.inner.filter_by_values(filter_a, filter_b)
        }
    }
}

impl DictionaryMut<String, PyVocabulary> for PyDictionary {
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<String>, word_b: HashRef<String>) -> DirectionTuple<usize, usize> {
        self.inner.insert_hash_ref::<D>(word_a, word_b)
    }

    fn insert_value<D: Direction>(&mut self, word_a: String, word_b: String) -> DirectionTuple<usize, usize> {
        self.inner.insert_value::<D>(word_a, word_b)
    }

    fn insert<D: Direction>(&mut self, word_a: impl Into<String>, word_b: impl Into<String>) -> DirectionTuple<usize, usize> {
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

impl From<Dictionary<String, VocabularyImpl<String>>> for PyDictionary {
    fn from(value: Dictionary<String, VocabularyImpl<String>>) -> Self {
        Self { inner: value.map(|value| value.clone()).into() }
    }
}

impl From<Dictionary<String, PyVocabulary>> for PyDictionary {
    #[inline(always)]
    fn from(inner: Dictionary<String, PyVocabulary>) -> Self {
        Self { inner: inner.into() }
    }
}

pub(crate) fn dictionary_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDictionary>()?;
    m.add_class::<PyDictIter>()?;
    m.add_class::<SolvedMetadata>()?;
    m.add_class::<DirectionKind>()?;
    m.add_class::<LanguageKind>()?;
    Ok(())
}