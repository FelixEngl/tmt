use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::path::Path;
use itertools::{chain, Itertools};
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not};
use nom::character::complete::{char, multispace1, satisfy, space0};
use nom::combinator::{cond, map, not, opt, peek, recognize};
use nom::IResult;
use nom::multi::many1;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use crate::topicmodel::dictionary::loader::file_parser::{base_parser_method, FileParserResult, FunctionBasedLineWiseReader, LineWiseDictionaryReader};
use crate::topicmodel::dictionary::loader::helper::{space_only0, take_bracket, OptionalEntry};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OmegaWikiWord<T> {
    word: T,
    meta: Option<T>
}

impl<T> OmegaWikiWord<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> OmegaWikiWord<R> {
        OmegaWikiWord {
            word: (&mapper)(self.word),
            meta: self.meta.map(mapper)
        }
    }
}

impl<T> Display for OmegaWikiWord<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.meta {
            None => {
                write!(f, "{}", self.word)
            }
            Some(ref value) => {
                write!(f, "{} ({})", self.word, value)
            }
        }

    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OmegaWikiEntry<T> {
    lang_a: Vec<OmegaWikiWord<T>>,
    lang_b: Vec<OmegaWikiWord<T>>
}

impl<T> OmegaWikiEntry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> OmegaWikiEntry<R> {
        OmegaWikiEntry {
            lang_a: self.lang_a.into_iter().map(|value| value.map(&mapper)).collect(),
            lang_b: self.lang_b.into_iter().map(|value| value.map(&mapper)).collect()
        }
    }
}

impl<T> Display for OmegaWikiEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.lang_a.iter().join(" ; "), self.lang_b.iter().join(" ; "))
    }
}

fn parse_word<'a, E: nom::error::ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, OmegaWikiWord<&'a str>, E> {
    map(
        pair(
            preceded(
                space0,
                map(recognize(
                    alt((
                        is_not("(\t;"),
                        terminated(is_a("()"), peek(not(alt((multispace1, is_a(";"))))))
                    ))
                ), |value: &str | value.trim()),
            ),
            opt(preceded(space0, take_bracket!('(', ')')))
        ),
        |(a, b)| {
            OmegaWikiWord {
                word: a,
                meta: b
            }
        }
    )(s)
}

fn parse_line<'a, E: nom::error::ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Option<OmegaWikiEntry<&'a str>>, E> {
    cond(
        !s.starts_with('#'),
        map(
            tuple(
                (
                    parse_word,
                    opt(
                        many1(
                            preceded(
                                delimited(space0, char(';'), space0),
                                parse_word,
                            )
                        )
                    ),
                    delimited(space_only0, satisfy(|c| c == '\t'), space_only0),
                    parse_word,
                    opt(
                        many1(
                            preceded(
                                delimited(space0, char(';'), space0),
                                parse_word,
                            )
                        )
                    ),
                )
            ),
            |(a, a_alt, _, b, b_alt)| {
                OmegaWikiEntry {
                    lang_a: chain!(std::iter::once(a), a_alt.into_iter().flatten()).collect(),
                    lang_b: chain!(std::iter::once(b), b_alt.into_iter().flatten()).collect()
                }
            }
        )
    )(s)
}


pub type OptionalOmegaWikiEntry = OptionalEntry<OmegaWikiEntry<String>>;


fn parse_or_fail(content: &[u8]) -> FileParserResult<OptionalOmegaWikiEntry> {
    match base_parser_method(
        content,
        |s| parse_line::<nom::error::Error<&str>>(s)
    ) {
        Ok(value) => {
            Ok(OptionalEntry(value.map(|value| value.map(ToString::to_string))))
        }
        Err(value) => {
            Err(
                value
                    .map(
                        |value|
                            OptionalEntry(
                                value.map(|value| value.map(ToString::to_string))
                            )
                    )
            )
        }
    }
}

pub fn read_dictionary(file: impl AsRef<Path>) -> io::Result<FunctionBasedLineWiseReader<File, OptionalOmegaWikiEntry>> {
    // todo: english    german als letzte zeile filtern
    Ok(LineWiseDictionaryReader::new(
        File::options().read(true).open(file)?,
        parse_or_fail
    ))
}


#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::helper::test::execute_test_read_for;
    use crate::topicmodel::dictionary::loader::omega_wiki::{read_dictionary};

    #[test]
    fn can_read(){
        let reader = read_dictionary(r#".\dictionaries\dicts.info\OmegaWiki.txt"#).expect("This should read!");
        execute_test_read_for(reader, 0, 0)
    }
}