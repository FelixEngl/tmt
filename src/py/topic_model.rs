use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::Range;
use std::sync::Arc;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use topic_model::{DocumentLength, DocumentTo, Probability, TopicTo, WordFrequency, WordTo};
use topicmodel::topic_model;
use crate::py::vocabulary::PyVocabulary;
use crate::topicmodel;
use crate::topicmodel::enums::{ReadError, TopicModelVersion, WriteError};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::topic_model::{BasicTopicModel, BasicTopicModelWithVocabulary, DocumentId, TopicId, TopicMeta, TopicModel, TopicModelInferencer, TopicModelWithDocumentStats, TopicModelWithVocabulary, WordId, WordMeta, WordMetaWithWord};
use crate::topicmodel::vocabulary::{Vocabulary};

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyTopicModel {
    inner: TopicModel<String, PyVocabulary>
}

impl PyTopicModel {
    pub fn wrap(inner: TopicModel<String, PyVocabulary>) -> Self {
        Self{inner}
    }
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

    pub fn save(&self, path: &str) -> PyResult<usize> {
        Ok(self.inner.save(path, TopicModelVersion::V1, true, true)?)
    }

    #[staticmethod]
    pub fn load(path: &str) -> PyResult<PyTopicModel> {
        Ok(Self { inner: TopicModel::<_,PyVocabulary>::load(path, true)?.0 })
    }

    pub fn __repr__(&self) -> String {
        format!("PyTopicModel({:?})", self.inner)
    }

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    pub fn get_doc_probability(
        &self,
        doc: Vec<String>,
        alpha: f64,
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

    pub fn vocabulary(&self) -> PyVocabulary {
        self.inner.vocabulary().clone()
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
            fn seems_equal_to<Q, VOther>(&self, other: &impl TopicModelWithVocabulary<Q, VOther>) -> bool where String: Borrow<Q>,Q: Hash + Eq + Borrow<String>, VOther: Vocabulary<Q>;
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