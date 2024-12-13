use crate::translate::entropies::{FDivergenceCalculator};
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
use ldatranslate_voting::traits::VotingMethodMarker;
use pyo3::exceptions::PyValueError;
use pyo3::{pyclass, pymethods, PyResult};
use std::num::NonZeroUsize;
use std::sync::Arc;
use itertools::Itertools;
use strum::{AsRefStr, Display, EnumString, ParseError};
use crate::translate::dictionary_meta::coocurrence::NormalizeMode;
use crate::translate::dictionary_meta::{MetaTagTemplate, SparseVectorFactory};
use crate::translate::dictionary_meta::vertical_boost_1::VerticalScoreBoostConfig;

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
    pub divergence_config: Option<Arc<VerticalScoreBoostConfig>>,
    /// The config for a coocurrence
    pub vertical_coocurrence: Option<Arc<HorizontalScoreBootConfig>>,
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
pub struct HorizontalScoreBootConfig {
    pub alpha: Option<f64>,
    pub calculator: FDivergenceCalculator,
    pub mode: NormalizeMode,
    pub field_config: FieldConfig,
    pub normalize_to_one: bool
}

impl HorizontalScoreBootConfig {
    pub fn new(alpha: Option<f64>, calculator: FDivergenceCalculator, mode: NormalizeMode, field_config: FieldConfig, normalize_to_one: bool) -> Self {
        Self { alpha, calculator, mode, field_config, normalize_to_one }
    }
}

pub enum BoostScoreBy {
    Domains(Vec<DictMetaTagIndex>),
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
    ) -> Self {
        Self {
            epsilon,
            voting,
            threshold,
            keep_original_word,
            top_candidate_limit,
            divergence_config: divergence_config.map(Arc::new),
            vertical_coocurrence: vertical_coocurrence.map(Arc::new),
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
            divergence_config: self.divergence_config.clone(),
            vertical_coocurrence: self.vertical_coocurrence.clone(),
        }
    }
}
