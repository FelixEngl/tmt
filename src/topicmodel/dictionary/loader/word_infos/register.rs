use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIter, EnumString, IntoStaticStr};
use tinyset::Fits64;
use crate::register_python;
use crate::topicmodel::dictionary::loader::iate_reader::AdministrativeStatus;
use crate::topicmodel::dictionary::metadata::domain_matrix::DomainModelIndex;
use crate::topicmodel::dictionary::metadata::loaded::impl_try_from_as_unpack;
use crate::topicmodel::dictionary::word_infos::Domain;

register_python! {
    enum Register;
}

/// In sociolinguistics, a register is a variety of language used for a particular purpose or particular communicative situation
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumCount, EnumIter)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum Register {
    #[strum(to_string = "humor.", serialize = "humor", serialize = "hum.", serialize = "hum")]
    Humor = 0,
    #[strum(to_string = "vulg.", serialize = "vulg", serialize = "derb")]
    Vulg = 1,
    #[strum(to_string = "techn.", serialize = "techn")]
    Techn = 2,
    #[strum(to_string = "coll.", serialize = "coll", serialize = "ugs", serialize = "ugs.")]
    #[strum(serialize = "ug")]
    Coll = 3,
    /// Gehoben
    #[strum(to_string = "geh.", serialize = "geh")]
    Geh = 4,
    #[strum(to_string = "slang", serialize = "slang.", serialize = "sl.", serialize = "jargon")]
    Slang = 5,
    #[strum(to_string = "iron.", serialize = "iron")]
    Iron = 6,
    #[strum(to_string = "formal", serialize = "formal.", serialize = "formell")]
    Formal = 7,
    #[strum(to_string = "euphem.", serialize = "euphem")]
    Euphem = 8,
    #[strum(to_string = "literary", serialize = "literary.")]
    Literary = 9,
    #[strum(to_string = "dialect", serialize = "dialect.")]
    Dialect = 10,
    /// DictCC
    #[strum(to_string = "archaic", serialize = "veraltet", serialize = "veraltend")]
    #[strum(serialize = "dated", serialize = "alt", serialize = "obs.")]
    #[strum(serialize = "altertümlich", serialize = "veraltentd", serialize = "frühere Bezeichnung")]
    #[strum(serialize = "ancient name", serialize = "becoming dated")]
    #[strum(serialize = "altertümelnd", serialize = "slightly dated")]
    Archaic = 11,
    /// DictCC
    #[strum(to_string = "rare", serialize = "selten", serialize = "very rare")]
    Rare = 12,
    /// DictCC -
    #[strum(to_string = "pej.")]
    Pejorativ = 13,
    /// DictCC - also figurative
    #[strum(to_string = "fig.")]
    Figurative = 14,
    #[strum(to_string = "also fig.", serialize = "auch fig.")]
    AlsoFigurative = 15,
    /// spelling variant (less common)
    #[strum(to_string = "spv.", serialize = "Rsv.")]
    SpellingVariant = 16,
    /// official language; administration
    #[strum(to_string = "adm.")]
    Admin = 17,
    /// Übertragen: giftig -> virulently
    #[strum(to_string = "übtr.")]
    Transfer = 18,
    /// Netzjargon
    #[strum(to_string = "Chat-Jargon", serialize = "internet slang", serialize = "chat jargon")]
    NetJargon = 19,
    /// Informal
    #[strum(to_string = "informell")]
    Informal = 20,
    /// Quantity
    #[strum(to_string = "Mengenangabe")]
    QuantityInformation = 21,
    #[strum(to_string = "IATEPreferred")]
    IATEPreferred = 22
}

impl_try_from_as_unpack! {
    Register => Register
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Register {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl DomainModelIndex for Register {
    #[inline(always)]
    fn get(self) -> usize {
        Domain::COUNT + (self as u64) as usize
    }
}

impl Fits64 for Register {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Register::try_from(x).unwrap()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
}

impl TryFrom<AdministrativeStatus> for Register {
    type Error = AdministrativeStatus;

    fn try_from(value: AdministrativeStatus) -> Result<Self, AdministrativeStatus> {
        match value {
            AdministrativeStatus::Obsolete | AdministrativeStatus::Deprecated => {
                Ok(Self::Archaic)
            }
            AdministrativeStatus::Preferred => {
                Ok(Self::IATEPreferred)
            }
            other => {
                Err(other)
            }
        }
    }
}

