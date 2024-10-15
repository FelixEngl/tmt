use std::fs::File;
use std::io;
use std::path::Path;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not};
use nom::character::complete::{char, satisfy, space0};
use nom::character::is_alphanumeric;
use nom::combinator::{cond, map, opt, peek, recognize};
use nom::IResult;
use nom::multi::many1;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use crate::topicmodel::dictionary::loader::file_parser::{base_parser_method, FileParserResult, FunctionBasedLineWiseReader, LineWiseDictionaryReader};
use crate::topicmodel::dictionary::loader::helper::{merge_list_opt, space_only0, take_bracket};

#[derive(Debug)]
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

#[derive(Debug)]
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

fn parse_word<'a, E: nom::error::ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, OmegaWikiWord<&'a str>, E> {
    map(
        pair(
            preceded(
                space0,
                map(recognize(
                    alt((
                        is_not("(\t;"),
                        terminated(is_a("()"), peek(is_alphanumeric))
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
                    lang_a: merge_list_opt(a, a_alt),
                    lang_b: merge_list_opt(b, b_alt)
                }
            }
        )
    )(s)
}

fn parse_or_fail(content: &[u8]) -> FileParserResult<Option<OmegaWikiEntry<String>>> {
    match base_parser_method(
        content,
        |s| parse_line::<nom::error::Error<&str>>(s)
    ) {
        Ok(None) => {
            Ok(None)
        }
        Ok(Some(value)) => {
            Ok(Some(value.map(ToString::to_string)))
        }
        Err(value) => {
            Err(value.map(|value| value.map(|value| value.map(ToString::to_string))))
        }
    }
}

pub fn read_dictionary(file: impl AsRef<Path>) -> io::Result<FunctionBasedLineWiseReader<File, Option<OmegaWikiEntry<String>>>> {
    // todo: english    german als letzte zeile filtern
    Ok(LineWiseDictionaryReader::new(
        File::options().read(true).open(file)?,
        parse_or_fail
    ))
}


#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::omega_wiki::{read_dictionary};

    #[test]
    fn can_read(){
        let mut reader = read_dictionary(r#"C:\git\tmt\data\dictionaries\dicts.info\OmegaWiki.txt"#).expect("This should read!");
        let mut ct = 0usize;
        for value in reader {
            match value {
                Ok(Some(value)) => {
                    ct += 1;
                }
                Ok(None) => {}
                Err(err) => {
                    println!("{err}")
                }
            }
        }
        println!("CT: {ct}")
    }
}