use itertools::repeat_n;
use once_cell::sync::OnceCell;
use pyo3::exceptions::PyValueError;
use pyo3::{pyclass, pymethods, PyResult};
use crate::py::helpers::LanguageHintValue;
use crate::py::topic_model::PyTopicModel;
use crate::py::vocabulary::PyVocabulary;
use crate::toolkit::partial_ord_iterator::PartialOrderIterator;
use crate::topicmodel::topic_model::{DocumentLength, DocumentTo, Probability, TopicModel, TopicTo, WordFrequency, WordTo};
use crate::topicmodel::vocabulary::{BasicVocabulary, Vocabulary, VocabularyMut};

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyTopicModelBuilder {
    voc: Vocabulary<String>,
    topics: Vec<Vec<f64>>,
    used_vocab_frequency: OnceCell<WordTo<WordFrequency>>,
    doc_topic_distributions: Option<DocumentTo<TopicTo<Probability>>>,
    document_lengths: Option<DocumentTo<DocumentLength>>,
}

impl PyTopicModelBuilder {
    fn set_probability(&mut self, topic_id: usize, word_id: usize, probability: f64) {
        if let Some(found) = self.topics.get_mut(topic_id) {
            if let Some(found) = found.get_mut(word_id) {
                *found = probability;
            } else {
                let mut result = vec![f64::NAN; self.voc.len()];
                result.insert(word_id, probability);
            }
        } else {
            while self.topics.len() <= topic_id {
                self.topics.push(vec![f64::NAN; self.voc.len()]);
            }
            unsafe {
                self.topics.get_unchecked_mut(topic_id).insert(word_id, probability);
            }
        }
    }

    fn set_frequency(&mut self, word_id: usize, frequency: u64) {
        if let Some(target) = self.used_vocab_frequency.get_mut() {
            if target.len() <= word_id {
                let to_add = word_id - target.len() + 1;
                target.reserve(to_add);
                target.extend(repeat_n(0, to_add));
            }
            target[word_id] = frequency;
        } else {
            let mut value = vec![0u64; self.voc.len()];
            value[word_id] = frequency;
            self.used_vocab_frequency.set(value).expect("This shouldn't be initialized right now!")
        }
    }
}

#[pymethods]
impl PyTopicModelBuilder {
    #[new]
    pub fn new(language: Option<LanguageHintValue>) -> Self {
        Self {
            voc: Vocabulary::new(language.map(Into::into)),
            topics: Default::default(),
            used_vocab_frequency: Default::default(),
            doc_topic_distributions: Default::default(),
            document_lengths: Default::default(),
        }
    }

    pub fn set_frequency(&mut self, word: String, frequency: u64) {
        self.set_frequency(self.voc.add_value(word), frequency);
    }

    pub fn add_word(&mut self, topic_id: usize, word: String, probability: f64, frequency: Option<u64>) -> PyResult<()> {
        if !probability.is_normal() {
            return Err(PyValueError::new_err("The probability has to be a normal number!"))
        }
        let word_id = self.voc.add_value(word);
        self.set_probability(topic_id, word_id, probability);
        if let Some(frequency) = frequency {
            self.set_frequency(topic_id, frequency);
        }
        Ok(())
    }


    pub fn set_doc_topic_distributions(&mut self, doc_topic_distributions: Option<DocumentTo<TopicTo<Probability>>>) {
        self.doc_topic_distributions = doc_topic_distributions;
    }
    pub fn set_document_lengths(&mut self, document_lengths: Option<DocumentTo<DocumentLength>>) {
        self.document_lengths = document_lengths;
    }

    pub fn build(&self) -> PyResult<PyTopicModel> {
        let mut topics = self.topics.clone();

        let iter = topics
            .iter()
            .flatten()
            .cloned();

        if iter.clone().any(|value| value.is_nan()) {
            let min_value = iter
                .min_partial()
                .map_err(|err| PyValueError::new_err(err.to_string()))?
                .ok_or_else(|| PyValueError::new_err("There has to be some probability given!"))? - f64::EPSILON;

            for value in topics.iter_mut().flatten() {
                if value.is_nan() {
                    *value = min_value
                }
            }
        }

        let mut model = TopicModel::new(
            topics,
            PyVocabulary::from(self.voc.clone()),
            self.used_vocab_frequency.get().cloned().unwrap_or_default(),
            self.doc_topic_distributions.clone().unwrap_or_default(),
            self.document_lengths.clone().unwrap_or_default(),
        );

        model.normalize_in_place();

        Ok(PyTopicModel::wrap(model))
    }
}