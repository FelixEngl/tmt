use std::fmt::{Display, Formatter};
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
    #[strum(to_string = "{0}")] Language(Language),
    #[strum(to_string = "{0}")] Domain(Domain),
    #[strum(to_string = "{0}")] Register(Register),
    #[strum(to_string = "{0}")] GrammaticalGender(GrammaticalGender),
    #[strum(to_string = "{0}")] PartOfSpeech(PartOfSpeech),
    #[strum(to_string = "{0}")] PartOfSpeechTag(PartOfSpeechTag),
    #[strum(to_string = "{0}")] Region(Region),
    #[strum(to_string = "{0}")] GrammaticalNumber(GrammaticalNumber),
    #[strum(to_string = "{0}")] RawId(u64),
    #[strum(to_string = "{0}")] String(String),
}

#[derive(Debug, Clone, Error)]
#[error("The resolved value was expected to be of type {0} but got {1:?}!")]
pub struct WrongResolvedValueError(pub &'static str, pub ResolvedValue);

macro_rules! impl_try_from_as_unpack {
    ($($i: ident => $a: ty);+ $(;)?) => {
        $(
        const _: () = {
            use $crate::topicmodel::dictionary::metadata::ex::{ResolvedValue, WrongResolvedValueError};
            impl TryFrom<ResolvedValue> for $a {
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

impl TryInto<u64> for ResolvedValue {
    type Error = WrongResolvedValueError;

    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            ResolvedValue::RawId(int) => Ok(int),
            other => Err(WrongResolvedValueError("int", other))
        }
    }
}

impl TryInto<String> for ResolvedValue {
    type Error = WrongResolvedValueError;

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            ResolvedValue::String(value) => Ok(value),
            other => Err(WrongResolvedValueError("String", other))
        }
    }
}

pub(crate) use impl_try_from_as_unpack;

impl ResolvedValue {
    pub fn resolve_with_interner<B: string_interner::backend::Backend<Symbol=S>, S: Symbol, H: BuildHasher>(&self, interner: &mut string_interner::StringInterner<B, H>) -> Result<S, WrongResolvedValueError> {
        match self {
            ResolvedValue::String(value) => { Ok(interner.get_or_intern(value)) }
            other => Err(WrongResolvedValueError("", other.clone()))
        }
    }

    pub fn try_resolve_with_interner<B: string_interner::backend::Backend<Symbol=S>, S: Symbol, H: BuildHasher>(&self, interner: &string_interner::StringInterner<B, H>) -> Result<Option<S>, WrongResolvedValueError> {
        match self {
            ResolvedValue::String(value) => { Ok(interner.get(value)) }
            other => Err(WrongResolvedValueError("", other.clone()))
        }
    }
}

impl From<&str> for ResolvedValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
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
            .with::<Language>()
            .with::<Domain>()
            .with::<Register>()
            .with::<GrammaticalGender>()
            .with::<PartOfSpeech>()
            .with::<Region>()
            .with::<GrammaticalNumber>()
            .with::<PartOfSpeechTag>()
            .with::<String>()
            .with::<u64>()
            .build_output()
        }
        input: {
            builder()
            .with::<Language>()
            .with::<Domain>()
            .with::<Register>()
            .with::<GrammaticalGender>()
            .with::<PartOfSpeech>()
            .with::<Region>()
            .with::<GrammaticalNumber>()
            .with::<PartOfSpeechTag>()
            .with::<String>()
            .with::<u64>()
            .build_input()
        }
    }
);

impl_py_stub!(ResolvedValue: ResolvedValueType);
