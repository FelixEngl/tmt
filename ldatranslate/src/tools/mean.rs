use std::cmp::Ordering::Equal;
use pyo3::pyclass;
use rstats::{Median, Stats, RE};
use strum::{AsRefStr, Display, EnumString};
use ldatranslate_toolkit::register_python;

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
register_python!(enum MeanMethod;);

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

