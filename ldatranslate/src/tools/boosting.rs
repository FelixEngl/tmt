use pyo3::pyclass;
use strum::{AsRefStr, Display, EnumString};
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::model::Probability;

/// Setting if to keep the original word from language A
#[cfg_attr(
    feature = "gen_python_api",
    pyo3_stub_gen::derive::gen_stub_pyclass_enum
)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(
    Debug, Default, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, AsRefStr, Display, EnumString,
)]
pub enum BoostMethod {
    #[default]
    Linear,
    Sum,
    MultPow,
    Pipe,
    Mult,
}

impl BoostMethod {
    pub fn boost(&self, probability: Probability, boost: f64, factor: f64) -> f64 {
        let boosted = match self {
            BoostMethod::Linear => {
                probability + probability * boost * factor
            }
            BoostMethod::Sum => {
                boost * factor + probability
            }
            BoostMethod::MultPow => {
                probability * boost.powf(factor)
            }
            BoostMethod::Mult => {
                probability * boost
            }
            BoostMethod::Pipe => {
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



register_python!(enum BoostMethod;);