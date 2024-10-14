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
use crate::variable_names::{EPSILON, NUMBER_OF_VOTERS, RECIPROCAL_RANK, SCORE, SCORE_CANDIDATE};
use crate::voting::{VotingMethod, VotingMethodContext, VotingMethodMarker, VotingResult, VotingWithLimit};
use crate::voting::aggregations::{Aggregation, AggregationError};
use crate::voting::aggregations::AggregationKind::{AvgOf, GAvgOf, SumOf};
use crate::voting::display::{DisplayTree, IndentWriter};
use crate::voting::traits::{LimitableVotingMethodMarker, RootVotingMethodMarker};
use crate::voting::VotingExpressionError::{Eval, NoValue};
use pyo3::{Bound, pyclass, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use serde::{Deserialize, Serialize};
use crate::py::voting::PyVoting;
use crate::voting::parser::InterpretedVoting::Limited;
use crate::voting::py::{PyContextWithMutableVariables, PyExprValue};

/// An empty voting method if nothing works
pub struct EmptyVotingMethod;

impl LimitableVotingMethodMarker for EmptyVotingMethod {}
impl VotingMethodMarker for EmptyVotingMethod {}

impl VotingMethod for EmptyVotingMethod {
    fn execute<A, B>(&self, _: &mut A, _: &mut [B]) -> VotingResult<Value> where A: VotingMethodContext, B: VotingMethodContext {
        return Err(NoValue)
    }
}

/// All possible buildin votings
#[derive(Debug, Copy, Clone, EnumString, IntoStaticStr, Display, VariantArray, Serialize, Deserialize)]
#[pyclass]
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
    pub fn from_string_py(s: &str) -> PyResult<Self> {
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

    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: VotingMethodContext, B: VotingMethodContext {
        fn get_value_or_fail<A: Context>(context: &A, name: &str) -> VotingResult<Value> {
            if let Some(found) = context.get_value(name) {
                Ok(found.clone())
            } else {
                Err(Eval(EvalexprError::VariableIdentifierNotFound(name.to_string())))
            }
        }

        fn collect_simple<B, F>(voters: &[B], f: F) -> VotingResult<Vec<f64>> where B: Context, F: Fn(&B) -> EvalexprResult<Value> {
            Ok(voters.iter().map(f).map_ok(|value: Value| value.as_number()).collect::<EvalexprResult<EvalexprResult<Vec<_>>>>()??)
        }

        match self {
            BuildInVoting::OriginalScore => {
                get_value_or_fail(global_context, SCORE_CANDIDATE)
            }
            BuildInVoting::Voters => {
                get_value_or_fail(global_context, NUMBER_OF_VOTERS)
            }
            BuildInVoting::CombSum => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                Ok(SumOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::GCombSum => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;

                Ok(GAvgOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::CombSumTop => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;

                Ok(Aggregation::new(SumOf, NonZeroUsize::new(2)).calculate_desc(calculated.into_iter())?.into())
            }
            BuildInVoting::CombSumPow2 => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(Value::Float(found.as_number().expect("Expected a number for score in CombSumPow2").powi(2)))
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;

                Ok(SumOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::CombMax => {
                Ok(
                    collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                        Ok(found.clone())
                    } else {
                        Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                    })?.iter().max_partial()
                        .map_err(|_| AggregationError::NoMaxFound)?
                        .expect("Expected a number for score in CombMax")
                        .clone()
                        .into()
                )
            }
            BuildInVoting::RR => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(RECIPROCAL_RANK) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                })?;
                Ok(SumOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::RRPow2 => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(RECIPROCAL_RANK) {
                    Ok(found.as_number().expect("Expected a number for rr in RRPow2").powi(2).into())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                })?;
                Ok(SumOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::CombSumRR => {
                let calculated = collect_simple(voters, |value| if let Some(score) = value.get_value(SCORE) {
                    if let Some(rr) = value.get_value(RECIPROCAL_RANK) {
                        Ok(Value::Float(score.as_number().expect("Expected a number for score in CombSumRR") * rr.as_number().expect("Expected a number for rr in CombSumRR")))
                    } else {
                        Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                    }
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                Ok(SumOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::CombSumRRPow2 => {
                let calculated = collect_simple(voters, |value| if let Some(score) = value.get_value(SCORE) {
                    if let Some(rr) = value.get_value(RECIPROCAL_RANK) {
                        Ok(Value::Float(score.as_number().expect("Expected a number for the score in CombSumRRPow2") * rr.as_number().expect("Expected a number for rr in CombSumRRPow2").powi(2)))
                    } else {
                        Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                    }
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                Ok(SumOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::CombSumPow2RR => {
                let calculated = collect_simple(voters, |value| if let Some(score) = value.get_value(SCORE) {
                    if let Some(rr) = value.get_value(RECIPROCAL_RANK) {
                        Ok(Value::Float(score.as_number().expect("Expected a number for the score in CombSumPow2RR").powi(2) * rr.as_number().expect("Expected a number for rr in CombSumPow2RR")))
                    } else {
                        Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                    }
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                Ok(SumOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::CombSumPow2RRPow2 => {
                let calculated = collect_simple(voters, |value| if let Some(score) = value.get_value(SCORE) {
                    if let Some(rr) = value.get_value(RECIPROCAL_RANK) {
                        Ok(Value::Float(score.as_number().expect("Expected a number for the score in CombSumPow2RRPow2").powi(2) * rr.as_number().expect("Expected a number for rr in CombSumPow2RRPow2").powi(2)))
                    } else {
                        Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                    }
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                Ok(SumOf.aggregate(calculated.into_iter())?.into())
            }
            BuildInVoting::ExpCombMnz => {
                let n_voters = get_value_or_fail(global_context, NUMBER_OF_VOTERS)?.as_int()?;
                Ok((BuildInVoting::CombSumPow2.execute(global_context, voters)?.as_number()? + (n_voters as f64)).into())
            }
            BuildInVoting::WCombSum => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                let trans = SumOf.aggregate(calculated.iter().copied())?;
                let trans_avg = AvgOf.aggregate(calculated.into_iter())?;
                let n_voters = get_value_or_fail(global_context, NUMBER_OF_VOTERS)?.as_int()?;
                Ok(((trans + trans_avg) / (n_voters + 1) as f64).into())
            }
            BuildInVoting::WCombSumG => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                let trans = SumOf.aggregate(calculated.iter().copied())?;
                let trans_avg = GAvgOf.aggregate(calculated.into_iter())?;
                let n_voters = get_value_or_fail(global_context, NUMBER_OF_VOTERS)?.as_int()?;
                Ok(((trans + trans_avg) / (n_voters + 1) as f64).into())
            }
            BuildInVoting::WGCombSum => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                let trans = SumOf.aggregate(calculated.iter().map(|value| value.ln()))?;
                let trans_avg = AvgOf.aggregate(calculated.into_iter())?;
                let n_voters = get_value_or_fail(global_context, NUMBER_OF_VOTERS)?.as_int()?;
                Ok(((trans + trans_avg.ln()) / (n_voters + 1) as f64).exp().into())
            }
            BuildInVoting::PCombSum => {
                if voters.is_empty() {
                    get_value_or_fail(global_context, EPSILON)
                } else {
                    let trans = SumOf.aggregate(collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                        Ok(found.clone())
                    } else {
                        Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                    })?.into_iter())?;
                    let max_rr = voters.iter().map(|value| if let Some(found) = value.get_value(RECIPROCAL_RANK) {
                        found.as_number()
                    } else {
                        Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                    }).process_results(|values| {
                        values.max_partial().expect("Failed to find a maximum!")
                    })?.expect("Failed to find a maximum!");
                    Ok(((trans / get_value_or_fail(global_context, NUMBER_OF_VOTERS)?.as_number()?) + max_rr).into())
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

pub(crate) fn register_py_voting_buildin(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BuildInVoting>()?;
    Ok(())
}