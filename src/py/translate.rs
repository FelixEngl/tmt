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

use std::num::NonZeroUsize;
use std::ops::Deref;
use derive_more::From;
use evalexpr::{DefaultNumericTypes, EvalexprNumericTypesConvert, Value};
use pyo3::{pyclass, pyfunction, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use crate::py::dictionary::PyDictionary;
use crate::py::helpers::{KeepOriginalWordArg, VotingArg};
use crate::py::topic_model::PyTopicModel;
use crate::py::variable_provider::PyVariableProvider;
use crate::py::voting::{PyVoting, PyVotingRegistry};
use crate::register_python;
use crate::topicmodel::dictionary::metadata::domain_voting::{vote_for_domains_with_targets, VoteConfig};
use crate::translate::{KeepOriginalWord, TranslateConfig};
use crate::voting::parser::input::ParserInput;
use crate::voting::parser::{parse};
use crate::translate::translate_topic_model as translate;
use crate::voting::{VotingMethod, VotingMethodContext, VotingResult};
use crate::voting::py::{PyExprValue, PyVotingModel};
use crate::voting::traits::VotingMethodMarker;

/// The config for a translation
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyVoteConfig {
    /// The epsilon to be used, if it is none it is determined heuristically.
    pub epsilon: Option<f64>,
    /// The threshold of the probabilities allowed to be used as voters
    pub threshold: Option<f64>,
    /// Limits the number of accepted candidates to N. If not set keep all.
    pub top_candidate_limit: Option<NonZeroUsize>,
    /// Declares a field that boosts the score iff present.
    pub boost_with: Option<Value>
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyVoteConfig {
    #[new]
    #[pyo3(signature = (epsilon=None, threshold=None, top_candidate_limit=None, boost_with=None))]
    pub fn new(epsilon: Option<f64>, threshold: Option<f64>, top_candidate_limit: Option<usize>, boost_with: Option<PyExprValue>) -> Self {
        Self {
            epsilon,
            threshold,
            top_candidate_limit: top_candidate_limit.and_then(|x| NonZeroUsize::new(x)),
            boost_with: boost_with.map(|v| v.into())
        }
    }
}

impl PyVoteConfig {
    fn to_vote_config(self, voting: VotingArg, voting_registry: Option<PyVotingRegistry>) -> PyResult<VoteConfig<Wrapper>> {
        let voting = match voting {
            VotingArg::Voting(voting) => {
                Wrapper::Internal(voting)
            }
            VotingArg::Parseable(voting) => {
                match parse::<nom::error::Error<_>>(ParserInput::new(&voting, voting_registry.unwrap_or_default().registry())) {
                    Ok((_, value)) => {
                        Wrapper::Internal(value.into())
                    }
                    Err(err) => {
                        return Err(PyValueError::new_err(err.to_string()))
                    }
                }
            }
            VotingArg::BuildIn(build_in) => {
                Wrapper::Internal(build_in.into())
            }
            VotingArg::PyCallable(def) => {
                Wrapper::External(def)
            }
        };

        Ok(
            VoteConfig::new(
                voting,
                self.epsilon,
                self.threshold,
                self.top_candidate_limit,
                self.boost_with,
            )
        )
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyTranslationConfig {
    epsilon: Option<f64>,
    threshold: Option<f64>,
    keep_original_word: KeepOriginalWord,
    top_candidate_limit: Option<NonZeroUsize>,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyTranslationConfig {
    #[new]
    #[pyo3(signature = (epsilon=None, threshold=None, keep_original_word=None, top_candidate_limit=None))]
    pub fn new(
        epsilon: Option<f64>,
        threshold: Option<f64>,
        keep_original_word: Option<KeepOriginalWordArg>,
        top_candidate_limit: Option<usize>,
    ) -> PyResult<Self> {
        Ok(Self{
             epsilon,
             threshold,
             keep_original_word: keep_original_word
                 .unwrap_or(KeepOriginalWordArg::Value(KeepOriginalWord::Never))
                 .try_into()
                 .map_err(|value: <KeepOriginalWordArg as TryInto<KeepOriginalWord>>::Error| PyValueError::new_err(value.to_string()))?,
             top_candidate_limit: top_candidate_limit.map(|value| NonZeroUsize::new(value)).flatten()
        })
    }
}

impl PyTranslationConfig {
    fn to_translation_config(self, voting: VotingArg, voting_registry: Option<PyVotingRegistry>) -> PyResult<TranslateConfig<Wrapper>> {
        let voting = match voting {
            VotingArg::Voting(voting) => {
                Wrapper::Internal(voting)
            }
            VotingArg::Parseable(voting) => {
                match parse::<nom::error::Error<_>>(ParserInput::new(&voting, voting_registry.unwrap_or_default().registry())) {
                    Ok((_, value)) => {
                        Wrapper::Internal(value.into())
                    }
                    Err(err) => {
                        return Err(PyValueError::new_err(err.to_string()))
                    }
                }
            }
            VotingArg::BuildIn(build_in) => {
                Wrapper::Internal(build_in.into())
            }
            VotingArg::PyCallable(def) => {
                Wrapper::External(def)
            }
        };

        Ok(
            TranslateConfig::new(
                voting,
                self.epsilon,
                self.threshold,
                self.keep_original_word,
                self.top_candidate_limit,
            )
        )
    }
}

#[derive(Clone, Debug, From)]
enum Wrapper<'a> {
    External(PyVotingModel<'a>),
    Internal(PyVoting)
}

impl VotingMethod for Wrapper<'_> {
    fn execute<A, B, NumericTypes: EvalexprNumericTypesConvert>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<NumericTypes>, NumericTypes>
    where
        A : VotingMethodContext<NumericTypes>,
        B : VotingMethodContext<NumericTypes>
    {
        match self {
            Wrapper::External(value) => {
                value.execute(global_context, voters)
            }
            Wrapper::Internal(value) => {
                value.execute(global_context, voters)
            }
        }
    }
}

impl VotingMethodMarker for Wrapper<'_> {}



#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
#[pyo3(signature = (topic_model, dictionary, voting, config, provider=None, voting_registry=None))]
pub fn translate_topic_model<'a>(
    topic_model: &PyTopicModel,
    dictionary: &PyDictionary,
    // VotingArg<'a>
    voting: VotingArg<'a>,
    config: PyTranslationConfig,
    provider: Option<&PyVariableProvider>,
    voting_registry: Option<PyVotingRegistry>
) -> PyResult<PyTopicModel> {
    let cfg =config.to_translation_config(voting, voting_registry)?;
    let read = dictionary.get();
    match translate::<DefaultNumericTypes, _, _, _, _, _, _>(topic_model, read.deref(), &cfg, provider) {
        Ok(result) => {
            Ok(result)
        }
        Err(err) => {
            Err(PyValueError::new_err(err.to_string()))
        }
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
#[pyo3(signature = (topic_model, dictionary, voting, config, provider=None, voting_registry=None))]
pub fn vote_for_domains<'a>(
    topic_model: &PyTopicModel,
    dictionary: &PyDictionary,
    // VotingArg<'a>
    voting: VotingArg<'a>,
    config: PyVoteConfig,
    provider: Option<&PyVariableProvider>,
    voting_registry: Option<PyVotingRegistry>
) -> PyResult<Vec<Vec<f64>>> {
    let cfg = config.to_vote_config(voting, voting_registry)?;
    let read = dictionary.get();
    match vote_for_domains_with_targets::<DefaultNumericTypes, _, _, _, _, _, _>(topic_model, read.deref(), &cfg, provider) {
        Ok(result) => {
            Ok(result)
        }
        Err(err) => {
            Err(PyValueError::new_err(err.to_string()))
        }
    }
}


register_python! {
    struct PyTranslationConfig;
    fn translate_topic_model;
}