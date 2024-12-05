mod voting;
mod count;
mod dict_meta;
mod iter;
mod count_weighted;
pub mod topic_associated;
mod dictionary_association;

use ndarray::{ArrayBase, Data, Dimension};
use num::{Float, FromPrimitive};
pub use dict_meta::*;
pub use count::*;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictionaryMetaIndex;
use ldatranslate_topicmodel::model::WordId;

pub trait HorizontalDictionaryMetaProbabilityProvider: Send + Sync {
    fn whole_topic_model(&self) -> &SparseMetaVector;

    fn for_topic(&self, topic_id: usize) -> Option<&SparseMetaVector>;

    fn for_word_in_topic(&self, topic_id: usize, word_id: WordId) -> Option<&SparseMetaVector>;
}

pub trait VerticalDictionaryMetaProbabilityProvider: Send + Sync {
    fn whole_topic_model<T>(&self, idx: T) -> &SparseMetaVector
    where
        T: DictionaryMetaIndex + Copy + Clone;

    fn for_topic<T>(&self, topic_id: usize, idx: T) -> Option<&SparseMetaVector>
    where
        T: DictionaryMetaIndex + Copy + Clone;
}

pub trait Similarity {
    type Error<A>: std::error::Error;
    fn calculate<S1, S2, A, D>(
        &self,
        p: &ArrayBase<S1, D>,
        q: &ArrayBase<S2, D>,
    ) -> Result<A, Self::Error<A>>
    where
        S1: Data<Elem = A>,
        S2: Data<Elem = A>,
        D: Dimension,
        A: Float + FromPrimitive;
}