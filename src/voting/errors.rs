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

use evalexpr::EvalexprError;
use pyo3::PyErr;
use thiserror::Error;
use crate::voting::aggregations::AggregationError;
use crate::voting::constants::TMTNumericTypes;
use crate::voting::parser::voting_function::IndexOrRange;

/// Errors when parsing a voting.
#[derive(Debug, Error)]
pub enum VotingExpressionError {
    #[error(transparent)]
    Eval(#[from] EvalexprError<TMTNumericTypes>),
    #[error(transparent)]
    Agg(#[from] AggregationError),
    #[error("The tuple {0} with length {2} does not have a value at {1}!")]
    TupleGet(String, IndexOrRange, usize),
    #[error("No value for working with was found!")]
    NoValue,
    #[error(transparent)]
    PythonError(PyErr)
}
