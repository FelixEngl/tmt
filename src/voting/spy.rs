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

use std::sync::Mutex;
use evalexpr::{Value};
use crate::variable_provider::variable_names::{CANDIDATE_ID, SCORE, SCORE_CANDIDATE, TOPIC_ID, VOTER_ID};
use crate::voting::{VotingMethod, VotingMethodContext, VotingMethodMarker, VotingResult};
use crate::voting::constants::TMTNumericTypes;
use crate::voting::traits::RootVotingMethodMarker;

/// Allows to spy on the voting method
pub struct Spy<V: VotingMethodMarker + ?Sized> {
    spy_history: Mutex<Vec<(usize, (usize, f64, Value), Vec<(usize, f64)>)>>,
    inner: V,
}

impl<V> Spy<V> where V: VotingMethodMarker {
    pub fn new(inner: V) -> Self {
        Self { inner, spy_history: Default::default() }
    }

    #[allow(dead_code)]
    pub fn spy_history(&self) -> &Mutex<Vec<(usize, (usize, f64, Value), Vec<(usize, f64)>)>> {
        &self.spy_history
    }
}

impl<V> VotingMethod for Spy<V> where V: VotingMethodMarker {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<TMTNumericTypes>>
    where
        A : VotingMethodContext,
        B : VotingMethodContext
    {
        Ok(self.execute_with_voters(global_context, voters)?.0)
    }

    fn execute_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(Value<TMTNumericTypes>, &'a [B])>
    where
        A: VotingMethodContext,
        B: VotingMethodContext
    {
        let (result, voters) = self.inner.execute_with_voters(global_context, voters)?;

        let entry = (
            global_context.get_value(TOPIC_ID).unwrap().as_int()? as usize,
            (
                global_context.get_value(CANDIDATE_ID).unwrap().as_int()? as usize,
                global_context.get_value(SCORE_CANDIDATE).unwrap().as_number()?,
                result.clone()
            ),
            voters.iter().map(|value| {
                (
                    value.get_value(VOTER_ID).unwrap().as_int().unwrap() as usize,
                    value.get_value(SCORE).unwrap().as_number().unwrap(),
                )
            }).collect()
        );

        let mut lock = self.spy_history.lock().unwrap();
        lock.push(entry);
        drop(lock);

        return Ok((result, voters))
    }
}

impl<V> VotingMethodMarker for Spy<V> where V: VotingMethodMarker{}

/// Allows to limit the voting to the top n elements
pub trait IntoSpy: RootVotingMethodMarker {
    #[allow(dead_code)]
    fn spy(self) -> Spy<Self>;
}

impl<T> IntoSpy for T where T: Sized + RootVotingMethodMarker {
    fn spy(self) -> Spy<Self> {
        Spy::new(self)
    }
}
