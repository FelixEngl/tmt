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
mod views;
mod meta_small;
pub use inferencer::*;
pub use traits::*;

use approx::relative_eq;
use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io;
use std::io::{Write};
use std::marker::PhantomData;
use std::ops::{Range};
use std::sync::{Arc};

use crate::model::meta::*;
use crate::vocabulary::{BasicVocabulary, MappableVocabulary, SearchableVocabulary, VocabularyMut};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use ldatranslate_toolkit::normal_number::IsNormalNumber;
use ldatranslate_translate::VoterInfoProvider;
use crate::model::meta_small::{SmallTopicMeta, SmallTopicStats, SmallWordMeta};
use crate::model::views::{ImportanceIndex, MetaView, PositionIndex, TopicMetaView};

pub type TopicTo<T> = Vec<T>;
pub type WordTo<T> = Vec<T>;
pub type PositionTo<T> = Vec<T>;
pub type DocumentTo<T> = Vec<T>;
pub type ImportanceTo<T> = Vec<T>;
pub type ImportanceRankTo<T> = Vec<T>;
pub type Probability = f64;

/// The direct rank, created by the order of the probabilities and then
pub type Rank = usize;

/// The rank, when grouping the topic by probabilities
pub type ImportanceRank = usize;
pub type WordId = usize;
pub type TopicId = usize;
pub type Position = usize;
pub type Importance = usize;
pub type DocumentId = usize;
pub type WordFrequency = u64;
pub type DocumentLength = u64;

/// A topic model
#[derive(Debug, Serialize, Deserialize)]
pub struct TopicModel<T, V> {
    // topic to word
    // Row = Topic
    // Col = Word
    topics: TopicTo<WordTo<Probability>>,
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    vocabulary: V,
    used_vocab_frequency: WordTo<WordFrequency>,
    doc_topic_distributions: DocumentTo<TopicTo<Probability>>,
    document_lengths: DocumentTo<DocumentLength>,
    topic_metas: TopicTo<SmallTopicMeta>,
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



impl<T, V> TopicModel<T, V> {
    pub fn get_meta_for(&self, topic_id: TopicId, word_id: WordId) -> Option<WordMeta> {
        let probability = *self.topics.get(topic_id)?.get(word_id)?;
        let small_meta = unsafe {
            self.topic_metas.get_unchecked(topic_id).by_word.get_unchecked(word_id)
        };
        Some(
            WordMeta {
                topic_id,
                word_id,
                probability,
                position: small_meta.position,
                importance: small_meta.importance,
            }
        )
    }

    pub unsafe fn get_meta_for_unchecked(&self, topic_id: TopicId, word_id: WordId) -> WordMeta {
        let probability = *self.topics.get_unchecked(topic_id).get_unchecked(word_id);
        let small_meta = unsafe {
            self.topic_metas.get_unchecked(topic_id).by_word.get_unchecked(word_id)
        };
        WordMeta {
            topic_id,
            word_id,
            probability,
            position: small_meta.position,
            importance: small_meta.importance,
        }
    }

    pub fn meta_view_for<'a>(&'a self, topic_id: TopicId) -> Option<TopicMetaView<'a>> {
        Some(
            TopicMetaView {
                probabilities_ref: self.topics.get(topic_id)?,
                stats_ref: unsafe{self.topic_metas.get_unchecked(topic_id)},
                topic_id
            }
        )
    }

    pub unsafe fn meta_view_for_unchecked(&self, topic_id: TopicId) -> TopicMetaView {
        TopicMetaView {
            probabilities_ref: self.topics.get_unchecked(topic_id),
            stats_ref: unsafe{self.topic_metas.get_unchecked(topic_id)},
            topic_id
        }
    }

    pub fn position_index(&self, topic_id: TopicId) -> Option<PositionIndex> {
        self.meta_view_for(topic_id).map(|value| value.to_position_index())
    }

    pub fn importance_index(&self, topic_id: TopicId) -> Option<ImportanceIndex> {
        self.meta_view_for(topic_id).map(|value| value.to_importance_index())
    }

    pub fn topic_stats_for(&self, topic_id: TopicId) -> Option<TopicStats> {
        let meta = self.topic_metas.get(topic_id)?;
        Some(
            TopicStats {
                topic_id,
                min_value: meta.stats.min,
                max_value: meta.stats.max,
                average_value: meta.stats.avg,
                sum_value: meta.stats.sum,
            }
        )
    }
}


impl<T, V> TopicModel<T, V> where
    T: Hash + Eq + Ord,
    V: BasicVocabulary<T> + Sync + Send
{
    fn recalculate_statistics(&mut self) {
        self.topic_metas = unsafe {
            Self::calculate_topic_metas(&self.topics, &self.vocabulary)
        };
    }

    unsafe fn calculate_topic_metas(topics: &TopicTo<WordTo<Probability>>, vocabulary: &(impl BasicVocabulary<T> + Sync + Send)) -> TopicTo<SmallTopicMeta> {
        struct SortHelper<'a, Q, V> where V: BasicVocabulary<Q> {
            word_id: WordId,
            probability: Probability,
            vocabulary: &'a V,
            _word_type: PhantomData<Q>,
        }

        impl<'a, Q, V> SortHelper<'a, Q, V> where Q: Hash + Eq, V: BasicVocabulary<Q> {
            fn word(&self) -> &Q {
                self.vocabulary.get_value_by_id(self.word_id).expect("There should be no problem with enpacking it here!")
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
                                other.vocabulary.get_value_by_id(other.word_id).unwrap().cmp(
                                    self.vocabulary.get_value_by_id(self.word_id).unwrap()
                                )
                            )
                        }
                    }
                    Some(Ordering::Equal) => {
                        Some(
                            other.vocabulary.get_value_by_id(other.word_id).unwrap().cmp(
                                self.vocabulary.get_value_by_id(self.word_id).unwrap()
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

        topics.par_iter().map(|topic| {
            let position_to_word_id_and_prob =
                topic
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

            let mut word_id_to_importance: Vec<(Importance, WordId)> = importance_to_word_ids
                .into_iter()
                .enumerate()
                .flat_map(|(importance, words)| {
                    words.into_iter().map(move |value| (importance, value))
                }).collect_vec();
            word_id_to_importance.sort_by_key(|value| value.1);

            let max_importance = word_id_to_importance.iter().map(|(v, _)| *v).max().unwrap();

            let word_id_to_position: Vec<_> = position_to_word_id_and_prob
                .into_iter()
                .enumerate()
                .map(|(position, (word_id, prob))| (word_id, prob, position))
                .sorted_by_key(|(word_id, _, _)| *word_id)
                .collect_vec();

            let mut by_word: WordTo<SmallWordMeta> = word_id_to_position
                .into_iter()
                .zip_eq(word_id_to_importance.into_iter())
                .map(|((word_id_1, prob, position), (importance, word_id_2))| {
                    assert_eq!(word_id_1, word_id_2, "Word ids {} {} are not compatible!", word_id_1, word_id_2);
                    (word_id_1, prob, position, importance)
                })
                .map(|(_, _, position, importance)| {
                    SmallWordMeta {
                        position,
                        importance
                    }
                }).collect_vec();
            by_word.shrink_to_fit();

            let mut max_value: f64 = f64::MIN;
            let mut min_value: f64 = f64::MAX;
            let mut sum_value: f64 = 0.0;

            for &value in topic.iter() {
                max_value = max_value.max(value);
                min_value = min_value.min(value);
                sum_value += value;
            }
            let stats = SmallTopicStats {
                max: max_value,
                min: min_value,
                sum: sum_value,
                avg: sum_value / (by_word.len() as f64),
                max_importance
            };
            SmallTopicMeta {
                stats,
                by_word
            }
        }).collect()
    }
}

impl<T, V> FullTopicModel<T, V> for TopicModel<T, V> where
    T: Hash + Eq + Ord,
    V: SearchableVocabulary<T> + Sync + Send
{
    fn normalize_in_place(&mut self) {
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

    fn new(
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
}


impl<T, V> TopicModel<T, V> where T: Hash + Eq + Ord + Clone, V: Clone + VocabularyMut<T> + Sync + Send {
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

    type TopicMetas<'a> = MetaView<'a, T, V> where Self: 'a;
    type TopicMeta<'a> = TopicMetaView<'a> where Self: 'a;
    type WordMeta<'a> = WordMeta where Self: 'a;

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

    fn topic_metas<'a>(&'a self) -> Self::TopicMetas<'a> {
        MetaView::new(self)
    }

    fn get_topic_meta<'a>(&'a self, topic_id: usize) -> Option<Self::TopicMeta<'a>> {
        self.meta_view_for(topic_id)
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

    fn get_word_meta(&self, topic_id: TopicId, word_id: WordId) -> Option<WordMeta> {
        self.get_meta_for(topic_id, word_id)
    }


    fn get_all_similar_important<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<Vec<WordMeta>> {
        let metas = self.topic_metas.get(topic_id)?;
        let targ_imp = metas.by_word.get(word_id)?.importance;
        if metas.by_word.len() > 1024 {
            Some(
                metas.by_word
                    .par_iter()
                    .enumerate()
                    .filter_map(|(word_id, v)| {
                        if v.importance == targ_imp {
                            Some(unsafe{
                                self.get_meta_for_unchecked(topic_id, word_id)
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            )
        } else {
            Some(
                metas.by_word
                    .iter()
                    .enumerate()
                    .filter_map(|(word_id, v)| {
                        if v.importance == targ_imp {
                            Some(unsafe{
                                self.get_meta_for_unchecked(topic_id, word_id)
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            )
        }


    }

    fn get_words_for_topic_sorted(&self, topic_id: TopicId) -> Option<PositionTo<WordId>> {
        let value = Arc::into_inner(self.meta_view_for(topic_id)?.to_position_index().sorted_by_pos).expect("This unpacking never fails!");
        Some(value)
    }


    fn get_n_best_for_topic(&self, topic_id: usize, n: usize) -> Option<PositionTo<WordId>> {
        let metas = self.topic_metas.get(topic_id)?;
        if n == 0 {
            return Some(Vec::new())
        }

        Some(
            metas.by_word
                .iter()
                .enumerate()
                .filter(|&(_, v)|  v.position < n)
                .sorted_unstable_by_key(|(_, v)| v.position)
                .map(|(k, _)| k)
                .collect()
        )
    }

    fn get_n_best_for_topics(&self, n: usize) -> TopicTo<Vec<WordId>> {
        self.topic_ids().map(
            |topic_id|
                self.get_n_best_for_topic(topic_id, n)
                    .expect("Never fails.")
        ).collect()
    }

}

impl<T, V> VoterInfoProvider for TopicModel<T, V> where V: BasicVocabulary<T> {
    type VoterMeta<'b> = WordMeta where Self: 'b;

    fn get_voter_meta<'a>(&'a self, column: usize, row: usize) -> Option<WordMeta> {
        self.get_word_meta(column, row)
    }
}

impl<T, V> BasicTopicModelWithVocabulary<T, V> for TopicModel<T, V> where V: BasicVocabulary<T> {
    fn vocabulary(&self) -> &V {
        &self.vocabulary
    }
}

impl<T, V> TopicModelWithVocabulary<T, V> for TopicModel<T, V> where T: Hash + Eq, V: SearchableVocabulary<T> {
    delegate::delegate! {
        to self.vocabulary {
            fn get_id<Q: ?Sized>(&self, word: &Q) -> Option<usize> where T: Borrow<Q>, Q: Hash + Eq;

            #[call(contains_value)]
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
            if let Some(found) = other.get_id(word) {
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
        for (topic_id, topic_entries) in self.get_n_best_for_topics(n).iter().enumerate() {
            if topic_id != 0 {
                out.write(b"\n")?;
            }
            let topic = self.get_topic_meta(topic_id).expect("All words should be known!");
            write!(out, "Topic({topic_id}):")?;
            for it in topic_entries.iter() {
                let word_meta = topic.get_word_meta(*it).expect("All words should be known!");
                out.write(b"\n")?;
                write!(
                    out,
                    "    {}: {} ({})",
                    self.vocabulary.get_value_by_id(word_meta.word_id).unwrap(),
                    word_meta.probability,
                    word_meta.rank()
                )?;
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
                write!(f, "\n        '{}'({}): {}", self.vocabulary.get_value_by_id(word_id).unwrap(), word_id, probability)?;
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
pub mod test {
    use arcstr::ArcStr;
    use crate::enums::TopicModelVersion;
    use crate::model::{BasicTopicModel, FullTopicModel, TopicModel, TopicModelInferencer, TopicModelWithVocabulary};
    use crate::vocabulary::{EfficientStringVocabulary, Vocabulary, VocabularyMut};
    use itertools::{assert_equal, Itertools};
    use bincode;

    pub fn create_test_data() -> TopicModel<ArcStr, Vocabulary<ArcStr>> {
        let mut voc: EfficientStringVocabulary = Vocabulary::default();
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
    fn the_new_sorts_work_as_expected(){
        let test = create_test_data();
        println!("{:#?}", test.get_all_similar_important(0, 4).expect("Should returns something"))
    }

    #[test]
    fn can_load_and_unlad_json(){
        let topic_model = create_test_data();

        let ser = serde_json::to_string_pretty(&topic_model).unwrap();
        println!("{}", ser);
        let topic: TopicModel<String, Vocabulary<String>> = serde_json::from_str(&ser).unwrap();

        for (a, b) in topic_model.topic_metas.iter().zip_eq(topic.topic_metas.iter()) {
            assert_equal(a.by_word.as_slice(), b.by_word.as_slice());
        }
    }

    #[test]
    fn test_complete_functionality(){
        let mut t = create_test_data();
        println!("{:?}", t.topics);
        t.normalize_in_place();
        println!("{:?}", t.topics);
    }

    #[test]
    fn can_load_and_unlad_binary(){
        let topic_model = create_test_data();

        let ser = bincode::serialize(&topic_model).unwrap();
        let topic: TopicModel<String, Vocabulary<String>> = bincode::deserialize(&ser).unwrap();

        for (a, b) in topic_model.topic_metas.iter().zip_eq(topic.topic_metas.iter()) {
            assert_equal(a.by_word.iter(), b.by_word.iter());
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