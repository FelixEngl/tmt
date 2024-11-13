use std::marker::PhantomData;
use std::num::NonZeroUsize;
use evalexpr::ContextWithMutableVariables;
use crate::toolkit::evalexpr::{ContextExtender, ExtensionLevel};
use crate::topicmodel::vocabulary::BasicVocabulary;
use crate::translate::{TopicMeta, TranslatableTopicMatrix, VoterInfoProvider, VoterMeta};

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
    pub fn new(voter_id: usize, score: f64, rank: NonZeroUsize, importance: NonZeroUsize) -> Self {
        Self { voter_id, score, rank, importance }
    }
}

impl ContextExtender for StandardVoterMeta {
    const EXTENSION_LEVEL: ExtensionLevel = ExtensionLevel::Voter;

    fn extend_context(&self, _: &mut impl ContextWithMutableVariables) {}
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
    const EXTENSION_LEVEL: ExtensionLevel = ExtensionLevel::Topic;

    fn extend_context(&self, _: &mut impl ContextWithMutableVariables) {}
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