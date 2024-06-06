use evalexpr::{ContextWithMutableVariables, Value};
use crate::voting::display::DisplayTree;
use crate::voting::VotingResult;

/// Marks a struct as voting executable. An executable does only know a single context that can be modified.
pub(crate) trait VotingExecutable: DisplayTree {
    fn execute(&self, context: &mut impl ContextWithMutableVariables) -> VotingResult<Value>;
}
