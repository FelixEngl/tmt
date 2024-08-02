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
use derive_more::From;
use evalexpr::{Value};
use pyo3::{Bound, pyclass, pyfunction, pymethods, PyResult, wrap_pyfunction};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use crate::py::dictionary::PyDictionary;
use crate::py::helpers::{KeepOriginalWordArg, VotingArg};
use crate::py::topic_model::PyTopicModel;
use crate::py::variable_provider::PyVariableProvider;
use crate::py::vocabulary::PyVocabulary;
use crate::py::voting::{PyVoting, PyVotingRegistry};
use crate::translate::{KeepOriginalWord, register_py_translate, TranslateConfig};
use crate::voting::parser::input::ParserInput;
use crate::voting::parser::{parse};
use crate::translate::translate_topic_model as translate;
use crate::topicmodel::topic_model::MappableTopicModel;
use crate::variable_names::{register_py_variable_names_module};
use crate::voting::{VotingMethod, VotingMethodContext, VotingResult};
use crate::voting::py::PyVotingModel;
use crate::voting::traits::VotingMethodMarker;


#[pyclass]
#[derive(Debug, Clone)]
pub struct PyTranslationConfig {
    epsilon: Option<f64>,
    threshold: Option<f64>,
    keep_original_word: KeepOriginalWord,
    top_candidate_limit: Option<NonZeroUsize>,
}


#[pymethods]
impl PyTranslationConfig {
    #[new]
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

#[derive(Clone, Debug, From)]
enum Wrapper<'a> {
    External(PyVotingModel<'a>),
    Internal(PyVoting)
}

impl VotingMethod for Wrapper<'_> {
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: VotingMethodContext, B: VotingMethodContext {
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

#[pyfunction]
pub fn translate_topic_model<'a>(
    topic_model: &PyTopicModel,
    dictionary: &PyDictionary,
    voting: VotingArg<'a>,
    config: PyTranslationConfig,
    provider: Option<&PyVariableProvider>,
    voting_registry: Option<PyVotingRegistry>
) -> PyResult<PyTopicModel> {
    let cfg =config.to_translation_config(voting, voting_registry)?;
    match translate(topic_model, dictionary, &cfg, provider) {
        Ok(result) => {
            Ok(PyTopicModel::wrap(result.map::<PyVocabulary>()))
        }
        Err(err) => {
            Err(PyValueError::new_err(err.to_string()))
        }
    }
}


pub(crate) fn translate_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTranslationConfig>()?;
    m.add_function(wrap_pyfunction!(translate_topic_model, m)?)?;
    register_py_translate(m)?;
    register_py_variable_names_module(m)?;
    Ok(())
}