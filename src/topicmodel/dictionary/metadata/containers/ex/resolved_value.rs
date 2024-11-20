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
pub enum ResolvedValue {
    #[strum(to_string = "({0:?})")] Language((Language, u32)),
    #[strum(to_string = "({0:?})")] Domain((Domain, u32)),
    #[strum(to_string = "({0:?})")] Register((Register, u32)),
    #[strum(to_string = "({0:?})")] GrammaticalGender((GrammaticalGender, u32)),
    #[strum(to_string = "({0:?})")] PartOfSpeech((PartOfSpeech, u32)),
    #[strum(to_string = "({0:?})")] PartOfSpeechTag((PartOfSpeechTag, u32)),
    #[strum(to_string = "({0:?})")] Region((Region, u32)),
    #[strum(to_string = "({0:?})")] GrammaticalNumber((GrammaticalNumber, u32)),
    #[strum(to_string = "({0:?})")] RawId((u64, u32)),
    #[strum(to_string = "({0:?})")] String((String, u32)),
}

#[derive(Debug, Clone, Error)]
#[error("The resolved value was expected to be of type {0} but got {1:?}!")]
pub struct WrongResolvedValueError(pub &'static str, pub ResolvedValue);

macro_rules! impl_try_from_as_unpack {
    ($($i: ident => $a: ty);+ $(;)?) => {
        $(
        const _: () = {
            use $crate::topicmodel::dictionary::metadata::ex::{ResolvedValue, WrongResolvedValueError};
            impl TryFrom<ResolvedValue> for ($a, u32) {
                type Error = WrongResolvedValueError;

                fn try_from(value: ResolvedValue) -> Result<Self, Self::Error> {
                    match value {
                        ResolvedValue::$i(value) => Ok(value),
                        other => Err(WrongResolvedValueError(stringify!($a), other))
                    }
                }
            }
        };
        )+
    };
}

impl TryInto<(u64, u32)> for ResolvedValue {
    type Error = WrongResolvedValueError;

    fn try_into(self) -> Result<(u64, u32), Self::Error> {
        match self {
            ResolvedValue::RawId(int) => Ok(int),
            other => Err(WrongResolvedValueError("int", other))
        }
    }
}

impl TryInto<(usize, u32)> for ResolvedValue {
    type Error = WrongResolvedValueError;

    fn try_into(self) -> Result<(usize, u32), Self::Error> {
        match self {
            ResolvedValue::RawId((a, b)) => Ok((a as usize, b)),
            other => Err(WrongResolvedValueError("int", other))
        }
    }
}

impl TryInto<(String, u32)> for ResolvedValue {
    type Error = WrongResolvedValueError;

    fn try_into(self) -> Result<(String, u32), Self::Error> {
        match self {
            ResolvedValue::String(value) => Ok(value),
            other => Err(WrongResolvedValueError("String", other))
        }
    }
}

pub(crate) use impl_try_from_as_unpack;

impl ResolvedValue {
    pub fn resolve_with_interner<B: string_interner::backend::Backend<Symbol=S>, S: Symbol, H: BuildHasher>(&self, interner: &mut string_interner::StringInterner<B, H>) -> Result<(S, u32), WrongResolvedValueError> {
        match self {
            ResolvedValue::String(value) => { Ok((interner.get_or_intern(&value.0), value.1)) }
            other => Err(WrongResolvedValueError("", other.clone()))
        }
    }

    pub fn try_resolve_with_interner<B: string_interner::backend::Backend<Symbol=S>, S: Symbol, H: BuildHasher>(&self, interner: &string_interner::StringInterner<B, H>) -> Result<Option<(S, u32)>, WrongResolvedValueError> {
        match self {
            ResolvedValue::String(value) => { Ok(interner.get(&value.0).map(|v| (v, value.1))) }
            other => Err(WrongResolvedValueError("", other.clone()))
        }
    }
}

impl From<(&str, u32)> for ResolvedValue {
    fn from((value, count): (&str, u32)) -> Self {
        Self::String((value.to_string(), count))
    }
}

impl IntoPy<PyObject> for ResolvedValue {
    delegate::delegate! {
        to match self {
            ResolvedValue::Language(value) => value,
            ResolvedValue::Domain(value) => value,
            ResolvedValue::Register(value) => value,
            ResolvedValue::GrammaticalGender(value) => value,
            ResolvedValue::PartOfSpeech(value) => value,
            ResolvedValue::PartOfSpeechTag(value) => value,
            ResolvedValue::Region(value) => value,
            ResolvedValue::GrammaticalNumber(value) => value,
            ResolvedValue::String(value) => value,
            ResolvedValue::RawId(value) => value,
        } {
            fn into_py(self, py: Python<'_>) -> PyObject;
        }
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
