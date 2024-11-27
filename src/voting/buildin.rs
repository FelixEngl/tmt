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
use evalexpr::{Context, EvalexprError, EvalexprResult, Value};
use itertools::Itertools;
use strum::{Display, EnumString, IntoStaticStr, VariantArray};
use crate::toolkit::partial_ord_iterator::PartialOrderIterator;
use crate::variable_provider::variable_names::{EPSILON, NUMBER_OF_VOTERS, RECIPROCAL_RANK, SCORE, SCORE_CANDIDATE};
use crate::voting::{VotingContext, VotingMethod, VotingMethodContext, VotingMethodMarker, VotingResult, VotingWithLimit};
use crate::voting::aggregations::{Aggregation, AggregationError};
use crate::voting::aggregations::AggregationKind::{AvgOf, GAvgOf, SumOf};
use crate::voting::display::{DisplayTree, IndentWriter};
use crate::voting::traits::{LimitableVotingMethodMarker, RootVotingMethodMarker};
use crate::voting::VotingExpressionError::{NoValue};
use pyo3::{pyclass, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use serde::{Deserialize, Serialize};
use crate::py::voting::PyVoting;
use crate::register_python;
use crate::voting::constants::TMTNumericTypes;
use crate::voting::parser::InterpretedVoting::Limited;
use crate::voting::py::{PyContextWithMutableVariables, PyExprValue};

/// An empty voting method if nothing works
pub struct EmptyVotingMethod;

impl LimitableVotingMethodMarker for EmptyVotingMethod {}
impl VotingMethodMarker for EmptyVotingMethod {}

impl VotingMethod for EmptyVotingMethod {
    fn execute<A, B>(&self, _: &mut A, _: &mut [B]) -> VotingResult<Value> where A: VotingMethodContext, B: VotingMethodContext {
        Err(NoValue)
    }
}

/// All possible buildin votings
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, EnumString, IntoStaticStr, Display, VariantArray, Serialize, Deserialize)]
pub enum BuildInVoting {
    OriginalScore,
    Voters,
    CombSum,
    GCombSum,
    CombSumTop,
    CombSumPow2,
    CombMax,
    RR,
    RRPow2,
    CombSumRR,
    CombSumRRPow2,
    CombSumPow2RR,
    CombSumPow2RRPow2,
    ExpCombMnz,
    WCombSum,
    WCombSumG,
    WGCombSum,
    PCombSum
}

// TODO: Causes Panic
// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl BuildInVoting {
    pub fn limit(&self, limit: usize) -> PyResult<PyVoting> {
        if let Some(limit) = NonZeroUsize::new(limit) {
            Ok(
                PyVoting::from(
                    Limited(
                        VotingWithLimit::new(
                            limit,
                            Box::new((*self).into())
                        )
                    )
                )
            )
        } else {
            Err(PyValueError::new_err("Limit has to be greater than 0!".to_string()))
        }
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }

    #[staticmethod]
    #[pyo3(name="from_string")]
    pub fn from_string_py(s: &str) -> PyResult<BuildInVoting> {
        s.try_into().map_err(|value: strum::ParseError| PyValueError::new_err(value.to_string()))
    }

    //noinspection DuplicatedCode
    pub fn __call__(&self, mut global_context: PyContextWithMutableVariables, mut voters: Vec<PyContextWithMutableVariables>) -> PyResult<(PyExprValue, Vec<PyContextWithMutableVariables>)>{
        let used_voters= voters.as_mut_slice();
        match self.execute(&mut global_context, used_voters) {
            Ok(value) => {
                Ok((value.into(), used_voters.iter().cloned().collect()))
            }
            Err(err) => {
                Err(PyValueError::new_err(err.to_string()))
            }
        }
    }

    pub fn __reduce__(&self) -> String {
        format!("BuildInVoting.{self}")
    }
}

impl RootVotingMethodMarker for BuildInVoting {}
impl LimitableVotingMethodMarker for BuildInVoting {}
impl VotingMethodMarker for BuildInVoting {}

impl VotingMethod for BuildInVoting {

    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<TMTNumericTypes>>
    where
        A: VotingMethodContext,
        B: VotingMethodContext
    {

        fn collect_simple<B, F>(voters: &[B], f: F) -> VotingResult<Vec<f64>>
        where
            B: Context<NumericTypes=TMTNumericTypes>,
            F: Fn(&B) -> VotingResult<Value<TMTNumericTypes>>
        {
            Ok(voters.iter().map(f).map_ok(|value: Value| value.as_number()).collect::<VotingResult<EvalexprResult<Vec<_>>>>()??)
        }

        match self {
            BuildInVoting::OriginalScore => {
                global_context.get_vote_value(SCORE_CANDIDATE).cloned()
            }
            BuildInVoting::Voters => {
                global_context.get_vote_value(NUMBER_OF_VOTERS).cloned()
            }
            BuildInVoting::CombSum => {
                let calculated = collect_simple(voters, |value| value.get_vote_value(SCORE).cloned())?;
                Ok(Value::from_as_float(SumOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::GCombSum => {
                let calculated = collect_simple(voters, |value| value.get_vote_value(SCORE).cloned())?;
                Ok(Value::from_as_float(GAvgOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::CombSumTop => {
                let calculated = collect_simple(voters, |value| value.get_vote_value(SCORE).cloned())?;
                Ok(Value::from_as_float(Aggregation::new(SumOf, NonZeroUsize::new(2)).calculate_desc(calculated.into_iter())?))
            }
            BuildInVoting::CombSumPow2 => {
                let calculated = collect_simple(voters, |value| Ok(Value::from_float(value.get_vote_value(SCORE)?.as_number()?.powi(2))))?;
                Ok(Value::from_as_float(SumOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::CombMax => {
                Ok(
                    Value::from_as_float(
                        collect_simple(voters, |value| value.get_vote_value(SCORE).cloned())?
                            .iter()
                            .max_partial()
                            .map_err(|_| AggregationError::NoMaxFound)?
                            .expect("Expected a number for score in CombMax")
                            .clone()
                    )
                )
            }
            BuildInVoting::RR => {
                let calculated = collect_simple(voters, |value| value.get_vote_value(RECIPROCAL_RANK).cloned())?;
                Ok(Value::from_as_float(SumOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::RRPow2 => {
                let calculated = collect_simple(voters, |value| Ok(
                    Value::from_as_float(value.get_vote_value(RECIPROCAL_RANK)?.as_number().expect("Expected a number for rr in RRPow2").powi(2))
                ))?;
                Ok(Value::from_as_float(SumOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::CombSumRR => {
                let calculated = collect_simple(voters, |value| {
                    let score = value.get_vote_value(SCORE)?.as_number().expect("Expected a number for score in CombSumRR");
                    let rr = value.get_vote_value(RECIPROCAL_RANK)?.as_number().expect("Expected a number for rr in CombSumRR");
                    Ok(Value::Float(score * rr))
                })?;
                Ok(Value::from_as_float(SumOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::CombSumRRPow2 => {
                let calculated = collect_simple(voters, |value|{
                    let score = value.get_vote_value(SCORE)?.as_number().expect("Expected a number for score in CombSumRRPow2");
                    let rr = value.get_vote_value(RECIPROCAL_RANK)?.as_number().expect("Expected a number for rr in CombSumRRPow2");
                    Ok(Value::Float(score * rr.powi(2)))
                })?;
                Ok(Value::from_as_float(SumOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::CombSumPow2RR => {
                let calculated = collect_simple(voters, |value|{
                    let score = value.get_vote_value(SCORE)?.as_number().expect("Expected a number for score in CombSumPow2RR");
                    let rr = value.get_vote_value(RECIPROCAL_RANK)?.as_number().expect("Expected a number for rr in CombSumPow2RR");
                    Ok(Value::Float(score.powi(2) * rr))
                })?;
                Ok(Value::from_as_float(SumOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::CombSumPow2RRPow2 => {
                let calculated = collect_simple(voters, |value| {
                    let score = value.get_vote_value(SCORE)?.as_number().expect("Expected a number for score in CombSumPow2RRPow2");
                    let rr = value.get_vote_value(RECIPROCAL_RANK)?.as_number().expect("Expected a number for rr in CombSumPow2RRPow2");
                    Ok(Value::Float(score.powi(2) * rr.powi(2)))
                })?;
                Ok(Value::from_as_float(SumOf.aggregate(calculated.into_iter())?))
            }
            BuildInVoting::ExpCombMnz => {
                let n_voters = global_context.get_vote_value(NUMBER_OF_VOTERS)?.as_int()?;
                Ok(Value::from_as_float(BuildInVoting::CombSumPow2.execute(global_context, voters)?.as_number()? + (n_voters as f64)))
            }
            BuildInVoting::WCombSum => {
                let calculated = collect_simple(voters, |value| value.get_vote_value(SCORE).cloned())?;
                let trans = SumOf.aggregate(calculated.iter().copied())?;
                let trans_avg = AvgOf.aggregate(calculated.into_iter())?;
                let n_voters = global_context.get_vote_value(NUMBER_OF_VOTERS)?.as_int()?;
                Ok(Value::from_as_float((trans + trans_avg) / (n_voters + 1) as f64))
            }
            BuildInVoting::WCombSumG => {
                let calculated = collect_simple(voters, |value| value.get_vote_value(SCORE).cloned())?;
                let trans = SumOf.aggregate(calculated.iter().copied())?;
                let trans_avg = GAvgOf.aggregate(calculated.into_iter())?;
                let n_voters = global_context.get_vote_value(NUMBER_OF_VOTERS)?.as_int()?;
                Ok(Value::from_as_float((trans + trans_avg) / (n_voters + 1) as f64))
            }
            BuildInVoting::WGCombSum => {
                let calculated = collect_simple(voters, |value| value.get_vote_value(SCORE).cloned())?;
                let trans = SumOf.aggregate(calculated.iter().map(|value| value.ln()))?;
                let trans_avg = AvgOf.aggregate(calculated.into_iter())?;
                let n_voters = global_context.get_vote_value(NUMBER_OF_VOTERS)?.as_int()?;
                Ok(Value::from_as_float(((trans + trans_avg.ln()) / (n_voters + 1) as f64).exp()))
            }
            BuildInVoting::PCombSum => {
                if voters.is_empty() {
                    global_context.get_vote_value(EPSILON).cloned()
                } else {
                    let trans = SumOf.aggregate(collect_simple(voters, |value| value.get_vote_value(SCORE).cloned())?.into_iter())?;
                    let max_rr = voters.iter().map(|value| if let Some(found) = value.get_value(RECIPROCAL_RANK) {
                        found.as_number()
                    } else {
                        Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                    }).process_results(|values| {
                        values.max_partial().expect("Failed to find a maximum!")
                    })?.expect("Failed to find a maximum!");
                    Ok(Value::from_as_float((trans / global_context.get_vote_value(NUMBER_OF_VOTERS)?.as_number()?) + max_rr))
                }
            }
        }
    }
}
impl DisplayTree for BuildInVoting {
    fn fmt(&self, f: &mut IndentWriter<'_, impl Write>) -> std::fmt::Result {
        let s: &str = self.into();
        write!(f, "{s}")
    }
}


register_python! {
    enum BuildInVoting;
}