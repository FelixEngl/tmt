use std::fmt::{Debug, Display, Formatter};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, space0};
use nom::combinator::{map, map_res, opt, value};
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::many1;
use nom::sequence::{delimited, preceded, terminated};
use crate::topicmodel::dictionary::loader::helper::take_bracket;
use crate::topicmodel::dictionary::loader::word_infos::{GrammaticalGender, PartialWordType, WordType};


pub trait DictCCParserError<I>: ParseError<I> + FromExternalError<I, strum::ParseError>{}

impl<T, I> DictCCParserError<I> for T where T:  ParseError<I> + FromExternalError<I, strum::ParseError>{}


#[derive(Debug, Clone)]
pub enum WordEntryElement<T> {
    Word(T),
    PartialWord(T, PartialWordType),
    Info(GrammaticalGender),
    Contextualisation(T),
    Abbreviation(T),
    Combination(T),
    Placeholder
}

impl<T> WordEntryElement<T> {
    pub fn map<R, F: FnOnce(T) -> R>(self, mapper: F) -> WordEntryElement<R> {
        match self {
            WordEntryElement::Word(value) => WordEntryElement::Word(mapper(value)),
            WordEntryElement::PartialWord(value, typ) => WordEntryElement::PartialWord(mapper(value), typ),
            WordEntryElement::Info(value) => WordEntryElement::Info(value),
            WordEntryElement::Contextualisation(value) => WordEntryElement::Contextualisation(mapper(value)),
            WordEntryElement::Abbreviation(value) => WordEntryElement::Abbreviation(mapper(value)),
            WordEntryElement::Combination(value) => WordEntryElement::Combination(mapper(value)),
            WordEntryElement::Placeholder => WordEntryElement::Placeholder
        }
    }
}

impl<T> Display for WordEntryElement<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WordEntryElement::Word(value) => {
                write!(f, "{value}")
            }
            WordEntryElement::Info(value) => {
                write!(f, "{{{value}}}")
            }
            WordEntryElement::Contextualisation(value) => {
                write!(f, "[{value}]")
            }
            WordEntryElement::Abbreviation(value) => {
                write!(f, "<{value}>")
            }
            WordEntryElement::Combination(value) => {
                write!(f, "({value})")
            }
            WordEntryElement::Placeholder => {
                write!(f, "...")
            }
            WordEntryElement::PartialWord(value, typ) => {
                match typ {
                    PartialWordType::Prefix => write!(f, "{value}..."),
                    PartialWordType::Suffix => write!(f, "...{value}")
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct WordEntry<T>(pub Vec<WordEntryElement<T>>);

impl<T> WordEntry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> WordEntry<R> {
        self.0.into_iter().map(|value| value.map(
            |value| mapper(value)
        )).collect_vec().into()
    }
}

impl<T> From<Vec<WordEntryElement<T>>> for WordEntry<T> {
    fn from(value: Vec<WordEntryElement<T>>) -> Self {
        Self(value)
    }
}

impl<T> Display for WordEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" "))
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct WordTypes(Vec<WordType>);

impl From<Vec<WordType>> for WordTypes {
    fn from(value: Vec<WordType>) -> Self {
        Self(value)
    }
}

impl Display for WordTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.iter().join(" "), f)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct WordCategories<T>(Vec<T>);

impl<T> WordCategories<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> WordCategories<R> {
        self.0.into_iter().map(|value| mapper(value)).collect_vec().into()
    }
}

impl<T> From<Vec<T>> for WordCategories<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> Display for WordCategories<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter().join("] ["))
    }
}


#[derive(Debug, Clone)]
pub struct Entry<T>(pub WordEntry<T>, pub WordEntry<T>, Option<WordTypes>, Option<WordCategories<T>>);

impl<T> Entry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> Entry<R> {
        Entry(
            self.0.map(|value| mapper(value)),
            self.1.map(|value| mapper(value)),
            self.2,
            self.3.map(|value| value.map(|value| mapper(value))),
        )
    }
}

impl<T> From<(WordEntry<T>,WordEntry<T>, Option<WordTypes>, Option<WordCategories<T>>)> for Entry<T> {
    fn from(value: (WordEntry<T>, WordEntry<T>, Option<WordTypes>, Option<WordCategories<T>>)) -> Self {
        Entry(value.0, value.1, value.2, value.3)
    }
}

impl<T> Display for Entry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}\t", &self.0, &self.1)?;
        if let Some(ref types) = self.2 {
            write!(f, "{types}")?;
        }
        write!(f, "\t")?;
        if let Some(ref categories) = self.3 {
            write!(f, "{categories}")
        } else {
            Ok(())
        }

    }
}


fn parse_entry<'a, E: DictCCParserError<&'a str>>(s: &'a str) -> IResult<&'a str, WordEntry<&'a str>, E> {
    map(
        many1(
            delimited(
                space0,
                alt((
                    map_res(take_bracket!('{', '}'), |value: &str| value.parse().map(|g: GrammaticalGender| WordEntryElement::Info(g))),
                    map(take_bracket!('(', ')'), WordEntryElement::Combination),
                    map(take_bracket!('[', ']'), WordEntryElement::Contextualisation),
                    map(take_bracket!('<', '>'), WordEntryElement::Abbreviation),
                    map(preceded(tag("..."), is_not("{[(< \t")), |value| WordEntryElement::PartialWord(value, PartialWordType::Suffix)),
                    value(WordEntryElement::Placeholder, tag("...")),
                    map(is_not("{[(< \t"), |value: &str| {
                        if value.ends_with("...") {
                            WordEntryElement::PartialWord(value, PartialWordType::Prefix)
                        } else {
                            WordEntryElement::Word(value)
                        }
                    }),
                )),
                space0
            )
        ),
        WordEntry::from
    )(s)
}


fn parse_word_type<'a, E: DictCCParserError<&'a str>>(s: &'a str) -> IResult<&'a str, WordTypes, E> {
    map(
        many1(
            terminated(
                map_res(is_not(" .\t"), |value: &str| value.try_into()),
                opt(char(' '))
            )
        ),
        WordTypes::from
    )(s)
}



fn parse_word_category<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, WordCategories<&'a str>, E> {
    map(
        many1(
            terminated(
                take_bracket!('[', ']'),
                opt(char(' '))
            )
        ),
        WordCategories
    )(s)
}


fn parse_line<'a, E: DictCCParserError<&'a str>>(s: &'a str) -> IResult<&'a str, Entry<&'a str>, E> {
    map(
        nom::sequence::tuple((
            terminated(
                parse_entry,
                char('\t')
            ),
            terminated(
                parse_entry,
                char('\t')
            ),
            terminated(
                opt(parse_word_type),
                char('\t')
            ),
            terminated(
                opt(parse_word_category),
                multispace0
            )
        )),
        Entry::from
    )(s)
}