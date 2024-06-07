use std::fmt::Write;
use std::num::NonZeroUsize;
use evalexpr::{ContextWithMutableVariables, Value};
use crate::voting::{VotingMethod, VotingResult, VotingWithLimit};
use crate::voting::display::{DisplayTree, IndentWriter};

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


impl<T> VotingMethod for Box<T> where T: VotingMethodMarker {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        self.as_ref().execute(global_context, voters)
    }
}

impl<T> VotingMethodMarker for Box<T> where T: VotingMethodMarker {}
impl<T> DisplayTree for Box<T> where T: DisplayTree {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        DisplayTree::fmt(self.as_ref(), f)
    }
}