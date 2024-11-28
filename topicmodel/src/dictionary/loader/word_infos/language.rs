use std::fmt::{Display, Formatter};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods, FromPyObject, IntoPy, PyObject, Python};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};
use tinyset::Fits64;
use ldatranslate_toolkit::{impl_py_stub, register_python};
use crate::dictionary::direction::{Language as DirLang, LanguageMarker};
use crate::dictionary::loader::helper::gen_freedict_tei_reader::{LangAttribute as FreeDictLangAttribute};
use crate::dictionary::loader::helper::gen_iate_tbx_reader::{LangAttribute as IateLangAttribute};
use crate::dictionary::loader::helper::gen_ms_terms_reader::{LangAttribute as MsTermsAttribute};
use crate::dictionary::metadata::ex::impl_try_from_as_unpack;


register_python! {
    enum Language;
    struct LanguageDirection;
}



#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(name = "DictionaryLanguage", eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr)]
#[derive(TryFromPrimitive, IntoPrimitive)]
#[derive(Serialize, Deserialize)]
#[repr(u16)]
/// The recognized
pub enum Language {
    #[strum(to_string = "en", serialize = "english", serialize = "English")]
    English = 0,
    #[strum(to_string = "de", serialize = "german", serialize = "Dt.", serialize = "German")]
    German = 1,
    #[strum(to_string = "it", serialize = "Italian", serialize = "italian", serialize = "Ital.")]
    Italian = 2,
    #[strum(to_string = "fr", serialize = "french", serialize = "from French", serialize = "French")]
    French = 3,
    #[strum(to_string = "latin", serialize = "Latin", serialize = "Lat.", serialize = "lat.")]
    Latin = 4
}

impl_try_from_as_unpack! {
    Language => Language
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Language {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl Language {
    pub fn to(self, other: Language) -> LanguageDirection {
        LanguageDirection::new(self, other)
    }
}

impl Fits64 for Language {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Language::try_from(x as u16).unwrap()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        (self as u16) as u64
    }
}

impl From<FreeDictLangAttribute> for Language {
    fn from(value: FreeDictLangAttribute) -> Self {
        match value {
            FreeDictLangAttribute::En => {
                Language::English
            }
            FreeDictLangAttribute::De => {
                Language::German
            }
        }
    }
}

impl From<IateLangAttribute> for Language {
    fn from(value: IateLangAttribute) -> Self {
        match value {
            IateLangAttribute::En => {
                Language::English
            }
            IateLangAttribute::De => {
                Language::German
            }
        }
    }
}

impl From<MsTermsAttribute> for Language {
    fn from(value: MsTermsAttribute) -> Self {
        match value {
            MsTermsAttribute::EnGb | MsTermsAttribute::EnUs => {
                Language::English
            }
            MsTermsAttribute::DeDe => {
                Language::German
            }
        }
    }
}



#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(name = "DictionaryLanguageDirection", eq, hash, frozen)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct LanguageDirection {
    inner: [Language; 2]
}

impl AsRef<[Language; 2]> for LanguageDirection {
    fn as_ref(&self) -> &[Language; 2] {
        &self.inner
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl LanguageDirection {
    #[new]
    pub const fn new(lang_a: Language, lang_b: Language) -> Self {
        Self { inner: [lang_a, lang_b] }
    }

    #[getter]
    pub fn lang_a(&self) -> Language {
        self.inner[0]
    }

    #[getter]
    pub fn lang_b(&self) -> Language {
        self.inner[1]
    }

    /// Returns true if this contains [other] as any position
    #[doc(hidden)]
    fn __contains__(&self, other: Language) -> bool {
        self.contains(&other)
    }

    #[doc(hidden)]
    fn __getitem__(&self, language_kind: LanguageMarker) -> Language {
        match language_kind {
            LanguageMarker::A => self.inner[0],
            LanguageMarker::B => self.inner[1]
        }
    }

    /// Returns an inverted variant
    pub const fn invert(&self) -> LanguageDirection {
        Self {
            inner: [self.inner[1], self.inner[0]]
        }
    }

    /// Returns an inverted variant
    #[doc(hidden)]
    fn __invert__(&self) -> LanguageDirection {
        self.invert()
    }

    /// Returns an inverted variant
    #[doc(hidden)]
    fn __neg__(&self) -> LanguageDirection {
        self.invert()
    }

    /// Returns true if this points from [lang_a] to [lang_b]
    pub fn is_direction_in(&self, lang_a: Language, lang_b: Language) -> bool {
        self.inner[0] == lang_a && self.inner[1] == lang_b
    }
}


impl LanguageDirection {

    pub const EN_DE: LanguageDirection = LanguageDirection::new(Language::English, Language::German);
    pub const DE_EN: LanguageDirection = LanguageDirection::EN_DE.invert();

    /// Returns true if this contains [other] as any position
    pub fn contains(&self, other: &Language) -> bool {
        self.inner[0].eq(other) || self.inner[1].eq(other)
    }

    pub fn get<L: DirLang>(&self) -> Language {
        if L::LANG.is_a() {
            self.inner[0]
        } else {
            self.inner[1]
        }
    }

    pub fn check_lang<L: DirLang>(&self, other: Language) -> bool {
        if L::LANG.is_a() {
            self.inner[0] == other
        } else {
            self.inner[1] == other
        }
    }

    pub fn same_lang<L: DirLang>(&self, other: &LanguageDirection) -> bool {
        if L::LANG.is_a() {
            self.inner[0] == other.inner[0]
        } else {
            self.inner[1] == other.inner[1]
        }
    }
}

impl Display for LanguageDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}[A] to {}[B])", self.inner[0], self.inner[1])
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(FromPyObject)]
pub enum LanguageDirectionArg {
    Tuple(Language, Language),
    Direction(LanguageDirection)
}

impl IntoPy<PyObject> for LanguageDirectionArg {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            LanguageDirectionArg::Tuple(lang_a, lang_b) => {
                LanguageDirection::new(lang_a, lang_b).into_py(py)
            }
            LanguageDirectionArg::Direction(value) => {
                value.into_py(py)
            }
        }
    }
}

impl_py_stub!(LanguageDirectionArg {
    output: {
        builder()
        .with::<LanguageDirection>()
        .with::<(Language, Language)>()
        .build_output()
    }
    input: {
        LanguageDirection::type_output()
    }
});


