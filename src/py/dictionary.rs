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
use crate::topicmodel::dictionary::direction::{DirectionKind, DirectionTuple, LanguageKind};
use crate::topicmodel::dictionary::iterators::{DictionaryWithMetaIterator};
use crate::topicmodel::dictionary::metadata::ex::{MetadataManagerEx, LoadedMetadataEx, MetaField};
use crate::topicmodel::dictionary::metadata::{MetaIterOwned, MetadataManager};
use crate::topicmodel::dictionary::*;
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, SearchableVocabulary, Vocabulary};
use itertools::{EitherOrBoth, Itertools};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::{pyclass, pymethods, Bound, PyRef, PyResult};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use arcstr::ArcStr;
use camino::Utf8PathBuf;
use either::Either;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::PyTuple;
use strum::{EnumCount, IntoEnumIterator};
use crate::py::tokenizer::PyAlignedArticleProcessor;
use crate::{define_py_method, register_python, type_def_wrapper};
use crate::py::aliases::{UnderlyingPyVocabulary, UnderlyingPyWord};
use crate::tokenizer::Tokenizer;
use crate::toolkit::from_str_ex::ParseEx;
use crate::toolkit::special_python_values::{PyEither, PyEitherOrBoth};
use crate::topicmodel::dictionary::io::{ReadableDictionary, WriteModeLiteral, WriteableDictionary};
use crate::topicmodel::dictionary::len::Len;
use crate::topicmodel::dictionary::metadata::dict_meta_topic_matrix::TopicVector;
use crate::topicmodel::dictionary::search::{SearchInput, SearchType, SearchTypeLiteral};

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
        target_language: Option<LanguageKind>,
        ignores_ascii_case: Option<bool>,
    ) -> PyResult<SearchResultContainer> {
        let ignores_ascii_case = ignores_ascii_case.unwrap_or(false);
        let read = self.get();
        let searcher = read.inner.get_searcher();
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
                    Some(LanguageKind::A) => {
                        PyEitherOrBoth::left(PyEither::left(Vec::with_capacity(0)))
                    }
                    Some(LanguageKind::B) => {
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
            LanguageKind::A
        )
    }

    /// Returns the vocabulary b
    #[getter]
    #[pyo3(name = "voc_b")]
    fn voc_b_py(&self) -> PyVocabulary {
        PyVocabulary::new_from_dict(
            self.inner.clone(),
            LanguageKind::B
        )
    }

    /// Returns the topic vetor vor a specific word. Can be None if the word does not exist or
    /// no metadata is set.
    pub fn topic_vector_a(&self, word: &str) -> Option<TopicVector> {
        let read = self.get();
        let entry = read.voc_a().get_id(word)?;
        read.metadata().get_meta_a(entry)?.topic_vector()
    }

    /// Returns the topic vetor vor a specific word. Can be None if the word does not exist or
    /// no metadata is set.
    pub fn topic_vector_b(&self, word: &str) -> Option<TopicVector> {
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
    ) -> PyResult<(usize, usize, DirectionKind)> {
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
        PyMetaIter::new(self, DirectionKind::AToB)
    }

    fn iter_meta_b(&self) -> PyMetaIter {
        PyMetaIter::new(self, DirectionKind::BToA)
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
                        Ok(
                            Some(
                                DictionaryWithMetaProcessResult::with_unprocessed(
                                    processed.into(),
                                    result.iter().map(|value| value.0).join(" ").into()
                                )
                            )
                        )
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
        let to_keep: HashSet<_> = HashSet::from_iter(py_args.extract::<Vec<MetaField>>()?);
        let mut r = self.get_mut();
        let mut result = Vec::with_capacity(MetaField::COUNT - to_keep.len());
        for value in MetaField::iter() {
            if to_keep.contains(&value) {
                continue
            }
            let rem = r.metadata_mut().drop_field(value);
            result.push((value, rem));
        }
        Ok(result)
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




#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
pub struct PyMetaIter {
    inner: MetaIterOwned<DefaultDict, ArcStr, Vocabulary<ArcStr>, MetadataManagerEx>,
}

unsafe impl Send for PyMetaIter {}
unsafe impl Sync for PyMetaIter {}

impl PyMetaIter {
    pub fn new(inner: &PyDictionary, direction: DirectionKind) -> Self {
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