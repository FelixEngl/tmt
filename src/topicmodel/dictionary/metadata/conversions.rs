use std::fmt::Formatter;
use itertools::Itertools;
use serde::de::{Error, Unexpected, Visitor};
use serde::Deserializer;
use string_interner::{DefaultSymbol, Symbol};
use string_interner::symbol::SymbolU32;

/// Converter for a vec filled with [DefaultSymbol]
pub(crate) fn convert_vec_defaultsymbol(value: &Vec<DefaultSymbol>) -> Vec<usize> {
    value.iter().map(|value| value.to_usize()).collect_vec()
}

/// The inverse of [convert_vec_defaultsymbol]
pub(crate) fn convert_vec_usize<'de, D>(value: Vec<usize>) -> Result<Vec<DefaultSymbol>, D::Error> where D: Deserializer<'de>  {
    value.into_iter().map(|value|
        DefaultSymbol::try_from_usize(value)
            .ok_or_else(||
                Error::invalid_value(
                    Unexpected::Unsigned(value as u64),
                    &SymbolU32Visitor
                )
            )
    ).collect::<Result<Vec<_>, _>>()
}



/// Helper for SymbolU32 serialisation
struct SymbolU32Visitor;

impl<'de> Visitor<'de> for SymbolU32Visitor {
    type Value = SymbolU32;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("The default symbols are between 0 and u32::MAX-1.")
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> where E: Error {
        match DefaultSymbol::try_from_usize(v as usize) {
            None => {
                Err(E::invalid_value(
                    Unexpected::Unsigned(v as u64),
                    &self
                ))
            }
            Some(value) => {
                Ok(value)
            }
        }
    }
}