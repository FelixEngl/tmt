use std::fmt::{Display, Formatter};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods, FromPyObject, IntoPy, PyObject, Python};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};
use tinyset::Fits64;
use crate::{impl_py_stub, register_python};
use crate::topicmodel::dictionary::direction::{Language as DirLang, LanguageKind};
use crate::topicmodel::dictionary::loader::helper::gen_freedict_tei_reader::{LangAttribute as FreeDictLangAttribute};
use crate::topicmodel::dictionary::loader::helper::gen_iate_tbx_reader::{LangAttribute as IateLangAttribute};
use crate::topicmodel::dictionary::loader::iate_reader::{AdministrativeStatus};
use crate::topicmodel::dictionary::loader::helper::gen_ms_terms_reader::{LangAttribute as MsTermsAttribute, ETermNoteElement};
use crate::topicmodel::dictionary::metadata::loaded::impl_try_from_as_unpack;


register_python! {
    enum Language;
    struct LanguageDirection;
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(name = "DictionaryLanguageDirection", eq, hash, frozen)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct LanguageDirection {
    #[pyo3(get)]
    pub lang_a: Language,
    #[pyo3(get)]
    pub lang_b: Language
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl LanguageDirection {
    #[new]
    pub const fn new(lang_a: Language, lang_b: Language) -> Self {
        Self {
            lang_a,
            lang_b
        }
    }

    /// Returns true if this contains [other] as any position
    #[doc(hidden)]
    fn __contains__(&self, other: Language) -> bool {
        self.contains(other)
    }

    #[doc(hidden)]
    fn __getitem__(&self, language_kind: LanguageKind) -> Language {
        match language_kind {
            LanguageKind::A => self.lang_a,
            LanguageKind::B => self.lang_b
        }
    }

    /// Returns an inverted variant
    pub const fn invert(&self) -> LanguageDirection {
        Self {
            lang_a: self.lang_b,
            lang_b: self.lang_a
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
        self.lang_a == lang_a && self.lang_b == lang_b
    }
}


impl LanguageDirection {

    pub const EN_DE: LanguageDirection = LanguageDirection::new(Language::English, Language::German);
    pub const DE_EN: LanguageDirection = LanguageDirection::EN_DE.invert();

    /// Returns true if this contains [other] as any position
    pub fn contains(&self, other: Language) -> bool {
        self.lang_a == other || self.lang_b == other
    }

    pub fn get<L: DirLang>(&self) -> Language {
        if L::LANG.is_a() {
            self.lang_a
        } else {
            self.lang_b
        }
    }

    pub fn check_lang<L: DirLang>(&self, other: Language) -> bool {
        if L::LANG.is_a() {
            self.lang_a == other
        } else {
            self.lang_b == other
        }
    }

    pub fn same_lang<L: DirLang>(&self, other: &LanguageDirection) -> bool {
        if L::LANG.is_a() {
            self.lang_a == other.lang_a
        } else {
            self.lang_b == other.lang_b
        }
    }
}

impl Display for LanguageDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}[A] to {}[B])", self.lang_a, self.lang_b)
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
                LanguageDirection{lang_a, lang_b}.into_py(py)
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


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(name = "DictionaryLanguage", eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr)]
#[derive(TryFromPrimitive, IntoPrimitive)]
#[derive(Serialize, Deserialize)]
#[repr(u64)]
/// The recognized
pub enum Language {
    #[strum(serialize = "en", serialize = "english")]
    English = 0,
    #[strum(serialize = "de", serialize = "german", serialize = "Dt.")]
    German = 1,
    #[strum(serialize = "italian", serialize = "Ital.")]
    Italian = 2,
    #[strum(serialize = "french", serialize = "from French")]
    French = 3,
    #[strum(serialize = "latin", serialize = "Lat.", serialize = "lat.")]
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
    fn to(self, other: Language) -> LanguageDirection {
        LanguageDirection::new(self, other)
    }
}

impl Fits64 for Language {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Language::try_from(x).unwrap()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
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
