use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};
use tinyset::Fits64;
use crate::register_python;
use crate::topicmodel::dictionary::metadata::ex::impl_try_from_as_unpack;

register_python!(
    enum Region;
);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum Region {
    #[strum(
        to_string = "BE",
        serialize = "eBr.",
        serialize = "Br.",
        serialize = "BR.",
        serialize = "Br,",
        serialize = "also Br.",
        serialize = "Britain",
        serialize = "British",
        serialize = "British-Army",
        serialize = "British-Columbia",
        serialize = "British-Isles",
        serialize = "British-Royal-Navy",
        serialize = "British-airforce",
        serialize = "UK",
    )]
    BritishEnglish = 0,
    #[strum(
        to_string = "AE",
        serialize = "eAm.",
        serialize = "Am.",
        serialize = "AM.",
        serialize = "Am .",
        serialize = "mainly Am.",
        serialize = "North-American",
        serialize = "North-America",
        serialize = "US",
    )]
    AmericanEnglish = 1,
    #[strum(
        to_string = "Aus.",
        serialize = "Austr.",
    )]
    AustralianEnglish = 2,
    #[strum(
        to_string = "NZ",
    )]
    NewZealandEnglish = 3,
    #[strum(
        to_string = "Can.",
        serialize = "Canada",
        serialize = "Canadian",
    )]
    CanadianEnglish = 4,
    #[strum(to_string = "Irish", serialize = "Ir.", serialize = "Irl.")]
    IrishEnglish = 5,
    #[strum(to_string = "Ind.")]
    IndianEnglish = 6,
    #[strum(to_string = "S.Afr.", serialize = "South Africa")]
    SouthAfricanEnglish = 7,
    #[strum(
        to_string = "Scot.",
        serialize = "Sc.",
        serialize = "Scottish",
    )]
    ScottishEnglish = 8,
    #[strum(
        to_string = "österr.",
        serialize = "Ös.",
        serialize = "Austria",
        serialize = "Austrian",
    )]
    AustrianGerman = 9,
    #[strum(to_string = "südd.", serialize = "Süddt.")]
    SouthGerman = 10,
    #[strum(
        to_string = "nordd.",
        serialize = "Norddt.",
        serialize = "North-German",
    )]
    NorthGerman = 11,
    #[strum(to_string = "ostd.", serialize = "Ostdt.")]
    EastGerman = 12,
    #[strum(
        to_string = "schweiz.",
        serialize = "Schw.",
        serialize = "Swiss",
        serialize = "Swiss-German",
    )]
    SwissGerman = 13,
    #[strum(to_string = "regional")]
    Regional = 14,
    #[strum(to_string = "Mittelwestdt.")]
    MiddleWestGerman = 15,
    #[strum(to_string = "Südwestdt.")]
    SouthWestGerman = 16,
    #[strum(to_string = "Nordwestdt.")]
    NorthWestGerman = 17,
    #[strum(to_string = "BW", serialize = "Württemberg", serialize = "BW.")]
    BadenWuerttembergGerman = 18,
    #[strum(to_string = "Mittelostdt.")]
    MiddleEastGerman = 19,
    #[strum(to_string = "Südostdt.")]
    SouthEastGerman = 20,
    #[strum(to_string = "Nordostdt.")]
    NorthEastGerman = 21,
    #[strum(to_string = "Mitteldt.")]
    MiddleGerman = 22,
    #[strum(
        to_string = "Bayr.",
        serialize = "Bavaria",
        serialize = "Bavarian",
    )]
    BavarianGerman = 23,
    #[strum(to_string = "Northern Irish")]
    NorthernIrish = 24,
    #[strum(
        to_string = "Oberdt.",
        serialize = "Upper-German",
    )]
    UpperGerman = 25,
    #[strum(to_string = "Ostös.")]
    EastAustrianGerman = 26,
    #[strum(to_string = "Berlin")]
    BerlinGerman = 27,
    #[strum(to_string = "Schwäb.", serialize = "Sachsen")]
    SwabianGerman = 28,
    #[strum(to_string = "Westös.")]
    WestAustrianGerman = 29,
    #[strum(to_string = "Wien")]
    ViennaGerman = 30,
    #[strum(to_string = "Tirol")]
    TyrolGerman = 31,
    #[strum(to_string = "Northern English")]
    NorthEnglish = 32,
    #[strum(to_string = "DDR")]
    DDRGerman = 33,
    #[strum(to_string = "Pfalz")]
    PfalzGerman = 34,
    #[strum(to_string = "Südtirol")]
    SouthTyrolGerman = 35,
    #[strum(to_string = "Ostmitteldt.")]
    EastMiddleGerman = 36,
    #[strum(to_string = "SE Asia")]
    SouthEastAsianEnglish = 37,
    #[strum(to_string = "Hessen")]
    HesseGerman = 38,
    #[strum(to_string = "Lux.")]
    LuxenbourgGerman = 39,
    #[strum(to_string = "Welch")]
    WelchEnglish = 40,
    #[strum(to_string = "Rheinl.")]
    RhinelandPalatinateGerman = 41,
    #[strum(to_string = "Sächs.")]
    SaxonyGerman = 42,
    #[strum(to_string = "Westdt.")]
    WestGerman = 43,
    #[strum(to_string = "Lie.")]
    LiechtensteinGerman = 44,
    #[strum(to_string = "Westfalen")]
    WestphaliaGerman = 45,
    #[strum(to_string = "Südostös.")]
    SouthEastAustrianGerman = 46,
    #[strum(to_string = "Nordostös.")]
    NorthEastAustrianGerman = 47,
    #[strum(to_string = "Nordwestös.")]
    NorthWestAustrianGerman = 48,
    #[strum(to_string = "Südwestös.")]
    SouthWestAustrianGerman = 49,
    #[strum(to_string = "Westschw.")]
    WestSwissGerman = 50,
    #[strum(to_string = "Nordirl.")]
    NorthIrishEnglish = 51,
    #[strum(to_string = "Mittelös.")]
    MiddleAustrianGerman = 52,
    #[strum(to_string = "Franken")]
    FranconianGerman = 53,
    #[strum(to_string = "Ostschw.")]
    EastSwissGerman = 54,
    #[strum(to_string = "Germanic")]
    Germanic = 55
}

impl_try_from_as_unpack! {
    Region => Region
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Region {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}


impl TryFrom<crate::topicmodel::dictionary::loader::helper::gen_ms_terms_reader::LangAttribute> for Region {
    type Error = ();

    fn try_from(value: crate::topicmodel::dictionary::loader::helper::gen_ms_terms_reader::LangAttribute) -> Result<Self, Self::Error> {
        match value {
            crate::topicmodel::dictionary::loader::helper::gen_ms_terms_reader::LangAttribute::EnGb => {
                Ok(Region::BritishEnglish)
            }
            crate::topicmodel::dictionary::loader::helper::gen_ms_terms_reader::LangAttribute::EnUs => {
                Ok(Region::AmericanEnglish)
            }
            _ => Err(())
        }
    }
}


impl Fits64 for Region {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Region::try_from(x).unwrap()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
}


