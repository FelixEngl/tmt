use std::num::NonZeroUsize;
use crate::voting::{VotingMethod, VotingWithLimit};

pub trait RootVotingMethodMarker: VotingMethodMarker {}
pub trait LimitableVotingMethodMarker: VotingMethodMarker {}

/// A marker for methods that can be dynamically referenced without generics
pub trait VotingMethodMarker: VotingMethod + Sync + Send {}

/// Allows to limit the voting to the top n elements
pub trait IntoVotingWithLimit: LimitableVotingMethodMarker {
    fn with_limit(self, limit: NonZeroUsize) -> VotingWithLimit<Self>;
}

impl<T> IntoVotingWithLimit for T where T: Sized + LimitableVotingMethodMarker {
    fn with_limit(self, limit: NonZeroUsize) -> VotingWithLimit<Self> {
        VotingWithLimit::new(limit, self)
    }
}