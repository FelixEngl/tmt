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

#![allow(dead_code)]

pub mod meta;
mod traits;
mod classic_serialisation;
mod inferencer;

pub use classic_serialisation::*;
pub use inferencer::*;
pub use traits::*;

use approx::relative_eq;
use std::borrow::Borrow;
use std::cmp::{min, Ordering, Reverse};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io;
use std::io::{BufRead, ErrorKind, Read, Write};
use std::marker::PhantomData;
use std::ops::{DerefMut, Range};
use std::str::FromStr;
use std::sync::Arc;

use crate::toolkit::normal_number::IsNormalNumber;
use crate::topicmodel::model::meta::*;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::traits::ToParseableString;
use crate::topicmodel::vocabulary::{BasicVocabulary, LoadableVocabulary, MappableVocabulary, StoreableVocabulary, VocabularyMut};
use itertools::Itertools;
use rand_distr::Distribution;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};


pub(crate) type TopicTo<T> = Vec<T>;
pub(crate) type WordTo<T> = Vec<T>;
pub(crate) type PositionTo<T> = Vec<T>;
pub(crate) type DocumentTo<T> = Vec<T>;
pub(crate) type ImportanceRankTo<T> = Vec<T>;
pub(crate) type Probability = f64;

/// The direct rank, created by the order of the probabilities and then
pub(crate) type Rank = usize;

/// The rank, when grouping the topic by probabilities
pub(crate) type ImportanceRank = usize;
pub(crate) type WordId = usize;
pub(crate) type TopicId = usize;
pub(crate) type Position = usize;
pub(crate) type Importance = usize;
pub(crate) type DocumentId = usize;
pub(crate) type WordFrequency = u64;
pub(crate) type DocumentLength = u64;

/// A topic model
#[derive(Debug, Serialize, Deserialize)]
pub struct TopicModel<T, V> {
    // topic to word
    topics: TopicTo<WordTo<Probability>>,
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    vocabulary: V,
    used_vocab_frequency: WordTo<WordFrequency>,
    doc_topic_distributions: DocumentTo<TopicTo<Probability>>,
    document_lengths: DocumentTo<DocumentLength>,
    topic_metas: TopicTo<TopicMeta>,
    #[serde(skip)]
    _word_type: PhantomData<T>
}

unsafe impl<T, V> Send for TopicModel<T, V>{}
unsafe impl<T, V> Sync for TopicModel<T, V>{}

impl<T, V> Clone for TopicModel<T, V> where V: Clone {
    fn clone(&self) -> Self {
        Self {
            topics: self.topics.clone(),
            vocabulary: self.vocabulary.clone(),
            used_vocab_frequency: self.used_vocab_frequency.clone(),
            doc_topic_distributions: self.doc_topic_distributions.clone(),
            document_lengths: self.document_lengths.clone(),
            topic_metas: self.topic_metas.clone(),
            _word_type: PhantomData
        }
    }
}


impl<T, V> TopicModel<T, V> where
    T: Hash + Eq + Ord,
    V: VocabularyMut<T>
{
    pub fn new(
        topics: TopicTo<WordTo<Probability>>,
        vocabulary: V,
        used_vocab_frequency: WordTo<WordFrequency>,
        doc_topic_distributions: DocumentTo<TopicTo<Probability>>,
        document_lengths: DocumentTo<DocumentLength>,
    ) -> Self {
        let topic_content = unsafe {
            Self::calculate_topic_metas(&topics, &vocabulary)
        };

        Self {
            topics,
            vocabulary,
            used_vocab_frequency,
            doc_topic_distributions,
            document_lengths,
            topic_metas: topic_content,
            _word_type: PhantomData
        }
    }

    unsafe fn calculate_topic_metas(topics: &TopicTo<WordTo<Probability>>, vocabulary: &impl BasicVocabulary<T>) -> TopicTo<TopicMeta> {
        struct SortHelper<'a, Q, V> where V: BasicVocabulary<Q> {
            word_id: WordId,
            probability: Probability,
            vocabulary: &'a V,
            _word_type: PhantomData<Q>,
        }

        impl<'a, Q, V> SortHelper<'a, Q, V> where Q: Hash + Eq, V: BasicVocabulary<Q> {
            fn word(&self) -> &HashRef<Q> {
                self.vocabulary.get_value(self.word_id).expect("There should be no problem with enpacking it here!")
            }
        }

        impl<Q, V> Eq for SortHelper<'_, Q, V> where V: BasicVocabulary<Q> {}

        impl<Q, V> PartialEq<Self> for SortHelper<'_, Q, V> where V: BasicVocabulary<Q> {
            fn eq(&self, other: &Self) -> bool {
                self.probability.eq(&other.probability)
            }
        }

        impl<Q, V> PartialOrd for SortHelper<'_, Q, V> where Q: Hash + Eq + Ord, V: BasicVocabulary<Q> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                match self.probability.partial_cmp(&other.probability) {
                    None => {
                        if self.probability.is_normal_number() {
                            Some(Ordering::Greater)
                        } else if other.probability.is_normal_number() {
                            Some(Ordering::Less)
                        } else {
                            Some(
                                other.vocabulary.get_value(other.word_id).unwrap().cmp(
                                    self.vocabulary.get_value(self.word_id).unwrap()
                                )
                            )
                        }
                    }
                    Some(Ordering::Equal) => {
                        Some(
                            other.vocabulary.get_value(other.word_id).unwrap().cmp(
                                self.vocabulary.get_value(self.word_id).unwrap()
                            )
                        )
                    }
                    otherwise => otherwise
                }
            }
        }

        impl<Q, V> Ord for SortHelper<'_, Q, V> where Q: Hash + Eq + Ord, V: BasicVocabulary<Q> {
            fn cmp(&self, other: &Self) -> Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

        topics.par_iter().enumerate().map(|(topic_id, topic)| {
            let position_to_word_id_and_prob = topic
                .iter()
                .copied()
                .enumerate()
                .sorted_by_key(|(word_id, prob)| Reverse(SortHelper {
                    word_id: *word_id,
                    probability: *prob,
                    vocabulary: vocabulary,
                    _word_type: PhantomData
                }))
                .collect_vec();

            let mut current_value = position_to_word_id_and_prob.first().unwrap().1;
            let mut current_sink = Vec::new();
            let mut importance_to_word_ids: Vec<Vec<WordId>> = Vec::new();

            for (word_id, probability) in &position_to_word_id_and_prob {
                if current_value.ne(probability) {
                    importance_to_word_ids.push(current_sink);
                    current_sink = Vec::new();
                    current_value = *probability;
                }
                current_sink.push(*word_id);
            }

            if !current_sink.is_empty() {
                importance_to_word_ids.push(current_sink);
            }

            let mut word_id_to_importance: Vec<_> = importance_to_word_ids
                .into_iter()
                .enumerate()
                .flat_map(|(importance, words)| {
                    words.into_iter().map(move |value| (importance, value))
                }).collect_vec();
            word_id_to_importance.sort_by_key(|value| value.1);

            let word_id_to_position: Vec<_> = position_to_word_id_and_prob
                .into_iter()
                .enumerate()
                .map(|(position, (word_id, prob))| (word_id, prob, position))
                .sorted_by_key(|(word_id, _, _)| *word_id)
                .collect_vec();

            let mut topic_content = word_id_to_position.into_iter().zip_eq(word_id_to_importance.into_iter()).map(|((word_id_1, prob, position), (importance, word_id_2))| {
                assert_eq!(word_id_1, word_id_2, "Word ids {} {} are not compatible!", word_id_1, word_id_2);
                (word_id_1, prob, position, importance)
            }).zip_eq(topic.into_iter().enumerate()).map(|((word_id_1, probability_1, position, importance), (word_id, probability))| {
                assert_eq!(word_id, word_id_1, "Word ids {} {} are not compatible in zipping!", word_id, word_id_1);
                assert_eq!(*probability, probability_1,
                           "Probabilities fir the ids {}({}) {}({}) are not compatible in zipping!",
                           word_id, probability,
                           word_id_1, probability_1);
                Arc::new(
                    WordMeta {
                        topic_id,
                        word_id,
                        probability: probability_1,
                        position,
                        importance,
                    }
                )
            }).collect_vec();
            topic_content.shrink_to_fit();
            (topic_id, topic_content)
        }).map(|(topic_id, topic_content)| {
            let position_to_meta: PositionTo<_> = topic_content.iter().sorted_by_key(|value| value.position).cloned().collect_vec();

            let mut importance_to_meta: ImportanceRankTo<_> = Vec::new();

            for value in position_to_meta.iter() {
                while importance_to_meta.len() <= value.importance {
                    importance_to_meta.push(Vec::new())
                }
                importance_to_meta.get_unchecked_mut(value.importance).push(value.clone());
            }

            let mut max_value: f64 = f64::MIN;
            let mut min_value: f64 = f64::MAX;
            let mut sum_value: f64 = 0.0;

            for value in &topic_content {
                max_value = max_value.max(value.probability);
                min_value = min_value.min(value.probability);
                sum_value += value.probability;
            }


            let stats = TopicStats {
                topic_id,
                max_value,
                min_value,
                sum_value,
                average_value: sum_value / (topic_content.len() as f64)
            };

            TopicMeta::new(stats, topic_content, position_to_meta, importance_to_meta)
        }).collect()
    }

    fn recalculate_statistics(&mut self) {
        self.topic_metas = unsafe {
            Self::calculate_topic_metas(&self.topics, &self.vocabulary)
        };
    }

    pub fn normalize_in_place(&mut self) {
        for topic in self.topics.iter_mut() {
            let sum: f64 = topic.iter().sum();
            topic.iter_mut().for_each(|value| {
                *value /= sum
            });
        }

        for probabilities in self.doc_topic_distributions.iter_mut() {
            let sum: f64 = probabilities.iter().sum();
            probabilities.iter_mut().for_each(|value| {
                *value /= sum
            });
        }

        self.recalculate_statistics();
    }
}

impl<T, V> TopicModel<T, V> where T: Hash + Eq + Ord, V: Clone + VocabularyMut<T> {
    pub fn normalize(&self) -> Self {
        let mut target = self.clone();
        target.normalize_in_place();
        target
    }
}

impl<T, V> TopicModel<T, V> {
    fn calculate_topic_stats(topics: &Vec<Vec<f64>>) -> Vec<TopicStats> {
        topics.iter().enumerate().map(|(topic_id, topic)| {
            let mut max_value: f64 = f64::MIN;
            let mut min_value: f64 = f64::MAX;
            let mut sum_value: f64 = 0.0;

            for &value in topic {
                max_value = max_value.max(value);
                min_value = min_value.min(value);
                sum_value += value;
            }


            TopicStats {
                topic_id,
                max_value,
                min_value,
                sum_value,
                average_value: sum_value / (topic.len() as f64)
            }
        }).collect()
    }
}

impl<T, V> BasicTopicModel for TopicModel<T, V> where V: BasicVocabulary<T> {
    fn topic_count(&self) -> usize {
        self.topics.len()
    }

    fn vocabulary_size(&self) -> usize {
        self.vocabulary.len()
    }

    fn topic_ids(&self) -> Range<usize> {
        0..self.topics.len()
    }

    fn contains_topic_id(&self, topic_id: TopicId) -> bool {
        topic_id < self.topics.len()
    }

    fn word_ids(&self) -> Range<WordId> {
        self.vocabulary.ids()
    }

    fn contains_word_id(&self, word_id: WordId) -> bool {
        self.vocabulary.contains_id(word_id)
    }

    fn topics(&self) -> &TopicTo<WordTo<Probability>> {
        &self.topics
    }

    fn get_topic(&self, topic_id: usize) -> Option<&WordTo<f64>> {
        self.topics.get(topic_id)
    }

    fn topic_metas(&self) -> &Vec<TopicMeta> {
        &self.topic_metas
    }

    fn get_topic_meta(&self, topic_id: usize) -> Option<&TopicMeta> {
        self.topic_metas.get(topic_id)
    }

    fn used_vocab_frequency(&self) -> &WordTo<WordFrequency> {
        &self.used_vocab_frequency
    }


    fn get_probability(&self, topic_id: TopicId, word_id: WordId) -> Option<&Probability> {
        self.topics.get(topic_id)?.get(word_id)
    }

    fn get_topic_probabilities_for(&self, word_id: WordId) -> Option<TopicTo<Probability>> {
        if self.contains_word_id(word_id) {
            Some(self.topics.iter().map(|value| unsafe{value.get_unchecked(word_id).clone()}).collect())
        } else {
            None
        }
    }

    fn get_word_meta(&self, topic_id: TopicId, word_id: WordId) -> Option<&Arc<WordMeta>> {
        self.topic_metas.get(topic_id)?.by_words.get(word_id)
    }

    fn get_word_metas_for(&self, word_id: WordId) -> Option<TopicTo<&Arc<WordMeta>>> {
        if self.contains_word_id(word_id) {
            Some(self.topic_metas.iter().map(|value| unsafe{value.by_words.get_unchecked(word_id)}).collect())
        } else {
            None
        }
    }

    fn get_all_similar_important(&self, topic_id: usize, word_id: usize) -> Option<&Vec<Arc<WordMeta>>> {
        let topic = self.topic_metas.get(topic_id)?;
        topic.by_importance.get(topic.by_words.get(word_id)?.importance)
    }

    fn get_words_for_topic_sorted(&self, topic_id: TopicId) -> Option<&[Arc<WordMeta>]> {
        let metas = self.topic_metas.get(topic_id)?;
        Some(&metas.by_position)
    }


    fn get_n_best_for_topic(&self, topic_id: usize, n: usize) -> Option<&[Arc<WordMeta>]> {
        let metas = self.topic_metas.get(topic_id)?;
        Some(&metas.by_position[..min(n, metas.by_position.len())])
    }

    fn get_n_best_for_topics(&self, n: usize) -> Option<Vec<&[Arc<WordMeta>]>> {
        self.topic_ids().map(|topic_id| self.get_n_best_for_topic(topic_id, n)).collect()
    }
}

impl<T, V> BasicTopicModelWithVocabulary<T, V> for TopicModel<T, V> where V: BasicVocabulary<T> {
    fn vocabulary(&self) -> &V {
        &self.vocabulary
    }

    fn get_word_meta_with_word<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<WordMetaWithWord<'a, HashRef<T>>>  where V: 'a  {
        let topic_meta = self.get_topic_meta(topic_id)?;
        let word_meta = topic_meta.by_words.get(word_id)?;
        let word = self.vocabulary.get_value(word_meta.word_id)?;
        Some(WordMetaWithWord::new(word, word_meta))
    }

    fn get_all_similar_important_with_word_for<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<Vec<WordMetaWithWord<'a, HashRef<T>>>> where V: 'a {
        Some(
            self.get_all_similar_important(topic_id, word_id)?
                .iter()
                .map(|value| WordMetaWithWord::new(self.vocabulary.get_value(value.word_id).unwrap(), value))
                .collect()
        )
    }
}

impl<T, V> TopicModelWithVocabulary<T, V> for TopicModel<T, V> where T: Hash + Eq, V: VocabularyMut<T> {
    delegate::delegate! {
        to self.vocabulary {
            fn get_id<Q: ?Sized>(&self, word: &Q) -> Option<usize> where T: Borrow<Q>, Q: Hash + Eq;
            fn contains<Q: ?Sized>(&self, word: &Q) -> bool where T: Borrow<Q>, Q: Hash + Eq;
        }
    }

    fn seems_equal_to<Q, Voc>(&self, other: &impl TopicModelWithVocabulary<Q, Voc>) -> bool
    where T: Borrow<Q>,
          Q: Hash + Eq + Borrow<T>,
          Voc: BasicVocabulary<Q>
    {
        self.topic_count() == other.topic_count()
            && self.vocabulary_size() == other.vocabulary_size()
            && self.vocabulary.iter().enumerate().all(|(word_id, word)| {
            if let Some(found) = other.get_id(word.as_ref()) {
                self.used_vocab_frequency.get(word_id) == other.used_vocab_frequency().get(found)
            } else {
                false
            }
        })
            && self.topics
            .iter()
            .zip_eq(other.topics())
            .all(|(topic, other_topic)| {
                self.vocabulary
                    .iter()
                    .enumerate()
                    .all(|(word_id, word)| {
                        unsafe {
                            // all accesses are already checked by the checks above!
                            let value = topic.get_unchecked(word_id);
                            let other_word_id = other.get_id(word).expect("All words should be known!");
                            let value_other = other_topic.get_unchecked(other_word_id);
                            relative_eq!(*value, *value_other)
                        }
                    })
            })
    }
}

impl<T, V> TopicModelWithDocumentStats for TopicModel<T, V> {
    fn document_count(&self) -> usize {
        self.document_lengths.len()
    }

    fn document_ids(&self) -> Range<DocumentId> {
        0..self.document_lengths.len()
    }

    fn doc_topic_distributions(&self) -> &DocumentTo<TopicTo<Probability>> {
        &self.doc_topic_distributions
    }

    fn document_lengths(&self) -> &DocumentTo<DocumentLength> {
        &self.document_lengths
    }
}

impl<T: Display, V> TopicModel<T, V> where V: BasicVocabulary<T> {

    pub fn show_to(&self, n: usize, out: &mut impl Write) -> io::Result<()> {
        for (topic_id, topic_entries) in self.get_n_best_for_topics(n).ok_or(io::Error::from(ErrorKind::Other))?.iter().enumerate() {
            if topic_id != 0 {
                out.write(b"\n")?;
            }
            write!(out, "Topic({topic_id}):")?;
            for it in topic_entries.iter() {
                out.write(b"\n")?;
                write!(out, "    {}: {} ({})", self.vocabulary.get_value(it.word_id).unwrap(), it.probability, it.rank())?;
            }
        }
        Ok(())
    }

    pub fn show(&self, n: usize) -> io::Result<()> {
        let mut str = Vec::new();
        self.show_to(n, &mut str)?;
        println!("{}", String::from_utf8(str).unwrap());
        Ok(())
    }

    pub fn show_10(&self) -> io::Result<()>{
        self.show(10)
    }
}

impl<T: Display, V> Display for TopicModel<T, V> where V: Display + BasicVocabulary<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Topic Model:")?;
        for (topic_id, topic) in self.topics.iter().enumerate() {
            write!(f, "\n    Topic({topic_id})")?;
            for (word_id, probability) in topic.iter().enumerate() {
                write!(f, "\n        '{}'({}): {}", self.vocabulary.get_value(word_id).unwrap(), word_id, probability)?;
            }
        }
        write!(f, "\n{}", self.vocabulary)
    }
}


/// Allows to map a topic model to another one.
pub trait MappableTopicModel<T, V> where T: Clone + Hash + Eq, V: MappableVocabulary<T> {
    fn map<VNew>(self) -> TopicModel<T, VNew> where VNew: BasicVocabulary<T>;
}

impl<T, V> MappableTopicModel<T, V> for TopicModel<T, V> where T: Clone + Hash + Eq, V: MappableVocabulary<T>  {
    fn map<VNew>(self) -> TopicModel<T, VNew> where VNew: BasicVocabulary<T> {
        TopicModel {
            vocabulary: self.vocabulary.map(|value| value.clone()),
            document_lengths: self.document_lengths,
            doc_topic_distributions: self.doc_topic_distributions,
            used_vocab_frequency: self.used_vocab_frequency,
            topics: self.topics,
            topic_metas: self.topic_metas,
            _word_type: PhantomData
        }
    }
}




#[cfg(test)]
mod test {
    use crate::topicmodel::enums::TopicModelVersion;
    use crate::topicmodel::model::{TopicModel, TopicModelInferencer, TopicModelWithVocabulary};
    use crate::topicmodel::vocabulary::{StringVocabulary, Vocabulary, VocabularyMut};
    use itertools::{assert_equal, Itertools};


    pub fn create_test_data() -> TopicModel<String, Vocabulary<String>> {
        let mut voc: StringVocabulary = Vocabulary::default();
        voc.add("plane");
        voc.add("aircraft");
        voc.add("airplane");
        voc.add("flyer");
        voc.add("airman");
        voc.add("airfoil");
        voc.add("wing");
        voc.add("deck");
        voc.add("hydrofoil");
        voc.add("foil");
        voc.add("bearing surface");

        TopicModel::new(
            vec![
                vec![0.019, 0.018, 0.012, 0.009, 0.008, 0.007, 0.008, 0.008, 0.008, 0.008, 0.008],
                vec![0.02, 0.002, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001],
            ],
            voc,
            vec![10, 5, 8, 1, 2, 3, 1, 1, 1, 1, 2],
            vec![
                vec![0.7, 0.2],
                vec![0.8, 0.3],
            ],
            vec![
                200,
                300
            ]
        )
    }

    #[test]
    fn can_load_and_unlad_json(){
        let topic_model = create_test_data();

        let ser = serde_json::to_string_pretty(&topic_model).unwrap();
        println!("{}", ser);
        let topic: TopicModel<String, Vocabulary<String>> = serde_json::from_str(&ser).unwrap();

        for (a, b) in topic_model.topic_metas.iter().zip_eq(topic.topic_metas.iter()) {
            assert_equal(a.by_words.clone(), b.by_words.clone());
            assert_equal(a.by_position.clone(), b.by_position.clone());
            for (k, v) in a.by_importance.clone().into_iter().zip_eq(b.by_importance.clone().into_iter()) {
                assert_equal(k, v);
            }
        }
    }

    #[test]
    fn can_load_and_unlad_binary(){
        let topic_model = create_test_data();

        let ser = bincode::serialize(&topic_model).unwrap();
        let topic: TopicModel<String, Vocabulary<String>> = bincode::deserialize(&ser).unwrap();

        for (a, b) in topic_model.topic_metas.iter().zip_eq(topic.topic_metas.iter()) {
            assert_equal(a.by_words.clone(), b.by_words.clone());
            assert_equal(a.by_position.clone(), b.by_position.clone());
            for (k, v) in a.by_importance.clone().into_iter().zip_eq(b.by_importance.clone().into_iter()) {
                assert_equal(k, v);
            }
        }
    }


    #[test]
    fn can_load_and_unload(){
        let topic_model = create_test_data();

        const P: &str = "test2\\def";

        let _ = std::fs::create_dir("test2");
        topic_model.save(P, TopicModelVersion::V1, true, true).unwrap();

        let (loaded, _) = TopicModel::load_string_model(P, false).unwrap();

        assert!(topic_model.seems_equal_to(&loaded));

        topic_model.show_10().unwrap();
        topic_model.normalize().show_10().unwrap();

        std::fs::remove_dir_all(P).unwrap();


    }

    #[test]
    fn try_infer(){
        let before = std::time::Instant::now();
        let model = TopicModel::load_string_model(
            r"E:\git\ldatranslation\bambergdictionary\dictionaryprocessor\lda\translations\aligned-v1_never\en\lda\combSum_2-limit_a-aligned-v1",
            true
        ).unwrap().0;
        println!("{}", (std::time::Instant::now() - before).as_secs());
        // model.show_10().unwrap();
        let infer = TopicModelInferencer::new(&model, 0.001.into(), 0.1);
        let inferred = infer.get_doc_probability_for_default(vec!["hello".to_string(), "religion".to_string()], true);
        println!("{:?}", inferred.0);
    }
}