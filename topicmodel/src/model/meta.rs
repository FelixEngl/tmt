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
use std::sync::Arc;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, SerializeStruct};

use crate::model::{Importance, ImportanceRank, ImportanceRankTo, Position, PositionTo, Probability, Rank, TopicId, WordId, WordTo};

/// The precalculated stats of a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStats {
    pub topic_id: usize,
    pub max_value: f64,
    pub min_value: f64,
    pub average_value: f64,
    pub sum_value: f64,
}


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

    fn new_with(stats: TopicStats, by_words: WordTo<Arc<WordMeta>>) -> TopicMeta {
        let mut position_to_meta: PositionTo<Arc<WordMeta>> = by_words.clone();
        position_to_meta.sort_by_key(|value| value.position);

        let mut importance_to_meta: ImportanceRankTo<_> = Vec::new();

        for value in position_to_meta.iter() {
            while importance_to_meta.len() <= value.importance {
                importance_to_meta.push(Vec::new())
            }
            unsafe{importance_to_meta.get_unchecked_mut(value.importance).push(value.clone());}
        }

        TopicMeta::new(
            stats,
            by_words,
            position_to_meta,
            importance_to_meta
        )
    }
}

impl Serialize for TopicMeta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        if serializer.is_human_readable() {
            let mut ser = serializer.serialize_struct("TopicMeta", 2)?;
            ser.serialize_field("stats", &self.stats)?;
            ser.serialize_field("bywords", &self.by_words)?;
            ser.end()
        } else {
            let mut ser = serializer.serialize_seq(Some(2))?;
            ser.serialize_element(&self.stats)?;
            ser.serialize_element(&self.by_words)?;
            ser.end()
        }
    }
}


impl<'de> Deserialize<'de> for TopicMeta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct TopicMetaVisitor;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Stats, ByWords }

        impl<'de> Visitor<'de> for TopicMetaVisitor {
            type Value = TopicMeta;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a TopicMeta")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let stats_field = seq.next_element()?.ok_or_else(|| de::Error::missing_field("stats"))?;
                let by_words_field: Vec<Arc<WordMeta>> = seq.next_element()?.ok_or_else(|| de::Error::missing_field("bywords"))?;
                let mut position_to_meta: PositionTo<Arc<WordMeta>> = by_words_field.clone();
                position_to_meta.sort_by_key(|value| value.position);

                let mut importance_to_meta: ImportanceRankTo<_> = Vec::new();

                for value in position_to_meta.iter() {
                    while importance_to_meta.len() <= value.importance {
                        importance_to_meta.push(Vec::new())
                    }
                    unsafe{importance_to_meta.get_unchecked_mut(value.importance).push(value.clone());}
                }

                Ok(
                    TopicMeta::new(
                        stats_field,
                        by_words_field,
                        position_to_meta,
                        importance_to_meta
                    )
                )
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut stats_field = None;
                let mut by_words_field = None;
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Stats => {
                            if stats_field.is_some() {
                                return Err(de::Error::duplicate_field("stats"));
                            }
                            stats_field = Some(map.next_value::<TopicStats>()?);
                        }
                        Field::ByWords => {
                            if by_words_field.is_some() {
                                return Err(de::Error::duplicate_field("bywords"));
                            }
                            by_words_field = Some(map.next_value::<WordTo<Arc<WordMeta>>>()?)
                        }
                    }
                }
                let stats_field = stats_field.ok_or_else(|| de::Error::missing_field("stats"))?;
                let by_words_field = by_words_field.ok_or_else(|| de::Error::missing_field("bywords"))?;

                Ok(TopicMeta::new_with(stats_field, by_words_field))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_struct(
                "TopicMeta",
                &["stats", "bywords"],
                TopicMetaVisitor
            )
        } else {
            deserializer.deserialize_seq(TopicMetaVisitor)
        }


    }
}

/// The meta for a word.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

