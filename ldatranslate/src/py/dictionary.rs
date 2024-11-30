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

use std::collections::{HashMap, HashSet};
use crate::py::helpers::LanguageHintValue;
use crate::py::vocabulary::PyVocabulary;
use ldatranslate_topicmodel::dictionary::direction::{DirectionMarker, DirectedElement, LanguageMarker};
use ldatranslate_topicmodel::dictionary::iterators::{DictionaryWithMetaIterator};
use ldatranslate_topicmodel::dictionary::metadata::ex::{MetadataManagerEx, LoadedMetadataEx, MetaField};
use ldatranslate_topicmodel::dictionary::metadata::{MetaIterOwned, MetadataManager};
use ldatranslate_topicmodel::dictionary::*;
use ldatranslate_topicmodel::language_hint::LanguageHint;
use ldatranslate_topicmodel::vocabulary::{AnonymousVocabulary, SearchableVocabulary, Vocabulary};
use itertools::{EitherOrBoth, Itertools};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::{pyclass, pymethods, Bound, PyRef, PyResult};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use arcstr::ArcStr;
use camino::Utf8PathBuf;
use either::Either;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::PyTuple;
use strum::{EnumCount, IntoEnumIterator};
use crate::py::tokenizer::PyAlignedArticleProcessor;
use ldatranslate_toolkit::{define_py_method, register_python, type_def_wrapper};
use crate::py::aliases::{UnderlyingPyVocabulary, UnderlyingPyWord};
use ldatranslate_tokenizer::Tokenizer;
use ldatranslate_toolkit::from_str_ex::ParseEx;
use ldatranslate_toolkit::special_python_values::{PyEither, PyEitherOrBoth};
use ldatranslate_topicmodel::dictionary::io::{ReadableDictionary, WriteModeLiteral, WriteableDictionary};
use ldatranslate_topicmodel::dictionary::len::Len;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaVector;
use ldatranslate_topicmodel::dictionary::search::{SearchInput, SearchType, SearchTypeLiteral};

pub type DefaultDict = EfficientDictWithMetaDefault;

/// A dictionary for bidirectional dictionaries.
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyDictionary {
    inner: Arc<RwLock<DefaultDict>>,
}


impl PyDictionary {
    pub fn new(dict: DefaultDict) -> Self {
        Self {
            inner: Arc::new(RwLock::new(dict))
        }
    }

    pub fn get<'a>(&'a self) -> RwLockReadGuard<'a, DefaultDict> {
        self.inner.read().unwrap()
    }

    pub fn get_mut<'a>(&'a mut self) -> RwLockWriteGuard<'a, DefaultDict> {
        self.inner.write().unwrap()
    }

    pub fn into_inner(self) -> Arc<RwLock<DefaultDict>> {
        self.inner
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
        Ok(PyDictionary::new(DefaultDict::deserialize(deserializer)?))
    }
}


type_def_wrapper!(
    SearchResultContainer<PyEitherOrBoth<PyEither<Vec<String>, HashMap<String, HashMap<String, HashSet<String>>>>, PyEither<Vec<String>, HashMap<String, HashMap<String, HashSet<String>>>>>> with into
);

type_def_wrapper!(
      SearchTypeUnion<PyEither<SearchType, SearchTypeLiteral>>
);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyDictionary {
    #[new]
    pub fn new_py(language_a: Option<LanguageHintValue>, language_b: Option<LanguageHintValue>) -> Self {
        Self {
            inner: Arc::new(
                RwLock::new(
                    DictionaryWithMeta::new_with(
                        language_a,
                        language_b
                    )
                )
            )
        }
    }

    /// A search function for the dictionary, requires a query or matcher as first argument.
    ///
    /// :param query: The query can be a single words, multiple words or a matcher function.
    ///
    /// :param search_type: Determines which kind of search is execute. (default: SearchType.ExactMatch)
    ///                     See SearchType for more search types.
    ///                     Typing is not supported when using a matcher as query.
    ///
    /// :param threshold: The threshold is needed when using any kind of distance in as search.
    ///
    /// :param target_language: The target language can either be A, B or None and determines which vocabularies are searched. (see LanguageKind)
    ///                         None indicates to search in both vocabularies. (default: None)
    ///
    /// :param ignores_ascii_case: If the flag is set, the most searches ignore the case while comparing words. (default: false)
    ///                            The feature is not available for autocompletion or when using a custom matcher.
    /// :returns: Returns a tuple that can look like this (depending on the query):
    ///     When no target language is set:
    ///         - For simple query types: (list[str], list[str])
    ///         - For complex query types: (dict[str, dict[str, set[str]]], dict[str, dict[str, set[str]]])
    ///     When A is set as target language:
    ///         - For simple query types: (list[str], None)
    ///         - For complex query types: (dict[str, dict[str, set[str]]], None)
    ///     When B is set as target language:
    ///         - For simple query types: (None, list[str])
    ///         - For complex query types: (None, dict[str, dict[str, set[str]]])
    ///     The dict for a complex result is the following:
    ///         {<query_string>: {<prefix>: {<values>}}}
    ///     The list for simple queries basically only contains all the results in a bulk result.
    #[pyo3(signature = (query, search_type = None, threshold = None, target_language = None, ignores_ascii_case = None))]
    fn search<'py>(
        &self,
        query: SearchInput<'py>,
        search_type: Option<SearchTypeUnion>,
        threshold: Option<PyEither<usize, f64>>,
        target_language: Option<LanguageMarker>,
        ignores_ascii_case: Option<bool>,
    ) -> PyResult<SearchResultContainer> {
        let ignores_ascii_case = ignores_ascii_case.unwrap_or(false);
        let read = self.get();
        let searcher = read.get_searcher();
        let search_type = search_type.map(|value| {
            match value.into_inner().into_inner() {
                Either::Left(value) => {
                    Ok(value)
                }
                Either::Right(value) => {
                    value.parse_ex()
                }
            }
        }).transpose().map_err(|value| PyValueError::new_err(format!("Failed to parse argument: {value}")))?;



        let result = searcher.search(
            query,
            search_type,
            target_language,
            threshold.map(|value| value.into_inner()),
            ignores_ascii_case,
        )?;

        let result = match result {
            None => {
                drop(read);
                match target_language {
                    None => {
                        PyEitherOrBoth::both(
                            PyEither::left(Vec::with_capacity(0)),
                            PyEither::left(Vec::with_capacity(0))
                        )
                    }
                    Some(LanguageMarker::A) => {
                        PyEitherOrBoth::left(PyEither::left(Vec::with_capacity(0)))
                    }
                    Some(LanguageMarker::B) => {
                        PyEitherOrBoth::right(PyEither::left(Vec::with_capacity(0)))
                    }
                }
            }
            Some(value) => {
                fn convert_to_left(value: Vec<(usize, String)>) -> Vec<String> {
                    value.into_iter().map(|(_, v)| v.to_string()).collect_vec()
                }

                fn convert_to_right(
                    voc: &Vocabulary<ArcStr>,
                    value: HashMap<String, Vec<(String, Vec<usize>)>>
                ) -> HashMap<String, HashMap<String, HashSet<String>>>
                {
                    value.into_iter().map(|(k, v)| {
                        (
                            k,
                            v.into_iter().into_grouping_map().fold_with(
                                |_, _| HashSet::new(),
                                |mut acc, _, v| {
                                    acc.extend(
                                        v.into_iter()
                                            .map(|value|
                                                voc.id_to_entry(value)
                                                    .expect("A search never returns an id that is invalid!")
                                                    .to_string()
                                            )
                                    );
                                    acc
                                }
                            )
                        )
                    }).collect()
                }

                match value {
                    Either::Left(a) => {
                        drop(read);
                        match a {
                            EitherOrBoth::Both(a, b) => {
                                PyEitherOrBoth::both(
                                    PyEither::left(convert_to_left(a)),
                                    PyEither::left(convert_to_left(b)),
                                )
                            }
                            EitherOrBoth::Left(a) => {
                                PyEitherOrBoth::left(PyEither::left(convert_to_left(a)))
                            }
                            EitherOrBoth::Right(b) => {
                                PyEitherOrBoth::right(PyEither::left(convert_to_left(b)))
                            }
                        }
                    }
                    Either::Right(b) => {
                        match b {
                            EitherOrBoth::Both(a, b) => {
                                PyEitherOrBoth::both(
                                    PyEither::right(convert_to_right(read.voc_a(), a)),
                                    PyEither::right(convert_to_right(read.voc_b(), b)),
                                )
                            }
                            EitherOrBoth::Left(a) => {
                                PyEitherOrBoth::left(
                                    PyEither::right(convert_to_right(read.voc_a(), a))
                                )
                            }
                            EitherOrBoth::Right(b) => {
                                PyEitherOrBoth::right(
                                    PyEither::right(convert_to_right(read.voc_b(), b))
                                )
                            }
                        }
                    }
                }
            }
        };

        Ok(result.into())
    }



    /// Returns the dictionaries that where used to create this dictionary.
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

    /// Allows to set the translation languages. This is usually not necessary, except you build your own.
    #[setter]
    fn set_translation_direction(&mut self, direction: (Option<LanguageHintValue>, Option<LanguageHintValue>)) {
        let mut write = self.get_mut();
        write.set_language_a(direction.0.map(|value| value.into()));
        write.set_language_b(direction.1.map(|value| value.into()));
    }

    /// Returns the vocabulary a
    #[getter]
    #[pyo3(name = "voc_a")]
    fn voc_a_py(&self) -> PyVocabulary {
        PyVocabulary::new_from_dict(
            self.inner.clone(),
            LanguageMarker::A
        )
    }

    /// Returns the vocabulary b
    #[getter]
    #[pyo3(name = "voc_b")]
    fn voc_b_py(&self) -> PyVocabulary {
        PyVocabulary::new_from_dict(
            self.inner.clone(),
            LanguageMarker::B
        )
    }

    /// Returns the topic vetor vor a specific word. Can be None if the word does not exist or
    /// no metadata is set.
    pub fn topic_vector_a(&self, word: &str) -> Option<DictMetaVector> {
        let read = self.get();
        let entry = read.voc_a().get_id(word)?;
        read.metadata().get_meta_a(entry)?.topic_vector()
    }

    /// Returns the topic vetor vor a specific word. Can be None if the word does not exist or
    /// no metadata is set.
    pub fn topic_vector_b(&self, word: &str) -> Option<DictMetaVector> {
        let read = self.get();
        let entry = read.voc_b().get_id(word)?;
        read.metadata().get_meta_b(entry)?.topic_vector()
    }

    /// Returns true iff there is any content in the unaltered voc.
    fn has_unaltered_voc(&self) -> bool {
        self.get().metadata().has_content_for_field(
            MetadataManagerEx::unprocessed_field().expect("The metadata needs a unprocessed field.")
        )
    }

    /// The length of the metadata
    fn meta_len(&self) -> (usize, usize) {
        self.get().metadata().len()
    }

    /// Returns the number of words that know their unaltered vocabulary
    fn count_unaltered_voc(&self) -> (usize, usize) {
        self.get().metadata().count_metas_with_content_for_field(
            MetadataManagerEx::unprocessed_field().expect("The metadata needs a unprocessed field.")
        )
    }

    /// Returns true if voc a contains the value
    fn voc_a_contains(&self, value: &str) -> bool {
        self.get().voc_a().contains_value(value)
    }

    /// Returns true if voc a contains the value
    fn voc_b_contains(&self, value: &str) -> bool {
        self.get().voc_b().contains_value(value)
    }

    /// Returns true if any voc contains the value
    fn __contains__(&self, value: &str) -> bool {
        self.voc_a_contains(value) || self.voc_b_contains(value)
    }

    fn switch_a_to_b(&self) -> Self {
        Self {
            inner: Arc::new(
                RwLock::new(
                    DictionaryWithMeta::clone(&self.get()).switch_languages()
                )
            )
        }
    }

    /// Insert a translation from word a to b with the provided data.
    pub fn add(
        &mut self,
        word_a: (String, Option<LoadedMetadataEx>),
        word_b: (String, Option<LoadedMetadataEx>)
    ) -> PyResult<(usize, usize, DirectionMarker)> {
        let (a_word, a_solved) = word_a;
        let (b_word, b_solved) = word_b;

        let mut write = self.inner.write().unwrap();

        match write.insert_translation_ref_with_meta_invariant(
            a_word.into(),
            a_solved.as_ref(),
            b_word.into(),
            b_solved.as_ref(),
        ) {
            Ok((a, b)) => {
                Ok((a, b, DirectionMarker::Invariant))
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
        format!("PyDictionary({:?})", self.inner)
    }

    fn __str__(&self) -> String {
        format!("PyDictionary({:?})", self.inner)
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
            DefaultDict::from_path_with_extension(path)
                .map_err(|value| PyValueError::new_err(value.to_string()))?
                .into()
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

    fn iter_meta_a(&self) -> PyMetaIter {
        PyMetaIter::new(self, DirectionMarker::AToB)
    }

    fn iter_meta_b(&self) -> PyMetaIter {
        PyMetaIter::new(self, DirectionMarker::BToA)
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

        Ok(PyDictionary { inner: Arc::new(RwLock::new(created)) })
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

    pub fn process_and_filter<'py>(&self, lang_a_proc: ProcessAndFilterDictionaryMethod<'py>, lang_b_proc: ProcessAndFilterDictionaryMethod<'py>) -> PyResult<Self> {
        let read = self.get();
        Ok(
            Self {
                inner: Arc::new(RwLock::new(read.filter_and_process(
                    |value| {
                        lang_a_proc.call(value.to_string()).map(
                            |value| {
                                value.map(|(word, unstemmed)| {
                                    if let Some(u) = unstemmed {
                                        DictionaryWithMetaProcessResult::with_unprocessed(word.into(), u.into())
                                    } else {
                                        DictionaryWithMetaProcessResult::new(word.into())
                                    }
                                })
                            }
                        )
                    },
                    |value| {
                        lang_b_proc.call(value.to_string()).map(
                            |value| {
                                value.map(|(word, unstemmed)| {
                                    if let Some(u) = unstemmed {
                                        DictionaryWithMetaProcessResult::with_unprocessed(word.into(), u.into())
                                    } else {
                                        DictionaryWithMetaProcessResult::new(word.into())
                                    }
                                })
                            }
                        )
                    },
                )?))
            }
        )
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

                fn apply_tokenizer_and_filer(tokenizer: &Tokenizer, value: &str) -> Result<Option<DictionaryWithMetaProcessResult<ArcStr>>, ()> {
                    let result = tokenizer.process(value).collect_vec();
                    let processed = result.iter().filter(|value| !value.1.lemma.is_empty() && value.1.is_word()).map(|value| value.1.lemma()).join(" ");
                    if processed.is_empty() {
                        Ok(None)
                    } else {
                        let value: ArcStr = processed.trim().into();
                        if value.is_empty() {
                            Ok(None)
                        } else {
                            Ok(
                                Some(
                                    DictionaryWithMetaProcessResult::with_unprocessed(
                                        value,
                                        result.iter().map(|value| value.0).join(" ").trim().into()
                                    )
                                )
                            )
                        }
                    }
                }

                Ok(
                    Self {
                        inner: Arc::new(RwLock::new(read.filter_and_process(
                            |value| apply_tokenizer_and_filer(&a_tok, value.as_str()),
                            |value| apply_tokenizer_and_filer(&b_tok, value.as_str()),
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
        self.render_html_view(path, crc32, thread_limit).map_err(|err| PyValueError::new_err(format!("{err}")))
    }


    /// Returns a len object, containing the counts of the single parts
    #[pyo3(name = "len")]
    fn len_py(&self) -> Len {
        self.get().len()
    }

    /// Returns a list of fields that hold data.
    fn get_fields_with_content(&self) -> Vec<MetaField> {
        let read = self.get();
        MetaField::iter().filter(|value| read.metadata().has_content_for_field(*value)).collect_vec()
    }

    /// Drops the metadata from a field.
    /// Returns true iff data was lost.
    fn drop_metadata_field(&mut self, field: MetaField) -> bool {
        self.get_mut().metadata_mut().drop_field(field)
    }

    /// Drops all fields except the one in the args.
    /// Returns a list of dropped fields and if data was dropped.
    #[pyo3(signature = (*py_args))]
    fn drop_all_except(&mut self, py_args: &Bound<'_, PyTuple>) -> PyResult<Vec<(MetaField, bool)>> {
        Ok(
            self.drop_all_except_impl(
                &HashSet::from_iter(py_args.extract::<Vec<MetaField>>()?)
            )
        )
    }
}


impl PyDictionary {

    fn render_html_view(&self, path: impl AsRef<Path>, crc32: Option<bool>, thread_limit: Option<usize>) -> Result<(), std::io::Error> {
        let path = Utf8PathBuf::from_path_buf(path.as_ref().to_path_buf()).map_err(|err| PyValueError::new_err(format!("Failed to convert the path to utf8! {err:?}")))?;
        match thread_limit {
            None => self.get().generate_html(path, crc32.unwrap_or(true)),
            Some(thread_limit) => {
                let pool = rayon::ThreadPoolBuilder::new().num_threads(thread_limit).build().map_err(|err| PyValueError::new_err(format!("Failed to convert the path to utf8! {err:?}")))?;
                pool.install(|| { self.get().generate_html(path, crc32.unwrap_or(true)) })
            }
        }
    }
    fn drop_all_except_impl(&mut self, to_keep: &HashSet<MetaField>) -> Vec<(MetaField, bool)> {
        let mut r = self.get_mut();
        let mut result = Vec::with_capacity(MetaField::COUNT - to_keep.len());
        for value in MetaField::iter() {
            if to_keep.contains(&value) {
                continue
            }
            let rem = r.metadata_mut().drop_field(value);
            result.push((value, rem));
        }
        result
    }
}

define_py_method!{
    FilterDictionary(word: String, loaded: Option<LoadedMetadataEx>) -> bool
}

define_py_method!{
    ProcessAndFilterDictionary(word: String) -> Option<(String, Option<String>)>
}



impl From<Dictionary<UnderlyingPyWord, UnderlyingPyVocabulary>> for PyDictionary {
    fn from(value: Dictionary<UnderlyingPyWord, UnderlyingPyVocabulary>) -> Self {
        Self { inner: Arc::new(RwLock::new(value.map(|value| value.clone()).into())) }
    }
}

impl From<DefaultDict> for PyDictionary {
    fn from(value: DefaultDict) -> Self {
        Self { inner: Arc::new(RwLock::new(value)) }
    }
}

impl FromVoc<UnderlyingPyWord, UnderlyingPyVocabulary> for PyDictionary {
    fn from_voc(voc_a: UnderlyingPyVocabulary, voc_b: UnderlyingPyVocabulary) -> Self {
        Self {
            inner: Arc::new(RwLock::new(DictionaryWithMeta::from_voc(voc_a, voc_b)))
        }
    }

    fn from_voc_lang_a(voc: UnderlyingPyVocabulary, other_lang: Option<LanguageHint>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(DictionaryWithMeta::from_voc_lang_a(voc, other_lang)))
        }
    }

    fn from_voc_lang_b(other_lang: Option<LanguageHint>, voc: UnderlyingPyVocabulary) -> Self {
        Self {
            inner: Arc::new(RwLock::new(DictionaryWithMeta::from_voc_lang_b(other_lang, voc)))
        }
    }
}





#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
pub struct PyDictIter {
    inner: DictionaryWithMetaIterator<DefaultDict, ArcStr, Vocabulary<ArcStr>, MetadataManagerEx>,
}

unsafe impl Send for crate::py::dictionary::PyDictIter {}
unsafe impl Sync for crate::py::dictionary::PyDictIter {}

impl crate::py::dictionary::PyDictIter {
    pub fn new(inner: &PyDictionary) -> Self {
        Self {
            inner: DictionaryWithMetaIterator::new(inner.inner.clone())
        }
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::py::dictionary::PyDictIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<((usize, String, Option<LoadedMetadataEx>), (usize, String, Option<LoadedMetadataEx>), DirectionMarker)> {
        let DirectedElement {
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




#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
pub struct PyMetaIter {
    inner: MetaIterOwned<DefaultDict, ArcStr, Vocabulary<ArcStr>, MetadataManagerEx>,
}

unsafe impl Send for PyMetaIter {}
unsafe impl Sync for PyMetaIter {}

impl PyMetaIter {
    pub fn new(inner: &PyDictionary, direction: DirectionMarker) -> Self {
        Self {
            inner: MetaIterOwned::new(inner.inner.clone(), direction)
        }
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyMetaIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<(usize, String, Option<LoadedMetadataEx>)> {
        self.inner.next().map(|(word_id, word, meta)| {
            (word_id, word.to_string(), meta)
        })
    }
}

register_python! {
    struct PyDictionary;
    struct PyDictIter;
    struct PyMetaIter;
}

#[cfg(test)]
mod test  {
    use std::collections::HashSet;
    use camino::Utf8PathBuf;
    use strum::IntoEnumIterator;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, DictionaryFilterable, DictionaryWithVocabulary};
    use ldatranslate_topicmodel::dictionary::direction::DirectionMarker;
    use ldatranslate_topicmodel::dictionary::io::WriteableDictionary;
    use ldatranslate_topicmodel::dictionary::metadata::ex::MetaField;
    use ldatranslate_topicmodel::dictionary::metadata::MetadataManager;
    use crate::py::dictionary::PyDictionary;
    use crate::py::tokenizer::PyAlignedArticleProcessor;

    #[test]
    fn see(){
        let dict = PyDictionary::load(r#"E:\git\tmt\test\dictionary_final3.dat.zst"#.into()).unwrap();
        let mut set = HashSet::new();
        for value in dict.get().iter_with_meta_dir(DirectionMarker::Invariant) {
            assert!(set.insert((value.a.0, value.b.0)))
        }
    }

    #[test]
    fn test_processing(){
        fn dict_path(v: &str) -> Utf8PathBuf {
            Utf8PathBuf::from("./test").join("dict").join(format!("dictionary_20241130{}.dat.zst", v))
        }

        fn view_path(v: &str) -> Utf8PathBuf {
            Utf8PathBuf::from("./test").join("view").join(format!("dictionary_20241130{}", v))
        }


        let dict = {

            let mut dict = PyDictionary::load(dict_path("").into()).unwrap();

            dict.render_html_view(
                view_path(""), None, None
            ).unwrap();

            let to_keep = [
                MetaField::Domains,
                MetaField::Languages,
                MetaField::Registers,
                MetaField::Genders,
                MetaField::Pos,
                MetaField::PosTag,
                MetaField::Numbers,
                MetaField::Regions,
            ];
            dict.drop_all_except_impl(&HashSet::from_iter(to_keep));

            for field in MetaField::iter() {
                if to_keep.contains(&field) {
                    continue;
                }
                for value in dict.get().metadata().meta_a() {
                    let (general, rest) = value.get_raw_metadata(field);
                    if let Some(general) = general {
                        assert!(general.is_empty(), "Failed for general in {field}!")
                    }
                    for (i, t) in rest.into_iter().enumerate().filter_map(|(i, v)| v.map(|v| (i, v))) {
                        assert!(t.is_empty(), "Failed for {i} in {field}!")
                    }
                }
                for value in dict.get().metadata().meta_b() {
                    let (general, rest) = value.get_raw_metadata(field);
                    if let Some(general) = general {
                        assert!(general.is_empty(), "Failed for general in {field}!")
                    }
                    for (i, t) in rest.into_iter().enumerate().filter_map(|(i, v)| v.map(|v| (i, v))) {
                        assert!(t.is_empty(), "Failed for {i} in {field}!")
                    }
                }
            }

            dict.get().write_to_path_with_extension(dict_path("_proc1")).unwrap();

            dict.get().generate_html(view_path("_proc1"), false).unwrap();

            dict
        };

        println!("Len0 {}", dict.get().len());
        let domain_counts = dict.get().metadata().domain_count();
        println!("Vec {}", domain_counts);

        // ca 7 min

        let dict = {
            let proc = PyAlignedArticleProcessor::from_json(
                r#"{"builders":{"de":{"unicode":true,"words_dict":null,"normalizer_option":{"create_char_map":false,"classifier":{"stop_words":{"inner":[3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,16,135,194,0,0,0,0,115,114,110,109,16,68,194,0,111,16,65,1,5,115,108,16,2,0,0,0,0,12,115,114,110,109,101,16,69,199,194,1,100,16,65,0,16,142,0,0,1,115,102,99,16,3,1,12,0,30,51,117,110,109,108,98,16,5,0,16,136,0,116,16,65,1,0,115,110,16,2,1,11,105,101,16,2,0,16,129,200,0,16,139,0,16,130,218,207,1,101,16,65,1,115,16,65,0,16,147,0,16,159,128,1,5,8,21,24,195,122,115,110,109,16,69,111,101,16,65,203,38,16,130,218,207,194,1,115,16,65,5,0,115,110,16,66,0,110,16,65,194,218,207,194,1,151,115,101,16,66,0,27,110,108,16,2,194,1,115,16,65,1,12,26,32,42,115,114,110,109,105,16,5,0,0,0,0,32,115,114,110,109,108,16,69,1,101,16,65,1,115,16,65,0,1,157,114,101,99,16,3,123,165,114,99,16,2,171,16,138,1,114,16,65,1,8,14,42,100,117,111,105,101,97,16,5,108,16,151,0,16,143,197,1,5,229,109,105,101,16,67,203,0,16,134,1,97,16,65,204,245,0,1,1,214,0,114,101,99,32,3,1,12,0,0,20,117,116,115,114,105,16,5,24,1,32,188,128,195,192,158,16,134,194,1,162,119,103,16,2,194,152,101,16,65,155,116,16,65,1,5,116,98,16,2,55,1,116,32,65,1,0,61,1,110,101,32,2,1,14,105,97,16,2,242,101,16,65,210,1,0,114,110,109,16,3,0,16,144,194,0,1,115,100,16,66,12,1,1,0,0,0,11,0,54,1,115,110,109,104,99,32,5,29,1,32,182,1,0,106,1,106,1,116,110,100,32,3,40,16,146,1,4,43,109,101,100,16,3,51,1,32,139,18,1,32,136,56,1,59,1,116,101,32,2,203,203,182,192,1,13,17,195,101,97,16,3,39,1,32,142,160,1,32,142,1,99,16,65,1,9,110,99,16,2,0,0,0,0,143,1,116,114,99,32,3,105,1,116,32,65,198,198,1,8,57,19,117,105,101,97,16,4,0,115,16,65,193,206,202,0,0,114,110,16,2,1,0,11,1,7,0,11,1,117,111,105,97,32,4,149,1,32,139,1,0,244,1,0,0,104,100,98,32,3,171,1,32,134,218,1,0,135,1,5,2,108,105,104,32,3,0,16,146,1,0,0,0,232,1,110,101,99,32,3,85,111,108,99,16,2,203,1,32,135,194,212,1,1,0,115,100,32,2,1,14,110,108,16,66,1,26,40,111,105,101,16,3,0,0,0,115,110,109,16,67,1,101,16,65,199,1,101,16,65,79,2,1,0,0,0,116,115,100,32,3,1,0,110,109,16,2,115,1,32,130,0,0,0,114,110,109,16,3,1,9,111,105,16,2,30,2,26,2,115,101,32,66,0,1,115,114,16,2,128,2,0,0,116,108,32,2,218,16,138,249,1,32,146,1,0,55,2,5,0,8,0,0,0,114,110,108,105,103,32,5,160,2,100,32,65,81,2,0,0,115,100,32,66,1,0,192,1,9,0,114,108,101,32,3,30,1,32,143,1,108,16,65,175,16,139,194,199,206,56,16,135,1,4,188,164,16,2,1,16,24,48,80,195,111,105,101,97,16,5,0,0,114,109,16,66,55,1,32,138,198,1,0,231,2,105,97,32,2,1,14,119,117,16,2,244,2,32,154,188,192,1,0,7,0,32,0,137,0,155,0,189,0,252,0,11,1,38,1,84,1,112,1,138,1,177,1,210,1,221,1,228,1,19,2,169,2,188,2,195,122,119,118,117,115,111,110,109,107,106,105,104,103,102,101,100,98,97,32,19,235,0,0,0,0,0,0,0,71,3,0,0,0,0,0,0,240,0,183,189]},"separators":[" ",",",":",".","\n","\r\n","(","[","{",")","]","}","!","\t","?","\"","'","|","`","-","_"]},"lossy":true},"segmenter_option":{"allow_list":null},"stemmer":["German",false],"vocabulary":null},"en":{"unicode":true,"words_dict":null,"normalizer_option":{"create_char_map":false,"classifier":{"stop_words":{"inner":[3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,16,129,0,16,130,1,4,118,117,16,2,196,0,16,135,194,193,16,115,16,65,203,200,197,0,16,139,0,16,143,0,0,121,100,16,66,35,39,16,65,1,110,16,65,194,0,0,1,10,0,16,19,22,29,34,116,115,114,110,109,108,105,103,102,98,16,74,63,16,134,211,197,68,16,135,196,0,16,151,203,0,16,140,196,58,16,130,194,204,1,6,10,14,63,18,116,108,105,102,101,99,16,70,0,16,142,193,0,106,1,5,121,117,111,101,16,4,78,16,139,210,207,211,1,97,111,97,16,2,86,16,146,89,16,134,109,96,56,1,119,110,105,101,16,68,66,16,136,199,1,5,18,117,111,105,16,67,52,16,138,197,0,16,144,196,150,16,142,193,199,1,6,156,88,117,114,111,101,16,4,101,137,105,101,16,2,1,143,143,118,115,100,16,3,0,16,155,207,1,101,16,65,1,0,115,101,16,66,1,114,16,65,15,16,130,1,115,16,65,0,1,115,109,16,2,143,1,14,32,111,105,101,97,16,4,0,16,132,1,116,16,65,0,16,134,44,1,115,39,16,66,1,208,10,0,116,115,110,102,16,68,4,1,32,134,211,147,16,129,206,215,14,1,11,1,115,114,32,2,11,16,134,64,1,4,12,0,0,121,117,111,105,101,97,16,70,176,16,130,0,0,0,119,116,114,16,67,1,9,111,101,16,2,0,102,16,65,0,16,157,1,0,60,1,108,99,32,66,84,16,130,226,207,1,101,16,65,1,115,16,65,0,1,116,114,16,2,65,1,76,1,1,0,180,0,0,0,20,0,31,0,119,118,117,116,114,110,102,32,71,110,1,32,144,130,39,16,65,118,1,32,162,87,1,1,0,110,39,32,66,210,207,211,1,0,16,0,18,1,111,101,97,32,3,144,1,109,32,65,250,0,48,1,1,0,6,0,0,0,36,0,117,116,111,104,98,97,32,70,144,1,32,143,1,39,16,65,1,0,116,110,16,2,0,115,16,65,199,116,16,130,1,115,16,65,0,0,195,1,195,1,0,0,1,0,8,0,121,115,114,110,109,105,32,70,114,1,32,151,211,196,1,0,156,1,237,0,7,0,39,0,114,111,105,101,97,32,5,0,111,16,65,1,5,111,104,16,66,223,1,32,136,1,0,241,1,116,100,32,2,0,1,112,110,16,2,207,114,16,65,194,231,1,114,32,65,20,2,0,0,114,110,32,2,28,2,183,1,108,99,32,2,0,109,16,65,0,0,1,0,5,0,13,0,43,2,121,111,105,101,97,32,5,212,1,32,2,116,108,32,2,204,1,30,2,117,110,32,2,1,0,9,0,17,0,54,0,199,1,111,105,104,101,97,32,5,90,2,90,2,65,2,0,0,118,114,108,100,32,4,34,1,0,0,118,102,32,2,207,1,101,16,65,1,115,16,65,1,18,114,39,16,66,211,1,111,16,65,1,0,43,0,118,0,123,0,141,0,220,0,132,2,23,1,80,1,97,1,107,2,127,1,132,1,158,1,213,1,232,1,236,1,8,2,20,2,70,2,121,119,118,117,116,115,114,111,110,109,108,106,105,104,102,101,100,99,98,97,32,20,181,0,0,0,0,0,0,0,214,2,0,0,0,0,0,0,175,79,249,173]},"separators":[" ",",",":",".","\n","\r\n","(","[","{",")","]","}","!","\t","?","\"","'","|","`","-","_"]},"lossy":true},"segmenter_option":{"allow_list":null},"stemmer":["English",false],"vocabulary":null}}}"#
            ).unwrap();
            let dict = dict.process_with_tokenizer(proc).unwrap();
            dict.get().write_to_path_with_extension(dict_path("_proc2")).unwrap();
            dict.get().generate_html(view_path("_proc2"), false).unwrap();
            dict
        };

        println!("Len1 {}", dict.get().len());
        let domain_counts = dict.get().metadata().domain_count();
        println!("Vec {}", domain_counts);

        let dict = {
            let dict = dict.get().filter_by_values(
                |value| value.split_whitespace().take(3).count() < 3 && value.starts_with(|v| char::is_alphanumeric(v)) && value.ends_with(|v| char::is_alphanumeric(v)),
                |value| value.split_whitespace().take(3).count() < 3 && value.starts_with(|v| char::is_alphanumeric(v)) && value.ends_with(|v| char::is_alphanumeric(v))
            );
            dict.write_to_path_with_extension(dict_path("_proc3")).unwrap();
            dict.generate_html(view_path("_proc3"), false).unwrap();
            dict
        };

        println!("Len2: {}", dict.len());
        let domain_counts = dict.metadata().domain_count();
        println!("Vec {}", domain_counts);

        let dict = {
            let dict = dict.filter_by_values(
                |value| !value.contains(' '),
                |value| !value.contains(' ')
            );
            dict.write_to_path_with_extension(dict_path("_proc4")).unwrap();
            dict.generate_html(view_path("_proc4"), false).unwrap();
            dict
        };

        println!("Len4: {}", dict.len());
        let domain_counts = dict.metadata().domain_count();
        println!("Vec {}", domain_counts);
    }
}