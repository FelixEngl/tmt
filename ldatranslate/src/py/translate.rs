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

use std::fs::File;
use std::io::Write;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::path::PathBuf;
use derive_more::From;
use evalexpr::{Value};
use pyo3::{pyclass, pyfunction, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use crate::py::dictionary::PyDictionary;
use crate::py::helpers::{KeepOriginalWordArg, LanguageHintValue, VotingArg};
use crate::py::topic_model::PyTopicModel;
use crate::py::variable_provider::PyVariableProvider;
use ldatranslate_voting::py::{PyVoting, PyVotingRegistry};
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
use ldatranslate_topicmodel::language_hint::LanguageHint;
use crate::translate::{KeepOriginalWord, TranslateConfig};
use ldatranslate_voting::parser::input::ParserInput;
use ldatranslate_voting::parser::{parse};
use crate::translate::translate_topic_model as translate;
use ldatranslate_voting::{VotingMethod, VotingMethodContext, VotingResult};
use ldatranslate_voting::constants::TMTNumericTypes;
use ldatranslate_voting::py::{PyVotingModel};
use ldatranslate_voting::traits::VotingMethodMarker;
use crate::py::word_counts::NGramStatistics;
use crate::tools::boosting::BoostMethod;
use crate::tools::mean::MeanMethod;
use crate::tools::memory::MemoryReporter;
use crate::tools::boost_norms::BoostNorm;
use crate::tools::tf_idf::Idf;
use crate::translate::dictionary_meta::coocurrence::NormalizeMode;
use crate::translate::dictionary_meta::vertical_boost_1::{ScoreModifierCalculator};
use crate::translate::entropies::{FDivergence};

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyBasicBoostConfig {
    pub divergence: FDivergence,
    pub alpha: Option<f64>,
    pub target_fields: Option<Vec<DictMetaTagIndex>>,
    pub invert_target_fields: bool,
    pub score_modifier_calculator: ScoreModifierCalculator,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyBasicBoostConfig {
    #[new]
    #[pyo3(signature = (divergence, alpha=None, target_fields=None, invert_target_fields=None, score_modifier_calculator=None))]
    pub fn new(
        divergence: FDivergence,
        alpha: Option<f64>,
        target_fields: Option<Vec<DictMetaTagIndex>>,
        invert_target_fields: Option<bool>,
        score_modifier_calculator: Option<ScoreModifierCalculator>,
    ) -> PyResult<Self> {
        Ok(Self{
            divergence,
            alpha,
            target_fields,
            invert_target_fields: invert_target_fields.unwrap_or(false),
            score_modifier_calculator: score_modifier_calculator.unwrap_or_default(),
        })
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyVerticalBoostConfig {
    pub divergence: PyBasicBoostConfig,
    pub transformer: BoostNorm,
    pub factor: Option<f64>
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyVerticalBoostConfig {
    #[new]
    #[pyo3(signature = (divergence, transformer=None, factor=None))]
    pub fn new(
        divergence: PyBasicBoostConfig,
        transformer: Option<BoostNorm>,
        factor: Option<f64>
    ) -> PyResult<Self> {
        Ok(Self{
            divergence,
            transformer: transformer.unwrap_or_default(),
            factor
        })
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyHorizontalBoostConfig {
    pub divergence: PyBasicBoostConfig,
    pub mode: NormalizeMode,
    pub alpha: Option<f64>,
    pub linear_transformed: bool,
    pub mean_method: MeanMethod,
    pub transform: BoostMethod,
    pub factor: Option<f64>
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyHorizontalBoostConfig {
    #[new]
    #[pyo3(signature = (divergence, mean_method=None, normalize_mode=None, alpha=None::<Option<f64>>, linear_transformed=None, transform=None, factor=None))]
    pub fn new(
        divergence: PyBasicBoostConfig,
        mean_method: Option<MeanMethod>,
        normalize_mode: Option<NormalizeMode>,
        alpha: Option<Option<f64>>,
        linear_transformed: Option<bool>,
        transform: Option<BoostMethod>,
        factor: Option<f64>
    ) -> PyResult<Self> {
        Ok(Self{
            divergence,
            mode: normalize_mode.unwrap_or_default(),
            alpha: alpha.unwrap_or(Some(0.15)),
            linear_transformed: linear_transformed.unwrap_or_default(),
            transform: transform.unwrap_or_default(),
            mean_method: mean_method.unwrap_or_default(),
            factor
        })
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyNGramBoostConfig {
    pub boost_lang_a: Option<PyNGramLanguageBoost>,
    pub boost_lang_b: Option<PyNGramLanguageBoost>,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyNGramBoostConfig {
    #[new]
    #[pyo3(signature = (boost_lang_a=None, boost_lang_b=None))]
    pub fn new(
        boost_lang_a: Option<PyNGramLanguageBoost>,
        boost_lang_b: Option<PyNGramLanguageBoost>,
    ) -> PyResult<Self> {
        let boost_lang_a = boost_lang_a.or_else(|| if boost_lang_b.is_some() {None} else {Some(Default::default())});
        Ok(Self{
            boost_lang_a,
            boost_lang_b
        })
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyNGramLanguageBoost {
    pub idf: Idf,
    pub boosting: BoostMethod,
    pub norm: BoostNorm,
    pub factor: Option<f64>,
    pub fallback_language: Option<LanguageHint>
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyNGramLanguageBoost {
    #[new]
    #[pyo3(signature = (idf=None, boosting=None, norm=None, factor=None, fallback_language=None))]
    pub fn new(
        idf: Option<Idf>,
        boosting: Option<BoostMethod>,
        norm: Option<BoostNorm>,
        factor: Option<f64>,
        fallback_language: Option<LanguageHintValue>
    ) -> PyResult<Self> {
        Ok(Self{
            idf: idf.unwrap_or(Idf::InverseDocumentFrequency),
            boosting: boosting.unwrap_or(BoostMethod::Mult),
            norm: norm.unwrap_or(BoostNorm::Off),
            factor,
            fallback_language: fallback_language.map(|v| v.into())
        })
    }
}

impl Default for PyNGramLanguageBoost {
    fn default() -> Self {
        Self {
            idf: Idf::InverseDocumentFrequency,
            boosting: BoostMethod::Mult,
            norm: BoostNorm::Off,
            factor: None
        }
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
    vertical_config: Option<PyVerticalBoostConfig>,
    horizontal_config: Option<PyHorizontalBoostConfig>,
    ngram_config: Option<PyNGramBoostConfig>,
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyTranslationConfig {
    #[new]
    #[pyo3(signature = (epsilon=None, threshold=None, keep_original_word=None, top_candidate_limit=None, vertical_config=None, horizontal_config=None, ngram_config=None))]
    pub fn new(
        epsilon: Option<f64>,
        threshold: Option<f64>,
        keep_original_word: Option<KeepOriginalWordArg>,
        top_candidate_limit: Option<usize>,
        vertical_config: Option<PyVerticalBoostConfig>,
        horizontal_config: Option<PyHorizontalBoostConfig>,
        ngram_config: Option<PyNGramBoostConfig>,
    ) -> PyResult<Self> {
        Ok(Self{
            epsilon,
            threshold,
            keep_original_word: keep_original_word
                .unwrap_or(KeepOriginalWordArg::Value(KeepOriginalWord::Never))
                .try_into()
                .map_err(|value: <KeepOriginalWordArg as TryInto<KeepOriginalWord>>::Error| PyValueError::new_err(value.to_string()))?,
            top_candidate_limit: top_candidate_limit.map(|value| NonZeroUsize::new(value)).flatten(),
            vertical_config,
            horizontal_config,
            ngram_config
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
                self.vertical_config.map(Into::into),
                self.horizontal_config.map(Into::into),
                self.ngram_config.map(Into::into),
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
    fn execute<A, B>(&self, global_context: &mut A, voters: &mut [B]) -> VotingResult<Value<TMTNumericTypes>>
    where
        A : VotingMethodContext,
        B : VotingMethodContext
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
#[pyo3(signature = (topic_model, dictionary, voting, config, provider=None, voting_registry=None, idf_source=None))]
pub fn translate_topic_model<'a>(
    topic_model: &PyTopicModel,
    dictionary: &PyDictionary,
    // VotingArg<'a>
    voting: VotingArg<'a>,
    config: PyTranslationConfig,
    provider: Option<&PyVariableProvider>,
    voting_registry: Option<PyVotingRegistry>,
    idf_source: Option<NGramStatistics>
) -> PyResult<PyTopicModel> {
    let cfg =config.to_translation_config(voting, voting_registry)?;
    let read = dictionary.get();
    log::info!("Start with translation.");
    match translate(topic_model, read.deref(), &cfg, provider, idf_source) {
        Ok(result) => {
            log::info!("Memory usage: {}", MemoryReporter::instant_report());
            Ok(result)
        }
        Err(err) => {
            Err(PyValueError::new_err(err.to_string()))
        }
    }
}

/// Saves some ratings
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
pub fn save_ratings(
    path: PathBuf,
    ratings: Vec<(u64, Vec<(u64, f64)>)>
) -> PyResult<()> {
    let mut writer = zstd::Encoder::new(
        File::options().write(true).create_new(true).open(path)?,
        0
    )?;
    bincode::serialize_into(
        &mut writer,
        &ratings
    ).map_err(|err| PyValueError::new_err(err.to_string()))?;
    writer.flush()?;
    Ok(())
}

/// Loads some ratings
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
pub fn load_ratings(
    path: PathBuf,
) -> PyResult<Vec<(u64, Vec<(u64, f64)>)>> {
    bincode::deserialize_from(
        zstd::Decoder::new(
            File::options().read(true).open(path)?
        )?
    ).map_err(|err| PyValueError::new_err(err.to_string()))
}


register_python! {
    struct PyTranslationConfig;
    struct PyVerticalBoostConfig;
    struct PyHorizontalBoostConfig;
    struct PyBasicBoostConfig;
    struct PyNGramBoostConfig;
    struct PyNGramLanguageBoost;
    fn translate_topic_model;
    fn save_ratings;
    fn load_ratings;
}