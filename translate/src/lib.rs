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

use std::ops::Deref;
use evalexpr::{ContextWithMutableVariables, EvalexprNumericTypesConvert};
use strum::{Display, EnumIs};
use rayon::prelude::*;
use rayon::prelude::IntoParallelIterator;


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[derive(Display, EnumIs)]
pub enum ExtensionLevelKind {
    Global,
    Topic,
    Voter
}

impl ExtensionLevelKind {
    pub const fn is_at_least_on(&self, other: ExtensionLevelKind) -> bool {
        match self {
            ExtensionLevelKind::Global => true,
            ExtensionLevelKind::Topic => !other.is_voter(),
            ExtensionLevelKind::Voter => other.is_voter()
        }
    }
}

/// A trait that marks an element as an extender for a topic.
pub trait ContextExtender {
    /// Mars on which level it is to be applied
    const EXTENSION_LEVEL: ExtensionLevelKind;

    /// Allows to add model specific variables.
    /// Panics when invalid variables are added.
    fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, context: &mut impl ContextWithMutableVariables<NumericTypes=NumericTypes>);
}


/// Provides some info for some voters.
pub trait VoterInfoProvider: Send + Sync {
    /// Provides the metadata to a voter
    type VoterMeta<'a>: VoterMeta + Sync + 'a where Self: 'a;

    /// Get the meta for a specific voter.
    fn get_voter_meta(&self, column: usize, voter_id: usize) -> Option<Self::VoterMeta<'_>>;
}




/// The basic representation of a matrix the represents the topic to word - probability matrix
pub trait TopicModelLikeMatrix {
    type Iter<'a>: Iterator<Item = &'a Self::TopicLike> where Self: 'a;
    type ParIter<'a>: ParallelIterator<Item = &'a Self::TopicLike> + IndexedParallelIterator where Self: 'a;

    type TopicLike: TopicLike + Send + Sync;

    /// Returns the number of topics
    fn len(&self) -> usize;

    /// Get the topic like for `topic_id`
    fn get(&self, topic_id: usize) -> Option<&Self::TopicLike>;

    /// Get the topic like for `topic_id`
    unsafe fn get_unchecked(&self, topic_id: usize) -> &Self::TopicLike;

    /// Iterate the topics.
    fn iter<'a>(&'a self) -> Self::Iter<'a>;

    /// Iterate the topics.
    fn par_iter<'a>(&'a self) -> Self::ParIter<'a>;
}


/// The basic trait for anything that resembles a topic.
pub trait TopicLike {
    /// A sync iterator over the scores
    type Iter<'a>: Iterator<Item = &'a f64> where Self: 'a;

    /// An indexed parallel iterator over the scores.
    type ParIter<'a>: ParallelIterator<Item = &'a f64> + IndexedParallelIterator where Self: 'a;

    /// Returns the number of probability like values in the topic.
    /// This is also the number of voters because the probability like values are the scores of the voters.
    fn len(&self) -> usize;

    /// Get the probability like value for a specific `voter_id`
    fn get(&self, voter_id: usize) -> Option<&f64>;
    /// Get the probability like value for a specific `voter_id`
    unsafe fn get_unchecked(&self, voter_id: usize) -> &f64;

    /// Iterate the probability like values for each voter.
    fn iter<'a>(&'a self) -> Self::Iter<'a>;

    /// Iterate the probability like values for each voter.
    fn par_iter<'a>(&'a self) -> Self::ParIter<'a>;
}



/// Provides the metas for a topic
pub trait TopicMetas {

    /// A topic meta
    type TopicMeta<'a>: TopicMeta + Send + Sync + 'a where Self: 'a;

    type Iter<'a>: Iterator<Item = Self::TopicMeta<'a>> where Self: 'a;

    type ParIter<'a>: ParallelIterator<Item = Self::TopicMeta<'a>> + IndexedParallelIterator where Self: 'a;

    /// Returns the meta for a specific topic id
    fn get<'a>(&'a self, topic_id: usize) -> Option<Self::TopicMeta<'a>>;

    /// Returns the meta for a specific topic id
    unsafe fn get_unchecked<'a>(&'a self, topic_id: usize) -> Self::TopicMeta<'a>;

    /// The number of topics
    fn len(&self) -> usize;

    fn iter<'a>(&'a self) -> Self::Iter<'a>;
    fn par_iter<'a>(&'a self) -> Self::ParIter<'a>;
}


/// The meta for a topic
pub trait TopicMeta: ContextExtender {
    /// The id of the associated topic
    fn topic_id(&self) -> usize;
    /// The maximum score in the topic of all voters.
    fn max_score(&self) -> f64;
    /// The minimum score in the topic of all voters.
    fn min_score(&self) -> f64;
    /// The average score in the topic of all voters.
    fn avg_score(&self) -> f64;
    /// The sum of all scores in the topic of all voters.
    fn sum_score(&self) -> f64;
}

/// The meta for a voter.
pub trait VoterMeta: ContextExtender {
    /// The id of the voter
    fn voter_id(&self) -> usize;

    /// The score associated to the voter in the topic.
    /// Usually a probability, but can be anything.
    fn score(&self) -> f64;

    /// The rank of this voter when voters in a topics are sorted by the scores.
    /// The rank is unique for each voter per topic and starts at 1 (highest).
    /// <div class="warning">
    /// To keep the results of the voting consistent, the order of voters with similar scores has
    /// to be similar. This means that the order of a voter has to be absolute in comparison to all
    /// other voters!
    /// </div>
    fn rank(&self) -> usize;

    /// The importance is similar to the rank. But voters with similar scores have the same
    /// importance.
    /// The importance starts at 1 (highest) and goes up for every new score.
    fn importance(&self) -> usize;
}

// impl<T, R> ContextExtender for T
// where
//     T: Deref<Target = R>,
//     R: VoterMeta
// {
//     const EXTENSION_LEVEL: ExtensionLevelKind = T::EXTENSION_LEVEL;
//
//     delegate::delegate! {
//         to self.deref() {
//             fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, context: &mut impl ContextWithMutableVariables<NumericTypes=NumericTypes>);
//         }
//     }
// }

// impl<T, R> VoterMeta for T
// where
//     T: Deref<Target = R>,
//     R: VoterMeta
// {
//     delegate::delegate! {
//         to self.deref() {
//             fn voter_id(&self) -> usize;
//             fn score(&self) -> f64;
//             fn rank(&self) -> usize;
//             fn importance(&self) -> usize;
//         }
//     }
// }


impl<T, L> TopicModelLikeMatrix for T
where
    T: Deref<Target=[L]>,
    L: TopicLike + Send + Sync + 'static
{
    type Iter<'a> = std::slice::Iter<'a, Self::TopicLike> where Self: 'a;

    type ParIter<'a> = rayon::slice::Iter<'a, Self::TopicLike> where Self: 'a;

    type TopicLike = L;

    #[inline(always)]
    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, topic_id: usize) -> Option<&Self::TopicLike> {
        <[_]>::get(self, topic_id)
    }

    unsafe fn get_unchecked(&self, topic_id: usize) -> &Self::TopicLike {
        <[_]>::get_unchecked(self, topic_id)
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        <&[_]>::into_iter(self)
    }

    fn par_iter<'a>(&'a self) -> Self::ParIter<'a> {
        <&[_]>::into_par_iter(self)
    }
}

impl<T> TopicLike for T
where
    T: Deref<Target=[f64]>
{
    type Iter<'a> = std::slice::Iter<'a, f64> where Self: 'a;
    type ParIter<'a> = rayon::slice::Iter<'a, f64> where Self: 'a;

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, voter_id: usize) -> Option<&f64> {
        <[_]>::get(self, voter_id)
    }

    unsafe fn get_unchecked(&self, voter_id: usize) -> &f64 {
        <[_]>::get_unchecked(self, voter_id)
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        <&[_]>::into_iter(self)
    }

    fn par_iter<'a>(&'a self) -> Self::ParIter<'a> {
        <&[_]>::into_par_iter(self)
    }
}