use evalexpr::EvalexprError;
use thiserror::Error;
use crate::voting::aggregations::AggregationError;
use crate::voting::parser::structs::IndexOrRange;

#[derive(Debug, Error)]
pub enum VotingExpressionError {
    #[error(transparent)]
    Eval(#[from] EvalexprError),
    #[error(transparent)]
    Agg(#[from] AggregationError),
    #[error("The tuple {0} with length {2} does not have a value at {1}!")]
    TupleGet(String, IndexOrRange, usize),
    #[error("No value for working with was found!")]
    NoValue,
}
