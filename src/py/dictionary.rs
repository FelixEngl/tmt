use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter, Write};
use itertools::Itertools;
use pyo3::{Bound, FromPyObject, IntoPy, pyclass, pymethods, PyObject, PyRef, PyResult, Python};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyAnyMethods, PyModule, PyModuleMethods};
use pyo3::types::PyFunction;
use serde::{Deserialize, Serialize};
use crate::py::vocabulary::PyVocabulary;
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, Dictionary, DictionaryFilterable, DictionaryMut, DictionaryWithMeta, DictionaryWithVocabulary};
use crate::topicmodel::dictionary::direction::{A, AToB, B, BToA, Direction, DirectionKind, DirectionTuple, Invariant, Language, LanguageKind, Translation};
use crate::topicmodel::dictionary::iterators::{DictionaryWithMetaIterator, DictIter};
use crate::topicmodel::dictionary::metadata::SolvedMetadata;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{VocabularyImpl, VocabularyMut};

#[derive(FromPyObject, Clone, Debug, Serialize, Deserialize)]
pub enum SingleOrVec<T> {
    Single(#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))] T),
    Vec(#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))] Vec<T>),
}

impl<T> SingleOrVec<T> {
    pub fn to_vec(self) -> Vec<T> {
        match self {
            SingleOrVec::Single(value) => {vec![value]}
            SingleOrVec::Vec(value) => {value}
        }
    }
}

impl<T> AsRef<[T]> for SingleOrVec<T> {
    fn as_ref(&self) -> &[T] {
        match self {
            SingleOrVec::Single(value) => {
                std::slice::from_ref(value)
            }
            SingleOrVec::Vec(values) => {
                values.as_slice()
            }
        }
    }
}

impl<T> IntoPy<PyObject> for SingleOrVec<T> where T: IntoPy<PyObject> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            SingleOrVec::Single(value) => {
                value.into_py(py)
            }
            SingleOrVec::Vec(values) => {
                values.into_py(py)
            }
        }
    }
}


#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyDictionaryEntry {
    word_a: String,
    word_b: String,
    dictionary_a: Option<HashSet<String>>,
    dictionary_b: Option<HashSet<String>>,
    meta_value_a: Option<HashSet<String>>,
    meta_value_b: Option<HashSet<String>>,
    unstemmed_a: Option<HashMap<String, HashSet<String>>>,
    unstemmed_b: Option<HashMap<String, HashSet<String>>>,
}


#[pymethods]
impl PyDictionaryEntry {
    #[new]
    pub fn new(
        word_a: String,
        word_b: String,
        dictionary_a: Option<SingleOrVec<String>>,
        dictionary_b: Option<SingleOrVec<String>>,
        meta_value_a: Option<SingleOrVec<String>>,
        meta_value_b: Option<SingleOrVec<String>>,
        unstemmed_a: Option<HashMap<String, Option<SingleOrVec<String>>>>,
        unstemmed_b: Option<HashMap<String, Option<SingleOrVec<String>>>>,
    ) -> Self {
        Self {
            word_a,
            word_b,
            dictionary_a: dictionary_a.map(|x| {
                let mut set = HashSet::new();
                set.extend(x.to_vec());
                set
            }),
            dictionary_b: dictionary_b.map(|x| {
                let mut set = HashSet::new();
                set.extend(x.to_vec());
                set
            }),
            meta_value_a: meta_value_a.map(|x| {
                let mut set = HashSet::new();
                set.extend(x.to_vec());
                set
            }),
            meta_value_b: meta_value_b.map(|x| {
                let mut set = HashSet::new();
                set.extend(x.to_vec());
                set
            }),
            unstemmed_a: Self::convert_map(unstemmed_a),
            unstemmed_b: Self::convert_map(unstemmed_b),
        }
    }

    #[getter]
    pub fn word_a(&self) -> PyResult<String> {
        Ok(self.word_a.clone())
    }

    #[getter]
    pub fn word_b(&self) -> PyResult<String> {
        Ok(self.word_b.clone())
    }

    #[getter]
    pub fn dictionary_a(&self) -> PyResult<Option<HashSet<String>>> {
        Ok(self.dictionary_a.clone())
    }
    #[setter]
    pub fn set_dictionary_a(&mut self, value: Option<SingleOrVec<String>>) -> PyResult<()> {
        self.dictionary_a = value.map(|x| {
            let mut set = HashSet::new();
            set.extend(x.to_vec());
            set
        });
        Ok(())
    }
    #[getter]
    pub fn dictionary_b(&self) -> PyResult<Option<HashSet<String>>> {
        Ok(self.dictionary_b.clone())
    }
    #[setter]
    pub fn set_dictionary_b(&mut self, value: Option<SingleOrVec<String>>) -> PyResult<()> {
        self.dictionary_b = value.map(|x| {
            let mut set = HashSet::new();
            set.extend(x.to_vec());
            set
        });
        Ok(())
    }
    #[getter]
    pub fn meta_a(&self) -> PyResult<Option<HashSet<String>>> {
        Ok(self.meta_value_a.clone())
    }
    #[setter]
    pub fn set_meta_a(&mut self, value: Option<SingleOrVec<String>>) -> PyResult<()> {
        self.meta_value_a = value.map(|x| {
            let mut set = HashSet::new();
            set.extend(x.to_vec());
            set
        });
        Ok(())
    }
    #[getter]
    pub fn meta_b(&self) -> PyResult<Option<HashSet<String>>> {
        Ok(self.meta_value_b.clone())
    }
    #[setter]
    pub fn set_meta_b(&mut self, value: Option<SingleOrVec<String>>) -> PyResult<()> {
        self.meta_value_b = value.map(|x| {
            let mut set = HashSet::new();
            set.extend(x.to_vec());
            set
        });
        Ok(())
    }
    #[getter]
    pub fn unstemmed_a(&self) -> PyResult<Option<HashMap<String, HashSet<String>>>> {
        Ok(self.unstemmed_a.clone())
    }



    #[setter]
    pub fn set_unstemmed_a(&mut self, value: Option<HashMap<String, Option<SingleOrVec<String>>>>) -> PyResult<()> {
        self.unstemmed_a = Self::convert_map(value);
        Ok(())
    }
    #[getter]
    pub fn unstemmed_b(&self) -> PyResult<Option<HashMap<String, HashSet<String>>>> {
        Ok(self.unstemmed_b.clone())
    }

    #[setter]
    pub fn set_unstemmed_b(&mut self, value: Option<HashMap<String, Option<SingleOrVec<String>>>>) -> PyResult<()> {
        self.unstemmed_b = Self::convert_map(value);
        Ok(())
    }

    pub fn set_dictionary_a_value(&mut self, value: &str) -> PyResult<()> {
        self.set_dictionary_value::<A>(value);
        Ok(())
    }

    pub fn set_dictionary_b_value(&mut self, value: &str) -> PyResult<()> {
        self.set_dictionary_value::<B>(value);
        Ok(())
    }

    pub fn set_meta_a_value(&mut self, value: &str) -> PyResult<()> {
        self.set_meta_value::<A>(value);
        Ok(())
    }

    pub fn set_meta_b_value(&mut self, value: &str) -> PyResult<()> {
        self.set_meta_value::<B>(value);
        Ok(())
    }


    pub fn set_unstemmed_word_a(&mut self, word: &str, unstemmed_meta: Option<&str>) -> PyResult<()> {
        self.set_unstemmed_word::<A>(word, unstemmed_meta);
        Ok(())
    }
    pub fn set_unstemmed_word_b(&mut self, word: &str, unstemmed_meta: Option<&str>) -> PyResult<()> {
        self.set_unstemmed_word::<B>(word, unstemmed_meta);
        Ok(())
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    pub fn __str___(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }
}

impl PyDictionaryEntry {
    fn convert_map(value: Option<HashMap<String, Option<SingleOrVec<String>>>>) -> Option<HashMap<String, HashSet<String>>> {
        value.map(
            |value| {
                value.into_iter().map(|(k, v)|{
                    (k, v.map(|value| {
                        let mut x = HashSet::new();
                        x.extend(value.to_vec());
                        x
                    }).unwrap_or_else(|| HashSet::with_capacity(0)))
                }).collect::<HashMap<_, _>>()
            }
        )
    }

    pub fn set_dictionary_value<L: Language>(&mut self, value: &str) {
        let target = if L::LANG.is_a() {
            &mut self.dictionary_a
        } else {
            &mut self.dictionary_b
        };
        target.get_or_insert_with(|| HashSet::with_capacity(1)).insert(value.to_string());
    }

    pub fn set_meta_value<L: Language>(&mut self, value: &str) {
        let target = if L::LANG.is_a() {
            &mut self.meta_value_a
        } else {
            &mut self.meta_value_b
        };
        target.get_or_insert_with(|| HashSet::with_capacity(1)).insert(value.to_string());
    }

    pub fn set_unstemmed_word<L: Language>(&mut self, word: &str, unstemmed: Option<&str>) {
        let target = if L::LANG.is_a() {
            &mut self.unstemmed_a
        } else {
            &mut self.unstemmed_b
        };
        match target.get_or_insert_with(HashMap::new).entry(word.to_string()) {
            Entry::Occupied(mut value) => {
                if let Some(unstemmed) = unstemmed {
                    value.get_mut().insert(unstemmed.to_string());
                }
            }
            Entry::Vacant(value) => {
                let mut x = HashSet::with_capacity(1);
                if let Some(unstemmed) = unstemmed {
                    x.insert(unstemmed.to_string());
                }
                value.insert(x);
            }
        }
    }
}

impl Display for PyDictionaryEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "(A: {}, B: {}, A_Dicts: [{}], B_Dicts: [{}], A_Meta: [{}], B_Meta: [{}], [{}], [{}])",
               self.word_a,
               self.word_b,
               self.dictionary_a.as_ref().map_or("".to_string(), |value| value.iter().join(", ")),
               self.dictionary_b.as_ref().map_or("".to_string(), |value| value.iter().join(", ")),
               self.meta_value_a.as_ref().map_or("".to_string(), |value| value.iter().join(", ")),
               self.meta_value_b.as_ref().map_or("".to_string(), |value| value.iter().join(", ")),
               self.unstemmed_a.as_ref().map_or("".to_string(), |value| value.iter().map(|(a, b)| format!("({a}, {{{}}})", b.iter().join(", "))).join(", ")),
               self.unstemmed_b.as_ref().map_or("".to_string(), |value| value.iter().map(|(a, b)| format!("({a}, {{{}}})", b.iter().join(", "))).join(", ")),
        )
    }
}

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

    #[getter]
    pub fn voc_a(&self) -> PyVocabulary {
        self.inner.voc_a().clone()
    }

    #[getter]
    pub fn voc_b(&self) -> PyVocabulary {
        self.inner.voc_b().clone()
    }

    pub fn voc_a_contains(&self, value: &str) -> bool {
        self.inner.voc_a().contains(value)
    }

    pub fn voc_b_contains(&self, value: &str) -> bool {
        self.inner.voc_b().contains(value)
    }

    pub fn __contains__(&self, value: &str) -> bool {
        return self.voc_a_contains(value) || self.voc_b_contains(value)
    }

    pub fn switch_a_to_b(&self) -> Self {
        self.clone().switch_languages()
    }

    pub fn add(&mut self, value: PyDictionaryEntry) -> (usize, usize, DirectionKind) {
        self.add_word_pair(
            value.word_a,
            value.word_b,
            value.dictionary_a,
            value.dictionary_b,
            value.meta_value_a,
            value.meta_value_b,
            value.unstemmed_a,
            value.unstemmed_b,
        )
    }

    pub fn add_word_pair(
        &mut self,
        word_a: String,
        word_b: String,
        dictionary_a: Option<HashSet<String>>,
        dictionary_b: Option<HashSet<String>>,
        meta_value_a: Option<HashSet<String>>,
        meta_value_b: Option<HashSet<String>>,
        unstemmed_a: Option<HashMap<String, HashSet<String>>>,
        unstemmed_b: Option<HashMap<String, HashSet<String>>>,
    ) -> (usize, usize, DirectionKind) {
        let result = self.inner.insert_value::<Invariant>(word_a, word_b);

        let meta = self.inner.metadata_mut();

        if let Some(dictionary_a) = dictionary_a {
            meta.set_dictionaries_for::<A>(result.a, &dictionary_a.into_iter().collect_vec())
        }
        if let Some(dictionary_b) = dictionary_b {
            meta.set_dictionaries_for::<B>(result.b, &dictionary_b.into_iter().collect_vec())
        }

        if let Some(meta_value_a) = meta_value_a {
            meta.set_meta_tags_for::<A>(result.a, &meta_value_a.into_iter().collect_vec())
        }
        if let Some(meta_value_b) = meta_value_b {
            meta.set_meta_tags_for::<B>(result.b, &meta_value_b.into_iter().collect_vec())
        }

        if let Some(unstemmed_a) = unstemmed_a {
            for (k, v) in unstemmed_a.into_iter() {
                meta.set_unstemmed_words_origins_for::<A>(result.a, &k, &v.into_iter().collect_vec())
            }
        }
        if let Some(unstemmed_b) = unstemmed_b {
            for (k, v) in unstemmed_b.into_iter() {
                meta.set_unstemmed_words_origins_for::<B>(result.b, &k, &v.into_iter().collect_vec())
            }
        }
        return result.to_tuple();
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
        let writer = File::options().write(true).create_new(true).open(path)?;
        let mut writer = BufWriter::with_capacity(1024*32, writer);
        match serde_json::to_writer(&mut writer, &self) {
            Ok(_) => {
                writer.flush()?;
                Ok(())
            }
            Err(err) => {
                return Err(PyValueError::new_err(err.to_string()))
            }
        }
    }

    #[staticmethod]
    pub fn load(path: &str) -> PyResult<Self> {
        let reader = File::options().read(true).open(path)?;
        let mut reader = BufReader::with_capacity(1024*32, reader);
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

    pub fn filter<'py>(&self, filter_a: Bound<'py, PyFunction>, filter_b: Bound<'py, PyFunction>) -> PyResult<Self> {
        let created = self.inner.create_subset_with_filters(
            |dict, word, meta|{
                let value = dict.id_to_word::<A>(word).unwrap().to_string();
                let solved = meta.cloned().map(|value| value.to_solved_metadata());
                filter_a.call1((value, solved)).expect("This should not fail!").extract::<bool>().expect("You can only return a boolean!")
            },
            |dict, word, meta|{
                let value = dict.id_to_word::<B>(word).unwrap().to_string();
                let solved = meta.cloned().map(|value| value.to_solved_metadata());
                filter_b.call1((value, solved)).expect("This should not fail!").extract::<bool>().expect("You can only return a boolean!")
            },
        );

        Ok(PyDictionary { inner: created })
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

    fn switch_languages(self) -> Self where Self: Sized {
        Self {
            inner: self.inner.switch_languages()
        }
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
    m.add_class::<DirectionKind>()?;
    m.add_class::<LanguageKind>()?;
    m.add_class::<SolvedMetadata>()?;
    m.add_class::<PyDictionaryEntry>()?;
    m.add_class::<PyDictionary>()?;
    m.add_class::<PyDictIter>()?;
    Ok(())
}