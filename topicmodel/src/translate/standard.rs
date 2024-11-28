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

use std::marker::PhantomData;
use std::num::NonZeroUsize;
use evalexpr::{ContextWithMutableVariables, EvalexprNumericTypesConvert};
use ldatranslate_translate::{ContextExtender, ExtensionLevelKind, TopicMeta, VoterInfoProvider, VoterMeta};
use crate::translate::TranslatableTopicMatrix;
use crate::vocabulary::BasicVocabulary;

pub type StandardTopicModelLikeMatrix = Vec<Vec<f64>>;

pub struct StandardTranslateableTopicMatrix<T, Voc> {
    topic_to_voters: Vec<Vec<f64>>,
    voter_meta: Vec<Vec<StandardVoterMeta>>,
    topic_meta: Vec<StandardTopicMeta>,
    vocabulary: Voc,
    _type: PhantomData<T>
}

impl<T, Voc> VoterInfoProvider for StandardTranslateableTopicMatrix<T, Voc>
where
    Voc: Sync + Send,
    T: Sync + Send
{
    type VoterMeta = StandardVoterMeta;

    fn get_voter_meta<'a>(&'a self, column: usize, voter_id: usize) -> Option<&'a Self::VoterMeta> {
        self.voter_meta.get(column)?.get(voter_id)
    }
}

impl<T, Voc> TranslatableTopicMatrix<T, Voc> for StandardTranslateableTopicMatrix<T, Voc>
where
    Voc: Sync + Send + BasicVocabulary<T>,
    T: Sync + Send
{
    type TopicToVoterMatrix = StandardTopicModelLikeMatrix;
    type TopicMetas = Vec<StandardTopicMeta>;

    fn len(&self) -> usize {
        self.topic_to_voters.len()
    }

    fn vocabulary(&self) -> &Voc {
        &self.vocabulary
    }

    fn matrix(&self) -> &Self::TopicToVoterMatrix {
        &self.topic_to_voters
    }

    fn matrix_meta(&self) -> &Self::TopicMetas {
        &self.topic_meta
    }
}


#[derive(Copy, Clone, Debug)]
pub struct StandardVoterMeta {
    voter_id: usize,
    score: f64,
    rank: NonZeroUsize,
    importance: NonZeroUsize
}

impl StandardVoterMeta {
    #[allow(dead_code)]
    pub fn new(voter_id: usize, score: f64, rank: NonZeroUsize, importance: NonZeroUsize) -> Self {
        Self { voter_id, score, rank, importance }
    }
}

impl ContextExtender for StandardVoterMeta {
    const EXTENSION_LEVEL: ExtensionLevelKind = ExtensionLevelKind::Voter;

    fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, _: &mut impl ContextWithMutableVariables<NumericTypes=NumericTypes>) {}
}

impl VoterMeta for StandardVoterMeta {
    #[inline(always)]
    fn voter_id(&self) -> usize {
        self.voter_id
    }

    #[inline(always)]
    fn score(&self) -> f64 {
        self.score
    }

    #[inline(always)]
    fn rank(&self) -> usize {
        self.rank.get()
    }

    #[inline(always)]
    fn importance(&self) -> usize {
        self.importance.get()
    }
}


#[derive(Copy, Clone, Debug)]
pub struct StandardTopicMeta {
    topic_id: usize,
    max_score: f64,
    min_score: f64,
    avg_score: f64,
    sum_score: f64,
}

impl ContextExtender for StandardTopicMeta {
    const EXTENSION_LEVEL: ExtensionLevelKind = ExtensionLevelKind::Topic;

    fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, _: &mut impl ContextWithMutableVariables<NumericTypes=NumericTypes>){}
}

impl TopicMeta for StandardTopicMeta {
    #[inline(always)]
    fn topic_id(&self) -> usize {
        self.topic_id
    }

    #[inline(always)]
    fn max_score(&self) -> f64 {
        self.max_score
    }

    #[inline(always)]
    fn min_score(&self) -> f64 {
        self.min_score
    }

    #[inline(always)]
    fn avg_score(&self) -> f64 {
        self.avg_score
    }

    #[inline(always)]
    fn sum_score(&self) -> f64 {
        self.sum_score
    }
}