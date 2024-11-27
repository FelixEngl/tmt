mod pos;
mod pos_tags;
mod region;
mod domain;
mod register;
mod language;
mod number;
mod gender;

use std::str::FromStr;
use derive_more::{From};
use serde::{Deserialize, Serialize};
use strum::{Display};
pub use domain::*;
pub use gender::*;
pub use language::*;
pub use number::*;
pub use pos::*;
pub use pos_tags::*;
pub use region::*;
pub use register::*;



#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum PartialWordType {
    Prefix,
    Suffix,
}


#[derive(From, Debug, Copy, Clone, Display)]
pub enum AnyWordInfo {
     // #[strum(to_string = "{0}")]
     Language(Language),
     // #[strum(to_string = "{0}")]
     Domain(Domain),
     // #[strum(to_string = "{0}")]
     Gender(GrammaticalGender),
     // #[strum(to_string = "{0}")]
     Number(GrammaticalNumber),
     // #[strum(to_string = "{0}")]
     POS(PartOfSpeech),
     // #[strum(to_string = "{0}")]
     POSTag(PartOfSpeechTag),
     // #[strum(to_string = "{0}")]
     Region(Region),
     // #[strum(to_string = "{0}")]
     Register(Register),
}

macro_rules! impl_parse {
    ($($i:ident),+ $(,)?) => {
        impl FromStr for AnyWordInfo {
            type Err = strum::ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                if let Ok(found) = s.parse::<$i>() {
                    return Ok(found.into());
                }
                )+
                Err(strum::ParseError::VariantNotFound)
            }
        }
    };
}

impl_parse!(
    Language,
    Domain,
    GrammaticalGender,
    GrammaticalNumber,
    PartOfSpeech,
    PartOfSpeechTag,
    Region,
    Register,
);