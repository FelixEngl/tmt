use serde::{Deserialize, Serialize};
use crate::model::{Importance, ImportanceRank, Position, Probability, Rank, WordTo};

/// The meta for a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallTopicMeta {
    pub stats: SmallTopicStats,
    pub by_word: WordTo<SmallWordMeta>,
}

/// The precalculated stats of a topic
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SmallTopicStats {
    pub max: Probability,
    pub min: Probability,
    pub avg: Probability,
    pub sum: Probability,
    pub max_importance: Importance
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct SmallWordMeta {
    /// The position in the topic model, starting from 0
    pub position: Position,
    /// The importance in the topic model, starting from 0
    pub importance: Importance,
}

impl SmallWordMeta {
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
