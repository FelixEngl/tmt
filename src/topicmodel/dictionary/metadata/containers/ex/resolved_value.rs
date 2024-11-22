use std::fmt::{Debug, Display};
use std::hash::BuildHasher;
use derive_more::From;
use pyo3::{FromPyObject, IntoPy, PyObject, Python};
use serde::{Deserialize, Serialize};
use string_interner::Symbol;
use thiserror::Error;
use crate::{impl_py_stub};
use crate::topicmodel::dictionary::word_infos::*;
use strum::Display;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[derive(FromPyObject)]
#[derive(From, Display)]
pub enum ResolvableValue {
    #[strum(to_string = "({0:?})")] Language(Language),
    #[strum(to_string = "({0:?})")] Domain(Domain),
    #[strum(to_string = "({0:?})")] Register(Register),
    #[strum(to_string = "({0:?})")] GrammaticalGender(GrammaticalGender),
    #[strum(to_string = "({0:?})")] PartOfSpeech(PartOfSpeech),
    #[strum(to_string = "({0:?})")] PartOfSpeechTag(PartOfSpeechTag),
    #[strum(to_string = "({0:?})")] Region(Region),
    #[strum(to_string = "({0:?})")] GrammaticalNumber(GrammaticalNumber),
    #[strum(to_string = "({0:?})")] RawId(u64),
    #[strum(to_string = "({0:?})")] String(String),
}

// macro_rules! impl_try_into {
//     ($($targ: ident => $t: ty);+ $(;)?) => {
//         $(
//         impl TryInto<$t> for ResolvableValue {
//             type Error = Self;
//
//             fn try_into(self) -> Result<$t, Self> {
//                 match self {
//                     ResolvableValue::$targ(value) => Ok(value),
//                     other => Err(other)
//                 }
//             }
//         }
//         )+
//     };
// }
//
//
// impl_try_into! {
//     Language => Language;
//     Domain => Domain;
//     Register => Register;
//     GrammaticalGender => GrammaticalGender;
//     PartOfSpeech => PartOfSpeech;
//     PartOfSpeechTag => PartOfSpeechTag;
//     Region => Region;
//     GrammaticalNumber => GrammaticalNumber;
//     RawId => u64;
// }

// impl TryInto<usize> for ResolvableValue {
//     type Error = Self;
//
//     fn try_into(self) -> Result<usize, Self::Error> {
//         match self {
//             ResolvableValue::RawId(v) => {Ok(v as usize)},
//             other => Err(other)
//         }
//     }
// }

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[derive(FromPyObject)]
pub struct ResolvedValue(pub ResolvableValue, pub u32);

impl Display for ResolvedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> From<(T, u32)> for ResolvedValue where T: Into<ResolvableValue> {
    fn from(value: (T, u32)) -> Self {
        Self(value.0.into(), value.1)
    }
}

#[derive(Debug, Clone, Error)]
#[error("The resolved value was expected to be of type {0} but got {1:?}!")]
pub struct WrongResolvedValueError<T: Debug + Clone>(pub &'static str, pub T);

macro_rules! impl_try_from_as_unpack {
    ($($i: ident => $a: ty);+ $(;)?) => {
        $(
        const _: () = {
            use $crate::topicmodel::dictionary::metadata::ex::{ResolvedValue, ResolvableValue, WrongResolvedValueError};
            impl TryFrom<ResolvedValue> for ($a, u32) {
                type Error = WrongResolvedValueError<ResolvedValue>;

                fn try_from(ResolvedValue(value, count): ResolvedValue) -> Result<Self, Self::Error> {
                    match value {
                        ResolvableValue::$i(value) => Ok((value, count)),
                        other => Err(WrongResolvedValueError(stringify!($a), ResolvedValue(other, count))),
                    }
                }
            }

            impl TryFrom<ResolvableValue> for $a {
                type Error = WrongResolvedValueError<ResolvableValue>;

                fn try_from(value: ResolvableValue) -> Result<Self, Self::Error> {
                    match value {
                        ResolvableValue::$i(value) => Ok(value),
                        other => Err(WrongResolvedValueError(stringify!($a), other))
                    }
                }
            }
        };
        )+
    };
}

macro_rules! impl_try_into_as_unpack {
    ($($i: ident => $a: ty);+ $(;)?) => {
        $(
            impl TryInto<($a, u32)> for ResolvedValue {
                type Error = WrongResolvedValueError<ResolvedValue>;

                fn try_into(self) -> Result<($a, u32), Self::Error> {
                    match self {
                        ResolvedValue(ResolvableValue::$i(int), value) => Ok((int as $a, value)),
                        other => Err(WrongResolvedValueError(stringify!($i), other))
                    }
                }
            }

            impl TryInto<$a> for ResolvableValue {
                type Error = WrongResolvedValueError<ResolvableValue>;

                fn try_into(self) -> Result<$a, Self::Error> {
                    match self {
                        ResolvableValue::$i(int) => Ok(int as $a),
                        other => Err(WrongResolvedValueError(stringify!($i), other))
                    }
                }
            }
        )+
    }
}

impl_try_into_as_unpack! {
    RawId => u64;
    RawId => usize;
}



pub(crate) use impl_try_from_as_unpack;

impl ResolvedValue {
    pub fn resolve_with_interner<B: string_interner::backend::Backend<Symbol=S>, S: Symbol, H: BuildHasher>(&self, interner: &mut string_interner::StringInterner<B, H>) -> Result<(S, u32), WrongResolvedValueError<ResolvedValue>> {
        match self {
            ResolvedValue(ResolvableValue::String(value), count) => { Ok((interner.get_or_intern(&value), *count)) }
            other => Err(WrongResolvedValueError("string", other.clone()))
        }
    }

    pub fn try_resolve_with_interner<B: string_interner::backend::Backend<Symbol=S>, S: Symbol, H: BuildHasher>(&self, interner: &string_interner::StringInterner<B, H>) -> Result<Option<(S, u32)>, WrongResolvedValueError<ResolvedValue>> {
        match self {
            ResolvedValue(ResolvableValue::String(value), count) => { Ok(interner.get(value).map(|v| (v, *count))) }
            other => Err(WrongResolvedValueError("string", other.clone()))
        }
    }
}

impl ResolvableValue {

    pub fn resolve_with_interner<B: string_interner::backend::Backend<Symbol=S>, S: Symbol, H: BuildHasher>(&self, interner: &mut string_interner::StringInterner<B, H>) -> Result<S, WrongResolvedValueError<ResolvableValue>> {
        match self {
            ResolvableValue::String(value) => { Ok(interner.get_or_intern(&value)) }
            other => Err(WrongResolvedValueError("string", other.clone()))
        }
    }

    pub fn try_resolve_with_interner<B: string_interner::backend::Backend<Symbol=S>, S: Symbol, H: BuildHasher>(&self, interner: &string_interner::StringInterner<B, H>) -> Result<Option<S>, WrongResolvedValueError<ResolvableValue>> {
        match self {
            ResolvableValue::String(value) => { Ok(interner.get(value)) }
            other => Err(WrongResolvedValueError("string", other.clone()))
        }
    }
}

impl From<&str> for ResolvableValue {
    fn from(value: &str) -> Self {
        ResolvableValue::String(value.to_string())
    }
}

impl IntoPy<PyObject> for ResolvableValue {
    delegate::delegate! {
        to match self {
            ResolvableValue::Language(value) => value,
            ResolvableValue::Domain(value) => value,
            ResolvableValue::Register(value) => value,
            ResolvableValue::GrammaticalGender(value) => value,
            ResolvableValue::PartOfSpeech(value) => value,
            ResolvableValue::PartOfSpeechTag(value) => value,
            ResolvableValue::Region(value) => value,
            ResolvableValue::GrammaticalNumber(value) => value,
            ResolvableValue::String(value) => value,
            ResolvableValue::RawId(value) => value,
        } {
            fn into_py(self, py: Python<'_>) -> PyObject;
        }
    }
}

impl IntoPy<PyObject> for ResolvedValue {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self.0, self.1).into_py(py)
    }
}

crate::impl_py_type_def_special!(
    ResolvedValueType; {
        output: {
            builder()
            .with::<(Language, u32)>()
            .with::<(Domain, u32)>()
            .with::<(Register, u32)>()
            .with::<(GrammaticalGender, u32)>()
            .with::<(PartOfSpeech, u32)>()
            .with::<(Region, u32)>()
            .with::<(GrammaticalNumber, u32)>()
            .with::<(PartOfSpeechTag, u32)>()
            .with::<(String, u32)>()
            .with::<(u64, u32)>()
            .build_output()
        }
        input: {
            builder()
            .with::<(Language, u32)>()
            .with::<(Domain, u32)>()
            .with::<(Register, u32)>()
            .with::<(GrammaticalGender, u32)>()
            .with::<(PartOfSpeech, u32)>()
            .with::<(Region, u32)>()
            .with::<(GrammaticalNumber, u32)>()
            .with::<(PartOfSpeechTag, u32)>()
            .with::<(String, u32)>()
            .with::<(u64, u32)>()
            .build_input()
        }
    }
);

impl_py_stub!(ResolvedValue: ResolvedValueType);
