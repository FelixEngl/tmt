use std::cmp::Ordering;
use crate::translate::language::LanguageOrigin;

#[derive(Debug, Clone, Copy)]
pub(super) struct Candidate {
    pub(super) candidate_word_id: LanguageOrigin<usize>,
    pub(super) relative_score: f64,
    _origin_word_id: usize
}


impl Candidate {
    pub fn new(
        candidate_word_id: LanguageOrigin<usize>,
        relative_score: f64,
        _origin_word_id: usize,
    ) -> Self {
        Self {
            candidate_word_id,
            relative_score,
            _origin_word_id
        }
    }
}

impl PartialEq<Self> for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.relative_score == other.relative_score
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        f64::partial_cmp(&other.relative_score, &self.relative_score)
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        f64::total_cmp(&other.relative_score, &self.relative_score)
    }
}
