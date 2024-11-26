use std::hash::Hash;
use std::ops::Deref;
use crate::topicmodel::model::meta::WordMeta;
use crate::topicmodel::model::{BasicTopicModel, FullTopicModel, TopicModelWithDocumentStats, TopicModelWithVocabulary};
use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary};
use crate::translate::{TranslatableTopicMatrix};
use crate::translate::{TopicMeta, TranslatableTopicMatrixWithCreate, VoterMeta, VoterInfoProvider};
use crate::topicmodel::model::meta::TopicMeta as TopicModelTopicMeta;
use evalexpr::{Context, ContextWithMutableVariables, EvalexprNumericTypesConvert};
use std::sync::Arc;
use crate::toolkit::evalexpr::{ContextExtender, ExtensionLevel};

impl<Model> VoterInfoProvider for Model where Model: BasicTopicModel {
    type VoterMeta = Arc<WordMeta>;

    #[allow(clippy::needless_lifetimes)]
    fn get_voter_meta<'a>(&'a self, column: usize, row: usize) -> Option<&'a Self::VoterMeta> {
        self.get_word_meta(column, row)
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

impl<Model, T, Voc> TranslatableTopicMatrix<T, Voc> for Model
where
    Model: TopicModelWithVocabulary<T, Voc> + TopicModelWithDocumentStats,
    Voc: BasicVocabulary<T>
{
    type TopicToVoterMatrix = Vec<Vec<f64>>;
    type TopicMetas = Vec<TopicModelTopicMeta>;

    #[inline(always)]
    fn len(&self) -> usize {
        self.k()
    }

    #[inline(always)]
    fn vocabulary(&self) -> &Voc {
        self.vocabulary()
    }

    #[inline(always)]
    fn matrix(&self) -> &Self::TopicToVoterMatrix {
        self.topics()
    }

    #[inline(always)]
    fn matrix_meta(&self) -> &Self::TopicMetas {
        self.topic_metas()
    }
}

impl ContextExtender for TopicModelTopicMeta {
    const EXTENSION_LEVEL: ExtensionLevel = ExtensionLevel::Topic;

    fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, _: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) {}
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

impl<T> ContextExtender for T where T: Deref<Target=WordMeta> {
    const EXTENSION_LEVEL: ExtensionLevel = ExtensionLevel::Voter;

    fn extend_context<NumericTypes: EvalexprNumericTypesConvert>(&self, _: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) {}
}

impl<T> VoterMeta for T where T: Deref<Target=WordMeta> {
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