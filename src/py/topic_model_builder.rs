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

use itertools::repeat_n;
use std::sync::OnceLock;
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
    used_vocab_frequency: OnceLock<WordTo<WordFrequency>>,
    doc_topic_distributions: Option<DocumentTo<TopicTo<Probability>>>,
    document_lengths: Option<DocumentTo<DocumentLength>>,
}

impl PyTopicModelBuilder {
    fn set_probability_impl(&mut self, topic_id: usize, word_id: usize, probability: f64) {
        if let Some(found) = self.topics.get_mut(topic_id) {
            if let Some(found) = found.get_mut(word_id) {
                *found = probability;
            } else {
                let target_len = self.voc.len() - found.len();
                found.reserve(target_len);
                found.extend(repeat_n(f64::NAN, target_len));
                found.insert(word_id, probability);
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

    fn set_frequency_impl(&mut self, word_id: usize, frequency: u64) {
        if let Some(target) = self.used_vocab_frequency.get_mut() {
            if target.len() <= word_id {
                let to_add = word_id - target.len();
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
    #[pyo3(signature = (language=None))]
    pub fn new(language: Option<LanguageHintValue>) -> Self {
        Self {
            voc: Vocabulary::new(language.map(Into::into)),
            topics: Default::default(),
            used_vocab_frequency: Default::default(),
            doc_topic_distributions: Default::default(),
            document_lengths: Default::default(),
        }
    }

    fn set_frequency(&mut self, word: String, frequency: u64) {
        let word_id = self.voc.add_value(word);
        self.set_frequency_impl(word_id, frequency);
    }

    #[pyo3(signature = (topic_id, word, probability, frequency=None))]
    fn add_word(&mut self, topic_id: usize, word: String, probability: f64, frequency: Option<u64>) -> PyResult<()> {
        if !probability.is_normal() {
            return Err(PyValueError::new_err("The probability has to be a normal number!"))
        }
        let word_id = self.voc.add_value(word);
        self.set_probability_impl(topic_id, word_id, probability);
        if let Some(frequency) = frequency {
            self.set_frequency_impl(topic_id, frequency);
        }
        Ok(())
    }

    #[pyo3(signature = (doc_topic_distributions=None))]
    fn set_doc_topic_distributions(&mut self, doc_topic_distributions: Option<DocumentTo<TopicTo<Probability>>>) {
        self.doc_topic_distributions = doc_topic_distributions;
    }

    #[pyo3(signature = (document_lengths=None))]
    fn set_document_lengths(&mut self, document_lengths: Option<DocumentTo<DocumentLength>>) {
        self.document_lengths = document_lengths;
    }

    #[pyo3(signature = (unset_words_become_smallest=None, normalize=None))]
    fn build(
        &self,
        unset_words_become_smallest: Option<bool>,
        normalize: Option<bool>
    ) -> PyResult<PyTopicModel> {
        let mut topics = self.topics.clone();

        if topics.iter().flatten().any(|value| value.is_nan()) {
            let min_value = topics
                .iter()
                .flatten()
                .cloned()
                .min_partial_filtered()
                .ok_or_else(|| PyValueError::new_err("There has to be some probability given!"))?;

            let min_value = if unset_words_become_smallest.unwrap_or_default() {
                min_value - f64::EPSILON
            } else {
                min_value
            };

            for topic in topics.iter_mut() {
                for value in topic.iter_mut() {
                    if value.is_nan() {
                        *value = min_value
                    }
                }
                if self.voc.len() > topic.len() {
                    let target_len = self.voc.len() - topic.len();
                    topic.reserve(target_len);
                    topic.extend(repeat_n(min_value, target_len))
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

        if normalize.unwrap_or_default() {
            model.normalize_in_place()
        }

        Ok(PyTopicModel::wrap(model))
    }
}

#[cfg(test)]
mod test {
    use crate::py::topic_model_builder::PyTopicModelBuilder;
    use crate::topicmodel::topic_model::DisplayableTopicModel;

    #[test]
    fn can_create(){
        let mut builder = PyTopicModelBuilder::new(None);
        builder.add_word(0, "hello".to_string(), 1.0, None).unwrap();
        builder.add_word(1, "hello".to_string(), 2.0, None).unwrap();
        builder.add_word(0, "beer".to_string(), 2.0, None).unwrap();
        builder.add_word(0, "vat".to_string(), 2.0, None).unwrap();
        builder.add_word(1, "cat".to_string(), 2.0, None).unwrap();
        builder.add_word(1, "beer".to_string(), 1.5, None).unwrap();
        builder.add_word(1, "leech".to_string(), 0.9, None).unwrap();
        let topic = builder.build(None, None).expect("ERR");
        topic.show_10().unwrap();
    }
}