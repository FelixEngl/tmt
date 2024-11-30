mod voting;
mod count;
mod dict_meta;
mod iter;

pub use iter::*;
pub use dict_meta::*;
pub use count::*;
use ldatranslate_topicmodel::model::WordId;
pub use voting::*;

pub trait DictionaryMetaProbabilityProvider: Send + Sync {
    fn whole_topic_model(&self) -> &SparseMetaVector;

    fn for_topic(&self, topic_id: usize) -> Option<&SparseMetaVector>;

    fn for_word_in_topic(&self, topic_id: usize, word_id: WordId) -> Option<&SparseMetaVector>;
}