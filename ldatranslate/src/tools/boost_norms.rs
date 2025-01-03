use pyo3::{pyclass, pymethods};
use strum::{AsRefStr, Display, EnumIs, EnumString};
use ldatranslate_toolkit::register_python;

#[cfg_attr(
    feature = "gen_python_api",
    pyo3_stub_gen::derive::gen_stub_pyclass_enum
)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(
    Debug, Copy, Default, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, AsRefStr, Display, EnumString, EnumIs
)]
pub enum BoostNorm {
    Off,
    #[default]
    Linear,
    Normalized,
}

#[cfg(not(feature = "gen_python_api"))]
#[pymethods]
impl BoostNorm {
    fn __str__(&self) -> String {
        self.to_string()
    }
}

impl BoostNorm {
    pub fn norm(&self, arr: &mut [f64]) {
        match self {
            BoostNorm::Off => {}
            BoostNorm::Linear => {
                // rstats::MutVecg::mlintrans()
                let mm = indxvec::Vecops::minmax(arr.as_ref());
                let range = mm.max - mm.min + f64::EPSILON;
                for c in arr.iter_mut() {
                    *c = (*c - mm.min + f64::EPSILON) / range
                }
            }
            BoostNorm::Normalized => {
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

register_python!(enum BoostNorm;);
