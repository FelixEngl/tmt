use std::sync::Mutex;
use evalexpr::{ContextWithMutableVariables, Value};
use crate::variable_names::{CANDIDATE_ID, SCORE, SCORE_CANDIDATE, TOPIC_ID, VOTER_ID};
use crate::voting::{VotingMethod, VotingMethodMarker, VotingResult};
use crate::voting::traits::RootVotingMethodMarker;

pub struct Spy<V: VotingMethodMarker + ?Sized> {
    spy_history: Mutex<Vec<(usize, (usize, f64, Value), Vec<(usize, f64)>)>>,
    inner: V,
}

impl<V> Spy<V> where V: VotingMethodMarker {
    pub fn new(inner: V) -> Self {
        Self { inner, spy_history: Default::default() }
    }


    pub fn spy_history(&self) -> &Mutex<Vec<(usize, (usize, f64, Value), Vec<(usize, f64)>)>> {
        &self.spy_history
    }
}

impl<V> VotingMethod for Spy<V> where V: VotingMethodMarker {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
        Ok(self.execute_with_voters(global_context, voters)?.0)
    }

    fn execute_with_voters<'a, A, B>(&self, global_context: &mut A, voters: &'a mut [B]) -> VotingResult<(Value, &'a [B])> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
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
    fn spy(self) -> Spy<Self>;
}

impl<T> IntoSpy for T where T: Sized + RootVotingMethodMarker {
    fn spy(self) -> Spy<Self> {
        Spy::new(self)
    }
}
