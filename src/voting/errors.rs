use evalexpr::EvalexprError;
use pyo3::PyErr;
use thiserror::Error;
use crate::voting::aggregations::AggregationError;
use crate::voting::parser::voting_function::IndexOrRange;

/// Errors when parsing a voting.
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
    #[error(transparent)]
    PythonError(PyErr)
}
