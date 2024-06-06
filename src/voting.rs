use std::fmt::{Debug, Display, Write};
use std::num::NonZeroUsize;
use evalexpr::{Context, ContextWithMutableVariables, Value};
use itertools::Itertools;
use nom::error::ParseError;
use nom::{AsChar, Compare, InputTake, InputTakeAtPosition};
use strum::VariantArray;
use crate::toolkit::evalexpr::CombineableContext;
use crate::toolkit::partial_ord_iterator::PartialOrderIterator;
use crate::translate::{NUMBER_OF_VOTERS, RANK};
pub use crate::voting::buildin::*;
pub use crate::voting::parser::structs::VotingFunction;
pub use crate::voting::errors::VotingExpressionError;

mod parser;
mod aggregations;
mod buildin;
pub mod registry;
mod walk;
pub mod display;
pub mod errors;

/// The result of a voting
pub type VotingResult<T> = Result<T, VotingExpressionError>;


/// Allows to limit the voting to the top n elements
pub trait IntoVotingWithLimit: VotingMethodMarker {
    fn with_limit(self, limit: NonZeroUsize) -> VotingWithLimit<Self>;
}

impl<T> IntoVotingWithLimit for T where T: Sized + VotingMethodMarker {
    fn with_limit(self, limit: NonZeroUsize) -> VotingWithLimit<Self> {
        VotingWithLimit::new(limit, self)
    }
}

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
}

/// A marker for methods that can be dynamically referenced without generics
pub trait VotingMethodMarker: VotingMethod {}

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

impl<T> VotingMethodMarker for Voting<T> where T: VotingMethodMarker {}

impl<T> VotingMethod for Voting<T> where T: VotingMethodMarker {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        global_context.set_value(NUMBER_OF_VOTERS.to_string(), (voters.len() as i64).into())?;
        self.expr.execute(global_context, voters)
    }
}


/// A voting with limits
pub struct VotingWithLimit<T: VotingMethodMarker + ?Sized> {
    /// The limit for the
    limit: NonZeroUsize,
    expr: T
}

impl<T> VotingWithLimit<T> where T: VotingMethodMarker {
    pub fn new(limit: NonZeroUsize, expr: T) -> Self {
        Self {
            limit,
            expr
        }
    }
}

impl<T> VotingMethodMarker for VotingWithLimit<T> where T: VotingMethodMarker {}

impl<T> VotingMethod for VotingWithLimit<T> where T: VotingMethodMarker {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        let voters = {
            let limit = self.limit.get();
            if limit < voters.len() {
                voters.sort_by_key(|value| value.get_value(RANK).unwrap().as_int().expect("Rank has to be an int!"));
                &mut voters[..limit]
            } else {
                voters
            }
        };

        global_context.set_value(NUMBER_OF_VOTERS.to_string(), (voters.len() as i64).into())?;

        self.expr.execute(global_context, voters)
    }
}





