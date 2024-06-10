#![allow(dead_code)]

use std::borrow::{Borrow};
use std::cmp::{min, Ordering, Reverse};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::iter::Map;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Range};
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use std::sync::Arc;
use approx::relative_eq;

use flate2::Compression;
use itertools::{Itertools, multiunzip, multizip};
use rand::thread_rng;
use rand_distr::Distribution;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::toolkit::normal_number::IsNormalNumber;

use crate::topicmodel::enums::{ReadError, TopicModelVersion, WriteError};
use crate::topicmodel::enums::ReadError::NotFinishedError;
use crate::topicmodel::traits::{ToParseableString};
use crate::topicmodel::io::{TopicModelFSRead, TopicModelFSWrite};
use crate::topicmodel::io::TopicModelIOError::PathNotFound;
use crate::topicmodel::math::{dirichlet_expectation_1d, dirichlet_expectation_2d, dot, transpose};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{LoadableVocabulary, MappableVocabulary, StoreableVocabulary, Vocabulary, VocabularyImpl, VocabularyMut};


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


/// The meta for a topic.
#[derive(Debug, Clone)]
pub struct TopicMeta {
    pub stats: TopicStats,
    pub by_words: WordTo<Arc<WordMeta>>,
    pub by_position: PositionTo<Arc<WordMeta>>,
    pub by_importance: ImportanceRankTo<Vec<Arc<WordMeta>>>
}

impl TopicMeta {
    pub fn new(
        stats: TopicStats,
        mut by_words: WordTo<Arc<WordMeta>>,
        mut by_position: PositionTo<Arc<WordMeta>>,
        mut by_importance: ImportanceRankTo<Vec<Arc<WordMeta>>>
    ) -> Self {
        by_words.shrink_to_fit();
        by_position.shrink_to_fit();
        by_importance.shrink_to_fit();

        Self {
            stats,
            by_words,
            by_position,
            by_importance
        }
    }
}

/// The meta for a word.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordMeta {
    pub topic_id: TopicId,
    pub word_id: WordId,
    pub probability: Probability,
    /// The position in the topic model, starting from 0
    pub position: Position,
    /// The importance in the topic model, starting from 0
    pub importance: Importance,
}

impl WordMeta {
    /// Returns the [self.probability] + 1
    #[inline]
    pub fn rank(&self) -> Rank {
        self.position + 1
    }

    /// Returns the [self.importance] + 1
    #[inline]
    pub fn importance_rank(&self) -> ImportanceRank {
        self.importance + 1
    }
}

impl Display for WordMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5}({})", self.probability, self.rank())
    }
}


/// Contains a reference to the associated word and the associated [WordMeta]
#[derive(Debug)]
pub struct WordMetaWithWord<'a, T> {
    pub word: &'a T,
    inner: &'a Arc<WordMeta>
}

impl<'a, T> WordMetaWithWord<'a, T> {
    pub fn new(word: &'a T, inner: &'a Arc<WordMeta>) -> Self {
        Self {
            word,
            inner
        }
    }
}

impl<'a, T> WordMetaWithWord<'a, T> {
    pub fn into_inner(self) -> &'a Arc<WordMeta> {
        self.inner
    }
}

impl<'a, T> Clone for WordMetaWithWord<'a, T> {
    fn clone(&self) -> Self {
        Self {
            word: self.word,
            inner: self.inner
        }
    }
}

impl<T> Deref for WordMetaWithWord<'_, T> {
    type Target = Arc<WordMeta>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

/// The precalculated stats of a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStats {
    pub topic_id: usize,
    pub max_value: f64,
    pub min_value: f64,
    pub average_value: f64,
    pub sum_value: f64,
}


/// A basic topic model fulfilling the bare minimum of
pub trait BasicTopicModel: Send + Sync {
    /// The number of topics in this model
    fn topic_count(&self) -> usize;

    /// The number of topics in this model
    #[inline]
    fn k(&self) -> usize {
        self.topic_count()
    }

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

/// A topicmodel with document stats
pub trait TopicModelWithDocumentStats {
    /// Returns the number of documents
    fn document_count(&self) -> usize;

    /// Returns all document ids
    fn document_ids(&self) -> Range<DocumentId>;

    /// Returns the topic distributions of the topic model
    fn doc_topic_distributions(&self) -> &DocumentTo<TopicTo<Probability>>;

    /// Returns the document lengths of the documents
    fn document_lengths(&self) -> &DocumentTo<DocumentLength>;
}

/// A basic topic model with a vocabulary
pub trait BasicTopicModelWithVocabulary<T, Voc>: BasicTopicModel where Voc: Vocabulary<T> {
    /// The vocabulary
    fn vocabulary(&self) -> &Voc;

    /// Get the word for the `word_id`
    #[inline]
    fn get_word<'a>(&'a self, word_id: WordId) -> Option<&'a HashRef<T>> where Voc: 'a {
        self.vocabulary().get_value(word_id)
    }

    /// Get the [WordMetaWithWord] of `word_id` of `topic_id`
    fn get_word_meta_with_word<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<WordMetaWithWord<'a, HashRef<T>>>  where Voc: 'a {
        let topic_meta = self.get_topic_meta(topic_id)?;
        let word_meta = topic_meta.by_words.get(word_id)?;
        let word = self.vocabulary().get_value(word_meta.word_id)?;
        Some(WordMetaWithWord::new(word, word_meta))
    }

    /// Get the [WordMetaWithWord] of `word_id` for all topics.
    fn get_word_metas_with_word<'a>(&'a self, word_id: usize) -> Option<TopicTo<WordMetaWithWord<'a, HashRef<T>>>> where Voc: 'a {
        self.topic_ids().map(|topic_id| self.get_word_meta_with_word(topic_id, word_id)).collect()
    }

    /// Get all [WordMetaWithWord] values with a similar importance in `topic_id` than `word_id`.
    /// (including the `word_id`)
    fn get_all_similar_important_with_word_for<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<Vec<WordMetaWithWord<'a, HashRef<T>>>> where Voc: 'a {
        Some(
            self.get_all_similar_important(topic_id, word_id)?
                .iter()
                .map(|value| WordMetaWithWord::new(self.vocabulary().get_value(value.word_id).unwrap(), value))
                .collect()
        )
    }
}

/// A topic model with an explicit vocabulary
pub trait TopicModelWithVocabulary<T, Voc>: BasicTopicModelWithVocabulary<T, Voc> where Voc: Vocabulary<T> {
    fn get_id<Q: ?Sized>(&self, word: &Q) -> Option<WordId> where T: Borrow<Q>, Q: Hash + Eq;
    fn contains<Q: ?Sized>(&self, word: &Q) -> bool where T: Borrow<Q>, Q: Hash + Eq;

    /// Get the probability of `word` of `topic_id`
    #[inline]
    fn get_probability_by_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<&Probability> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_probability(topic_id, self.get_id(word)?)
    }

    /// Get all probabilities of `word`
    #[inline]
    fn get_topic_probabilities_for_by_word<Q: ?Sized>(&self, word: &Q) -> Option<TopicTo<Probability>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_topic_probabilities_for(self.get_id(word)?)
    }

    /// Get the [WordMeta] of `word` of `topic_id`
    #[inline]
    fn get_word_meta_by_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<&Arc<WordMeta>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_word_meta(topic_id, self.get_id(word)?)
    }

    /// Get the [WordMetaWithWord] of `word` for all topics.
    #[inline]
    fn get_word_metas_with_word_by_word<'a, Q: ?Sized>(&'a self, word: &Q) -> Option<TopicTo<WordMetaWithWord<'a, HashRef<T>>>> where T: Borrow<Q>, Q: Hash + Eq, Voc: 'a {
        self.get_word_metas_with_word(self.get_id(word)?)
    }

    /// Get all [WordMeta] values with a similar importance in `topic_id` than `word`.
    /// (including the `word_id`)
    #[inline]
    fn get_all_similar_important_words_for_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<&Vec<Arc<WordMeta>>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_all_similar_important(topic_id, self.get_id(word)?)
    }

    /// Returns true iff the topic models seem similar.
    fn seems_equal_to<Q, VOther>(&self, other: &impl TopicModelWithVocabulary<Q, VOther>) -> bool
        where
            T: Borrow<Q>,
            Q: Hash + Eq + Borrow<T>,
            VOther: Vocabulary<Q>
    ;
}

/// A topic model that allows basic show methods
pub trait DisplayableTopicModel<T, Voc>: BasicTopicModelWithVocabulary<T, Voc> where T: Display, Voc: Vocabulary<T> + Display {
    fn show_to(&self, n: usize, out: &mut impl Write) -> io::Result<()> {
        for (topic_id, topic_entries) in self.get_n_best_for_topics(n).ok_or(io::Error::from(ErrorKind::Other))?.iter().enumerate() {
            if topic_id != 0 {
                out.write(b"\n")?;
            }
            write!(out, "Topic({topic_id}):")?;
            for it in topic_entries.iter() {
                out.write(b"\n")?;
                write!(out, "    {}: {} ({})", self.get_word(it.word_id).unwrap(), it.probability, it.rank())?;
            }
        }
        Ok(())
    }

    fn show(&self, n: usize) -> io::Result<()> {
        let mut str = Vec::new();
        self.show_to(n, &mut str)?;
        println!("{}", String::from_utf8(str).unwrap());
        Ok(())
    }

    fn show_10(&self) -> io::Result<()>{
        self.show(10)
    }
}

impl<TopicModel, T, Voc> DisplayableTopicModel<T, Voc> for TopicModel
    where TopicModel: BasicTopicModelWithVocabulary<T, Voc>,
          T:Display, Voc: Vocabulary<T> + Display
{}

/// A topic model
#[derive(Debug)]
pub struct TopicModel<T, V> {
    topics: TopicTo<WordTo<Probability>>,
    vocabulary: V,
    used_vocab_frequency: WordTo<WordFrequency>,
    doc_topic_distributions: DocumentTo<TopicTo<Probability>>,
    document_lengths: DocumentTo<DocumentLength>,
    topic_metas: TopicTo<TopicMeta>,
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

    unsafe fn calculate_topic_metas(topics: &TopicTo<WordTo<Probability>>, vocabulary: &impl Vocabulary<T>) -> TopicTo<TopicMeta> {
        struct SortHelper<'a, Q, V> where V: Vocabulary<Q> {
            word_id: WordId,
            probability: Probability,
            vocabulary: &'a V,
            _word_type: PhantomData<Q>,
        }

        impl<'a, Q, V> SortHelper<'a, Q, V> where Q: Hash + Eq, V: Vocabulary<Q> {
            fn word(&self) -> &HashRef<Q> {
                self.vocabulary.get_value(self.word_id).expect("There should be no problem with enpacking it here!")
            }
        }

        impl<Q, V> Eq for SortHelper<'_, Q, V> where V: Vocabulary<Q> {}

        impl<Q, V> PartialEq<Self> for SortHelper<'_, Q, V> where V: Vocabulary<Q> {
            fn eq(&self, other: &Self) -> bool {
                self.probability.eq(&other.probability)
            }
        }

        impl<Q, V> PartialOrd for SortHelper<'_, Q, V> where Q: Hash + Eq + Ord, V: Vocabulary<Q> {
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

        impl<Q, V> Ord for SortHelper<'_, Q, V> where Q: Hash + Eq + Ord, V: Vocabulary<Q> {
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

    pub fn normalize_in_place(mut self) -> Self {
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

        self
    }
}

impl<T, V> TopicModel<T, V> where T: Hash + Eq + Ord, V: Clone + VocabularyMut<T> {
    pub fn normalize(&self) -> Self {
        self.clone().normalize_in_place()
    }
}

impl<T, V> TopicModel<T, V> {
    pub fn is_already_finished(path: impl AsRef<Path>) -> bool {
        println!("{:}", path.as_ref().join(MARKER_FILE).to_str().unwrap());
        path.as_ref().join(MARKER_FILE).exists()
    }

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

impl<T, V> BasicTopicModel for TopicModel<T, V> where V: Vocabulary<T> {
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

    fn get_n_best_for_topic(&self, topic_id: usize, n: usize) -> Option<&[Arc<WordMeta>]> {
        let metas = self.topic_metas.get(topic_id)?;
        Some(&metas.by_position[..min(n, metas.by_position.len())])
    }

    fn get_n_best_for_topics(&self, n: usize) -> Option<Vec<&[Arc<WordMeta>]>> {
        self.topic_ids().map(|topic_id| self.get_n_best_for_topic(topic_id, n)).collect()
    }
}

impl<T, V> BasicTopicModelWithVocabulary<T, V> for TopicModel<T, V> where V: Vocabulary<T> {
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
              Voc: Vocabulary<Q>
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

impl<T: Display, V> TopicModel<T, V> where V: Vocabulary<T> {

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

impl<T: Display, V> Display for TopicModel<T, V> where V: Display + Vocabulary<T> {
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

impl TopicModel<String, VocabularyImpl<String>> {
    pub fn load_string_model(path: impl AsRef<Path>, allow_unfinished: bool) -> Result<(Self, TopicModelVersion), ReadError<Infallible>> {
        Self::load(path, allow_unfinished)
    }
}

const MODEL_ZIP_PATH: &str = "model.zip";
const PATH_TO_DOC_LENGTHS: &str = "doc\\doc_lengths.freq";
const PATH_TO_DOC_TOPIC_DISTS: &str = "doc\\doc_topic_dists.freq";
const PATH_TO_VOCABULARY_FREQ: &str = "voc\\vocabulary.freq";
const PATH_TO_VOCABULARY: &str = "voc\\vocabulary.txt";
const PATH_TO_MODEL: &str = "model\\topic.model";
const PATH_VERSION_INFO: &str = "version.info";
const MARKER_FILE: &str = "COMPLETED_TM";

impl<T: FromStr<Err=E> + Hash + Eq + Ord, E: Debug, V> TopicModel<T, V> where V: LoadableVocabulary<T, E> + VocabularyMut<T> {

    pub fn load(path: impl AsRef<Path>, allow_unfinished: bool) -> Result<(TopicModel<T, V>, TopicModelVersion), ReadError<E>> {
        if !allow_unfinished && !Self::is_already_finished(&path) {
            return Err(NotFinishedError(path.as_ref().to_path_buf()))
        }
        let reader = if path.as_ref().is_file() {
            TopicModelFSRead::open_zip(path)?
        } else {
            let z = path.as_ref().join(MODEL_ZIP_PATH);
            if z.exists() {
                TopicModelFSRead::open_zip(z)?
            } else {
                TopicModelFSRead::open_file_system(path)?
            }
        };
        Self::load_routine(reader)
    }

    fn load_routine(mut fs: TopicModelFSRead) -> Result<(TopicModel<T, V>, TopicModelVersion), ReadError<E>> {
        let mut buf = String::new();

        let version = match fs.create_reader_to(PATH_VERSION_INFO) {
            Ok((mut reader, _)) => {
                reader.read_to_string(&mut buf)?;
                buf.trim().parse()?
            }
            Err(PathNotFound(_)) => {
                TopicModelVersion::V1
            }
            Err(other) => return Err(other.into())
        };

        match &version {
            TopicModelVersion::V1 => {
                let doc_lengths = { Self::read_vec_u64(fs.create_reader_to(PATH_TO_DOC_LENGTHS)?.0) }?;
                let (inp, deflate) = fs.create_reader_to(PATH_TO_DOC_TOPIC_DISTS)?;
                let doc_topic_distributions = Self::read_matrix_f64(inp, deflate)?;
                let used_vocab_frequency = Self::read_vec_u64(fs.create_reader_to(PATH_TO_VOCABULARY_FREQ)?.0)?;
                let (inp, _) = fs.create_reader_to(PATH_TO_VOCABULARY)?;
                let vocabulary = V::load_from_input(&mut BufReader::new(inp))?;
                let (inp, deflate) = fs.create_reader_to(PATH_TO_MODEL)?;
                let topics = Self::read_matrix_f64(inp, deflate)?;
                Ok(
                    (
                        Self::new(
                            topics,
                            vocabulary,
                            used_vocab_frequency,
                            doc_topic_distributions,
                            doc_lengths
                        ),
                        version
                    )
                )
            }
            TopicModelVersion::V2 => {
                panic!("Unsupported")
            }
        }
    }


    fn read_vec_u64(inp: impl Read) -> Result<Vec<u64>, ReadError<E>> {
        BufReader::new(inp).lines().process_results(|lines| {
            lines.enumerate().map(|(pos, line)| line.trim().parse::<u64>().map_err(|err| ReadError::ParseInt {
                line: pos,
                position: 0,
                err
            })).collect::<Result<Vec<_>, _>>()
        })?
    }

    fn read_matrix_f64(inp: impl Read, deflate: bool) -> Result<Vec<Vec<f64>>, ReadError<E>> {
        let mut reader: Box<dyn BufRead> = if deflate {
            Box::new(BufReader::new(flate2::read::DeflateDecoder::new(inp)))
        } else {
            Box::new(BufReader::new(inp))
        };

        reader.deref_mut().lines().process_results(|lines| {
            lines.enumerate().filter_map(move |(line_no, mut line)| {
                line.retain(|value| !['\n', '\r', '\t'].contains(&value));
                if line.is_empty() {
                    None
                } else {
                    Some(line.trim().split(" ").enumerate().map(
                        |(pos, it)| it.replace(",", ".").parse::<f64>().map_err(|err| ReadError::ParseFloat {
                            line: line_no,
                            position: pos,
                            err
                        })
                    ).collect::<Result<Vec<f64>, _>>())
                }
            }).collect::<Result<Vec<_>, _>>()
        })?
    }
}

impl<T: ToParseableString, V> TopicModel<T, V> where V: StoreableVocabulary<T> {

    pub fn save(&self, path: impl AsRef<Path>, save_version: TopicModelVersion, deflate: bool, replace: bool) -> Result<usize, WriteError> {
        if Self::is_already_finished(&path) {
            if !replace {
                return Err(WriteError::AlreadyFinished)
            } else {
                if path.as_ref().exists() {
                    std::fs::remove_dir_all(&path)?;
                }
            }
        } else {
            if path.as_ref().exists() {
                std::fs::remove_dir_all(&path)?;
            }
        }

        let mut fs = if deflate {
            TopicModelFSWrite::create_zip(path.as_ref().join(MODEL_ZIP_PATH))
        } else {
            TopicModelFSWrite::create_file_system(&path)
        }?;


        let result = self.save_routine(&mut fs, save_version, false)?;
        match std::fs::File::create_new(path.as_ref().join(MARKER_FILE)) {
            Ok(_) => {}
            Err(err) => {
                match err.kind() {
                    ErrorKind::AlreadyExists => {}
                    _ => {
                        return Err(WriteError::IO(err))
                    }
                }
            }
        }
        Ok(result)
    }



    fn save_routine(&self, fs: &mut TopicModelFSWrite, save_version: TopicModelVersion, deflate: bool) -> Result<usize, WriteError> {
        let mut bytes_written = fs.create_writer_to(PATH_VERSION_INFO)?.write(save_version.as_ref().as_bytes())?;
        match save_version {
            TopicModelVersion::V1 => {
                bytes_written += self.vocabulary.save_to_output(&mut fs.create_writer_to(PATH_TO_VOCABULARY)?)?;
                bytes_written += fs.create_writer_to(PATH_TO_VOCABULARY_FREQ)?.write(self.used_vocab_frequency.iter().map(|value| value.to_string()).join("\n").as_bytes())?;
                bytes_written += fs.create_writer_to(PATH_TO_DOC_LENGTHS)?.write(self.document_lengths.iter().map(|value| value.to_string()).join("\n").as_bytes())?;
                bytes_written += Self::write_matrix_f64(&mut fs.create_writer_to(PATH_TO_DOC_TOPIC_DISTS)?, &self.doc_topic_distributions, deflate)?;
                bytes_written += Self::write_matrix_f64(&mut fs.create_writer_to(PATH_TO_MODEL)?, &self.topics, deflate)?;
            }
            TopicModelVersion::V2 => {
                panic!("Unsupported!")
            }
        }
        fs.create_writer_to(MARKER_FILE)?.write(&[])?;
        Ok(bytes_written)
    }

    fn write_matrix_f64(out: &mut impl Write, target: &Vec<Vec<f64>>, deflate: bool) -> io::Result<usize> {
        let mut write: Box<dyn Write> = if deflate {
            Box::new(BufWriter::new(flate2::write::DeflateEncoder::new(out, Compression::default())))
        } else {
            Box::new(BufWriter::new(out))
        };
        let mut bytes = 0usize;
        for doubles in target {
            let t = doubles.iter().map(|value| format!("{:.20}", value).replace(',', ".")).join(" ");
            bytes += write.write(t.as_bytes())?;
            bytes += write.write(b"\n")?;
        }
        Ok(bytes)
    }
}

pub trait MappableTopicModel<T, V> where T: Clone + Hash + Eq, V: MappableVocabulary<T> + From<Vec<T>> {
    fn map<VNew>(self) -> TopicModel<T, VNew> where VNew: From<Vec<T>>;
}

impl<T, V> MappableTopicModel<T, V> for TopicModel<T, V> where T: Clone + Hash + Eq, V: MappableVocabulary<T> + From<Vec<T>>  {
    fn map<VNew>(self) -> TopicModel<T, VNew> where VNew: From<Vec<T>> {
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


#[derive(Debug)]
pub enum WordIdOrUnknown<T> {
    WordId(WordId),
    Unknown(T)
}


pub struct TopicModelInferencer<'a, T, V, Model> where Model: TopicModelWithVocabulary<T, V>, V: Vocabulary<T> {
    topic_model: &'a Model,
    alpha: f64,
    gamma_threshold: f64,
    _word_type: PhantomData<(T, V)>
}

impl<'a, T, V, Model> TopicModelInferencer<'a, T, V, Model> where Model: TopicModelWithVocabulary<T, V>, V: Vocabulary<T> {
    pub fn new(topic_model: &'a Model, alpha: f64, gamma_threshold: f64) -> Self {
        Self { topic_model, alpha, gamma_threshold, _word_type: PhantomData }
    }
}

impl<'a, T, V, Model> TopicModelInferencer<'a, T, V, Model> where
    T: Hash + Eq,
    V: VocabularyMut<T>,
    Model: TopicModelWithVocabulary<T, V>
{
    pub const DEFAULT_MIN_PROBABILITY: f64 = 1E-10;
    pub const DEFAULT_MIN_PHI_VALUE: f64 = 1E-10;

    pub fn get_doc_probability_for_default(
        &self,
        doc: Vec<T>,
        per_word_topics: bool
    ) -> (Vec<(usize, f64)>, Option<Vec<(usize, Vec<usize>)>>, Option<Vec<(usize, Vec<(usize, f64)>)>>) {
        self.get_doc_probability_for(doc, Self::DEFAULT_MIN_PROBABILITY, Self::DEFAULT_MIN_PHI_VALUE, per_word_topics)
    }

    fn get_doc_probability(&self, doc: Vec<WordIdOrUnknown<T>>, minimum_probability: f64, minimum_phi_value: f64, per_word_topics: bool) -> (Vec<(usize, f64)>, Option<Vec<(usize, Vec<usize>)>>, Option<Vec<(usize, Vec<(usize, f64)>)>>) {
        let minimum_probability = 1E-10f64.max(minimum_probability);
        let minimum_phi_value = 1E-10f64.max(minimum_phi_value);
        let (bow, _) = self.doc_to_bow(doc);
        let (gamma, phis) = self.inference(
            vec![bow.iter().map(|(a, b)| (*a,*b)).collect_vec()],
            per_word_topics,
            1000
        );
        let norm_value = gamma[0].iter().sum::<f64>();
        let topic_dist = gamma[0].iter().map(|value| value / norm_value).collect_vec();

        let document_topics = topic_dist.into_iter().enumerate().filter(|(_, value)| *value > minimum_probability).collect_vec();

        if let Some(phis) = phis {
            let mut word_topic: Vec<(usize, Vec<usize>)> = Vec::new();  // contains word and corresponding topic
            let mut word_phi: Vec<(usize, Vec<(usize, f64)>)> = Vec::new();  // contains word and phi values
            for (word_type, _) in bow.iter() {
                let word_type = *word_type;
                let mut phi_values: Vec<(f64, usize)> = Vec::new();  // contains (phi_value, topic) pairing to later be sorted
                let mut phi_topic: Vec<(usize, f64)> = Vec::new();  // contains topic and corresponding phi value to be returned 'raw' to user
                for topic_id in self.topic_model.topic_ids() {
                    let v = phis[topic_id][word_type];
                    if v > minimum_phi_value {
                        phi_values.push((v, topic_id));
                        phi_topic.push((topic_id, v));
                    }
                }
                // list with ({word_id => [(topic_0, phi_value), (topic_1, phi_value) ...]).
                word_phi.push((word_type, phi_topic));
                // sorts the topics based on most likely topic
                // returns a list like ({word_id => [topic_id_most_probable, topic_id_second_most_probable, ...]).
                phi_values.sort_by(|a, b| b.0.total_cmp(&a.0));
                word_topic.push((word_type, phi_values.into_iter().map(|(_, b)| b).collect()))
            }
            (document_topics, Some(word_topic), Some(word_phi))
        } else {
            (document_topics, None, None)
        }
    }

    fn inference(&self, chunk: Vec<Vec<(usize, usize)>>, collect_stats: bool, iterations: usize) -> (Vec<Vec<f64>>, Option<Vec<Vec<f64>>>) {

        fn calculate_phi_norm(exp_e_log_theta_d: &Vec<f64>, exp_e_log_beta_d: &Vec<Vec<f64>>) -> Vec<f64> {
            dot(exp_e_log_theta_d, exp_e_log_beta_d).map(|value| value + f64::EPSILON).collect_vec()
        }

        fn calculate_gamma_d(alpha: f64, exp_e_log_theta_d: &Vec<f64>, exp_e_log_beta_d: &Vec<Vec<f64>>, counts: &Vec<usize>, phinorm: &Vec<f64>) -> Vec<f64> {
            let a = counts.iter().zip_eq(phinorm.iter()).map(|(ct, phi)| *ct as f64 / phi).collect_vec();
            let b = transpose(exp_e_log_beta_d).collect_vec();
            dot(&a, &b).zip_eq(exp_e_log_theta_d.iter()).map(|(dot, theta)| dot * theta + alpha).collect()
        }

        fn calculate_stats<'a>(exp_e_log_theta_d: &'a Vec<f64>, counts: &Vec<usize>, phinorm: &Vec<f64>) -> Map<Iter<'a, f64>, impl FnMut(&'a f64) -> Vec<f64> + 'a> {
            // transposing a 1d == not transposing in numpy exp_e_log_theta_d.T
            let b = counts.iter().zip_eq(phinorm.iter()).map(|(a, b)| *a as f64 / b).collect_vec();
            exp_e_log_theta_d.iter().map(move |a| b.iter().map(|b| a * b).collect_vec())
        }


        let gamma = rand_distr::Gamma::new(100., 1./100.)
            .unwrap()
            .sample_iter(&mut thread_rng())
            .take(self.topic_model.k() * chunk.len())
            .chunks(self.topic_model.k())
            .into_iter()
            .map(|value| value.collect_vec())
            .collect_vec();

        assert_eq!(chunk.len(), gamma.len());
        assert_eq!(self.topic_model.k(), gamma[0].len());

        let exp_e_log_theta = dirichlet_expectation_2d(&gamma).map(|values| values.iter().copied().map(f64::exp).collect_vec()).collect_vec();
        assert_eq!(chunk.len(), exp_e_log_theta.len());
        assert_eq!(self.topic_model.k(), exp_e_log_theta[0].len());

        let mut stats = if collect_stats {
            let mut stats: Vec<Vec<f64>> = Vec::with_capacity(self.topic_model.k());
            for _ in self.topic_model.topic_ids() {
                stats.push(vec![0.;self.topic_model.vocabulary_size()]);
            }
            Some(stats)
        } else {
            None
        };

        let mut converged = 0;

        let gamma = multizip((chunk.into_iter(), gamma.into_iter(), exp_e_log_theta.into_iter()))
            .enumerate()
            .map(|(_, (doc, mut gamma_d, mut exp_e_log_theta_d))| {
                let (ids, cts): (Vec<_>, Vec<_>) = multiunzip(doc.into_iter());
                let exp_e_log_beta_d = self.topic_model.topics().iter().map(|topic| ids.iter().map(|id| topic[*id]).collect_vec()).collect_vec();
                let mut phinorm = calculate_phi_norm(&exp_e_log_theta_d, &exp_e_log_beta_d);
                for _ in 0..iterations {
                    let last_gamma = std::mem::replace(
                        &mut gamma_d,
                        calculate_gamma_d(self.alpha, &exp_e_log_theta_d, &exp_e_log_beta_d, &cts, &phinorm)
                    );
                    exp_e_log_theta_d = dirichlet_expectation_1d(&gamma_d).map(|value| value.exp()).collect();
                    phinorm = dot(&exp_e_log_theta_d, &exp_e_log_beta_d).map(|value| value + f64::EPSILON).collect();
                    let meanchange =  gamma_d.iter().zip_eq(last_gamma.iter()).map(|(a, b)| f64::abs(a - b)).sum::<f64>() / (gamma_d.len() as f64);
                    if meanchange < self.gamma_threshold {
                        converged += 1;
                        break;
                    }
                }
                if let Some(stats) = &mut stats {
                    let calc = calculate_stats(&exp_e_log_theta_d, &cts, &phinorm).collect_vec();
                    for(values, to_add) in stats.iter_mut().zip(calc.into_iter()) {
                        for (pos, id) in ids.iter().enumerate() {
                            unsafe {
                                *values.get_unchecked_mut(*id) += to_add[pos];
                            }
                        }
                    }
                }
                gamma_d
            }).collect_vec();

        (gamma, stats)
    }

    fn doc_to_bow<Q>(&self, doc: Vec<WordIdOrUnknown<Q>>) -> (HashMap<WordId, usize>, Option<HashMap<Q, usize>>) where T: Borrow<Q>, Q: Eq + Hash {
        let mut counts: HashMap<WordId, usize> = HashMap::with_capacity(doc.len());
        let mut fallback = HashMap::new();
        for word in doc {
            match word {
                WordIdOrUnknown::WordId(value) => {
                    match counts.entry(value) {
                        Entry::Occupied(entry) => {
                            *entry.into_mut() += 1;
                        }
                        Entry::Vacant(vacant) => {
                            vacant.insert(1usize);
                        }
                    }
                }
                WordIdOrUnknown::Unknown(value) => {
                    match fallback.entry(value) {
                        Entry::Occupied(entry) => {
                            *entry.into_mut() += 1;
                        }
                        Entry::Vacant(vacant) => {
                            vacant.insert(1usize);
                        }
                    }
                }
            }
        }

        (counts, (!fallback.is_empty()).then_some(fallback))
    }

    pub fn get_doc_probability_for(&self, doc: Vec<T>, minimum_probability: f64, minimum_phi_value: f64, per_word_topics: bool) -> (Vec<(usize, f64)>, Option<Vec<(usize, Vec<usize>)>>, Option<Vec<(usize, Vec<(usize, f64)>)>>) {
        let doc = doc.into_iter().map(|value| match self.topic_model.get_id(&value) {
            None => {
                WordIdOrUnknown::Unknown(value)
            }
            Some(value) => {
                WordIdOrUnknown::WordId(value)
            }
        }).collect_vec();
        self.get_doc_probability(doc, minimum_probability,minimum_phi_value, per_word_topics)
    }
}



#[cfg(test)]
mod test {
    use crate::topicmodel::enums::TopicModelVersion;
    use crate::topicmodel::topic_model::{TopicModel, TopicModelInferencer, TopicModelWithVocabulary};
    use crate::topicmodel::vocabulary::{StringVocabulary, VocabularyImpl, VocabularyMut};


    pub fn create_test_data() -> TopicModel<String, VocabularyImpl<String>> {
        let mut voc: StringVocabulary = VocabularyImpl::new();
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
    fn can_load_and_unload(){
        let topic_model = create_test_data();
        const P: &str = "test\\def";

        std::fs::create_dir("test").unwrap();
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
            r"C:\git\ldatranslation_v3\bambergdictionary\dictionaryprocessor\lda\aligned-v2\en\trained_lda\lda",
            true
        ).unwrap().0;
        println!("{}", (std::time::Instant::now() - before).as_secs());
        // model.show_10().unwrap();
        let infer = TopicModelInferencer::new(&model, 0.001, 0.1);
        let inferred = infer.get_doc_probability_for_default(vec!["hello".to_string(), "religion".to_string()], true);
        println!("{:?}", inferred.0);
    }
}