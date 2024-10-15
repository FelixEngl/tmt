use std::fs::File;
use std::io;
use std::path::Path;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag_no_case};
use nom::character::complete::{line_ending, space0, space1};
use nom::combinator::{map, opt, recognize};
use nom::IResult;
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use crate::topicmodel::dictionary::loader::file_parser::{base_parser_method, FileParserResult, FunctionBasedLineWiseReader, LineWiseDictionaryReader};
use crate::topicmodel::dictionary::loader::helper::take_bracket;

#[derive(Debug)]
pub struct WikipediaWordEntry<T> {
    lang_a: T,
    meta_a: Option<T>,
    lang_b: T,
    meta_b: Option<T>,
    is_category: bool
}

impl<T> WikipediaWordEntry<T> {
    pub fn map<R, F: Fn(T) -> R>(self, mapper: F) -> WikipediaWordEntry<R> {
        WikipediaWordEntry {
            lang_a: (&mapper)(self.lang_a),
            meta_a: self.meta_a.map(&mapper),
            lang_b: (&mapper)(self.lang_b),
            meta_b: self.meta_b.map(&mapper),
            is_category: self.is_category,
        }
    }
}

fn parse_line<'a, E: nom::error::ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Option<WikipediaWordEntry<&'a str>>, E> {
    nom::combinator::cond(
        !s.starts_with('#'),
        map(
            delimited(
                space0,
                separated_pair(
                    pair(
                        recognize(is_not("(\t")),
                        opt(preceded(space0, take_bracket!('(', ')')))
                    ),
                    space1,
                    tuple(
                        (
                            nom::combinator::value(true, tag_no_case("Kategorie:")),
                            recognize(alt((is_not("("), line_ending))),
                            opt(preceded(space0, take_bracket!('(', ')')))
                        )
                    )
                ),
                space0
            ),
            |((lang_a, meta_a), (is_category, lang_b, meta_b))| {
                WikipediaWordEntry {
                    lang_a,
                    meta_a,
                    lang_b,
                    meta_b,
                    is_category
                }
            }
        )
    )(s)
}


fn parse_or_fail(content: &[u8]) -> FileParserResult<Option<WikipediaWordEntry<String>>> {
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

pub fn read_dictionary(file: impl AsRef<Path>) -> io::Result<FunctionBasedLineWiseReader<File, Option<WikipediaWordEntry<String>>>> {
    // todo: english    german als letzte zeile filtern
    Ok(LineWiseDictionaryReader::new(
        File::options().read(true).open(file)?,
        parse_or_fail
    ))
}
