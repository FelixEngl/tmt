use std::num::NonZeroUsize;
use pyo3::{Bound, FromPyObject, PyAny, pyclass, pyfunction, pymethods, PyResult, wrap_pyfunction};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use crate::py::dictionary::PyDictionary;
use crate::py::topic_model::PyTopicModel;
use crate::py::vocabulary::PyVocabulary;
use crate::py::voting::{PyVoting, PyVotingRegistry};
use crate::translate::{KeepOriginalWord, TranslateConfig};
use crate::voting::parser::input::ParserInput;
use crate::voting::parser::{parse};
use crate::translate::translate_topic_model as translate;
use crate::topicmodel::topic_model::MappableTopicModel;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyTranslationConfig {
    inner: TranslateConfig<PyVoting>
}

#[derive(FromPyObject)]
pub enum VotingArg<'a> {
    Voting(PyVoting),
    Parseable(String),
    #[pyo3(transparent)]
    CatchAll(&'a PyAny)
}

#[pymethods]
impl PyTranslationConfig {
    #[new]
    pub fn new<'a>(
        voting: VotingArg<'a>,
        epsilon: Option<f64>,
        threshold: Option<f64>,
        keep_original_word: Option<KeepOriginalWord>,
        top_candidate_limit: Option<usize>,
        voting_registry: Option<PyVotingRegistry>
    ) -> PyResult<Self> {
        let inner = match voting {
            VotingArg::Voting(voting) => {
                TranslateConfig::new(
                    epsilon,
                    voting,
                    threshold,
                    keep_original_word.unwrap_or(KeepOriginalWord::Never),
                    top_candidate_limit.map(|value| NonZeroUsize::new(value)).flatten()
                )
            }
            VotingArg::Parseable(voting) => {
                match parse::<nom::error::Error<_>>(ParserInput::new(&voting, voting_registry.unwrap_or_default().registry())) {
                    Ok((_, value)) => {
                        TranslateConfig::new(
                            epsilon,
                            value.into(),
                            threshold,
                            keep_original_word.unwrap_or(KeepOriginalWord::Never),
                            top_candidate_limit.map(|value| NonZeroUsize::new(value)).flatten()
                        )
                    }
                    Err(err) => {
                        return Err(PyValueError::new_err(err.to_string()))
                    }
                }
            }
            VotingArg::CatchAll(_) => {
                return Err(PyValueError::new_err("Not a PyVoting or a String!".to_string()))
            }
        };

        Ok(Self{ inner })
    }
}

#[pyfunction]
pub fn translate_topic_model(topic_model: &PyTopicModel, dictionary: &PyDictionary, config: &PyTranslationConfig) -> PyResult<PyTopicModel> {
    match translate(topic_model, dictionary, &config.inner) {
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
    Ok(())
}