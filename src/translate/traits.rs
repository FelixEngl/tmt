pub mod vec_ext;
pub mod standard;

use rayon::prelude::*;
use crate::topicmodel::vocabulary::BasicVocabulary;
use crate::toolkit::evalexpr::ContextExtender;

/// Provides some info for some voters.
pub trait VoterInfoProvider: Send + Sync {
    /// Provides the metadata to a voter
    type VoterMeta: VoterMeta + Sync;

    /// Get the meta for a specific voter.
    fn get_voter_meta<'a>(&'a self, column: usize, voter_id: usize) -> Option<&'a Self::VoterMeta>;
}

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
    type TopicToVoterMatrix: TopicModelLikeMatrix;

    type TopicMetas: TopicMetas;

    fn len(&self) -> usize;

    /// The vocabulary associated to this translatable matrix.
    fn vocabulary(&self) -> &Voc;

    /// The raw matrix. It is usually something like Vec<Vec<f64>>
    fn matrix(&self) -> &Self::TopicToVoterMatrix;

    /// The matrix meta. It is usually something like Vec<Vec<VoterMeta>>
    fn matrix_meta(&self) -> &Self::TopicMetas;
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
    type TopicMeta: TopicMeta + Send + Sync;

    type Iter<'a>: Iterator<Item = &'a Self::TopicMeta> where Self: 'a;
    type ParIter<'a>: ParallelIterator<Item = &'a Self::TopicMeta> + IndexedParallelIterator where Self: 'a;

    /// Returns the meta for a specific topic id
    fn get(&self, topic_id: usize) -> Option<&Self::TopicMeta>;

    /// Returns the meta for a specific topic id
    unsafe fn get_unchecked(&self, topic_id: usize) -> &Self::TopicMeta;

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
