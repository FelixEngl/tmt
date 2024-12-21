use std::cmp::Ordering::Equal;
use crate::translate::entropies::{FDivergenceCalculator};
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
use ldatranslate_voting::traits::VotingMethodMarker;
use pyo3::exceptions::PyValueError;
use pyo3::{pyclass, pymethods, PyResult};
use std::num::NonZeroUsize;
use std::sync::Arc;
use itertools::Itertools;
use rstats::{Median, MutVecg, Stats, RE};
use strum::{AsRefStr, Display, EnumIs, EnumString, ParseError};
use ldatranslate_topicmodel::model::Probability;
use crate::py::translate::{PyHorizontalBoostConfig, PyVerticalBoostConfig};
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


#[cfg_attr(
    feature = "gen_python_api",
    pyo3_stub_gen::derive::gen_stub_pyclass_enum
)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(
    Debug, Copy, Default, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, AsRefStr, Display, EnumString, EnumIs
)]
pub enum Transform {
    Off,
    #[default]
    Linear,
    Normalized,
}

impl Transform {
    pub fn transform(&self, arr: &mut [f64]) {
        match self {
            Transform::Off => {}
            Transform::Linear => {
                // rstats::MutVecg::mlintrans()
                let mm = indxvec::Vecops::minmax(arr.as_ref());
                let range = mm.max - mm.min + f64::EPSILON;
                for c in arr.iter_mut() {
                    *c = (*c - mm.min + f64::EPSILON) / range
                }
            }
            Transform::Normalized => {
                let sum: f64 = arr.iter().sum();
                if sum <= 0.0 {
                    return;
                }
                arr.iter_mut().for_each(|value| {
                    *value /= sum
                });
            }
        }
    }
}

register_python!(enum Transform;);


#[derive(Clone, Debug)]
pub struct VerticalScoreBoostConfig {
    pub field_config: FieldConfig,
    pub calculator: FDivergenceCalculator,
    pub transformer: Transform,
    pub factor: f64
}

impl VerticalScoreBoostConfig {
    pub fn new(field_config: FieldConfig, calculator: FDivergenceCalculator, transformer: Transform, factor: Option<f64>) -> Self {
        Self { field_config, calculator, transformer, factor: factor.unwrap_or(1.0) }
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
            value.factor
        )
    }
}

/// Setting if to keep the original word from language A
#[cfg_attr(
    feature = "gen_python_api",
    pyo3_stub_gen::derive::gen_stub_pyclass_enum
)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(
    Debug, Default, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, AsRefStr, Display, EnumString,
)]
pub enum MeanMethod {
    ArithmeticMean,
    LinearWeightedArithmeticMean,
    HarmonicMean,
    LinearWeightedHarmonicMean,
    GeometricMean,
    LinearWeightedGeometricMean,
    #[default]
    Median
}


impl MeanMethod {
    pub fn fails_on_empty(&self) -> bool {
        matches!(self,
            MeanMethod::GeometricMean | MeanMethod::LinearWeightedGeometricMean
            | MeanMethod::HarmonicMean | MeanMethod::LinearWeightedHarmonicMean
        )
    }

    pub fn apply<'a, S, T>(&self, value: S) -> Result<f64, RE>
    where
        S: Stats + Median<'a, T> + 'a,
        T: Into<f64> + PartialOrd + Copy
    {
        match self {
            MeanMethod::ArithmeticMean => {
                value.amean()
            }
            MeanMethod::LinearWeightedArithmeticMean => {
                value.awmean()
            }
            MeanMethod::HarmonicMean => {
                value.hmean()
            }
            MeanMethod::LinearWeightedHarmonicMean => {
                value.hwmean()
            }
            MeanMethod::GeometricMean => {
                value.gmean()
            }
            MeanMethod::LinearWeightedGeometricMean => {
                value.gwmean()
            }
            MeanMethod::Median => {
                Ok(value.qmedian_by(
                    &mut |a, b| a.partial_cmp(b).unwrap_or(Equal),
                    |v| v.clone().into()
                )?)
            }
        }
    }
}


/// Setting if to keep the original word from language A
#[cfg_attr(
    feature = "gen_python_api",
    pyo3_stub_gen::derive::gen_stub_pyclass_enum
)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(
    Debug, Default, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, AsRefStr, Display, EnumString,
)]
pub enum TransformMethod {
    #[default]
    Linear,
    Sum,
    MultPow,
    Pipe,
}

impl TransformMethod {
    pub fn boost(&self, probability: Probability, boost: f64, factor: f64) -> f64 {
        let boosted = match self {
            TransformMethod::Linear => {
                probability + probability * boost * factor
            }
            TransformMethod::Sum => {
                boost * factor + probability
            }
            TransformMethod::MultPow => {
                probability * boost.powf(factor)
            }
            TransformMethod::Pipe => {
                return probability
            }
        };
        if boosted <= 0.0 {
            f64::EPSILON
        } else {
            boosted
        }
    }
}



register_python!(enum MeanMethod; enum TransformMethod;);

#[derive(Debug, Clone)]
pub struct HorizontalScoreBootConfig {
    pub alpha: Option<f64>,
    pub calculator: FDivergenceCalculator,
    pub field_config: FieldConfig,
    pub mode: NormalizeMode,
    pub linear_transformed: TransformMethod,
    pub mean_method: MeanMethod,
    pub factor: f64
}



impl HorizontalScoreBootConfig {
    pub fn new(
        field_config: FieldConfig,
        calculator: FDivergenceCalculator,
        mode: NormalizeMode,
        alpha: Option<f64>,
        linear_transformed: TransformMethod,
        mean_method: MeanMethod,
        factor: Option<f64>
    ) -> Self {
        Self { alpha, calculator, mode, field_config, linear_transformed, mean_method, factor: factor.unwrap_or(1.0) }
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
            config.mean_method,
            config.factor
        )
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
            vertical_config: divergence_config.map(Arc::new),
            horizontal_config: vertical_coocurrence.map(Arc::new),
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
        }
    }
}
