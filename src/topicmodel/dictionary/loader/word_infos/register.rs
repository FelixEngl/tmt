use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIter, EnumString, IntoStaticStr};
use tinyset::Fits64;
use crate::register_python;
use crate::topicmodel::dictionary::loader::iate_reader::AdministrativeStatus;
use crate::topicmodel::dictionary::metadata::dict_meta_topic_matrix::DomainModelIndex;
use crate::topicmodel::dictionary::metadata::ex::impl_try_from_as_unpack;
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
#[repr(u16)]
pub enum Register {
    #[strum(to_string = "humor.", serialize = "humor", serialize = "hum.", serialize = "hum", serialize = "Humor")]
    Humor = 0,
    #[strum(
        to_string = "vulg.",
        serialize = "vulg",
        serialize = "derb",
        serialize = "slur",
        serialize = "vulgar",
        serialize = "Vulg",
    )]
    Vulg = 1,
    #[strum(
        to_string = "techn.",
        serialize = "techn",
        serialize = "Techn",
    )]
    Techn = 2,
    #[strum(
        to_string = "coll.",
        serialize = "coll",
        serialize = "ugs",
        serialize = "ugs.",
        serialize = "ug",
        serialize = "Coll",
    )]
    Coll = 3,
    /// Gehoben
    #[strum(
        to_string = "geh.",
        serialize = "geh",
        serialize = "Geh",
    )]
    Geh = 4,
    #[strum(
        to_string = "slang",
        serialize = "slang.",
        serialize = "sl.",
        serialize = "jargon",
        serialize = "Slang",
    )]
    Slang = 5,
    #[strum(to_string = "iron.", serialize = "iron")]
    Iron = 6,
    #[strum(
        to_string = "formal",
        serialize = "formal.",
        serialize = "formell",
        serialize = "polite",
    )]
    Formal = 7,
    #[strum(
        to_string = "euphem.",
        serialize = "euphem",
        serialize = "euphemistic",
    )]
    Euphem = 8,
    #[strum(
        to_string = "literary",
        serialize = "literary.",
        serialize = "literally",
    )]
    Literary = 9,
    #[strum(to_string = "dialect", serialize = "dialect.")]
    Dialect = 10,
    /// DictCC
    #[strum(
        to_string = "archaic",
        serialize = "veraltet",
        serialize = "veraltend",
        serialize = "dated",
        serialize = "alt",
        serialize = "obs.",
        serialize = "altertümlich",
        serialize = "veraltentd",
        serialize = "frühere Bezeichnung",
        serialize = "ancient name",
        serialize = "becoming dated",
        serialize = "Ancient",
        serialize = "Anglian",
        serialize = "historic",
        serialize = "historical",
        serialize = "obsolete",
    )]
    Archaic = 11,
    /// DictCC
    #[strum(
        to_string = "rare",
        serialize = "selten",
        serialize = "very rare",
        serialize = "uncommon",
    )]
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
    #[strum(
        to_string = "Chat-Jargon",
        serialize = "internet slang",
        serialize = "chat jargon",
        serialize = "Leet",
    )]
    NetJargon = 19,
    /// Informal
    #[strum(
        to_string = "informell",
        serialize = "impolite",
        serialize = "informal",
    )]
    Informal = 20,
    /// Quantity
    #[strum(to_string = "Mengenangabe")]
    QuantityInformation = 21,
    #[strum(to_string = "IATEPreferred")]
    IATEPreferred = 22,
    #[strum(to_string = "misspelling")]
    Miss = 23,
    #[strum(
        to_string = "common-gender",
        serialize = "gender-neutral",
    )]
    ComGend = 24
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
    fn as_index(self) -> usize {
        Domain::COUNT + (self as u64) as usize
    }
}

impl Fits64 for Register {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Register::try_from(x as u16).unwrap()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        (self as u16) as u64
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

