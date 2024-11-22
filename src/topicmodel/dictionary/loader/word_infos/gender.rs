use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use tinyset::Fits64;
use crate::register_python;
use crate::topicmodel::dictionary::loader::helper::gen_freedict_tei_reader::EGenElement;
use crate::topicmodel::dictionary::metadata::ex::impl_try_from_as_unpack;

register_python! {
    enum GrammaticalGender;
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumIter)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u16)]
pub enum GrammaticalGender {
    #[strum(to_string = "f", serialize = "female", serialize = "f.", serialize = "feminine")]
    Feminine = 0,
    #[strum(to_string = "m", serialize = "male", serialize = "m.", serialize = "masculine")]
    Masculine = 1,
    #[strum(to_string = "n", serialize = "neutral", serialize = "n.", serialize = "neuter")]
    Neutral = 2,
    #[strum(to_string = "not f")]
    NotFeminine = 3
}

impl_try_from_as_unpack! {
    GrammaticalGender => GrammaticalGender
}


// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl GrammaticalGender {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl Fits64 for GrammaticalGender {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        GrammaticalGender::try_from(x as u16).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        (self as u16) as u64
    }
}

impl From<EGenElement> for GrammaticalGender {
    fn from(value: EGenElement) -> Self {
        match value {
            EGenElement::Neut => {
                GrammaticalGender::Neutral
            }
            EGenElement::Masc => {
                GrammaticalGender::Masculine
            }
            EGenElement::Fem => {
                GrammaticalGender::Feminine
            }
        }
    }
}

