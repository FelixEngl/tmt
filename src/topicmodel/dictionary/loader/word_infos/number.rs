use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use tinyset::Fits64;
use crate::register_python;
use crate::topicmodel::dictionary::loader::helper::gen_freedict_tei_reader::ENumberElement;
use crate::topicmodel::dictionary::metadata::loaded::impl_try_from_as_unpack;

register_python! {
    enum GrammaticalNumber;
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumIter)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum GrammaticalNumber {
    #[strum(to_string = "sg", serialize = "sg.")]
    Singular = 0,
    #[strum(to_string = "pl", serialize = "pl.", serialize = "plural", serialize = "Pluralwort")]
    #[strum(serialize = "Pluralw")]
    Plural = 1,
    #[strum(to_string = "kein Singular", serialize = "no singular")]
    NoSingular = 2,
    #[strum(to_string = "kein Plural", serialize = "no plural", serialize = "keine Mehrzahl")]
    NoPlural = 3,
    #[strum(to_string = "usually pl")]
    UsuallyPlural = 4,
    #[strum(to_string = "sg only")]
    SingularOnly = 5,
    #[strum(to_string = "only plural")]
    PluralOnly = 6,
}

impl_try_from_as_unpack! {
    GrammaticalNumber => GrammaticalNumber
}


// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl GrammaticalNumber {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl Fits64 for GrammaticalNumber {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        GrammaticalNumber::try_from(x).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
}

impl From<ENumberElement> for GrammaticalNumber {
    fn from(value: ENumberElement) -> Self {
        match value {
            ENumberElement::Sg => {
                Self::Singular
            }
            ENumberElement::Pl => {
                Self::Plural
            }
        }
    }
}




