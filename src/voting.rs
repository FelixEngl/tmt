use std::fmt::{Display, Formatter, Write};
use std::num::NonZeroUsize;
use evalexpr::{ContextWithMutableVariables, Value};
use nom::Parser;
use nom::error::ParseError;
use crate::variable_names::{NUMBER_OF_VOTERS, RANK};
pub use crate::voting::buildin::*;
use crate::voting::display::{DisplayTree, IndentWriter};
pub use crate::voting::parser::voting_function::VotingFunction;
pub use crate::voting::errors::VotingExpressionError;
use crate::voting::traits::{LimitableVotingMethodMarker, RootVotingMethodMarker, VotingMethodMarker};

pub(crate) mod parser;
mod aggregations;
pub mod buildin;
pub mod registry;
mod walk;
pub mod display;
pub mod errors;
pub mod spy;
pub mod traits;

/// The result of a voting
pub type VotingResult<T> = Result<T, VotingExpressionError>;

/// Marks a struct as voting method.
pub trait VotingMethod {
    #[inline]
    fn execute_to_f64<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<f64>
        where
            A : ContextWithMutableVariables,
            B : ContextWithMutableVariables
    {
        Ok(self.execute(global_context,voters)?.as_number()?)
    }

    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value>
        where
            A : ContextWithMutableVariables,
            B : ContextWithMutableVariables;

    #[inline]
    fn execute_to_f64_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(f64, &'a [B])>
        where
            A : ContextWithMutableVariables,
            B : ContextWithMutableVariables
    {
        let (result, voters) = self.execute_with_voters(global_context,voters)?;
        Ok((result.as_number()?, voters))
    }

    fn execute_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(Value, &'a [B])>
        where
            A : ContextWithMutableVariables,
            B : ContextWithMutableVariables {
        Ok((self.execute(global_context, voters)?, voters))
    }
}


/// A normal voting without limits
#[repr(transparent)]
pub struct Voting<T: VotingMethodMarker + ?Sized> {
    expr: T
}
impl<T> Voting<T> where T: VotingMethodMarker {
    #[inline(always)]
    pub fn new(expr: T) -> Self {
        Self { expr }
    }
}
impl<T> RootVotingMethodMarker for Voting<T> where T: VotingMethodMarker {}
impl<T> LimitableVotingMethodMarker for Voting<T> where T: VotingMethodMarker {}
impl<T> VotingMethodMarker for Voting<T> where T: VotingMethodMarker {}
impl<T> VotingMethod for Voting<T> where T: VotingMethodMarker {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        global_context.set_value(NUMBER_OF_VOTERS.to_string(), (voters.len() as i64).into())?;
        self.expr.execute(global_context, voters)
    }

    fn execute_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(Value, &'a [B])> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        global_context.set_value(NUMBER_OF_VOTERS.to_string(), (voters.len() as i64).into())?;
        self.expr.execute_with_voters(global_context, voters)
    }
}


/// A voting with limits
#[derive(Debug, Clone)]
pub struct VotingWithLimit<T: ?Sized> {
    /// The limit for the votes
    limit: NonZeroUsize,
    expr: T
}
impl<T> VotingWithLimit<T> {
    pub fn new(limit: NonZeroUsize, expr: T) -> Self {
        Self {
            limit,
            expr
        }
    }

    fn slice_voters<'a, B, K, F>(&self, voters: &'a mut [B], key_provider: F) -> &'a mut [B] where F: FnMut(&B) -> K, K: Ord {
        let limit = self.limit.get();
        if limit < voters.len() {
            voters.sort_by_key(key_provider);
            &mut voters[..limit]
        } else {
            voters
        }
    }
}
impl<T> RootVotingMethodMarker for VotingWithLimit<T> where T: VotingMethodMarker {}
impl<T> VotingMethodMarker for VotingWithLimit<T> where T: VotingMethodMarker {}
impl<T> VotingMethod for VotingWithLimit<T> where T: VotingMethodMarker {

    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        let voters = self.slice_voters(voters, |value| value.get_value(RANK).unwrap().as_int().expect("Rank has to be an int!"));
        assert!(voters.len() <= self.limit.get());
        global_context.set_value(NUMBER_OF_VOTERS.to_string(), (voters.len() as i64).into())?;
        self.expr.execute(global_context, voters)
    }

    fn execute_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(Value, &'a [B])> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        let voters = self.slice_voters(voters, |value| value.get_value(RANK).unwrap().as_int().expect("Rank has to be an int!"));;
        assert!(voters.len() <= self.limit.get());
        global_context.set_value(NUMBER_OF_VOTERS.to_string(), (voters.len() as i64).into())?;
        self.expr.execute_with_voters(global_context, voters)
    }
}

impl<T> DisplayTree for VotingWithLimit<T> where T: DisplayTree {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        DisplayTree::fmt(&self.expr, f)?;
        write!(f, "({})", self.limit.get())
    }
}

impl<T> Display for VotingWithLimit<T> where T: DisplayTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut code_formatter = IndentWriter::new(f);
        DisplayTree::fmt(self, &mut code_formatter)
    }
}



