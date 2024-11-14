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
use crate::topicmodel::dictionary::direction::{DirectionKind, DirectionTuple};
use crate::topicmodel::dictionary::iterators::{DictionaryWithMetaIterator};
use crate::topicmodel::dictionary::metadata::ex::{MetadataManagerEx, LoadedMetadataEx};
use crate::topicmodel::dictionary::metadata::{MetadataManager};
use crate::topicmodel::dictionary::*;
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{SearchableVocabulary, Vocabulary};
use itertools::Itertools;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::{PyAnyMethods};
use pyo3::{pyclass, pymethods, PyRef, PyResult};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use camino::Utf8PathBuf;
use crate::py::tokenizer::PyAlignedArticleProcessor;
use crate::{define_py_method, register_python};
use crate::tokenizer::Tokenizer;
use crate::topicmodel::dictionary::io::{ReadableDictionary, WriteModeLiteral, WriteableDictionary};
use crate::topicmodel::dictionary::len::Len;

pub type DefaultDict = DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>;

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug, Default)]
#[repr(transparent)]
pub struct PyDictionary {
    wrapped: Arc<RwLock<DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>>>,
}


impl PyDictionary {
    pub fn new(dict: DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>) -> Self {
        Self {
            wrapped: Arc::new(RwLock::new(dict))
        }
    }

    pub fn get<'a>(&'a self) -> RwLockReadGuard<'a, DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>> {
        self.wrapped.read().unwrap()
    }

    pub fn get_mut<'a>(&'a mut self) -> RwLockWriteGuard<'a, DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>> {
        self.wrapped.write().unwrap()
    }
}

impl Serialize for PyDictionary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        DefaultDict::serialize(self.get().deref(), serializer)
    }
}

impl<'de> Deserialize<'de> for PyDictionary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        Ok(DefaultDict::deserialize(deserializer)?.into())
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyDictionary {
    #[new]
    pub fn new_py(language_a: Option<LanguageHintValue>, language_b: Option<LanguageHintValue>) -> Self {
        Self {
            wrapped: Arc::new(
                RwLock::new(
                    DictionaryWithMeta::new_with(
                        language_a,
                        language_b
                    )
                )
            )
        }
    }

    /// Returns the dictionaries contained in this dictionary.
    #[getter]
    fn known_dictionaries(&self) -> Vec<String> {
        self.get().known_dictionaries().into_iter().map(|value| value.to_string()).collect_vec()
    }

    /// Returns the translation direction. It goes from A to B (A, B)
    #[getter]
    fn translation_direction(&self) -> (Option<LanguageHint>, Option<LanguageHint>) {
        let read = self.get();
        let (a, b) = read.language_direction_a_to_b();
        (a.cloned(), b.cloned())
    }

    /// Allows to set the translation languages. This is usually not necessary, except youbuild your own.
    #[setter]
    fn set_translation_direction(&mut self, option: (Option<LanguageHintValue>, Option<LanguageHintValue>)) {
        let mut write = self.get_mut();
        write.set_language_a(option.0.map(|value| value.into()));
        write.set_language_b(option.1.map(|value| value.into()));
    }

    /// Returns the vocabulary a
    #[getter]
    #[pyo3(name = "voc_a")]
    fn voc_a_py(&self) -> PyVocabulary {
        self.get().voc_a().clone()
    }

    /// Returns the vocabulary b
    #[getter]
    #[pyo3(name = "voc_b")]
    fn voc_b_py(&self) -> PyVocabulary {
        self.get().voc_b().clone()
    }

    /// Returns true if voc a contains the value
    fn voc_a_contains(&self, value: &str) -> bool {
        self.get().voc_a().contains(value)
    }

    /// Returns true if voc a contains the value
    fn voc_b_contains(&self, value: &str) -> bool {
        self.get().voc_b().contains(value)
    }

    /// Returns true if any voc contains the value
    fn __contains__(&self, value: &str) -> bool {
        self.voc_a_contains(value) || self.voc_b_contains(value)
    }

    fn switch_a_to_b(&self) -> Self {
        Self {
            wrapped: Arc::new(
                RwLock::new(
                    DictionaryWithMeta::clone(&self.get()).switch_languages()
                )
            )
        }
    }

    /// Insert a translation from word a to b with
    pub fn add(
        &mut self,
        word_a: (String, Option<LoadedMetadataEx>),
        word_b: (String, Option<LoadedMetadataEx>)
    ) -> PyResult<(usize, usize, DirectionKind)> {
        let (a_word, a_solved) = word_a;
        let (b_word, b_solved) = word_b;

        let mut write = self.wrapped.write().unwrap();

        match write.insert_translation_ref_with_meta_invariant(
            HashRef::new(a_word),
            a_solved.as_ref(),
            HashRef::new(b_word),
            b_solved.as_ref(),
        ) {
            Ok((a, b)) => {
                Ok((a, b, DirectionKind::Invariant))
            }
            Err((_, err)) => {
                Err(PyValueError::new_err(format!("{err}")))
            }
        }
    }

    /// Returns the translations of the word, from a to b
    fn get_translation_a_to_b(&self, word: &str) -> Option<Vec<String>> {
        self.get()
            .translate_word_a_to_words_b(word)
            .map(|value|
                value
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect_vec()
            )
    }

    /// Returns the translations of the word, from b to a
    fn get_translation_b_to_a(&self, word: &str) -> Option<Vec<String>> {
        self.get()
            .translate_word_b_to_words_a(word)
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
        format!("PyDictionary({:?})", self.wrapped)
    }

    /// Writes the dictionary to the path, the mode is chosen based on the file ending.
    pub fn save(&self, path: PathBuf) -> PyResult<()> {
        let path = Utf8PathBuf::from_path_buf(path).map_err(|value| PyValueError::new_err(value.as_os_str().to_string_lossy().to_string()))?;
        self.get().write_to_path_with_extension(path).map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(())
    }

    /// Writes the dictionary to the path with the chosen mode
    pub fn save_as(&self, path: PathBuf, mode: WriteModeLiteral) -> PyResult<()> {
        let path = Utf8PathBuf::from_path_buf(path).map_err(|value| PyValueError::new_err(value.as_os_str().to_string_lossy().to_string()))?;
        self.get().write_to_path(
            mode.parse().map_err(|err| { PyValueError::new_err(format!("{err}")) })?,
            path
        ).map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(())
    }

    /// Loads the dictionary from the path with the provided mode
    #[staticmethod]
    pub fn load(path: PathBuf) -> PyResult<Self> {
        let path = Utf8PathBuf::from_path_buf(path).map_err(|value| PyValueError::new_err(value.as_os_str().to_string_lossy().to_string()))?;
        Ok(
            DictionaryWithMeta::from_path_with_extension(path)
                .map_err(|value| PyValueError::new_err(value.to_string()))?.into()
        )
    }

    /// Loads the dictionary from the path with the provided mode
    #[staticmethod]
    pub fn load_as(path: PathBuf, mode: WriteModeLiteral) -> PyResult<Self> {
        let path = Utf8PathBuf::from_path_buf(path).map_err(|value| PyValueError::new_err(value.as_os_str().to_string_lossy().to_string()))?;


        Ok(
            DictionaryWithMeta::from_path(
                mode.parse().map_err(|err| { PyValueError::new_err(format!("{err}")) })?,
                path
            ).map_err(|value| PyValueError::new_err(value.to_string()))?.into()
        )
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
        PyDictIter::new(self)
    }

    /// Filters a dictionary by the defined methods and returns a new instance.
    fn filter<'py>(&self, filter_a: FilterDictionaryMethod<'py>, filter_b: FilterDictionaryMethod<'py>) -> PyResult<Self> {
        let created = self.get().create_subset_with_filters(
            |dict, word, meta|{
                let value = dict.convert_id_a_to_word(word).unwrap().to_string();
                let solved = meta.cloned().map(LoadedMetadataEx::from);
                filter_a.call(value, solved).expect("This should not fail!")
            },
            |dict, word, meta|{
                let value = dict.convert_id_b_to_word(word).unwrap().to_string();
                let solved = meta.cloned().map(LoadedMetadataEx::from);
                filter_b.call(value, solved).expect("This should not fail!")
            },
        );

        Ok(PyDictionary { wrapped: Arc::new(RwLock::new(created)) })
    }

    /// Returns the meta for a specific word in a
    pub fn get_meta_a_of(&self, word: &str) -> Option<LoadedMetadataEx> {
        let read = self.get();
        let word_id = read.voc_a().get_id(word)?;
        let meta = read.metadata().get_meta_ref_a(read.voc_a(), word_id)?;
        Some(meta.into())
    }

    /// Returns the meta for a specific word in b
    pub fn get_meta_b_of(&self, word: &str) -> Option<LoadedMetadataEx> {
        let read = self.get();
        let word_id = read.voc_b().get_id(word)?;
        let meta = read.metadata().get_meta_ref_b(read.voc_b(), word_id)?;
        Some(meta.into())
    }

    /// Creates a new dictionary where the processor was applied.
    ///Requires that both languages (a+b) are properly set.
    pub fn process_with_tokenizer(
        &self,
        processor: PyAlignedArticleProcessor
    ) -> PyResult<Self> {
        let read = self.get();

        match read.language_direction_a_to_b() {
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

                Ok(
                    Self {
                        wrapped: Arc::new(RwLock::new(read.filter_and_process(
                            |value| {
                                apply_tokenizer_and_filer(&a_tok, value.as_str())
                            },
                            |value| {
                                apply_tokenizer_and_filer(&b_tok, value.as_str())
                            }
                        ).unwrap()))
                    }
                )
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


    /// Creates an html view of the vocabulary in this folder.
    /// If the thread limit is not set, the global thread pool will try to use the optimal number
    /// of threads for this CPU for complete usage.
    /// If the thread_limit is set to zero the number of threads will be chosen automatically.
    /// If the thread limit is grater than 0 this function will spawn n threads to create the necessary files.
    ///
    /// If crc32 is set to false, it does noch check the file contents before overriding it.
    /// Otherwise it it uses crc32 before overriding a file, to check if it is similar or not. (default)
    /// The algorithm used is the fast, SIMD-accelerated CRC32 (IEEE) checksum computation.
    ///
    /// The probability for a collision can be found here: https://preshing.com/20110504/hash-collision-probabilities/
    #[pyo3(signature = (path, crc32=None, thread_limit=None))]
    fn create_html_view_in(&self, path: PathBuf, crc32: Option<bool>, thread_limit: Option<usize>) -> PyResult<()> {
        let path = Utf8PathBuf::from_path_buf(path).map_err(|err| PyValueError::new_err(format!("Failed to convert the path to utf8! {err:?}")))?;
        match thread_limit {
            None => self.get().generate_html(path, crc32.unwrap_or(true)),
            Some(thread_limit) => {
                let pool = rayon::ThreadPoolBuilder::new().num_threads(thread_limit).build().map_err(|err| PyValueError::new_err(format!("Failed to convert the path to utf8! {err:?}")))?;
                pool.install(|| { self.get().generate_html(path, crc32.unwrap_or(true)) })
            }
        }.map_err(|err| PyValueError::new_err(format!("{err}")))
    }


    /// Returns a len object, containing the counts of the single parts
    #[pyo3(name = "len")]
    fn len_py(&self) -> Len {
        self.get().len()
    }

}

impl From<DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>> for PyDictionary {
    fn from(value: DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>) -> Self {
        Self::new(value)
    }
}


define_py_method!{
    FilterDictionary(word: String, loaded: Option<LoadedMetadataEx>) -> bool
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
pub struct PyDictIter {
    inner: DictionaryWithMetaIterator<DictionaryWithMeta<String, PyVocabulary, MetadataManagerEx>, String, PyVocabulary, MetadataManagerEx>,
}

unsafe impl Send for PyDictIter{}
unsafe impl Sync for PyDictIter{}

impl PyDictIter {
    pub fn new(inner: &PyDictionary) -> Self {
        Self {
            inner: DictionaryWithMetaIterator::new(inner.wrapped.clone())
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


impl From<Dictionary<String, Vocabulary<String>>> for PyDictionary {
    fn from(value: Dictionary<String, Vocabulary<String>>) -> Self {
        Self { wrapped: Arc::new(RwLock::new(value.map(|value| value.clone()).into())) }
    }
}

impl From<Dictionary<String, PyVocabulary>> for PyDictionary {
    #[inline(always)]
    fn from(inner: Dictionary<String, PyVocabulary>) -> Self {
        Self { wrapped: Arc::new(RwLock::new(inner.into())) }
    }
}

impl FromVoc<String, PyVocabulary> for PyDictionary {
    fn from_voc(voc_a: PyVocabulary, voc_b: PyVocabulary) -> Self {
        Self {
            wrapped: Arc::new(RwLock::new(DictionaryWithMeta::from_voc(voc_a, voc_b)))
        }
    }

    fn from_voc_lang_a(voc: PyVocabulary, other_lang: Option<LanguageHint>) -> Self {
        Self {
            wrapped: Arc::new(RwLock::new(DictionaryWithMeta::from_voc_lang_a(voc, other_lang)))
        }
    }

    fn from_voc_lang_b(other_lang: Option<LanguageHint>, voc: PyVocabulary) -> Self {
        Self {
            wrapped: Arc::new(RwLock::new(DictionaryWithMeta::from_voc_lang_b(other_lang, voc)))
        }
    }
}

register_python! {
    struct PyDictionary;
    struct PyDictIter;
}