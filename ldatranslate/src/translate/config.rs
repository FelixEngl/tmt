use crate::translate::entropies::{FDivergenceCalculator};
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
use ldatranslate_voting::traits::VotingMethodMarker;
use pyo3::exceptions::PyValueError;
use pyo3::{pyclass, pymethods, PyResult};
use std::num::NonZeroUsize;
use std::sync::Arc;
use itertools::{Itertools};
use strum::{AsRefStr, Display, EnumString, ParseError};
use ldatranslate_topicmodel::language_hint::LanguageHint;
use crate::py::translate::{PyHorizontalBoostConfig, PyNGramBoostConfig, PyNGramLanguageBoost, PyVerticalBoostConfig};
use crate::tools::boosting::BoostMethod;
use crate::tools::mean::MeanMethod;
use crate::tools::boost_norms::BoostNorm;
use crate::tools::tf_idf::Idf;
use crate::translate::dictionary_meta::coocurrence::NormalizeMode;
use crate::translate::dictionary_meta::{MetaTagTemplate, SparseVectorFactory};

/// Setting if to keep the original word from language A
#[cfg_attr(
    feature = "gen_python_api",
    pyo3_stub_gen::derive::gen_stub_pyclass_enum
)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(
    Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Default, AsRefStr, Display, EnumString,
)]
pub enum KeepOriginalWord {
    Always,
    IfNoTranslation,
    #[default]
    Never,
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl KeepOriginalWord {
    pub fn __str__(&self) -> String {
        self.to_string()
    }

    pub fn __repr__(&self) -> String {
        self.to_string()
    }

    #[staticmethod]
    #[pyo3(name = "from_string")]
    pub fn from_string_py(value: &str) -> PyResult<Self> {
        value
            .parse()
            .map_err(|value: ParseError| PyValueError::new_err(value.to_string()))
    }

    pub fn __reduce__(&self) -> String {
        format!("KeepOriginalWord.{self}")
    }

    pub fn __reduce_ex__(&self, _version: usize) -> String {
        format!("KeepOriginalWord.{self}")
    }
}

register_python! {
    enum KeepOriginalWord;
}


/// The config for a translation
#[derive(Debug)]
pub struct TranslateConfig<V: VotingMethodMarker> {
    /// The voting to be used
    pub voting: V,
    /// The epsilon to be used, if it is none it is determined heuristically.
    pub epsilon: Option<f64>,
    /// The threshold of the probabilities allowed to be used as voters
    pub threshold: Option<f64>,
    /// Set what to do with the original word
    pub keep_original_word: KeepOriginalWord,
    /// Limits the number of accepted candidates to N. If not set keep all.
    pub top_candidate_limit: Option<NonZeroUsize>,
    /// The config for a divergence applied to the base score.
    /// Right now we only support calculating on topic level.
    pub vertical_config: Option<Arc<VerticalScoreBoostConfig>>,
    /// The config for a coocurrence
    pub horizontal_config: Option<Arc<HorizontalScoreBootConfig>>,
    /// Word count boost config
    pub ngram_boost_config: Option<Arc<NGramBoostConfig>>
}

#[derive(Debug, Clone)]
pub struct FieldConfig {
    pub target_fields: Option<Vec<DictMetaTagIndex>>,
    pub invert_target_fields: bool,
}

impl FieldConfig {
    pub fn create_with_factory(&self, factory: &SparseVectorFactory) -> MetaTagTemplate {
        if let Some(targ_fields) = self.target_fields.as_ref() {
            if self.invert_target_fields {
                let value = DictMetaTagIndex::all().into_iter().copied().filter(|v| !targ_fields.contains(v)).collect_vec();
                factory.convert_to_template(&value)
            } else {
                factory.convert_to_template(targ_fields)
            }
        } else {
            factory.convert_to_template(&DictMetaTagIndex::all())
        }
    }

    pub fn new(target_fields: Option<Vec<DictMetaTagIndex>>, invert_target_fields: bool) -> Self {
        Self { target_fields, invert_target_fields }
    }
}


#[derive(Debug, Clone)]
pub struct NGramBoostConfig {
    pub boost_a: Option<NGramLanguageBoostConfig>,
    pub boost_b: Option<NGramLanguageBoostConfig>
}


impl From<PyNGramBoostConfig> for NGramBoostConfig {
    fn from(value: PyNGramBoostConfig) -> Self {
        Self {
            boost_a: value.boost_lang_a.map(Into::into),
            boost_b: value.boost_lang_b.map(Into::into),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NGramLanguageBoostConfig {
    pub idf: Idf,
    pub boosting: BoostMethod,
    pub norm: BoostNorm,
    pub only_positive_boost: bool,
    pub factor: f64,
    pub fallback_language: Option<LanguageHint>
}

impl NGramLanguageBoostConfig {
    pub fn new(idf: Idf, boosting: BoostMethod, norm: BoostNorm, only_positive_boost: Option<bool>, factor: Option<f64>, fallback_language: Option<LanguageHint>) -> Self {
        Self { idf, boosting, norm, only_positive_boost: only_positive_boost.unwrap_or(true), factor: factor.unwrap_or(1.0), fallback_language }
    }
}

impl From<PyNGramLanguageBoost> for NGramLanguageBoostConfig {
    fn from(value: PyNGramLanguageBoost) -> Self {
        Self::new(
            value.idf,
            value.boosting,
            value.norm,
            value.only_positive_boost,
            value.factor,
            value.fallback_language
        )
    }
}


#[derive(Clone, Debug)]
pub struct VerticalScoreBoostConfig {
    pub field_config: FieldConfig,
    pub calculator: FDivergenceCalculator,
    pub transformer: BoostNorm,
    pub factor: f64,
    pub only_positive_boost: bool,
}

impl VerticalScoreBoostConfig {
    pub fn new(field_config: FieldConfig, calculator: FDivergenceCalculator, transformer: BoostNorm, factor: Option<f64>, only_positive_boost: Option<bool>) -> Self {
        Self { field_config, calculator, transformer, factor: factor.unwrap_or(1.0), only_positive_boost: only_positive_boost.unwrap_or(false) }
    }
}


impl From<PyVerticalBoostConfig> for VerticalScoreBoostConfig {
    fn from(value: PyVerticalBoostConfig) -> Self {
        Self::new(
            FieldConfig::new(
                value.divergence.target_fields,
                value.divergence.invert_target_fields,
            ),
            FDivergenceCalculator::new(
                value.divergence.divergence,
                value.divergence.alpha,
                value.divergence.score_modifier_calculator
            ),
            value.transformer,
            value.factor,
            value.only_positive_boost
        )
    }
}



#[derive(Debug, Clone)]
pub struct HorizontalScoreBootConfig {
    pub alpha: Option<f64>,
    pub calculator: FDivergenceCalculator,
    pub field_config: FieldConfig,
    pub mode: NormalizeMode,
    pub linear_transformed: bool,
    pub transform: BoostMethod,
    pub mean_method: MeanMethod,
    pub factor: f64,
    pub only_positive_boost: bool
}


impl HorizontalScoreBootConfig {
    pub fn new(
        field_config: FieldConfig,
        calculator: FDivergenceCalculator,
        mode: NormalizeMode,
        alpha: Option<f64>,
        linear_transformed: bool,
        transform: BoostMethod,
        mean_method: MeanMethod,
        factor: Option<f64>,
        only_positive_boost: Option<bool>
    ) -> Self {
        Self { alpha, calculator, mode, field_config, linear_transformed, transform, mean_method, factor: factor.unwrap_or(1.0), only_positive_boost: only_positive_boost.unwrap_or(false) }
    }
}

impl From<PyHorizontalBoostConfig> for HorizontalScoreBootConfig {
    fn from(config: PyHorizontalBoostConfig) -> Self {
        Self::new(
            FieldConfig::new(
                config.divergence.target_fields,
                config.divergence.invert_target_fields,
            ),
            FDivergenceCalculator::new(
                config.divergence.divergence,
                config.divergence.alpha,
                config.divergence.score_modifier_calculator
            ),
            config.mode,
            config.alpha,
            config.linear_transformed,
            config.transform,
            config.mean_method,
            config.factor,
            config.only_positive_boost
        )
    }
}


impl<V> TranslateConfig<V>
where
    V: VotingMethodMarker,
{
    pub fn new(
        voting: V,
        epsilon: Option<f64>,
        threshold: Option<f64>,
        keep_original_word: KeepOriginalWord,
        top_candidate_limit: Option<NonZeroUsize>,
        divergence_config: Option<VerticalScoreBoostConfig>,
        vertical_coocurrence: Option<HorizontalScoreBootConfig>,
        ngram_boost_config: Option<NGramBoostConfig>,
    ) -> Self {
        Self {
            epsilon,
            voting,
            threshold,
            keep_original_word,
            top_candidate_limit,
            vertical_config: divergence_config.map(Arc::new),
            horizontal_config: vertical_coocurrence.map(Arc::new),
            ngram_boost_config: ngram_boost_config.map(Arc::new)
        }
    }
}

impl<'a, V> Clone for TranslateConfig<V>
where
    V: VotingMethodMarker + Clone,
{
    fn clone(&self) -> Self {
        Self {
            voting: self.voting.clone(),
            epsilon: self.epsilon,
            threshold: self.threshold,
            keep_original_word: self.keep_original_word,
            top_candidate_limit: self.top_candidate_limit,
            vertical_config: self.vertical_config.clone(),
            horizontal_config: self.horizontal_config.clone(),
            ngram_boost_config: self.ngram_boost_config.clone(),
        }
    }
}
