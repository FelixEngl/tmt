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

use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::model::{Importance, ImportanceRank, ImportanceRankTo, Position, PositionTo, PrimitiveWordMeta, Probability, Rank, TopicId, WordId, WordTo};

/// The precalculated stats of a topic
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TopicStats {
    pub topic_id: usize,
    pub max_value: f64,
    pub min_value: f64,
    pub average_value: f64,
    pub sum_value: f64,
}




/// The meta for a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMeta {
    pub stats: TopicStats,
    pub by_words: Arc<WordTo<Arc<WordMeta>>>,
    #[serde(skip, default)]
    by_position: OnceLock<Arc<PositionTo<WordId>>>,
    #[serde(skip, default)]
    by_importance: OnceLock<Arc<ImportanceRankTo<Vec<WordId>>>>
}


impl TopicMeta {
    pub fn new(
        stats: TopicStats,
        mut by_words: WordTo<Arc<WordMeta>>
    ) -> Self {
        by_words.shrink_to_fit();
        Self {
            stats,
            by_words: Arc::new(by_words),
            by_position: OnceLock::new(),
            by_importance: OnceLock::new()
        }
    }

    pub fn by_position(&self) -> &[WordId] {
        self.by_position.get_or_init(|| {
            let mut position_to_meta: PositionTo<WordId> = (0..self.by_words.len()).collect_vec();
            position_to_meta.shrink_to_fit();
            unsafe{
                position_to_meta
                    .sort_by_key(|&word_id| self.by_words.get_unchecked(word_id).position);
            }
            Arc::new(position_to_meta)
        })
    }

    pub fn by_position_iter<'a>(&'a self) -> impl IntoIterator<Item = &'a Arc<WordMeta>> + 'a {
        self.by_position().into_iter().map(|&value| {
            unsafe{self.by_words.get_unchecked(value)}
        })
    }

    pub fn by_importance(&self) -> &[Vec<WordId>] {
        self.by_importance.get_or_init(|| {
            let mut importance_to_meta: ImportanceRankTo<_> = Vec::new();
            for value in self.by_position_iter() {
                if importance_to_meta.len() <= value.importance {
                    importance_to_meta.resize_with(
                        value.importance + 1,
                        Vec::new
                    );
                }
                unsafe{importance_to_meta.get_unchecked_mut(value.importance).push(value.word_id);}
            }
            Arc::new(importance_to_meta)
        })
    }

    pub fn by_importance_iter<'a>(&'a self) -> impl IntoIterator<Item = Vec<&'a Arc<WordMeta>>> + 'a {
        self.by_importance().into_iter().map(|value| {
            value.iter().map(|&value| {
                unsafe{self.by_words.get_unchecked(value)}
            }).collect_vec()
        })
    }
}


/// The meta for a word.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    /// Returns the [self.importance] + 1
    #[inline]
    pub fn importance_rank(&self) -> ImportanceRank {
        self.importance + 1
    }
}

impl PrimitiveWordMeta for WordMeta {
    fn word_id(&self) -> WordId {
        self.word_id
    }

    fn probability(&self) -> Probability {
        self.probability
    }

    /// Returns the [self.probability] + 1
    #[inline]
    fn rank(&self) -> Rank {
        self.position + 1
    }
}

impl Display for WordMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5}({})", self.probability, self.rank())
    }
}


/// Contains a reference to the associated word and the associated [WordMeta]
#[derive(Debug)]
pub struct WordMetaWithWord<'a, T, M> {
    pub word: &'a T,
    inner: M
}

impl<'a, T, M> WordMetaWithWord<'a, T, M> {
    pub fn new(word: &'a T, inner: M) -> Self {
        Self {
            word,
            inner
        }
    }
}

impl<'a, T, M>WordMetaWithWord<'a, T, M> {
    pub fn into_inner(self) ->M {
        self.inner
    }
}

impl<'a, T, M> Clone for WordMetaWithWord<'a, T, M> where M: Clone {
    fn clone(&self) -> Self {
        Self {
            word: self.word,
            inner: self.inner.clone()
        }
    }
}

impl<'a, T, M> Deref for WordMetaWithWord<'a, T, M>{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

