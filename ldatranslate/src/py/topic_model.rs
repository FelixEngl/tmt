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

use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter, Read, Write};
use std::ops::Range;
use std::path::{PathBuf};
use itertools::Itertools;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ldatranslate_topicmodel::model::{DocumentLength, DocumentTo, PositionTo, Probability, TopicTo, WordFrequency, WordTo};
use crate::py::helpers::{LanguageHintValue};
use crate::py::topic_model_builder::PyTopicModelBuilder;
use crate::py::vocabulary::{PyVocabulary};
use ldatranslate_toolkit::partial_ord_iterator::PartialOrderIterator;
use ldatranslate_toolkit::{register_python};
use crate::py::aliases::{UnderlyingPyTopicModel, UnderlyingPyVocabulary, UnderlyingPyWord};
use ldatranslate_toolkit::special_python_values::{SingleOrVec};
use ldatranslate_topicmodel::language_hint::LanguageHint;
use ldatranslate_topicmodel::model::{BasicTopicModel, BasicTopicModelWithVocabulary, DocumentId, FullTopicModel, TopicId, TopicModel, TopicModelInferencer, TopicModelWithDocumentStats, TopicModelWithVocabulary, WordId};
use ldatranslate_topicmodel::model::meta::*;
use ldatranslate_topicmodel::vocabulary::{BasicVocabulary, Vocabulary, VocabularyMut};
use ldatranslate_translate::VoterInfoProvider;


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyTopicModel {
    inner: UnderlyingPyTopicModel
}

impl PyTopicModel {
    pub fn wrapped(&self) -> &UnderlyingPyTopicModel {
        &self.inner
    }
    pub fn wrap(inner: UnderlyingPyTopicModel) -> Self {
        Self{inner}
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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
                vocabulary.to_voc(),
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


    // fn save(&self, path: PathBuf) -> PyResult<usize> {
    //     Ok(self.inner.save(path, TopicModelVersion::V1, true, true)?)
    // }
    //
    // #[staticmethod]
    // fn load(path: PathBuf) -> PyResult<PyTopicModel> {
    //     Ok(Self { inner: TopicModel::<_, UnderlyingPyVocabulary>::load(path, true)?.0 })
    // }

    #[pyo3(signature = (n=None))]
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

    #[pyo3(signature = (doc, alpha, gamma_threshold, minimum_probability=None, minimum_phi_value=None, per_word_topics=None))]
    fn get_doc_probability(
        &self,
        doc: Vec<String>,
        alpha: SingleOrVec<f64>,
        gamma_threshold: f64,
        minimum_probability: Option<f64>,
        minimum_phi_value: Option<f64>,
        per_word_topics: Option<bool>
    ) -> (Vec<(usize, f64)>, Option<Vec<(usize, Vec<usize>)>>, Option<Vec<(usize, Vec<(usize, f64)>)>>) {
        TopicModelInferencer::<UnderlyingPyWord, UnderlyingPyVocabulary, Self>::new(
            &self,
            alpha,
            gamma_threshold
        ).get_doc_probability_for(
            doc.iter().map(|value| value.as_str()),
            minimum_probability.unwrap_or(TopicModelInferencer::<UnderlyingPyWord, UnderlyingPyVocabulary, PyTopicModel>::DEFAULT_MIN_PROBABILITY),
            minimum_phi_value.unwrap_or(TopicModelInferencer::<UnderlyingPyWord, UnderlyingPyVocabulary, PyTopicModel>::DEFAULT_MIN_PHI_VALUE),
            per_word_topics.unwrap_or_default()
        )
    }

    fn vocabulary(&self) -> PyVocabulary {
        self.inner.vocabulary().clone().into()
    }

    fn get_words_of_topic_sorted(&self, topic_id: usize) -> Option<Vec<(String, f64)>> {
        let probabilities = self.inner.topics().get(topic_id)?;
        self.get_words_for_topic_sorted(topic_id)
            .map(|value|
                value
                    .iter()
                    .map(|&word_id| {
                        let word = self.inner.vocabulary().get_value_by_id(word_id).unwrap().to_string();
                        let prob = unsafe{
                            *probabilities.get_unchecked(word_id)
                        };
                        (word, prob)
                    })
                    .collect_vec()
            )

    }

    fn get_topic_as_words(&self, topic_id: usize) -> Option<Vec<(usize, String, f64)>> {
        Some(
            self.inner.get_topic(topic_id)?.iter().enumerate().map(|(k, v)| {
                (k, self.inner.vocabulary().get_value_by_id(k).expect("This should not fail!").to_string(), *v)
            }).collect_vec()
        )
    }

    fn translate_by_provided_word_lists(&self, language_hint: LanguageHintValue, word_lists: SingleOrVec<Vec<String>>) -> PyResult<PyTopicModel> {
        if let Some(word_lists) = word_lists.as_vec() {
            if word_lists.len() != self.inner.topic_count() {
                return Err(PyValueError::new_err(format!("Expected {} lists, but got {}", self.inner.topic_count(), word_lists.len())))
            }
        }

        let min_value = self.inner.topics().iter().flatten().min_partial_filtered().unwrap().clone();

        let mut new_probability = Vec::new();

        let mut vocab_frequency: Vec<u64>;

        let voc = match word_lists {
            SingleOrVec::Single(value) => {
                let language_hint: LanguageHint = language_hint.into();
                let voc = Vocabulary::from((Some(language_hint), value.iter().map(UnderlyingPyWord::from).collect()));
                vocab_frequency = vec![0u64; voc.len()];
                for _ in 0..self.inner.topic_count() {
                    new_probability.push(vec![min_value; voc.len()]);
                }
                for topic_id in self.inner.topic_ids() {
                    let topic_old = self.inner.topics().get(topic_id).unwrap();
                    let topic_new = new_probability.get_mut(topic_id).unwrap();
                    for word_id in voc.ids() {
                        topic_new[word_id] = topic_old[word_id].clone();
                        vocab_frequency[word_id] = self.inner.used_vocab_frequency()[word_id];
                    }
                }
                voc
            }
            SingleOrVec::Vec(word_lists) => {
                let mut voc = Vocabulary::empty_for(language_hint.into());
                let word_lists = word_lists.into_iter().map(|values| {
                    values.into_iter().map(|value| {
                        voc.add::<UnderlyingPyWord>(value.into())
                    }).collect_vec()
                }).collect_vec();
                vocab_frequency = vec![0u64; voc.len()];
                for _ in 0..self.inner.topic_count() {
                    new_probability.push(vec![min_value; voc.len()]);
                }
                for (topic_id, word_ids) in word_lists.into_iter().enumerate() {
                    let topic_old = self.inner.topics().get(topic_id).unwrap();
                    let topic_new = new_probability.get_mut(topic_id).unwrap();
                    for (word_id_old, word_id_new) in word_ids.into_iter().enumerate() {
                        topic_new[word_id_new] = topic_old[word_id_old].clone();
                        vocab_frequency[word_id_new] = self.inner.used_vocab_frequency()[word_id_old];
                    }
                }
                voc
            }
        };

        let mut inner = TopicModel::new(
            new_probability,
            voc,
            vocab_frequency,
            self.doc_topic_distributions().clone(),
            self.document_lengths().clone()
        );

        inner.normalize_in_place();

        Ok(Self { inner })
    }


    fn save_json(&self, path: PathBuf) -> PyResult<()> {
        serde_json::to_writer(BufWriter::new(File::options().write(true).create_new(true).open(path)?), &self.inner).map_err(|value| PyValueError::new_err(value.to_string()))
    }


    #[staticmethod]
    fn load_json(path: PathBuf) -> PyResult<Self> {
        serde_json::from_reader(BufReader::new(File::options().read(true).open(path)?)).map_err(|value| PyValueError::new_err(value.to_string()))
    }

    fn save_binary(&self, path: PathBuf) -> PyResult<()> {
        self.save_to(
            File::options().write(true).create_new(true).open(path)?,
            true
        )?;
        Ok(())
    }

    #[staticmethod]
    fn load_binary(path: PathBuf) -> PyResult<Self> {
        Ok(
            Self::load_from(
                File::options().read(true).open(path)?,
                true
            )?
        )
    }


    fn normalize(&self) -> Self {
        self.inner.normalize().into()
    }

    #[pyo3(signature = (language=None))]
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

#[derive(Debug, Error)]
pub enum TopicModelSaveError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    BinarySerialisation(#[from] bincode::Error),
}

impl From<TopicModelSaveError> for PyErr {
    fn from(value: TopicModelSaveError) -> Self {
        match value {
            TopicModelSaveError::IO(value) => {
                value.into()
            }
            TopicModelSaveError::BinarySerialisation(value) => {
                PyRuntimeError::new_err(value.to_string())
            }
        }
    }
}

impl PyTopicModel {
    pub fn save_to<W: Write>(&self, writer: W, compressed: bool) -> Result<(), TopicModelSaveError> {
        let mut writer: Box<dyn Write> = if compressed {
            Box::new(zstd::Encoder::new(writer, 6)?)
        } else {
            Box::new(BufWriter::new(writer))
        };

        bincode::serialize_into(&mut writer, &self.inner)?;
        writer.flush()?;
        Ok(())
    }

    pub fn load_from<R: Read>(reader: R, compressed: bool) -> Result<Self, TopicModelSaveError> {
        let reader: Box<dyn Read> = if compressed {
            Box::new(zstd::Decoder::new(reader)?)
        } else {
            Box::new(BufReader::with_capacity(
                byte_unit::Byte::from_u64_with_unit(
                    250,
                    byte_unit::Unit::MB
                ).unwrap().as_u64() as usize,
                reader
            ))
        };
        Ok(bincode::deserialize_from(reader)?)
    }
}


impl BasicTopicModel for PyTopicModel {

    type TopicMeta<'a> = <UnderlyingPyTopicModel as BasicTopicModel>::TopicMeta<'a>;
    type TopicMetas<'a> = <UnderlyingPyTopicModel as BasicTopicModel>::TopicMetas<'a>;
    type WordMeta<'a> = <UnderlyingPyTopicModel as BasicTopicModel>::WordMeta<'a>;

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
            fn topic_metas(&self) -> Self::TopicMetas<'_>;

            /// Get the `TopicMeta` for `topic_id`
            fn get_topic_meta(&self, topic_id: TopicId) -> Option<Self::TopicMeta<'_>>;

            /// Get the [WordMeta] of `word_id` of `topic_id`
            fn get_word_meta(&self, topic_id: TopicId, word_id: WordId) -> Option<Self::WordMeta<'_>>;

            /// Get the word freuencies for each word.
            fn used_vocab_frequency(&self) -> &WordTo<WordFrequency>;

            /// Get the probability of `word_id` of `topic_id`
            fn get_probability(&self, topic_id: TopicId, word_id: WordId) -> Option<&Probability>;

            /// Get all probabilities of `word_id`
            fn get_topic_probabilities_for(&self, word_id: WordId) -> Option<TopicTo<Probability>>;

            /// Get all [WordMeta] values with a similar importance in `topic_id` than `word_id`.
            /// (including the `word_id`)
            fn get_all_similar_important(&self, topic_id: TopicId, word_id: WordId) -> Option<Vec<Self::WordMeta<'_>>>;

            /// Get the word ids sorted by position.

            fn get_words_for_topic_sorted(&self, topic_id: TopicId) -> Option<PositionTo<WordId>>;

            /// Get the `n` best [WordMeta] in `topic_id` by their position.
            fn get_n_best_for_topic(&self, topic_id: TopicId, n: usize) -> Option<PositionTo<WordId>>;

            /// Get the `n` best [WordMeta] for all topics by their position.
            fn get_n_best_for_topics(&self, n: usize) -> TopicTo<Vec<WordId>>;
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

impl BasicTopicModelWithVocabulary<UnderlyingPyWord, UnderlyingPyVocabulary> for PyTopicModel {
    delegate::delegate! {
        to self.inner {
            fn vocabulary(&self) -> &UnderlyingPyVocabulary;
            fn get_word<'a>(&'a self, word_id: usize) -> Option<&'a UnderlyingPyWord> where UnderlyingPyVocabulary: 'a;
            fn get_word_meta_with_word<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<WordMetaWithWord<'a, UnderlyingPyWord, <Self as BasicTopicModel>::WordMeta<'a>>>  where UnderlyingPyVocabulary: 'a;
            fn get_word_metas_with_word<'a>(&'a self, word_id: usize) -> Option<TopicTo<WordMetaWithWord<'a, UnderlyingPyWord, <Self as BasicTopicModel>::WordMeta<'a>>>> where UnderlyingPyVocabulary: 'a;
            fn get_all_similar_important_with_word_for<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<Vec<WordMetaWithWord<'a, UnderlyingPyWord, <Self as BasicTopicModel>::WordMeta<'a>>>> where UnderlyingPyVocabulary: 'a;
        }
    }
}

impl TopicModelWithVocabulary<UnderlyingPyWord, UnderlyingPyVocabulary> for PyTopicModel {
    delegate::delegate! {
        to self.inner {
            fn get_id<Q: ?Sized>(&self, word: &Q) -> Option<WordId> where UnderlyingPyWord: Borrow<Q>, Q: Hash + Eq;
            fn contains<Q: ?Sized>(&self, word: &Q) -> bool where UnderlyingPyWord: Borrow<Q>, Q: Hash + Eq;

            /// Get the probability of `word` of `topic_id`
            fn get_probability_by_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<&Probability> where UnderlyingPyWord: Borrow<Q>, Q: Hash + Eq;

            /// Get all probabilities of `word`
            fn get_topic_probabilities_for_by_word<Q: ?Sized>(&self, word: &Q) -> Option<TopicTo<Probability>> where UnderlyingPyWord: Borrow<Q>, Q: Hash + Eq;

            /// Get the [WordMeta] of `word` of `topic_id`
            fn get_word_meta_by_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<Self::WordMeta<'_>> where UnderlyingPyWord: Borrow<Q>, Q: Hash + Eq;

            /// Get the [WordMetaWithWord] of `word` for all topics.
            fn get_word_metas_with_word_by_word<'a, 'b, Q: ?Sized>(&'a self, word: &'b Q) -> Option<TopicTo<WordMetaWithWord<'a, UnderlyingPyWord, <Self as BasicTopicModel>::WordMeta<'a>>>> where UnderlyingPyWord: Borrow<Q>, Q: Hash + Eq, UnderlyingPyVocabulary: 'a;

            /// Get all [WordMeta] values with a similar importance in `topic_id` than `word`.
            /// (including the `word_id`)
            fn get_all_similar_important_words_for_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<Vec<<Self as BasicTopicModel>::WordMeta<'_>>> where UnderlyingPyWord: Borrow<Q>, Q: Hash + Eq;

            /// Returns true iff the topic models seem similar.
            fn seems_equal_to<Q, VOther>(&self, other: &impl TopicModelWithVocabulary<Q, VOther>) -> bool where UnderlyingPyWord: Borrow<Q>, Q: Hash + Eq + Borrow<UnderlyingPyWord>, VOther: BasicVocabulary<Q>;
        }
    }
}

impl Display for PyTopicModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<UnderlyingPyTopicModel> for PyTopicModel {
    fn from(inner: UnderlyingPyTopicModel) -> Self {
        Self { inner }
    }
}

impl VoterInfoProvider for PyTopicModel {
    type VoterMeta<'a> = <UnderlyingPyTopicModel as VoterInfoProvider>::VoterMeta<'a>;

    delegate::delegate! {
        to self.inner {
            fn get_voter_meta(&self, column: usize, voter_id: usize) -> Option<Self::VoterMeta<'_>>;
        }
    }
}

impl FullTopicModel<UnderlyingPyWord, UnderlyingPyVocabulary> for PyTopicModel {
    fn new(topics: TopicTo<WordTo<Probability>>, vocabulary: UnderlyingPyVocabulary, used_vocab_frequency: WordTo<WordFrequency>, doc_topic_distributions: DocumentTo<TopicTo<Probability>>, document_lengths: DocumentTo<DocumentLength>) -> Self
    where
        Self: Sized
    {
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

    fn normalize_in_place(&mut self) {
        self.inner.normalize_in_place()
    }
}




register_python!(struct PyTopicModel;);

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use arcstr::ArcStr;
    use crate::py::helpers::LanguageHintValue;
    use crate::py::topic_model::{PyTopicModel};
    use ldatranslate_topicmodel::model::{FullTopicModel, TopicModel, TopicModelWithVocabulary};
    use ldatranslate_topicmodel::vocabulary::MappableVocabulary;
    use crate::translate::test::create_test_data;

    #[test]
    fn special_translate_works(){
        let (voc_a, _, _) = create_test_data();
        let model_a = TopicModel::new(
            vec![
                vec![0.019, 0.018, 0.012, 0.009, 0.008, 0.008, 0.008, 0.008, 0.008, 0.008, 0.008],
                vec![0.02, 0.002, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001],
            ],
            voc_a.map(ArcStr::from),
            vec![10, 5, 8, 1, 2, 3, 1, 1, 1, 1, 2],
            vec![
                vec![0.7, 0.2],
                vec![0.8, 0.3]
            ],
            vec![
                200,
                300
            ]
        );
        let model = PyTopicModel::wrap(model_a);
        let tranlation = model.translate_by_provided_word_lists(
            LanguageHintValue::Value("LA".to_string()),
            vec![
                vec![
                    "a".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                    "d".to_string(),
                    "e".to_string(),
                    "f".to_string(),
                    "g".to_string(),
                    "h".to_string(),
                    "i".to_string(),
                    "j".to_string(),
                    "k".to_string(),
                ],
                vec![
                    "xxx".to_string(),
                    "b".to_string(),
                    "yyy".to_string(),
                    "d".to_string(),
                    "e".to_string(),
                    "f".to_string(),
                    "zzz".to_string(),
                    "h".to_string(),
                    "i".to_string(),
                    "j".to_string(),
                    "k".to_string(),
                ]
            ].into()
        ).unwrap();

        let mut vec = Vec::new();
        tranlation.save_to(
            &mut vec,
            true
        ).expect("save failed");

        let translation2 = PyTopicModel::load_from(
            Cursor::new(&vec),
            true
        ).expect("load failed");

        assert!(tranlation.inner.seems_equal_to(&translation2.inner));

        tranlation.show_top(Some(20)).unwrap()


    }
}