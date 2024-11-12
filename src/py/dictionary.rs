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

use crate::py::helpers::LanguageHintValue;
use crate::py::vocabulary::PyVocabulary;
use crate::topicmodel::dictionary::direction::{AToB, BToA, Direction, DirectionKind, DirectionTuple, Language, Translation, A, B};
use crate::topicmodel::dictionary::iterators::{DictIter, DictionaryWithMetaIterator};
use crate::topicmodel::dictionary::metadata::ex::{MetadataManagerEx, MetaField, LoadedMetadataEx};
use crate::topicmodel::dictionary::metadata::{MetadataManager};
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, Dictionary, DictionaryFilterable, DictionaryMut, DictionaryWithMeta, DictionaryWithVocabulary, FromVoc, MergingDictionary};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{SearchableVocabulary, Vocabulary};
use itertools::Itertools;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::{PyAnyMethods};
use pyo3::{pyclass, pymethods, Bound, PyAny, PyRef, PyResult};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter, Write};
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::path::PathBuf;
use camino::Utf8PathBuf;
use crate::py::tokenizer::PyAlignedArticleProcessor;
use crate::register_python;
use crate::tokenizer::Tokenizer;
use crate::topicmodel::dictionary::io::{ReadableDictionary, WriteMode, WriteableDictionary};

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PyDictionary {
    wrapped: DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyDictionary {
    #[new]
    #[pyo3(signature = (language_a=None, language_b=None))]
    pub fn new(language_a: Option<LanguageHintValue>, language_b: Option<LanguageHintValue>) -> Self {
        Self {
            wrapped: DictionaryWithMeta::new_with(
                language_a,
                language_b
            )
        }
    }

    #[getter]
    fn known_dictionaries(&self) -> Vec<String> {
        self.wrapped.known_dictionaries().into_iter().map(|value| value.to_string()).collect_vec()
    }

    fn get_all_values_from(&self, field: MetaField) {
        
    }

    #[getter]
    fn translation_direction(&self) -> (Option<LanguageHint>, Option<LanguageHint>) {
        (self.deref().language::<A>().cloned(), self.deref().language::<B>().cloned())
    }

    #[setter]
    fn set_translation_direction(&mut self, option: (Option<LanguageHintValue>, Option<LanguageHintValue>)) {
        self.wrapped.set_language::<A>(option.0.map(|value| value.into()));
        self.wrapped.set_language::<B>(option.1.map(|value| value.into()));
    }

    #[getter]
    #[pyo3(name = "voc_a")]
    fn voc_a_py(&self) -> PyVocabulary {
        self.wrapped.voc_a().clone()
    }

    #[getter]
    #[pyo3(name = "voc_b")]
    fn voc_b_py(&self) -> PyVocabulary {
        self.wrapped.voc_b().clone()
    }

    fn voc_a_contains(&self, value: &str) -> bool {
        self.wrapped.voc_a().contains(value)
    }

    fn voc_b_contains(&self, value: &str) -> bool {
        self.wrapped.voc_b().contains(value)
    }

    fn __contains__(&self, value: &str) -> bool {
        self.voc_a_contains(value) || self.voc_b_contains(value)
    }

    fn switch_a_to_b(&self) -> Self {
        self.clone().switch_languages()
    }

    pub fn add(
        &mut self,
        word_a: (String, LanguageHintValue, LoadedMetadataEx),
        word_b: (String, LanguageHintValue, LoadedMetadataEx)
    ) -> (usize, usize, DirectionKind) {
        let (a_word, a_hint, a_solved) = word_a;
        let (b_word, b_hint, b_solved) = word_b;
        if let Some(hint) = self.language::<A>() {

        }
        todo!()
    }

    fn get_translation_a_to_b(&self, word: &str) -> Option<Vec<String>> {
        self.wrapped
            .translate_value_to_values::<AToB, _>(word)
            .map(|value|
                value
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect_vec()
            )
    }

    fn get_translation_b_to_a(&self, word: &str) -> Option<Vec<String>> {
        self.wrapped
            .translate_value_to_values::<BToA, _>(word)
            .map(|value|
                value
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect_vec()
            )
    }

    fn __repr__(&self) -> String {
        format!("PyDictionary({:?})", self.wrapped)
    }

    fn __str__(&self) -> String {
        todo!()
        // self.inner.to_string()
    }

    #[pyo3(signature = (path, mode=None))]
    pub fn save(&self, path: PathBuf, mode: Option<WriteMode>) -> PyResult<()> {
        let path = Utf8PathBuf::from_path_buf(path).map_err(|value| PyValueError::new_err(value.as_os_str().to_string_lossy().to_string()))?;
        if let Some(mode) = mode {
            self.write_to_path(mode, path)
        } else {
            self.write_to_path_with_extension(path)
        }.map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(())
    }

    #[staticmethod]
    #[pyo3(signature = (path, mode=None))]
    pub fn load(path: PathBuf, mode: Option<WriteMode>) -> PyResult<Self> {
        let path = Utf8PathBuf::from_path_buf(path).map_err(|value| PyValueError::new_err(value.as_os_str().to_string_lossy().to_string()))?;
        if let Some(mode) = mode {
            Self::from_path(mode, path)
        } else {
            Self::from_path_with_extension(path)
        }.map_err(|value| PyValueError::new_err(value.to_string()))
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

    fn __iter__(&self) -> PyDictIter {
        PyDictIter::new(self.clone())
    }

    fn filter<'py>(&self, filter_a: Bound<'py, PyAny>, filter_b: Bound<'py, PyAny>) -> PyResult<Self> {
        let created = self.wrapped.create_subset_with_filters(
            |dict, word, meta|{
                let value = dict.id_to_word::<A>(word).unwrap().to_string();
                let solved = meta.cloned().map(LoadedMetadataEx::from);
                filter_a.call1((value, solved)).expect("This should not fail!").extract::<bool>().expect("You can only return a boolean!")
            },
            |dict, word, meta|{
                let value = dict.id_to_word::<B>(word).unwrap().to_string();
                let solved = meta.cloned().map(LoadedMetadataEx::from);
                filter_b.call1((value, solved)).expect("This should not fail!").extract::<bool>().expect("You can only return a boolean!")
            },
        );

        Ok(PyDictionary { wrapped: created })
    }

    pub fn get_meta_a_of(&self, word: &str) -> Option<LoadedMetadataEx> {
        let word_id = self.wrapped.voc_a().get_id(word)?;
        let meta = self.wrapped.metadata().get_meta_ref::<A>(self.voc_a(), word_id)?;
        Some(meta.into())
    }

    pub fn get_meta_b_of(&self, word: &str) -> Option<LoadedMetadataEx> {
        let word_id = self.wrapped.voc_b().get_id(word)?;
        let meta = self.wrapped.metadata().get_meta_ref::<B>(self.voc_b(), word_id)?;
        Some(meta.into())
    }

    pub fn process_with_tokenizer(
        &self,
        processor: PyAlignedArticleProcessor
    ) -> PyResult<Self> {
        match self.language_direction() {
            (Some(a), Some(b)) => {
                let a_tok = if let Some(a) = processor.get_tokenizers_for(a) {
                    a
                } else {
                    return Err(PyValueError::new_err(format!("Language A ({a}) is unknown to the processor!")))
                };

                let b_tok = if let Some(b) = processor.get_tokenizers_for(b) {
                    b
                } else {
                    return Err(PyValueError::new_err(format!("Language B ({b}) is unknown to the processor!")))
                };

                fn apply_tokenizer_and_filer(tokenizer: &Tokenizer, value: &str) -> Result<Option<HashRef<String>>, ()> {
                    let result = tokenizer.process(value).filter(|value| !value.1.lemma.is_empty() && value.1.is_word()).collect_vec();
                    if result.is_empty() {
                        Ok(None)
                    } else {
                        Ok(
                            Some(
                                HashRef::new(
                                    result.iter().map(|value| value.1.lemma()).join(" ")
                                )
                            )
                        )
                    }
                }

                Ok(self.filter_and_process(
                    |value| {
                        apply_tokenizer_and_filer(&a_tok, value.as_str())
                    },
                    |value| {
                        apply_tokenizer_and_filer(&b_tok, value.as_str())
                    }
                ).unwrap())
            }
            (Some(_), None) => {
                Err(PyValueError::new_err("Language B is unknown!"))
            }
            (None, Some(_)) => {
                Err(PyValueError::new_err("Language A is unknown!"))
            }
            _ => {
                Err(PyValueError::new_err("Language A and B are unknown!"))
            }
        }
    }

}

impl Deref for PyDictionary {
    type Target = DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>;

    fn deref(&self) -> &Self::Target {
        &self.wrapped
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
pub struct PyDictIter {
    inner: DictionaryWithMetaIterator<DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>, String, PyVocabulary, MetadataManagerEx>,
}

unsafe impl Send for PyDictIter{}
unsafe impl Sync for PyDictIter{}

impl PyDictIter {
    pub fn new(inner: PyDictionary) -> Self {
        Self { inner: inner.wrapped.into_iter() }
    }

    pub fn into_inner(self) -> PyDictionary {
        PyDictionary {
            wrapped: self.inner.into_inner()
        }
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyDictIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<((usize, String, Option<LoadedMetadataEx>), (usize, String, Option<LoadedMetadataEx>), DirectionKind)> {
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
        to self.wrapped {
            fn map_a_to_b(&self) -> &Vec<Vec<usize>>;

            fn map_b_to_a(&self) -> &Vec<Vec<usize>>;

            fn iter(&self) -> DictIter where Self: Sized;
        }
    }

    #[inline(always)]
    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        self.wrapped.translate_id_to_ids::<D>(word_id)
    }

    fn switch_languages(self) -> Self where Self: Sized {
        Self {
            wrapped: self.wrapped.switch_languages()
        }
    }

}

impl BasicDictionaryWithVocabulary<PyVocabulary> for PyDictionary {
    delegate::delegate! {
        to self.wrapped {
            fn voc_a(&self) -> &PyVocabulary;

            fn voc_b(&self) -> &PyVocabulary;
        }
    }
}

impl MergingDictionary<String, PyVocabulary> for PyDictionary {
    fn merge(self, other: impl Into<Self>) -> Self
    where
        Self: Sized
    {
        Self {
            wrapped: self.wrapped.merge(other.into().wrapped)
        }
    }
}

impl DictionaryWithVocabulary<String, PyVocabulary> for PyDictionary {

    #[inline(always)]
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        self.wrapped.can_translate_id::<D>(id)
    }

    #[inline(always)]
    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<String>> where PyVocabulary: 'a {
        self.wrapped.id_to_word::<D>(id)
    }

    #[inline(always)]
    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<String>)> where PyVocabulary: 'a {
        self.wrapped.ids_to_id_entry::<D>(ids)
    }

    #[inline(always)]
    fn ids_to_values<'a, D: Translation, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<&'a HashRef<String>> where PyVocabulary: 'a {
        self.wrapped.ids_to_values::<D, _>(ids)
    }

    #[inline(always)]
    fn translate_id<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<(usize, &'a HashRef<String>)>> where PyVocabulary: 'a {
        self.wrapped.translate_id::<D>(word_id)
    }

    #[inline(always)]
    fn translate_id_to_values<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<&'a HashRef<String>>> where PyVocabulary: 'a {
        self.wrapped.translate_id_to_values::<D>(word_id)
    }

    #[inline(always)]
    fn translate_value<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<(usize, &'a HashRef<String>)>> where String: Borrow<Q>, Q: Hash + Eq, PyVocabulary: 'a {
        self.wrapped.translate_value::<D, _>(word)
    }

    #[inline(always)]
    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>> where String: Borrow<Q>, Q: Hash + Eq {
        self.wrapped.translate_value_to_ids::<D, _>(word)
    }

    #[inline(always)]
    fn word_to_id<D: Translation, Q: ?Sized>(&self, id: &Q) -> Option<usize> where String: Borrow<Q>, Q: Hash + Eq {
        self.wrapped.word_to_id::<D, _>(id)
    }

    #[inline(always)]
    fn can_translate_word<D: Translation, Q: ?Sized>(&self, word: &Q) -> bool where String: Borrow<Q>, Q: Hash + Eq {
        self.wrapped.can_translate_word::<D, _>(word)
    }

    #[inline(always)]
    fn translate_value_to_values<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<&'a HashRef<String>>> where String: Borrow<Q>, Q: Hash + Eq, PyVocabulary: 'a {
        self.wrapped.translate_value_to_values::<D, _>(word)
    }
}

impl DictionaryFilterable<String, PyVocabulary> for PyDictionary {
    fn filter_and_process<'a, Fa, Fb, E>(&'a self, f_a: Fa, f_b: Fb) -> Result<Self, E>
    where
        Self: Sized,
        String: 'a,
        Fa: Fn(&'a HashRef<String>) -> Result<Option<HashRef<String>>, E>,
        Fb: Fn(&'a HashRef<String>) -> Result<Option<HashRef<String>>, E>
    {
        Ok(
            Self {
                wrapped: self.wrapped.filter_and_process(f_a, f_b)?
            }
        )
    }


    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized {
        Self {
            wrapped: self.wrapped.filter_by_ids(filter_a, filter_b)
        }
    }

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<String>) -> bool, Fb: Fn(&'a HashRef<String>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, String: 'a {
        Self {
            wrapped: self.wrapped.filter_by_values(filter_a, filter_b)
        }
    }
}

impl DictionaryMut<String, PyVocabulary> for PyDictionary {
    #[inline(always)]
    fn set_language<L: Language>(&mut self, value: Option<LanguageHint>) -> Option<LanguageHint> {
        self.wrapped.set_language::<L>(value)
    }

    fn insert_single_ref<L: Language>(&mut self, word: HashRef<String>) -> usize {
        self.wrapped.insert_single_ref::<L>(word)
    }

    unsafe fn reserve_for_single_value<L: Language>(&mut self, word_id: usize) {
        self.wrapped.reserve_for_single_value::<L>(word_id)
    }

    unsafe fn insert_raw_values<D: Direction>(&mut self, word_id_a: usize, word_id_b: usize) {
        self.wrapped.insert_raw_values::<D>(word_id_a, word_id_b)
    }

    #[inline(always)]
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<String>, word_b: HashRef<String>) -> DirectionTuple<usize, usize> {
        self.wrapped.insert_hash_ref::<D>(word_a, word_b)
    }

    #[inline(always)]
    fn insert_value<D: Direction>(&mut self, word_a: String, word_b: String) -> DirectionTuple<usize, usize> {
        self.wrapped.insert_value::<D>(word_a, word_b)
    }

    #[inline(always)]
    fn insert<D: Direction>(&mut self, word_a: impl Into<String>, word_b: impl Into<String>) -> DirectionTuple<usize, usize> {
        self.wrapped.insert::<D>(word_a, word_b)
    }
}

impl From<Dictionary<String, Vocabulary<String>>> for PyDictionary {
    fn from(value: Dictionary<String, Vocabulary<String>>) -> Self {
        Self { wrapped: value.map(|value| value.clone()).into() }
    }
}

impl From<Dictionary<String, PyVocabulary>> for PyDictionary {
    #[inline(always)]
    fn from(inner: Dictionary<String, PyVocabulary>) -> Self {
        Self { wrapped: inner.into() }
    }
}

impl FromVoc<String, PyVocabulary> for PyDictionary {
    fn from_voc(voc_a: PyVocabulary, voc_b: PyVocabulary) -> Self {
        Self {
            wrapped: DictionaryWithMeta::from_voc(voc_a, voc_b)
        }
    }

    fn from_voc_lang<L: Language>(voc: PyVocabulary, other_lang: Option<LanguageHint>) -> Self {
        Self {
            wrapped: DictionaryWithMeta::from_voc_lang::<L>(voc, other_lang)
        }
    }
}

register_python! {
    struct PyDictionary;
    struct PyDictIter;
}