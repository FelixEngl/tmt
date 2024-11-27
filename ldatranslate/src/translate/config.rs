use std::num::NonZeroUsize;
use ldatranslate_voting::traits::VotingMethodMarker;
use pyo3::{pyclass, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use strum::{AsRefStr, Display, EnumString, ParseError};
use ldatranslate_toolkit::register_python;

/// Setting if to keep the original word from language A
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Default)]
#[derive(AsRefStr, Display, EnumString)]
pub enum KeepOriginalWord {
    Always,
    IfNoTranslation,
    #[default]
    Never
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
    #[pyo3(name="from_string")]
    pub fn from_string_py(value: &str) -> PyResult<Self> {
        value.parse().map_err(|value: ParseError | PyValueError::new_err(value.to_string()))
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
}

impl<V> TranslateConfig<V> where V: VotingMethodMarker {
    pub fn new(voting: V, epsilon: Option<f64>, threshold: Option<f64>, keep_original_word: KeepOriginalWord, top_candidate_limit: Option<NonZeroUsize>) -> Self {
        Self { epsilon, voting, threshold, keep_original_word, top_candidate_limit }
    }
}

impl<'a, V> Clone for TranslateConfig<V> where V: VotingMethodMarker + Clone {
    fn clone(&self) -> Self {
        Self {
            voting: self.voting.clone(),
            epsilon: self.epsilon,
            threshold: self.threshold,
            keep_original_word: self.keep_original_word,
            top_candidate_limit: self.top_candidate_limit
        }
    }
}



