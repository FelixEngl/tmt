//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::num::NonZeroUsize;
use evalexpr::{Context, ContextWithMutableVariables, EvalexprError, EvalexprNumericTypesConvert, IterateVariablesContext, Value};
use crate::variable_provider::variable_names::{BOOST_SCORE, NUMBER_OF_VOTERS, RANK};
pub use crate::buildin::*;
pub use crate::constants::TMTNumericTypes;
use crate::display::{DisplayTree, IndentWriter};
pub use crate::parser::voting_function::VotingFunction;
pub use crate::errors::VotingExpressionError;
use crate::traits::{RootVotingMethodMarker, VotingMethodMarker};
use crate::VotingExpressionError::Eval;

pub mod variable_provider;
pub mod parser;
mod aggregations;
pub mod buildin;
pub mod registry;
mod walk;
pub mod display;
pub mod errors;
pub mod spy;
pub mod traits;
pub mod py;
pub mod constants;
pub mod interners;

/// The result of a voting
pub type VotingResult<T> = Result<T, VotingExpressionError>;


pub trait VotingContext<NumericTypes: EvalexprNumericTypesConvert = TMTNumericTypes>: Context<NumericTypes=NumericTypes> {
    fn get_vote_value(&self, name: &str) -> VotingResult<&Value<NumericTypes>> {
        if name == BOOST_SCORE {
            return match self.get_boost(BOOST_SCORE) {
                None => {
                    Ok(&Value::Empty)
                }
                Some(value) => {
                    Ok(value)
                }
            }
        }
        if let Some(found) = self.get_value(name) {
            Ok(found)
        } else {
            Err(Eval(EvalexprError::VariableIdentifierNotFound(name.to_string())))
        }
    }

    /// Resolves a boost variable with a given ``name``
    fn get_boost(&self, name: &str) -> Option<&Value<NumericTypes>> {
        let mut target_value = self.get_value(name)?;

        loop {
            target_value = match target_value {
                old @ Value::String(targ) => {
                    match self.get_value(targ) {
                        None => {
                            return Some(old)
                        }
                        Some(value) => {
                            value
                        }
                    }
                }
                value => return Some(value),
            };
        }
    }
}

impl<T, NumericTypes: EvalexprNumericTypesConvert> VotingContext<NumericTypes> for T where T: Context<NumericTypes=NumericTypes> {}


/// A voting method context allows to create a variable map to something that can me handled by python.
pub trait VotingMethodContext : ContextWithMutableVariables<NumericTypes=TMTNumericTypes> {
    fn variable_map(&self) -> HashMap<String, Value>;
}
impl<T> VotingMethodContext for T where T: ContextWithMutableVariables<NumericTypes=TMTNumericTypes> + IterateVariablesContext {
    fn variable_map(&self) -> HashMap<String, Value> {
        self.iter_variables().collect()
    }
}

/// Marks a struct as voting method.
pub trait VotingMethod{
    #[inline]
    fn execute_to_f64<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<f64>
    where
        A : VotingMethodContext,
        B : VotingMethodContext
    {
        Ok(self.execute(global_context, voters)?.as_number()?)
    }

    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<TMTNumericTypes>>
    where
        A : VotingMethodContext,
        B : VotingMethodContext;

    // #[inline]
    // fn execute_to_f64_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(f64, &'a [B])>
    //     where
    //         A : ContextWithMutableVariables,
    //         B : ContextWithMutableVariables
    // {
    //     let (result, voters) = self.execute_with_voters(global_context,voters)?;
    //     Ok((result.as_number()?, voters))
    // }

    fn execute_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(Value<TMTNumericTypes>, &'a [B])>
    where
        A : VotingMethodContext,
        B : VotingMethodContext {
        Ok((self.execute(global_context, voters)?, voters))
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

    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<TMTNumericTypes>>
    where
        A : VotingMethodContext,
        B : VotingMethodContext
    {
        let voters = self.slice_voters(voters, |value| value.get_value(RANK).unwrap().as_int().expect("Rank has to be an int!"));
        assert!(voters.len() <= self.limit.get());
        global_context.set_value(NUMBER_OF_VOTERS.to_string(), Value::from_as_int(voters.len()))?;
        self.expr.execute(global_context, voters)
    }

    fn execute_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(Value<TMTNumericTypes>, &'a [B])>
    where
        A: VotingMethodContext,
        B: VotingMethodContext
    {
        let voters = self.slice_voters(voters, |value| value.get_value(RANK).unwrap().as_int().expect("Rank has to be an int!"));
        assert!(voters.len() <= self.limit.get());
        global_context.set_value(NUMBER_OF_VOTERS.to_string(), Value::from_as_int(voters.len()))?;
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



