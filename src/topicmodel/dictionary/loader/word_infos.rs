use std::fmt::{Display, Formatter};
use strum::{Display, EnumString};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WordInfo<T> {
    Type(WordType),
    Gender(GrammaticalGender),
    Number(GrammaticalNumber),
    Other(T)
}

impl<T> From<T> for WordInfo<T> where T: AsRef<str> {
    fn from(value: T) -> Self {
        let s = value.as_ref();
        if let Ok(value) = s.parse() {
            WordInfo::Type(value)
        } else if let Ok(value) = s.parse() {
            WordInfo::Gender(value)
        } else if let Ok(value) = s.parse() {
            WordInfo::Number(value)
        } else {
            WordInfo::Other(value)
        }
    }
}

impl<T> WordInfo<T> {
    pub fn map<R, F: FnOnce(T) -> R>(self, mapper: F) -> WordInfo<R> {
        match self {
            WordInfo::Other(value) => WordInfo::Other(mapper(value)),
            WordInfo::Type(value) => WordInfo::Type(value),
            WordInfo::Gender(value) => WordInfo::Gender(value),
            WordInfo::Number(value) => WordInfo::Number(value),
        }
    }
}

impl<T> Display for WordInfo<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WordInfo::Type(value) => {
                Display::fmt(value, f)
            }
            WordInfo::Gender(value) => {
                Display::fmt(value, f)
            }
            WordInfo::Number(value) => {
                Display::fmt(value, f)
            }
            WordInfo::Other(value) => {
                Display::fmt(value, f)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq)]
pub enum WordType {
    #[strum(to_string = "noun")]
    Noun,
    #[strum(to_string = "adj")]
    Adjective,
    #[strum(to_string = "adv")]
    Adverb,
    #[strum(to_string = "verb")]
    Verb,
    #[strum(to_string = "conj")]
    Conjuction,
    #[strum(to_string = "pron")]
    Pronoun,
    #[strum(to_string = "prep")]
    Preposition,
    #[strum(to_string = "det")]
    Determiner,
    #[strum(to_string = "int")]
    Interjection
}

#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq)]
pub enum GrammaticalGender {
    #[strum(to_string = "f", serialize = "female")]
    Feminine,
    #[strum(to_string = "m", serialize = "male")]
    Masculine,
    #[strum(to_string = "n", serialize = "neutral")]
    Neutral
}

#[derive(Copy, Clone, Debug, Default, Display, EnumString, Eq, PartialEq)]
pub enum GrammaticalNumber {
    #[strum(to_string = "sg")]
    #[default]
    Singular,
    #[strum(to_string = "pl")]
    Plural
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PartialWordType {
    Prefix,
    Suffix,
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use crate::topicmodel::dictionary::loader::word_infos::GrammaticalGender::Feminine;
    use crate::topicmodel::dictionary::loader::word_infos::WordInfo;

    #[test]
    fn can_map(){
        let other = vec![WordInfo::Other("value"), WordInfo::Gender(Feminine)];
        println!("{other:?}");
        let other = other.into_iter().map(|value| value.map(|x| x.to_string())).collect_vec();
        println!("{other:?}");
    }
}