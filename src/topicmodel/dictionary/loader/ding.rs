use crate::topicmodel::dictionary::loader::file_parser::{base_parser_method, FileParserResult, FunctionBasedLineWiseReader, LineWiseDictionaryReader};
use crate::topicmodel::dictionary::loader::helper::take_nested_bracket_delimited;
use crate::topicmodel::dictionary::loader::word_infos::{PartialWordType, WordInfo};
use itertools::{chain, Itertools};
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag};
use nom::character::complete::{char, multispace0, space0};
use nom::combinator::{eof, map, not, opt, peek, recognize, value};
use nom::error::ParseError;
use nom::multi::many1;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io;
use std::path::Path;

/// The single elements that make up an entry
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DingWordEntryElement<T> {
    Word(T),
    /// A word that is only a partial word like a prefix or a suffix
    PartialWord(T, PartialWordType),
    /// A word that alternates between the contained values
    AlternatingWords(DingAlternatingWord<T>),
    Category(T),
    Contextualisation(T),
    Info(WordInfo<T>),
    Abbreviation(Abbreviation<T>),
    AlternateNotation(T, Option<Vec<T>>, Option<Abbreviation<T>>),
    /// Basically a placeholder for a word
    WordPlaceholder,
    /// Example:
    /// to put forward <> sth. -> to put forward, to put sth. forward, to put forward sth.
    InterchangeableWith,
}

impl<T> DingWordEntryElement<T> {
    pub fn is_word(&self) -> bool {
        match self {
            DingWordEntryElement::Word(_)
            | DingWordEntryElement::PartialWord(_, _)
            | DingWordEntryElement::AlternatingWords(_)
            | DingWordEntryElement::AlternateNotation(_, _, _)
            | DingWordEntryElement::WordPlaceholder => true,
            _ => false
        }
    }

    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingWordEntryElement<R> {
        match self {
            DingWordEntryElement::Word(value) => DingWordEntryElement::Word(mapper(value)),
            DingWordEntryElement::PartialWord(value, typ) => DingWordEntryElement::PartialWord(mapper(value), typ),
            DingWordEntryElement::Category(value) => DingWordEntryElement::Category(mapper(value)),
            DingWordEntryElement::Contextualisation(value) => DingWordEntryElement::Contextualisation(mapper(value)),
            DingWordEntryElement::Info(value) => DingWordEntryElement::Info(value.map(|value| mapper(value))),
            DingWordEntryElement::InterchangeableWith => DingWordEntryElement::InterchangeableWith,
            DingWordEntryElement::WordPlaceholder => DingWordEntryElement::WordPlaceholder,
            DingWordEntryElement::AlternatingWords(value) => DingWordEntryElement::AlternatingWords(value.map(mapper)),
            DingWordEntryElement::Abbreviation(value) => DingWordEntryElement::Abbreviation(value.map(mapper)),
            DingWordEntryElement::AlternateNotation(value, cont, abbr) =>
                DingWordEntryElement::AlternateNotation(
                    mapper(value),
                    cont.map(|value| value.into_iter().map(mapper).collect()),
                    abbr.map(|value| value.map(mapper))
                )
        }
    }
}

impl<T> Display for DingWordEntryElement<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DingWordEntryElement::Word(value) => {
                write!(f, "{value}")
            }
            DingWordEntryElement::Abbreviation(value) => {
                write!(f, "{value}")
            }
            DingWordEntryElement::AlternateNotation(value, values, abbrev) => {
                match (values, abbrev) {
                    (None, None) => {
                        write!(f, "<{value}>")
                    }
                    (Some(values), None) => {
                        write!(f, "<{value} {}>", values.iter().join(" "))
                    }
                    (None, Some(abbrev)) => {
                        write!(f, "<{value} {}>", abbrev)
                    }
                    (Some(values), Some(abbrev)) => {
                        write!(f, "<{value} {} {abbrev}>", values.iter().join(" "))
                    }
                }

            }
            DingWordEntryElement::Category(value) => {
                write!(f, "[{value}]")
            }
            DingWordEntryElement::Contextualisation(value) => {
                write!(f, "({value})")
            }
            DingWordEntryElement::Info(value) => {
                write!(f, "{{{value}}}")
            }
            DingWordEntryElement::InterchangeableWith => {
                write!(f, "<>")
            }
            DingWordEntryElement::WordPlaceholder => {
                write!(f, "…")
            }
            DingWordEntryElement::PartialWord(value, typ) => {
                match typ {
                    PartialWordType::Prefix => write!(f, "{value}…"),
                    PartialWordType::Suffix => write!(f, "…{value}")
                }
            }
            DingWordEntryElement::AlternatingWords(value) => {
                write!(f, "{value}")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Abbreviation<T>(pub T, pub Option<Vec<T>>);


impl<T> Abbreviation<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> Abbreviation<R> {
        match self.1 {
            None => {
                Abbreviation(mapper(self.0), None)
            }
            Some(values) => {
                Abbreviation(mapper(self.0), Some(values.into_iter().map(mapper).collect()))
            }
        }
    }
}

impl<T> Display for Abbreviation<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.1 {
            None => {
                write!(f, "/{}/", self.0)
            }
            Some(values) => {
                write!(f, "/{}, {}/", self.0, values.iter().join(", "))
            }
        }
    }
}

/// Represents a single element in an DingAltEntry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingAlternatingWordValue<T>(pub Vec<DingWordEntryElement<T>>);
impl<T> DingAlternatingWordValue<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingAlternatingWordValue<R> {
        self.0.into_iter().map(|value| value.map(mapper)).collect_vec().into()
    }
}
impl<T> From<Vec<DingWordEntryElement<T>>> for DingAlternatingWordValue<T> {
    fn from(value: Vec<DingWordEntryElement<T>>) -> Self {
        Self(value)
    }
}
impl<T> Display for DingAlternatingWordValue<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" "))
    }
}

/// Represents a word that can be alternated or altered by the following entries.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingAlternatingWord<T>(pub Vec<DingAlternatingWordValue<T>>);
impl<T> DingAlternatingWord<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingAlternatingWord<R> {
        DingAlternatingWord(self.0.into_iter().map(|value| value.map(mapper)).collect_vec())
    }
}

impl<T> Display for DingAlternatingWord<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" / "))
    }
}

/// Represents a complete ding word entry
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingWordEntry<T>(pub Vec<DingWordEntryElement<T>>);
impl<T> DingWordEntry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingWordEntry<R> {
        self.0.into_iter().map(|value| value.map(mapper)).collect_vec().into()
    }
}
impl<T> From<Vec<DingWordEntryElement<T>>> for DingWordEntry<T> {
    fn from(value: Vec<DingWordEntryElement<T>>) -> Self {
        Self(value)
    }
}
impl<T> Display for DingWordEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" "))
    }
}

/// Represents entries that are alternatives to each other
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingAlternativeEntries<T>(pub Vec<DingWordEntry<T>>);
impl<T> DingAlternativeEntries<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingAlternativeEntries<R> {
        self.0.into_iter().map(|value| value.map(mapper)).collect_vec().into()
    }
}
impl<T> From<Vec<DingWordEntry<T>>> for DingAlternativeEntries<T> {
    fn from(value: Vec<DingWordEntry<T>>) -> Self {
        Self(value)
    }
}
impl<T> Display for DingAlternativeEntries<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join("; "))
    }
}

/// A collection of alternative entries
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DingLanguageEntries<T>(pub Vec<DingAlternativeEntries<T>>);
impl<T> DingLanguageEntries<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: &F) -> DingLanguageEntries<R> {
        self.0.into_iter().map(|value| value.map(mapper)).collect_vec().into()
    }
}
impl<T> From<Vec<DingAlternativeEntries<T>>> for DingLanguageEntries<T> {
    fn from(value: Vec<DingAlternativeEntries<T>>) -> Self {
        Self(value)
    }
}
impl<T> Display for DingLanguageEntries<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" | "))
    }
}

/// An ding entry consisting of two language entries.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DingEntry<T>(pub DingLanguageEntries<T>, pub DingLanguageEntries<T>);
impl<T> DingEntry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> DingEntry<R> {
        DingEntry(self.0.map(&mapper), self.1.map(&mapper))
    }
}

impl<T> From<(DingLanguageEntries<T>, DingLanguageEntries<T>)> for DingEntry<T> {
    fn from((entry_a, entry_b): (DingLanguageEntries<T>, DingLanguageEntries<T>)) -> Self {
        Self(entry_a, entry_b)
    }
}
impl<T> Display for DingEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} :: {}", &self.0, &self.1)
    }
}

#[inline(always)]
fn parse_word_content<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(
        alt((
            is_not("{[(< \t:;|…/>\r\n"),
            recognize(pair(char(':'), not(char(':'))))
        ))
    )(s)
}


#[inline(always)]
fn parse_interchangeable<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    value(
        DingWordEntryElement::InterchangeableWith,
        alt((tag("<>"), tag("<->")))
    )(s)
}


#[inline(always)]
fn parse_word_content_no_comma<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(
        alt((
            is_not("{[\t;|…/"),
            recognize(pair(char(':'), not(char(':')))),
        ))
    )(s)
}


#[inline(always)]
fn parse_abbreviation<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Abbreviation<&'a str>, E> {
    // println!("parse_abbreviation {s}");
    map(
        preceded(
            space0,
            delimited(
                char('/'),
                pair(
                    preceded(space0, parse_word_content_no_comma),
                    opt(
                        many1(
                            delimited(
                                delimited(space0, is_a(";,"), space0),
                                parse_word_content_no_comma,
                                space0
                            )
                        )
                    )
                ),
                delimited(space0, char('/'), peek(not(preceded(space0, parse_word))))
            )
        ),
        |(a, b)| Abbreviation(a, b)
    )(s)
}

// Err: 20, Diff: 83
// Err: 24, Diff: 79
// Err: 17, Diff: 80

fn parse_non_word<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    alt(
        (
            map(take_nested_bracket_delimited('[', ']'), DingWordEntryElement::Category),
            map(take_nested_bracket_delimited('{', '}'), |value: &str| DingWordEntryElement::Info(value.into())),
            map(take_nested_bracket_delimited('(', ')'), DingWordEntryElement::Contextualisation),
            map(parse_abbreviation, DingWordEntryElement::Abbreviation),
            delimited(
                terminated(char('<'), peek(not(char('>')))),
                map(
                    tuple(
                        (
                            parse_word_content,
                            opt(
                                many1(
                                    preceded(space0, parse_word_content)
                                )
                            ),
                            opt(parse_abbreviation)
                        )
                    ),
                    |(a, b, abbrev)| DingWordEntryElement::AlternateNotation(a, b, abbrev)
                ),
                char('>')
            ),
            // map(preceded(opt(char('/')), take_bracket!('(', ')')), DingWordEntryElement::Contextualisation),
        )
    )(s)
}

fn parse_word<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    alt(
        (
            map(preceded(tag("…"), parse_word_content), |value| DingWordEntryElement::PartialWord(value, PartialWordType::Suffix)),
            value(DingWordEntryElement::WordPlaceholder, tag("…")),
            map(terminated(parse_word_content, tag("…")), |value| DingWordEntryElement::PartialWord(value, PartialWordType::Prefix)),
            map(parse_word_content, DingWordEntryElement::Word),
        )
    )(s)
}

fn parse_word_entry_element_no_alt<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    alt(
        (
            parse_non_word,
            parse_interchangeable,
            parse_word,
        )
    )(s)
}

fn parse_single_word_alternative<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingAlternatingWordValue<&'a str>, E> {
    // println!("parse_single_word_alternative {s}",);
    map(
        tuple(
            (
                opt(preceded(space0, map(take_nested_bracket_delimited('(', ')'), DingWordEntryElement::Contextualisation))),
                preceded(space0, parse_word),
                opt(
                    preceded(
                        peek(preceded(space0, is_a("{[(</"))),
                        many1(
                            preceded(
                                space0,
                                parse_non_word
                            )
                        )
                    )
                ),
            )
        ),
        |(context, value, rest)| {
            chain!(
                context,
                std::iter::once(value),
                rest.into_iter().flatten()
            ).collect_vec().into()
        }
    )(s)
}


fn parse_word_alternative<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingAlternatingWord<&'a str>, E> {
    map(
        pair(
            parse_single_word_alternative,
            many1(
                delimited(
                    preceded(space0, char('/')),
                    parse_single_word_alternative,
                    terminated(space0, peek(not(terminated(tag("/"), preceded(space0, alt((is_a("(;|["), tag("::"), eof)))))))
                )
            )
        ),
        |(first, following)| {
            DingAlternatingWord(chain!(std::iter::once(first), following).collect())
        }
    )(s)
}

fn parse_word_entry_element<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntryElement<&'a str>, E> {
    alt(
        (
            map(parse_word_alternative, DingWordEntryElement::AlternatingWords),
            parse_word_entry_element_no_alt
        )
    )(s)
}

fn parse_word_entry<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingWordEntry<&'a str>, E> {
    map(
        many1(
            delimited(
                space0,
                parse_word_entry_element,
                space0
            ),
        ),
        DingWordEntry::from
    )(s)
}


fn parse_alt_word_entries<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingAlternativeEntries<&'a str>, E> {
    map(
        pair(
            parse_word_entry,
            opt(
                many1(
                    preceded(
                        delimited(space0, char(';'), space0),
                        parse_word_entry
                    )
                )
            )
        ),
        |(first, following)| {
            match following {
                None => {
                    vec![first].into()
                }
                Some(value) => {
                    let mut data = Vec::with_capacity(1 + value.len());
                    data.push(first);
                    data.extend(value);
                    data.into()
                }
            }
        }
    )(s)
}


fn parse_language_entries<'a, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, DingLanguageEntries<&'a str>, E> {

    // map(
    //     many1(
    //         alt(
    //             (
    //                 parse_alt_word_entries,
    //                 preceded(
    //                     delimited(space0, char('|'), space0),
    //                     parse_alt_word_entries
    //                 ),
    //             )
    //         )
    //     ),
    //     DingLanguageEntries::from
    // )(s)

    map(
        pair(
            parse_alt_word_entries,
            opt(
                many1(
                    preceded(
                        delimited(space0, char('|'), space0),
                        parse_alt_word_entries
                    )
                )
            )
        ),
        |(first, following)| {
            match following {
                None => {
                    vec![first].into()
                }
                Some(value) => {
                    let mut data = Vec::with_capacity(1 + value.len());
                    data.push(first);
                    data.extend(value);
                    data.into()
                }
            }
        }
    )(s)
}

fn parse_line<'a, E: ParseError<&'a str>, const WITH_ERROR_CORRECTION: bool>(s: &'a str) -> IResult<&'a str, DingEntry<&'a str>, E> {
    if WITH_ERROR_CORRECTION {
        terminated(
            map(
                separated_pair(
                    parse_language_entries,
                    delimited(space0, tag("::"), space0),
                    preceded(
                        opt(terminated(char('|'), space0)),
                        parse_language_entries
                    )
                ),
                DingEntry::from
            ),
            multispace0
        )(s)
    } else {
        terminated(
            map(
                separated_pair(
                    parse_language_entries,
                    delimited(space0, tag("::"),space0),
                    parse_language_entries
                ),
                DingEntry::from
            ),
            multispace0
        )(s)
    }
}

fn parse_or_fail<'a>(content: &'a [u8]) -> FileParserResult<DingEntry<String>> {
    match base_parser_method(
        content,
        |s| parse_line::<nom::error::Error<&'a str>, true>(s)
    ) {
        Ok(value) => {
            Ok(value.map(ToString::to_string))
        }
        Err(value) => {
            Err(value.map(|value| value.map(ToString::to_string)))
        }
    }

    // let content = std::str::from_utf8(content)?;
    // let (left, entry) = parse_line::<nom::error::Error<&str>, true>(content).map_err(|err| {
    //     match err {
    //         nom::Err::Error(err) => {
    //             nom::Err::Error(nom::error::Error::from_error_kind(err.input.to_string(), err.code))
    //         },
    //         nom::Err::Incomplete(err) => {
    //             nom::Err::Incomplete(err)
    //         },
    //         nom::Err::Failure(err) => {
    //             nom::Err::Failure(nom::error::Error::from_error_kind(err.input.to_string(), err.code))
    //         }
    //     }
    // })?;
    // if !left.is_empty() {
    //     Err(DingDictionaryReaderError::Lost(left.to_string()))
    // } else {
    //     Ok(entry.map(|value| value.to_string()))
    // }
}




pub fn read_dictionary(file: impl AsRef<Path>) -> io::Result<FunctionBasedLineWiseReader<File, DingEntry<String>>> {
    Ok(LineWiseDictionaryReader::new(
        File::options().read(true).open(file)?,
        parse_or_fail
    ))
}




pub mod entry_processing {
    use crate::topicmodel::dictionary::loader::ding;
    use crate::topicmodel::dictionary::loader::ding::{Abbreviation, DingAlternatingWord, DingAlternatingWordValue, DingWordEntry, DingWordEntryElement};
    use crate::topicmodel::dictionary::metadata::loaded::{LoadedMetadataCollection, LoadedMetadataCollectionBuilder};
    use crate::topicmodel::dictionary::word_infos::*;
    use itertools::Itertools;
    use std::borrow::Cow;
    use std::fmt::{Debug, Display};


    pub fn process_translation_entry<T: AsRef<str> + Display>(ding::DingEntry(a, b): ding::DingEntry<T>) -> Translation<T> {
        Translation {
            a: process_single_entry(a),
            b: process_single_entry(b)
        }
    }

    fn try_parse_string<'a, T: AsRef<str>>(
        s: &'a str,
        builder: &mut LoadedMetadataCollectionBuilder<T>
    ) -> Option<Vec<&'a str>> {
        let s = s.trim();
        match s {
            "Dt., Ös. veraltend" => {
                builder.push_languages(Language::German);
                builder.push_regions(Region::AustrianGerman);
                builder.push_registers(Register::Archaic);
                return None
            }
            "Am., auch Br." => {
                builder.extend_regions([Region::AmericanEnglish, Region::BritishEnglish]);
                return None
            }
            _ => {}
        }

        if s.contains(|c| matches!(c, ' ' | '/' | ',')) {
            let mut cont = Vec::new();
            for value in s.split(|c| matches!(c, ' ' | '/' | ',')) {
                if let Some(fail) = try_parse_string(
                    value.trim(),
                    builder
                ) {
                    cont.extend(fail);
                }
            }
            if cont.is_empty() {
                None
            } else {
                Some(cont)
            }
        }  else {
            if let Ok(a) = s.parse() { builder.push_languages(a); return None; }
            if let Ok(a) = s.parse() { builder.push_regions(a); return None; }
            if let Ok(a) = s.parse() { builder.push_pos(a); return None; }
            if let Ok(a) = s.parse() { builder.push_genders(a); return None; }
            if let Ok(a) = s.parse() { builder.push_numbers(a); return None; }
            if let Ok(a) = s.parse() { builder.push_domains(a); return None; }
            if let Ok(a) = s.parse() { builder.push_registers(a); return None; }
            Some(vec![s])
        }
    }

    #[derive(Debug)]
    pub enum WordElement<T> {
        /// bla
        Word(T),
        /// bla...
        Prefix(T),
        /// ...bla
        Suffix(T),
        /// The following word can be interchanged with the previous word.
        /// Example:
        /// to put forward <> sth. -> to put forward, to put sth. forward, to put forward sth.
        InterchangeableInstruction,
        /// …
        Placeholder,
        /// The words can be interchanged, but one has to be put there
        AlternatingWord(Vec<WordEntry<T>>),

        /// The whole word can have a different spellings, stored in this entry.
        DifferentSpelling {
            words: Vec<T>,
            abbrev: Option<Vec<T>>
        }
    }

    pub struct Translation<T> {
        pub a: Entries<T>,
        pub b: Entries<T>
    }

    impl<T: AsRef<str> + Clone> Translation<T> {

        /// Buildup:
        /// 0.zip_eq(1)
        ///
        ///
        /// ```text
        /// Level 0:
        /// Alternatives:
        ///     Atomforscher {m}; Atomforscherin {f} | Atomforscher {pl}; Atomforscherinnen {pl} -- atomic scientist <nuclear scientist> | atomic scientist <nuclear scientist> s
        ///     -> Atomforscher {m}; Atomforscherin {f} -- atomic scientist <nuclear scientist>
        ///     -> Atomforscher {pl}; Atomforscherinnen {pl} -- atomic scientist <nuclear scientist> s
        ///
        /// Level 1:
        /// Interchangeables:
        ///     Atomforscher {m}; Atomforscherin {f} -- atomic scientist <nuclear scientist>
        ///     -> Atomforscher {m} -- atomic scientist <nuclear scientist>
        ///     -> Atomforscherin {f} -- atomic scientist <nuclear scientist>
        ///     Atomforscher {pl}; Atomforscherinnen {pl} -- atomic scientist <nuclear scientist> s
        ///     -> Atomforscher {pl} -- atomic scientist <nuclear scientist> s
        ///     -> Atomforscherinnen {pl} -- atomic scientist <nuclear scientist> s
        ///
        /// Level 2:
        /// Variants:
        ///     atomic scientist <nuclear scientist>
        ///     -> atomic scientist
        ///     -> nuclear scientist
        /// ```
        pub fn create_alternatives(&self) -> (Vec<Vec<Vec<(String, LoadedMetadataCollectionBuilder<String>)>>>, Vec<Vec<Vec<(String, LoadedMetadataCollectionBuilder<String>)>>>) {
            (self.a.create_alternatives(), self.b.create_alternatives())
        }
    }

    /// Denotes a complete language entry, sonsists of multiple single entries.
    pub struct Entries<T> {
        pub complete_entry: String,
        /// These translations are alternatives to each other and may have different meanings.
        /// They do not share any kind of meta informations and are only related to each other in some way.
        pub entries: Vec<AlternativeWords<T>>
    }

    impl<T: AsRef<str> + Clone> Entries<T> {
        pub fn create_alternatives(&self) -> Vec<Vec<Vec<(String, LoadedMetadataCollectionBuilder<String>)>>> {
            self.entries.iter().map(|value| {
                value.create_alternatives()
            }).collect()
        }
    }


    /// Multiple words with the same meaning but different translations.
    pub struct AlternativeWords<T> {
        pub single_entry: String,
        /// The words are interchangeable for each other. Usually different ways to write the same word.
        /// Like female and male versions.
        /// Usually they share all meta informations except gender.
        pub words: Vec<WordEntry<T>>
    }

    impl<T: AsRef<str> + Clone> AlternativeWords<T> {
        pub fn create_alternatives(&self) -> Vec<Vec<(String, LoadedMetadataCollectionBuilder<String>)>> {
            let mut data = self.words.iter().map(|value| value.create_alternatives().into_iter().map(
                |(value, meta)| {
                    (value, meta.map(|value| value.as_ref().to_string()))
                }
            ).collect_vec()).collect_vec();
            for value in data.iter_mut() {
                let mut normalized_meta = LoadedMetadataCollectionBuilder::with_name(None);
                for (_, b) in value.iter_mut() {
                    if let Some(x) = b.peek_domains() {
                        normalized_meta.extend_domains(x.into_iter().copied())
                    }
                    if let Some(x) = b.peek_registers() {
                        normalized_meta.extend_registers(x.into_iter().copied())
                    }
                }
                let build = normalized_meta.clone();
                for (_, b) in value {
                    let mut new = build.clone();
                    new.update_fields_with_other(b);
                    new.push_original_entry(self.single_entry.clone());
                    *b = new;
                    b.shrink();
                }
            }
            data
        }
    }

    #[derive(Debug)]
    pub struct WordEntry<T> {
        pub word_pattern_elements: Vec<WordElement<T>>,
        pub metadata: LoadedMetadataCollection<T>
    }

    impl<T: AsRef<str> + Clone> WordEntry<T> {
        fn create_interchange_entries<'a>(value_to_add: &Cow<'a, str>, targets: &[(Vec<Cow<'a, str>>, LoadedMetadataCollectionBuilder<T>)]) -> Vec<(Vec<Cow<'a, str>>, LoadedMetadataCollectionBuilder<T>)> {
            let mut new_to_add = Vec::new();
            for (words, meta) in targets.iter() {
                let mut cp = words.clone();
                let last = cp.pop().expect("There can't be an empty vec. Something went wrong!");
                cp.push(value_to_add.clone());
                cp.push(last);
                new_to_add.push((cp, meta.clone()));
            }
            new_to_add
        }

        fn handle_add_word<'a>(
            meta: &LoadedMetadataCollection<T>,
            value_to_add: Cow<'a, str>,
            output: &mut Vec<(Vec<Cow<'a, str>>, LoadedMetadataCollectionBuilder<T>)>,
            is_interchange: bool,
        ) {
            if output.is_empty() {
                let builder = meta.clone().to_builder();
                if is_interchange {
                    log::warn!("Interchangeable found at start!");
                }
                output.push((vec![value_to_add], builder));
            } else {
                let additional = if is_interchange {
                    Some(Self::create_interchange_entries(&value_to_add, output.as_slice()))
                } else {
                    None
                };
                for (words, _) in output.iter_mut() {
                    words.push(value_to_add.clone());
                }
                if let Some(additional) = additional {
                    output.extend(additional);
                }
            }
        }

        unsafe fn handle_containing_word<'a>(
            meta: &LoadedMetadataCollection<T>,
            value: &'a WordElement<T>,
            word_patterns_and_meta: &mut Vec<(Vec<Cow<'a, str>>, LoadedMetadataCollectionBuilder<T>)>,
            has_interchange: bool,
        ) {
            match value {
                WordElement::Word(value) => {
                    Self::handle_add_word(
                        meta,
                        Cow::Borrowed(value.as_ref()),
                        word_patterns_and_meta,
                        has_interchange
                    )
                }
                WordElement::Prefix(value) => {
                    Self::handle_add_word(
                        meta,
                        Cow::Owned(format!("{}…", value.as_ref())),
                        word_patterns_and_meta,
                        has_interchange
                    );
                }
                WordElement::Suffix(value) => {
                    Self::handle_add_word(
                        meta,
                        Cow::Owned(format!("…{}", value.as_ref())),
                        word_patterns_and_meta,
                        has_interchange
                    );
                }
                WordElement::Placeholder => {
                    Self::handle_add_word(
                        meta,
                        Cow::Borrowed("…"),
                        word_patterns_and_meta,
                        has_interchange
                    );
                }
                _ => unreachable!()
            }
        }

        pub fn create_alternatives<'a>(&'a self) -> Vec<(String, LoadedMetadataCollectionBuilder<T>)> {
            let mut word_patterns_and_meta: Vec<(Vec<Cow<'a, str>>, LoadedMetadataCollectionBuilder<T>)> = Vec::new();
            let mut iter = self.word_pattern_elements.iter();
            let mut has_interchange = false;

            while let Some(value) = iter.next() {
                match value {
                    WordElement::InterchangeableInstruction => {
                        has_interchange = true;
                    }
                    WordElement::AlternatingWord(value) => {

                        if word_patterns_and_meta.is_empty() {
                            for entr in value {
                                for (value, mut meta) in entr.create_alternatives() {
                                    meta.update_fields_with(&self.metadata);
                                    word_patterns_and_meta.push(
                                        (
                                            vec![Cow::Owned(value)],
                                            meta
                                        )
                                    )
                                }
                            }
                        } else {
                            if has_interchange {

                            }
                            let entries = value.iter().map(|value| value.create_alternatives()).collect_vec();
                            let new = Vec::with_capacity(word_patterns_and_meta.len() * entries.iter().map(|value| value.len()).sum::<usize>());
                            let old = std::mem::replace(&mut word_patterns_and_meta, new);
                            for words in entries {
                                for (word, additional_meta) in words {
                                    let word: Cow<'a, str> = Cow::Owned(word);
                                    if has_interchange {
                                        let mut to_add = Self::create_interchange_entries(
                                            &word,
                                            &old
                                        );
                                        for (_, m) in to_add.iter_mut(){
                                            m.update_fields_with_other(&additional_meta);
                                        }
                                        word_patterns_and_meta.extend(to_add);
                                    }
                                    for (mut new, mut meta) in old.iter().cloned() {
                                        new.push(word.clone());
                                        meta.update_fields_with_other(&additional_meta);
                                        word_patterns_and_meta.push((new, meta));
                                    }
                                }
                            }
                        }
                    }
                    WordElement::DifferentSpelling {
                        abbrev,
                        words: word
                    } => {
                        let value = Cow::Owned(word.iter().map(|v| v.as_ref()).join(" "));
                        if let Some((_, meta)) = word_patterns_and_meta.last() {
                            let mut meta = meta.clone();
                            if let Some(abbrev) = abbrev {
                                for v in abbrev {
                                    meta.push_abbreviations(v.clone());
                                }
                            }
                            word_patterns_and_meta.push((vec![value], meta))
                        } else {
                            let mut builder = self.metadata.clone().to_builder();
                            if let Some(abbrev) = abbrev {
                                for v in abbrev {
                                    builder.push_abbreviations(v.clone());
                                }
                            }
                            word_patterns_and_meta.push((vec![value], builder));
                        }
                    }
                    other => {
                        unsafe {
                            Self::handle_containing_word(
                                &self.metadata,
                                other,
                                &mut word_patterns_and_meta,
                                has_interchange
                            );
                        }
                        if has_interchange {
                            has_interchange = false;
                        }
                    }
                }
            }

            word_patterns_and_meta.into_iter().map(|(a, b)|{
                (a.iter().map(|value| value.as_ref()).join(" "), b)
            }).collect()
        }
    }




    fn process_word_element<T: AsRef<str>>(content: DingWordEntryElement<T>, builder: &mut LoadedMetadataCollectionBuilder<T>) -> Option<WordElement<T>> {
        match content {
            DingWordEntryElement::Category(category) => {
                if let Some(_) = try_parse_string(
                    category.as_ref(),
                    builder,
                ) {
                    builder.push_contextual_informations(category);
                }
                None
            }
            DingWordEntryElement::Contextualisation(contextualisation) => {
                if let Some(_) = try_parse_string(
                    contextualisation.as_ref(),
                    builder,
                ) {
                    builder.push_contextual_informations(contextualisation);
                }
                None
            }
            DingWordEntryElement::Info(info) => {
                match info {
                    WordInfo::Type(value) => {
                        builder.push_pos(value);
                    }
                    WordInfo::Gender(value) => {
                        builder.push_genders(value);
                    }
                    WordInfo::Number(value) => {
                        builder.push_numbers(value);
                    }
                    WordInfo::Other(value) => {
                        if let Some(_) = try_parse_string(
                            value.as_ref(),
                            builder
                        ) {
                            builder.push_unclassified(value)
                        }
                    }
                }
                None
            }
            DingWordEntryElement::Abbreviation(Abbreviation(a, b)) => {
                builder.push_abbreviations(a);
                if let Some(b) = b {
                    builder.extend_abbreviations(b);
                }
                None
            }
            DingWordEntryElement::Word(value) => {
                Some(WordElement::Word(value))
            }
            DingWordEntryElement::PartialWord(value, t) => {
                match t {
                    PartialWordType::Prefix => {
                        Some(WordElement::Prefix(value))
                    }
                    PartialWordType::Suffix => {
                        Some(WordElement::Suffix(value))
                    }
                }
            }
            DingWordEntryElement::InterchangeableWith => {
                Some(WordElement::InterchangeableInstruction)
            }
            DingWordEntryElement::AlternatingWords(DingAlternatingWord(alternating_words)) => {
                let alts = alternating_words.into_iter().map(|DingAlternatingWordValue(alternative)| {
                    let mut collected = LoadedMetadataCollectionBuilder::with_name(None);
                    let data = alternative.into_iter().filter_map(|value| {
                        process_word_element(value, &mut collected)
                    }).collect_vec();
                    WordEntry {
                        word_pattern_elements: data,
                        metadata: collected.build_consuming().unwrap()
                    }
                }).collect_vec();
                Some(WordElement::AlternatingWord(alts))
            }
            DingWordEntryElement::WordPlaceholder => {
                Some(WordElement::Placeholder)
            }
            DingWordEntryElement::AlternateNotation(a, b, c) => {
                Some(WordElement::DifferentSpelling {
                    words: if let Some(b) = b {
                        let mut x = Vec::with_capacity(1 + b.len());
                        x.push(a);
                        x.extend(b);
                        x
                    } else {
                        vec![a]
                    },
                    abbrev: c.map(|Abbreviation(a, b)|{
                        if let Some(b) = b {
                            let mut v = Vec::with_capacity(1 + b.len());
                            v.push(a);
                            v.extend(b);
                            v
                        } else {
                            vec![a]
                        }
                    })
                })
            }

        }
    }

    fn process_single_entry<T: AsRef<str> + Display>(dat: ding::DingLanguageEntries<T>) -> Entries<T> {
        let original = format!("{dat}");
        let result = dat.0.into_iter().map(
            |word_entries| { //
                let complete = format!("{word_entries}");
                let result =  word_entries.0.into_iter().map(
                    |DingWordEntry(entry_content)| {
                        let mut builder = LoadedMetadataCollectionBuilder::with_name(None);
                        let words = entry_content.into_iter().filter_map(
                            |value| { process_word_element(value, &mut builder) }
                        ).collect_vec();
                        WordEntry {
                            word_pattern_elements: words,
                            metadata: builder.build_consuming().expect("This boulder should never fail!")
                        }
                    }
                ).collect_vec();
                AlternativeWords {
                    single_entry: complete,
                    words: result
                }
            }
        ).collect_vec();
        Entries {
            complete_entry: original,
            entries: result
        }
    }
}


#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::ding::entry_processing::process_translation_entry;
    use crate::topicmodel::dictionary::loader::ding::{parse_line, parse_word_alternative, read_dictionary};
    use crate::topicmodel::dictionary::loader::helper::test::execute_test_read_for;
    use nom::error::VerboseError;
    use nom::Finish;

    #[test]
    fn can_parse_alt(){
        let result = parse_word_alternative::<VerboseError<_>>("jegliche/r/s").finish();
        match &result {
            Ok(value) => {
                println!("{:?}", value.1);
            }
            Err(value) => {
                println!("{}", value.to_string());
            }
        }
    }

    #[test]
    fn can_recognize_word_category() {

        const TEST_LINES: &[&'static str] = &[
            // "Aal {m} (auf der Speisekarte) [cook.] | Aal blau; blauer Aal | Aal grün; grüner Aal | Aal in Aspik; Aal in Gelee :: Eel (on a menu) | Eel au bleu; Eel steamed and served with Butter | Boiled Eel served with Parsley Sauce | Jellied Eel",
            // "A {n}; Ais {n};As {n}; Aisis {n}; Ases {n} [mus.] | A-Dur {n} :: A; A sharp; A flat; A double sharp; A double flat | A major",
            // "Abbau {m}; Zersetzung {f}; Degradierung {f} (von etw.) [chem.][biol.] | bakterieller Abbau | biologischer Abbau | chemischerAbbau | photochemischer Abbau; Abbau durch Licht | metabolischer Abbau | thermischer Abbau | Abbau durch Bakterien :: breakdown; decomposition; degradation (of sth.) | bacterialdegradation | biological breakdown/degradation; biodegradation | chemical breakdown/degradation | photochemicalbreakdown/degradation; photodegradation | metabolic breakdown | thermal degradation | bacterial decomposition",
            // "Ding {n}; Sache {f} | Dinge {pl}; Sachen {pl}; Krempel {m} | Dinge für sich behalten | die Dinge laufen lassen | den Dingen auf den Grund gehen | beim augenblicklichen Stand der Dinge | das Ding an sich | über solchen Dingen stehen | Er ist der Sache nicht ganz gewachsen. :: thing | things | to keep things to oneself | to let things slide | to get to the bottom of things | as things stand now; as things are now | the thing-in-itself | to be above such things | He is not really on top of things.",
            // "absolut; überhaupt {adv} (Verstärkung einer Aussage) | jegliche/r/s; absolut jeder | keinerlei; absolut kein | jeglichen Zweifel ausräumen | Ich habe absolut/überhaupt keinen Grund, dorthin zurückzukehren. :: whatsoever (postpositive) (used to emphasize an assertion) | any … whatsoever | no … whatsoever | to remove any doubt whatsoever | I have no reason whatsoever to return there.; I have no reason to return there whatsoever.",
            // "absondernd; sekretorisch; Sekretions…; sezernierend {adj} [biol.] | Sekretionskanälchen {n} | Sekretionsmechanismus {m} | Sekretionsnerv {n} | Gelenkschmiere sezernierend :: secretory | secretory canaliculus | secretory mechanism | secretory nerve | synoviparous",
            // "alterungsbeständig {adj} (Werkstoff) {adj} :: resistant to ageing [Br.]/aging [Am.]; ageing-resistant [Br.]; aging-resistant [Am.]; non-ageing [Br.]; non-aging [Am.] (of a material)",
            // "Abfallcontainer {f}; Müllcontainer {f} | Abfallcontainer {pl}; Müllcontainer {pl} :: waste/rubbish/garbage [Am.] container | waste/rubbish/garbage containers",
            // "Arzneimittelnebenwirkung {f}; unerwünschte Arzeimittelwirkung {f} [pharm.] | Arzneimittelnebenwirkungen {pl}; unerwünschte Arzeimittelwirkungen {pl} | schwerwiegende Nebenwirkung; schwerwiegende unerwünschte Arzneimittelwirkung :: advserse drug reaction; adverse drug effect; adverse effect | advserse drug reactions; adverse drug effects; adverse effects | serious adverse drug reaction /SADR/; serious adverse reaction / SAR/",
            // "Arztpraxis {f}; Ordination {f} [Ös.]; Arztambulatorium {n} [Südtirol] [med.] | Arztpraxen {pl}; Ordinationen {pl}; Arztambulatorien {pl} | Privatpraxis {f} | eine Arztpraxis / Ordination [Ös.] / ein Arztambulatorium [Südtirol] übernehmen :: medical practice; doctor's surgery [Br.]; medical office [Am.] | medical practices; doctor's surgeries; medical offices | private practice | to take over a medical practice/doctor's surgery [Br.] / medical office [Am.]",
            // "Bereitschaftszustand {m}; Bereitschaft {f} [electr.] [techn.] | Laufzeit im Bereitschaftszustand (Mobilgeräte usw.) | Bereitschaftsverlust {m} [electr.] | im Bereitschaftsbetrieb; in Wartestellung | im Bereitschaftsmodus / in Wartestellung / einsatzbereit sein :: standby condition; standby (readiness for immediate deployment) | standby time (of mobile devices etc.) | standby loss | under standby conditions | to be on standby",
            // "dümmster anzunehmender Nutzer; dümmster anzunehmender User /DAU/ [ugs.] [comp.] :: dumbest assumable user /DAU/; most stupid user imaginable [coll.]",
            // "Waschkessel {m} | Waschkessel {pl} :: washboiler <wash-boiler> <wash boiler> | washboilers",
            // "zur Zeit /z.Z., z.Zt./ :: at present, for the time being, at the time of",
            // "in der Regel /i. d. R./ :: generally; usually {adv}",
            // "West Virginia (US-Bundesstaat; Hauptstadt: Charleston) [geogr.] :: West Virginia /W.Va./ /W. Virg./ /WV/ (state of the US, capital: Charleston)",
            "sich differenzieren; sich verschieden entwickeln (von etw. / zu etw.) {vr} | Adulte Stammzellen differenzieren sich / entwickeln sich zum gewünschten Zelltyp. :: to differentiate (from sth. / (in)to sth.) | Adult stem cells differentiate into the required type of cell.",
        ];
        // todo: Requires handling of alternative with ageing [Br.]/aging [Am.]
        for value in TEST_LINES {
            let result = parse_line::<VerboseError<_>, false>(value).finish();

            match &result {
                Ok((_, b)) => {
                    println!("{value}\n\n{b}\n\n{b:?}");
                    assert_eq!(value.replace(' ', "").replace("\t", ""), b.to_string().replace(' ', "").replace("\t", ""));
                }
                Err(value) => {
                    println!("!!!!!!");
                    println!("{:?}", value);
                    println!("!!!!!!");
                }
            }
        }
    }

    #[test]
    fn can_read_file(){
        let value = read_dictionary(".\\dictionaries\\ding\\de-en.txt").unwrap();
        execute_test_read_for(value, 0, 0);
    }


    #[test]
    fn can_read_file2(){
        let value = read_dictionary(".\\dictionaries\\ding\\de-en.txt").unwrap();
        let mut diff = Vec::new();
        for entry in value.filter_map(|value| value.ok()){
            let is_atom = format!("{entry}") == "Atomforscher {m}; Atomforscherin {f} | Atomforscher {pl}; Atomforscherinnen {pl} :: atomic scientist <nuclear scientist> | atomic scientist <nuclear scientist> s";
            if is_atom {
                println!("{entry}");
            }
            let x = process_translation_entry(entry.clone());
            let (a, b) = x.create_alternatives();
            if a.len() != b.len() {
                diff.push((a, b));
            }
        }

        println!("{}", diff.len());


        // println!("{}", other.iter().join("\n"))
    }
}