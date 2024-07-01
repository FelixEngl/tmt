use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter};
use std::ops::Range;
use std::path::{PathBuf};
use std::sync::Arc;
use itertools::Itertools;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use topic_model::{DocumentLength, DocumentTo, Probability, TopicTo, WordFrequency, WordTo};
use topicmodel::topic_model;
use crate::py::helpers::{HasPickleSupport, LanguageHintValue, PyTopicModelStateValue};
use crate::py::topic_model_builder::PyTopicModelBuilder;
use crate::py::vocabulary::PyVocabulary;
use crate::toolkit::partial_ord_iterator::PartialOrderIterator;
use crate::{topicmodel};
use crate::topicmodel::enums::{ReadError, TopicModelVersion, WriteError};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::topic_model::{BasicTopicModel, BasicTopicModelWithVocabulary, DocumentId, SingleOrList, TopicId, TopicModel, TopicModelInferencer, TopicModelWithDocumentStats, TopicModelWithVocabulary, WordId};
use crate::topicmodel::topic_model::meta::*;
use crate::topicmodel::vocabulary::{BasicVocabulary, VocabularyMut};

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyTopicModel {
    inner: TopicModel<String, PyVocabulary>
}

impl PyTopicModel {
    pub fn wrap(inner: TopicModel<String, PyVocabulary>) -> Self {
        Self{inner}
    }
}


#[derive(Debug, Clone, FromPyObject)]
enum PlainTranslateArgs {
    List(Vec<String>),
    ListList(Vec<Vec<String>>),
}


#[pymethods]
impl PyTopicModel {
    #[new]
    pub fn new(
        topics: Vec<Vec<f64>>,
        vocabulary: PyVocabulary,
        used_vocab_frequency: Vec<u64>,
        doc_topic_distributions: Vec<Vec<f64>>,
        document_lengths: Vec<u64>,
    ) -> Self {
        Self {
            inner: TopicModel::new(
                topics,
                vocabulary,
                used_vocab_frequency,
                doc_topic_distributions,
                document_lengths
            )
        }
    }

    #[getter]
    #[pyo3(name="k")]
    fn py_k(&self) -> usize {
        self.inner.k()
    }

    #[pyo3(name="get_topic")]
    fn py_topic(&self, topic_id: usize) -> Option<Vec<f64>> {
        self.inner.get_topic(topic_id).cloned()
    }


    fn save(&self, path: PathBuf) -> PyResult<usize> {
        Ok(self.inner.save(path, TopicModelVersion::V1, true, true)?)
    }

    #[staticmethod]
    fn load(path: PathBuf) -> PyResult<PyTopicModel> {
        Ok(Self { inner: TopicModel::<_,PyVocabulary>::load(path, true)?.0 })
    }

    fn show_top(&self, n: Option<usize>) -> PyResult<()> {
        if let Some(n) = n {
            self.inner.show(n)?
        } else {
            self.inner.show_10()?
        };
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!("PyTopicModel({:?})", self.inner)
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn get_doc_probability(
        &self,
        doc: Vec<String>,
        alpha: SingleOrList,
        gamma_threshold: f64,
        minimum_probability: Option<f64>,
        minimum_phi_value: Option<f64>,
        per_word_topics: Option<bool>
    ) -> (Vec<(usize, f64)>, Option<Vec<(usize, Vec<usize>)>>, Option<Vec<(usize, Vec<(usize, f64)>)>>) {
        TopicModelInferencer::<String, PyVocabulary, Self>::new(
            &self,
            alpha,
            gamma_threshold
        ).get_doc_probability_for(
            doc,
            minimum_probability.unwrap_or(TopicModelInferencer::<String, PyVocabulary, PyTopicModel>::DEFAULT_MIN_PROBABILITY),
            minimum_phi_value.unwrap_or(TopicModelInferencer::<String, PyVocabulary, PyTopicModel>::DEFAULT_MIN_PHI_VALUE),
            per_word_topics.unwrap_or_default()
        )
    }

    fn vocabulary(&self) -> PyVocabulary {
        self.inner.vocabulary().clone()
    }

    fn get_words_of_topic_sorted(&self, topic_id: usize) -> Option<Vec<(String, f64)>> {
        self.get_words_for_topic_sorted(topic_id)
            .map(|value|
                value
                    .iter()
                    .map(|v| (self.inner.vocabulary().id_to_word(v.word_id).unwrap().to_string(), v.probability))
                    .collect_vec()
            )

    }

    fn translate_by_provided_word_lists(&self, language_hint: LanguageHintValue, word_lists: PlainTranslateArgs) -> PyResult<PyTopicModel> {
        if let PlainTranslateArgs::ListList(ref word_lists) = word_lists {
            if word_lists.len() != self.inner.topic_count() {
                return Err(PyValueError::new_err(format!("Expected {} lists, but got {}", self.inner.topic_count(), word_lists.len())))
            }
        }



        let min_value = self.inner.topics().iter().flatten().min_partial_filtered().unwrap().clone();

        let mut voc = PyVocabulary::new(Some(language_hint.into()), None);

        let mut new_probability = Vec::new();
        for _ in 0..self.inner.topic_count() {
            new_probability.push(vec![min_value; voc.len()]);
        }

        let mut vocab_frequency: Vec<u64>;

        match word_lists {
            PlainTranslateArgs::List(value) => {
                vocab_frequency = vec![0u64; voc.len()];
                let word_ids = value.into_iter().map(|value| {
                    voc.add(value)
                }).collect_vec();
                for topic_id in self.inner.topic_ids() {
                    let topic_old = self.inner.topics().get(topic_id).unwrap();
                    let topic_new = new_probability.get_mut(topic_id).unwrap();
                    for (word_id_old, word_id_new) in word_ids.iter().cloned().enumerate() {
                        topic_new[word_id_new] = topic_old[word_id_old].clone();
                        vocab_frequency[word_id_new] = self.inner.used_vocab_frequency()[word_id_old];
                    }
                }
            }
            PlainTranslateArgs::ListList(word_lists) => {
                vocab_frequency = vec![0u64; voc.len()];

                let word_lists = word_lists.into_iter().map(|values| {
                    values.into_iter().map(|value| {
                        voc.add(value)
                    }).collect_vec()
                }).collect_vec();

                for (topic_id, word_ids) in word_lists.into_iter().enumerate() {
                    let topic_old = self.inner.topics().get(topic_id).unwrap();
                    let topic_new = new_probability.get_mut(topic_id).unwrap();
                    for (word_id_old, word_id_new) in word_ids.into_iter().enumerate() {
                        topic_new[word_id_new] = topic_old[word_id_old].clone();
                        vocab_frequency[word_id_new] = self.inner.used_vocab_frequency()[word_id_old];
                    }
                }
            }
        };

        Ok(
            Self {
                inner: TopicModel::new(
                    new_probability,
                    voc,
                    vocab_frequency,
                    self.doc_topic_distributions().clone(),
                    self.document_lengths().clone()
                )
            }
        )
    }


    fn save_json(&self, path: PathBuf) -> PyResult<()> {
        serde_json::to_writer(BufWriter::new(File::options().write(true).create_new(true).open(path)?), &self.inner).map_err(|value| PyValueError::new_err(value.to_string()))
    }

    fn save_binary(&self, path: PathBuf) -> PyResult<()> {
        bincode::serialize_into(BufWriter::new(File::options().write(true).create_new(true).open(path)?), &self.inner).map_err(|value| PyValueError::new_err(value.to_string()))
    }
    #[staticmethod]
    fn load_json(path: PathBuf) -> PyResult<Self> {
        serde_json::from_reader(BufReader::new(File::options().read(true).open(path)?)).map_err(|value| PyValueError::new_err(value.to_string()))
    }

    #[staticmethod]
    fn load_binary(path: PathBuf) -> PyResult<Self> {
        bincode::deserialize_from(BufReader::new(File::options().read(true).open(path)?)).map_err(|value| PyValueError::new_err(value.to_string()))
    }


    fn normalize(&self) -> Self {
        self.inner.normalize().into()
    }

    fn __getnewargs__(&self) -> (Vec<Vec<f64>>, PyVocabulary, Vec<u64>, Vec<Vec<f64>>, Vec<u64>) {
        Default::default()
    }

    fn __getstate__(&self) -> HashMap<String, PyTopicModelStateValue> {
        let result = self.inner.get_py_state();
        return result.into_iter().map(|(k, value)| (k, value.into())).collect()
    }

    fn __setstate__(&mut self, state: HashMap<String, PyTopicModelStateValue>) -> PyResult<()> {
        let to_set = state.into_iter().map(|(k, v)| (k, v.into())).collect();
        self.inner = TopicModel::from_py_state(&to_set).map_err(|value| PyValueError::new_err(value.to_string()))?;
        Ok(())
    }

    #[staticmethod]
    fn builder(language: Option<LanguageHintValue>) -> PyTopicModelBuilder {
        PyTopicModelBuilder::new(language)
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

impl BasicTopicModel for PyTopicModel {
    delegate::delegate! {
        to self.inner {
            /// The number of topics in this model
            fn topic_count(&self) -> usize;

            /// The number of topics in this model
            #[inline]
            fn k(&self) -> usize;

            /// The size of the vocabulary for this model.
            fn vocabulary_size(&self) -> usize;

            /// A range over all topicIds
            fn topic_ids(&self) -> Range<TopicId>;

            /// Returns true if the `topic_id` is contained in self
            fn contains_topic_id(&self, topic_id: TopicId) -> bool;

            /// A range over all vocabulary ids
            fn word_ids(&self) -> Range<WordId>;

            /// Returns true if the `word_id` is contained in self
            fn contains_word_id(&self, word_id: WordId) -> bool;

            /// Returns the topics
            fn topics(&self) -> &TopicTo<WordTo<Probability>>;

            /// Get the topic for `topic_id`
            fn get_topic(&self, topic_id: TopicId) -> Option<&WordTo<Probability>>;

            /// The meta of the topic
            fn topic_metas(&self) -> &TopicTo<TopicMeta>;

            /// Get the `TopicMeta` for `topic_id`
            fn get_topic_meta(&self, topic_id: TopicId) -> Option<&TopicMeta>;

            /// Get the word freuencies for each word.
            fn used_vocab_frequency(&self) -> &WordTo<WordFrequency>;

            /// Get the probability of `word_id` of `topic_id`
            fn get_probability(&self, topic_id: TopicId, word_id: WordId) -> Option<&Probability>;

            /// Get all probabilities of `word_id`
            fn get_topic_probabilities_for(&self, word_id: WordId) -> Option<TopicTo<Probability>>;

            /// Get the [WordMeta] of `word_id` of `topic_id`
            fn get_word_meta(&self, topic_id: TopicId, word_id: WordId) -> Option<&Arc<WordMeta>>;

            /// Get all [WordMeta] for `word_id`
            fn get_word_metas_for(&self, word_id: WordId) -> Option<TopicTo<&Arc<WordMeta>>>;

            /// Get all [WordMeta] values with a similar importance in `topic_id` than `word_id`.
            /// (including the `word_id`)
            fn get_all_similar_important(&self, topic_id: TopicId, word_id: WordId) -> Option<&Vec<Arc<WordMeta>>>;

            fn get_words_for_topic_sorted(&self, topic_id: TopicId) -> Option<&[Arc<WordMeta>]>;

            /// Get the `n` best [WordMeta] in `topic_id` by their position.
            fn get_n_best_for_topic(&self, topic_id: TopicId, n: usize) -> Option<&[Arc<WordMeta>]>;

            /// Get the `n` best [WordMeta] for all topics by their position.
            fn get_n_best_for_topics(&self, n: usize) -> Option<TopicTo<&[Arc<WordMeta>]>>;
        }
    }
}

impl TopicModelWithDocumentStats for PyTopicModel {
    delegate::delegate! {
        to self.inner {
            /// Returns the number of documents
            fn document_count(&self) -> usize;

            /// Returns all document ids
            fn document_ids(&self) -> Range<DocumentId>;

            /// Returns the topic distributions of the topic model
            fn doc_topic_distributions(&self) -> &DocumentTo<TopicTo<Probability>>;

            /// Returns the document lengths of the documents
            fn document_lengths(&self) -> &DocumentTo<DocumentLength>;
        }
    }
}

impl BasicTopicModelWithVocabulary<String, PyVocabulary> for PyTopicModel {
    delegate::delegate! {
        to self.inner {
            fn vocabulary(&self) -> &PyVocabulary;
            fn get_word<'a>(&'a self, word_id: usize) -> Option<&'a HashRef<String>> where PyVocabulary: 'a;
            fn get_word_meta_with_word<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<WordMetaWithWord<'a, HashRef<String>>>  where PyVocabulary: 'a;
            fn get_word_metas_with_word<'a>(&'a self, word_id: usize) -> Option<TopicTo<WordMetaWithWord<'a, HashRef<String>>>> where PyVocabulary: 'a;
            fn get_all_similar_important_with_word_for<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<Vec<WordMetaWithWord<'a, HashRef<String>>>> where PyVocabulary: 'a;
        }
    }
}

impl TopicModelWithVocabulary<String, PyVocabulary> for PyTopicModel {
    delegate::delegate! {
        to self.inner {
            fn get_id<Q: ?Sized>(&self, word: &Q) -> Option<WordId> where String: Borrow<Q>, Q: Hash + Eq;
            fn contains<Q: ?Sized>(&self, word: &Q) -> bool where String: Borrow<Q>, Q: Hash + Eq;
            fn get_probability_by_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<&Probability> where String: Borrow<Q>, Q: Hash + Eq;
            fn get_topic_probabilities_for_by_word<Q: ?Sized>(&self, word: &Q) -> Option<TopicTo<Probability>> where String: Borrow<Q>, Q: Hash + Eq;
            fn get_word_meta_by_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<&Arc<WordMeta>> where String: Borrow<Q>, Q: Hash + Eq;
            fn get_word_metas_with_word_by_word<'a, Q: ?Sized>(&'a self, word: &Q) -> Option<TopicTo<WordMetaWithWord<'a, HashRef<String>>>> where String: Borrow<Q>, Q: Hash + Eq, PyVocabulary: 'a;
            fn get_all_similar_important_words_for_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<&Vec<Arc<WordMeta>>> where String: Borrow<Q>, Q: Hash + Eq;
            fn seems_equal_to<Q, VOther>(&self, other: &impl TopicModelWithVocabulary<Q, VOther>) -> bool where String: Borrow<Q>,Q: Hash + Eq + Borrow<String>, VOther: BasicVocabulary<Q>;
        }
    }
}

impl Display for PyTopicModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<TopicModel<String, PyVocabulary>> for PyTopicModel {
    fn from(inner: TopicModel<String, PyVocabulary>) -> Self {
        Self { inner }
    }
}

impl From<WriteError> for PyErr {
    fn from(err: WriteError) -> Self {
        PyValueError::new_err(err.to_string())
    }
}

impl From<ReadError<Infallible>> for PyErr {
    fn from(err: ReadError<Infallible>) -> Self {
        PyValueError::new_err(err.to_string())
    }
}


pub(crate) fn topic_model_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTopicModel>()?;
    Ok(())
}