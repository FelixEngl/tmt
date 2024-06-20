use std::num::NonZeroUsize;
use std::str::FromStr;
use derive_more::From;
use evalexpr::{ContextWithMutableVariables, Value};
use pyo3::{Bound, FromPyObject, pyclass, pyfunction, pymethods, PyResult, wrap_pyfunction};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use crate::py::dictionary::PyDictionary;
use crate::py::topic_model::PyTopicModel;
use crate::py::variable_provider::PyVariableProvider;
use crate::py::vocabulary::PyVocabulary;
use crate::py::voting::{PyVoting, PyVotingRegistry};
use crate::translate::{KeepOriginalWord, TranslateConfig};
use crate::voting::parser::input::ParserInput;
use crate::voting::parser::{parse};
use crate::translate::translate_topic_model as translate;
use crate::topicmodel::topic_model::MappableTopicModel;
use crate::voting::{BuildInVoting, VotingMethod, VotingResult};
use crate::voting::py::PyVotingModel;
use crate::voting::traits::VotingMethodMarker;

#[derive(FromPyObject)]
pub enum VotingArg<'a> {
    Voting(PyVoting),
    BuildIn(BuildInVoting),
    Parseable(String),
    PyCallable(PyVotingModel<'a>),
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyTranslationConfig {
    epsilon: Option<f64>,
    threshold: Option<f64>,
    keep_original_word: KeepOriginalWord,
    top_candidate_limit: Option<NonZeroUsize>,
}

#[derive(FromPyObject)]
pub enum KeepOriginalWordArg {
    String(String),
    Value(KeepOriginalWord)
}

impl TryInto<KeepOriginalWord> for KeepOriginalWordArg {
    type Error = <KeepOriginalWord as FromStr>::Err;

    fn try_into(self) -> Result<KeepOriginalWord, Self::Error> {
        match self {
            KeepOriginalWordArg::String(value) => {value.parse()}
            KeepOriginalWordArg::Value(value) => {Ok(value)}
        }
    }
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
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value> where A: ContextWithMutableVariables, B: ContextWithMutableVariables {
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
    m.add_class::<KeepOriginalWord>()?;
    m.add_function(wrap_pyfunction!(translate_topic_model, m)?)?;
    Ok(())
}