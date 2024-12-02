use std::ops::{Deref, Range};
use std::sync::Arc;
use evalexpr::{ContextWithMutableVariables, EvalexprNumericTypesConvert};
use itertools::Itertools;
use rayon::iter::{MapWith};
use rayon::prelude::*;
use rayon::range::Iter;
use ldatranslate_translate::{ContextExtender, ExtensionLevelKind, TopicMeta, TopicMetas, VoterInfoProvider};
use crate::model::{ImportanceTo, PositionTo, Probability, TopicModel, WordId};
use crate::model::meta::{TopicStats, WordMeta};
use crate::model::meta_small::{SmallTopicMeta, SmallWordMeta};


pub struct MetaViewTopicMetasIter<'a, T, V> {
    view: MetaView<'a, T, V>,
    index: Range<usize>
}

impl<'a, T, V> Iterator for  MetaViewTopicMetasIter<'a, T, V> {
    type Item = <MetaView<'a, T, V> as TopicMetas>::TopicMeta<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.view.meta_view_for(self.index.next()?)
    }
}


// pub struct MetaViewTopicMetasParIter<'a, T, V> {
//     index: Range<usize>,
//     view: MetaView<'a, T, V>,
// }
//
// impl<'a, T, V> ParallelIterator for MetaViewTopicMetasParIter<'a, T, V> {
//     type Item = TopicMetaView<'a>;
//
//     fn drive_unindexed<C>(self, consumer: C) -> C::Result
//     where
//         C: UnindexedConsumer<Self::Item>
//     {
//
//         self.iter.drive_unindexed(consumer)
//     }
// }
//
// impl<'a, T, V> IndexedParallelIterator for MetaViewTopicMetasParIter<'a, T, V> {
//     fn len(&self) -> usize {
//         self.iter.len()
//     }
//
//     fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
//         self.iter.drive(consumer)
//     }
//
//     fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
//         self.iter.with_producer(callback)
//     }
// }


#[derive(Debug, Copy)]
pub struct MetaView<'a, T, V> {
    topic_model: &'a TopicModel<T, V>
}

impl<'a, T, V> Clone for MetaView<'a, T, V> {
    fn clone(&self) -> Self {
        Self { topic_model: self.topic_model }
    }
}

impl<'a, T, V> MetaView<'a, T, V> {
    pub fn new(topic_model: &'a TopicModel<T, V>) -> Self {
        Self { topic_model }
    }
}

impl<'a, T, V> MetaView<'a, T, V> {
    pub fn meta_view_for(&self, topic_id: usize) -> Option<TopicMetaView<'a>> {
        self.topic_model.meta_view_for(topic_id)
    }

    pub fn get_meta_for(&self, topic_id: usize, word_id: usize) -> Option<WordMeta> {
        self.topic_model.get_meta_for(topic_id, word_id)
    }
}

impl<'a, T, V> VoterInfoProvider for MetaView<'a, T, V> {
    type VoterMeta<'b> = WordMeta where Self: 'b;

    fn get_voter_meta<'b>(&'b self, column: usize, voter_id: usize) -> Option<Self::VoterMeta<'b>> {
        self.get_meta_for(column, voter_id)
    }
}

impl<'a, T, V> TopicMetas for MetaView<'a, T, V>
{
    type TopicMeta<'b> = TopicMetaView<'a> where Self: 'b;
    type Iter<'b> = MetaViewTopicMetasIter<'a, T, V> where Self: 'b;
    type ParIter<'b> = MapWith<Iter<usize>, MetaView<'a, T, V>, fn(&mut MetaView<'a, T, V>, usize) -> TopicMetaView<'a>> where Self: 'b;

    fn get<'b>(&'b self, topic_id: usize) -> Option<Self::TopicMeta<'b>> {
        self.meta_view_for(topic_id)
    }

    unsafe fn get_unchecked<'b>(&'b self, topic_id: usize) -> Self::TopicMeta<'b> {
        self.topic_model.meta_view_for_unchecked(topic_id)
    }

    fn len(&self) -> usize {
        self.topic_model.topics.len()
    }

    fn iter<'b>(&'b self) -> Self::Iter<'b> {
        MetaViewTopicMetasIter {
            index: 0..self.len(),
            view: self.clone()
        }
    }

    fn par_iter<'b>(&'b self) -> Self::ParIter<'b> {
        (0..self.len()).into_par_iter().map_with(
            self.clone(),
            |k, v| {
                k.meta_view_for(v).expect("Does not fail")
            }
        )
    }
}



#[derive(Debug, Copy, Clone)]
pub struct TopicMetaView<'a> {
    pub(super) topic_id: usize,
    pub(super) stats_ref: &'a SmallTopicMeta,
    pub(super) probabilities_ref: &'a [Probability],
}

impl<'a> TopicMetaView<'a> {
    pub fn topic_id(&self) -> usize {
        self.topic_id
    }

    pub fn to_position_index(self) -> PositionIndex<'a> {
        let mut to_sort: Vec<WordId> = (0..self.probabilities_ref.len()).collect_vec();
        unsafe {
            to_sort.sort_unstable_by_key(|k|{
                self.by_word.get_unchecked(*k).position
            });
        }
        PositionIndex {
            sorted_by_pos: Arc::new(to_sort),
            view: self
        }
    }

    pub fn to_importance_index(self) -> ImportanceIndex<'a> {
        let mut importance_buckets: ImportanceTo<Vec<WordId>> = Vec::with_capacity(self.stats.max_importance + 1);
        importance_buckets.resize_with(self.stats.max_importance + 1, Vec::new);
        for (word_id, SmallWordMeta{
            importance,
            ..
        }) in self.stats_ref.by_word.iter().enumerate() {
            importance_buckets[*importance].push(word_id);
        }
        importance_buckets.par_iter_mut().for_each(|value| {
            value.sort_unstable_by_key(|word_id| unsafe {
                self.by_word.get_unchecked(*word_id).position
            })
        });
        ImportanceIndex {
            sorted_by_importance: Arc::new(importance_buckets),
            view: self
        }
    }

    pub fn topic_stats(&self) -> TopicStats {
        TopicStats {
            topic_id: self.topic_id,
            min_value: self.stats.min,
            max_value: self.stats.max,
            average_value: self.stats.avg,
            sum_value: self.stats.sum,
        }
    }

    pub fn get_word_meta(&self, word_id: WordId) -> Option<WordMeta> {
        if word_id < self.probabilities_ref.len() {
            Some(unsafe{self.get_word_meta_unchecked(word_id)})
        } else {
            None
        }
    }

    pub unsafe fn get_word_meta_unchecked(&self, word_id: WordId) -> WordMeta {
        let probability = self.probabilities_ref[word_id];
        let SmallWordMeta{
            position,
            importance
        } = self.by_word.get_unchecked(word_id);
        WordMeta {
            topic_id: self.topic_id,
            word_id,
            probability,
            position: *position,
            importance: *importance,
        }
    }
}



impl ContextExtender for TopicMetaView<'_> {
    const EXTENSION_LEVEL: ExtensionLevelKind = ExtensionLevelKind::Global;

    fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, _: &mut impl ContextWithMutableVariables<NumericTypes=NumericTypes>) {}
}

impl TopicMeta for TopicMetaView<'_> {
    fn topic_id(&self) -> usize {
        self.topic_id
    }

    fn max_score(&self) -> f64 {
        self.stats.max
    }

    fn min_score(&self) -> f64 {
        self.stats.min
    }

    fn avg_score(&self) -> f64 {
        self.stats.avg
    }

    fn sum_score(&self) -> f64 {
        self.stats.sum
    }
}

impl Deref for TopicMetaView<'_> {
    type Target = SmallTopicMeta;

    fn deref(&self) -> &Self::Target {
        self.stats_ref
    }
}


#[derive(Debug, Clone)]
pub struct PositionIndex<'a> {
    pub(super) sorted_by_pos: Arc<PositionTo<WordId>>,
    pub(super) view: TopicMetaView<'a>
}


#[derive(Debug, Clone)]
pub struct ImportanceIndex<'a> {
    pub(super) sorted_by_importance: Arc<ImportanceTo<Vec<WordId>>>,
    pub(super) view: TopicMetaView<'a>
}