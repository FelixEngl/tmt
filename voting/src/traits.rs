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

use std::fmt::Write;
use std::num::NonZeroUsize;
use evalexpr::{EvalexprNumericTypesConvert, Value};
use crate::{VotingMethod, VotingMethodContext, VotingResult, VotingWithLimit};
use crate::constants::TMTNumericTypes;
use crate::display::{DisplayTree, IndentWriter};

pub trait RootVotingMethodMarker: VotingMethodMarker {}
pub trait LimitableVotingMethodMarker: VotingMethodMarker {}

/// A marker for methods that can be dynamically referenced without generics
pub trait VotingMethodMarker: VotingMethod + Sync + Send {}

/// Allows to limit the voting to the top n elements
pub trait IntoVotingWithLimit<NumericTypes: EvalexprNumericTypesConvert = TMTNumericTypes>: LimitableVotingMethodMarker {
    #[allow(dead_code)]
    fn with_limit(self, limit: NonZeroUsize) -> VotingWithLimit<Self>;
}

impl<T> IntoVotingWithLimit for T where T: Sized + LimitableVotingMethodMarker {
    fn with_limit(self, limit: NonZeroUsize) -> VotingWithLimit<Self> {
        VotingWithLimit::new(limit, self)
    }
}


impl<T> VotingMethod for Box<T> where T: VotingMethodMarker {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<TMTNumericTypes>>
    where
        A: VotingMethodContext,
        B: VotingMethodContext {
        self.as_ref().execute(global_context, voters)
    }
}

impl<T> VotingMethodMarker for Box<T> where T: VotingMethodMarker {}
impl<T> DisplayTree for Box<T> where T: DisplayTree {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        DisplayTree::fmt(self.as_ref(), f)
    }
}