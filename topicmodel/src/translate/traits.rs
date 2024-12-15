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

use std::hash::Hash;
use evalexpr::{ContextWithMutableVariables, EvalexprNumericTypesConvert};
use ldatranslate_translate::{ContextExtender, ExtensionLevelKind, TopicMeta, TopicMetas, TopicModelLikeMatrix, VoterInfoProvider, VoterMeta};
use crate::model::{BasicTopicModel, FullTopicModel, TopicModelWithDocumentStats, TopicModelWithVocabulary};
use crate::vocabulary::{BasicVocabulary, SearchableVocabulary};
use crate::model::meta::{TopicMeta as TopicModelTopicMeta, WordMeta};

/// A matrix that associates voters to topics.
///
///
/// In regard to a topic model:
/// - The [TranslatableTopicMatrix] is the whole topic model.
///   - The topic-to-word-probability matrix is the [TopicModelLikeMatrix]
///
pub trait TranslatableTopicMatrix<T, Voc>: VoterInfoProvider
where
    Voc: BasicVocabulary<T>
{
    type TopicToVoterMatrix<'a>: TopicModelLikeMatrix + 'a where Self: 'a;

    type TopicMetas<'a>: TopicMetas + 'a where Self: 'a;

    /// The number of topics
    fn len(&self) -> usize;

    /// The vocabulary associated to this translatable matrix.
    fn vocabulary(&self) -> &Voc;

    /// The raw matrix. It is usually something like Vec<Vec<f64>>
    fn matrix<'a>(&'a self) -> Self::TopicToVoterMatrix<'a>;

    /// The matrix meta. It is usually something like Vec<Vec<VoterMeta>>
    fn matrix_meta<'a>(&'a self) -> Self::TopicMetas<'a>;
}


/// A topic matrix that is able to be reconstructed by some primitive data and a self reference to the original.
pub trait TranslatableTopicMatrixWithCreate<T, Voc>: TranslatableTopicMatrix<T, Voc> where Voc: BasicVocabulary<T> {
    fn create_new_from(
        topic_to_voter_probability_matrix: Vec<Vec<f64>>,
        associated_vocabulary: Voc,
        used_vocabulary_frequencies: Vec<u64>,
        original: &Self
    ) -> Self where Self: Sized;
}

impl<Model, T, Voc> TranslatableTopicMatrix<T, Voc> for Model
where
    Model: TopicModelWithVocabulary<T, Voc> + TopicModelWithDocumentStats,
    Voc: BasicVocabulary<T>,
    for<'a> <Model as BasicTopicModel>::TopicMetas<'a>: TopicMetas
{
    type TopicToVoterMatrix<'a> = &'a [Vec<f64>] where Self: 'a;
    type TopicMetas<'a> = <Self as BasicTopicModel>::TopicMetas<'a> where Self: 'a;

    #[inline(always)]
    fn len(&self) -> usize {
        self.k()
    }

    #[inline(always)]
    fn vocabulary(&self) -> &Voc {
        self.vocabulary()
    }

    #[inline(always)]
    fn matrix<'a>(&'a self) -> Self::TopicToVoterMatrix<'a> {
        self.topics().as_slice()
    }

    #[inline(always)]
    fn matrix_meta<'a>(&'a self) -> Self::TopicMetas<'a> {
        self.topic_metas()
    }
}


impl<Model, T, Voc> TranslatableTopicMatrixWithCreate<T, Voc> for Model
where
    Model: FullTopicModel<T, Voc> + TranslatableTopicMatrix<T, Voc> + TopicModelWithDocumentStats,
    T: Hash + Eq + Ord,
    Voc: SearchableVocabulary<T>
{
    fn create_new_from(
        topic_to_voter_probability_matrix: Vec<Vec<f64>>,
        associated_vocabulary: Voc,
        used_vocabulary_frequencies: Vec<u64>,
        old_model: &Self
    ) -> Self
    where
        Self: Sized
    {
        let mut new = Model::new(
            topic_to_voter_probability_matrix,
            associated_vocabulary,
            used_vocabulary_frequencies,
            old_model.doc_topic_distributions().clone(),
            old_model.document_lengths().clone()
        );
        new.normalize_in_place();
        new
    }
}

impl ContextExtender for TopicModelTopicMeta {
    const EXTENSION_LEVEL: ExtensionLevelKind = ExtensionLevelKind::Topic;

    fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, _: &mut impl ContextWithMutableVariables<NumericTypes=NumericTypes>) {}
}

impl TopicMeta for TopicModelTopicMeta {
    #[inline(always)]
    fn topic_id(&self) -> usize {
        self.stats.topic_id
    }

    #[inline(always)]
    fn max_score(&self) -> f64 {
        self.stats.max_value
    }

    #[inline(always)]
    fn min_score(&self) -> f64 {
        self.stats.min_value
    }

    #[inline(always)]
    fn avg_score(&self) -> f64 {
        self.stats.average_value
    }

    #[inline(always)]
    fn sum_score(&self) -> f64 {
        self.stats.sum_value
    }
}

impl ContextExtender for WordMeta {
    const EXTENSION_LEVEL: ExtensionLevelKind = ExtensionLevelKind::Voter;

    fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, _: &mut impl ContextWithMutableVariables<NumericTypes=NumericTypes>) {}
}

impl VoterMeta for WordMeta {
    #[inline(always)]
    fn voter_id(&self) -> usize {
        self.word_id
    }

    #[inline(always)]
    fn score(&self) -> f64 {
        self.probability
    }

    #[inline(always)]
    fn rank(&self) -> usize {
        self.position + 1
    }

    #[inline(always)]
    fn importance(&self) -> usize {
        self.importance + 1
    }
}