use std::fmt::Write;
use std::num::NonZeroUsize;
use evalexpr::{Context, ContextWithMutableVariables, EvalexprError, EvalexprResult, Value};
use itertools::Itertools;
use strum::{Display, EnumString, IntoStaticStr, VariantArray};
use crate::toolkit::partial_ord_iterator::PartialOrderIterator;
use crate::variable_names::{EPSILON, NUMBER_OF_VOTERS, RECIPROCAL_RANK, SCORE, SCORE_CANDIDATE};
use crate::voting::{VotingMethod, VotingMethodMarker, VotingResult, VotingWithLimit};
use crate::voting::aggregations::{Aggregation, AggregationError};
use crate::voting::aggregations::AggregationType::{AvgOf, GAvgOf, SumOf};
use crate::voting::display::{DisplayTree, IndentWriter};
use crate::voting::traits::LimitableVotingMethodMarker;
use crate::voting::VotingExpressionError::{Eval, NoValue};
use pyo3::{pyclass, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use serde::{Deserialize, Serialize};
use crate::py::voting::PyVoting;
use crate::voting::parser::ParseResult::Limited;

/// An empty voting method if nothing works
pub struct EmptyVotingMethod;

impl LimitableVotingMethodMarker for EmptyVotingMethod {}
impl VotingMethodMarker for EmptyVotingMethod {}

impl VotingMethod for EmptyVotingMethod {
    fn execute<A, B>(&self, _: &mut A, _: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
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
}

impl LimitableVotingMethodMarker for BuildInVoting {}
impl VotingMethodMarker for BuildInVoting {}
impl VotingMethod for BuildInVoting {

    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
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
                Ok(SumOf.calculate(calculated.into_iter())?.into())
            }
            BuildInVoting::GCombSum => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;

                Ok(GAvgOf.calculate(calculated.into_iter())?.into())
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

                Ok(SumOf.calculate(calculated.into_iter())?.into())
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
                Ok(SumOf.calculate(calculated.into_iter())?.into())
            }
            BuildInVoting::RRPow2 => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(RECIPROCAL_RANK) {
                    Ok(found.as_number().expect("Expected a number for rr in RRPow2").powi(2).into())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(RECIPROCAL_RANK.to_string()))
                })?;
                Ok(SumOf.calculate(calculated.into_iter())?.into())
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
                Ok(SumOf.calculate(calculated.into_iter())?.into())
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
                Ok(SumOf.calculate(calculated.into_iter())?.into())
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
                Ok(SumOf.calculate(calculated.into_iter())?.into())
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
                Ok(SumOf.calculate(calculated.into_iter())?.into())
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
                let trans = SumOf.calculate(calculated.iter().copied())?;
                let trans_avg = SumOf.calculate(calculated.into_iter())?;
                let n_voters = get_value_or_fail(global_context, NUMBER_OF_VOTERS)?.as_int()?;
                Ok(((trans + trans_avg) / (n_voters + 1) as f64).into())
            }
            BuildInVoting::WCombSumG => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                let trans = SumOf.calculate(calculated.iter().copied())?;
                let trans_avg = GAvgOf.calculate(calculated.into_iter())?;
                let n_voters = get_value_or_fail(global_context, NUMBER_OF_VOTERS)?.as_int()?;
                Ok(((trans + trans_avg) / (n_voters + 1) as f64).into())
            }
            BuildInVoting::WGCombSum => {
                let calculated = collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
                    Ok(found.clone())
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(SCORE.to_string()))
                })?;
                let trans = SumOf.calculate(calculated.iter().map(|value| value.ln()))?;
                let trans_avg = AvgOf.calculate(calculated.into_iter())?;
                let n_voters = get_value_or_fail(global_context, NUMBER_OF_VOTERS)?.as_int()?;
                Ok(((trans + trans_avg) / (n_voters + 1) as f64).into())
            }
            BuildInVoting::PCombSum => {
                if voters.is_empty() {
                    get_value_or_fail(global_context, EPSILON)
                } else {
                    let trans = SumOf.calculate(collect_simple(voters, |value| if let Some(found) = value.get_value(SCORE) {
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