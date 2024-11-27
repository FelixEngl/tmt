use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::path::Path;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag_no_case};
use nom::character::complete::{line_ending, space0, space1};
use nom::combinator::{map, opt, recognize};
use nom::IResult;
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use crate::dictionary::loader::file_parser::{base_parser_method, FileParserResult, FunctionBasedLineWiseReader, LineWiseDictionaryReader};
use crate::dictionary::loader::helper::{take_bracket, OptionalEntry};

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

impl<T> Display for WikipediaWordEntry<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lang_a)?;
        if let Some(ref value) = self.meta_a {
            write!(f, " ({})", value)?;
        }
        write!(f, "\t")?;
        if self.is_category {
            write!(f, "Kategorie:")?;
        }
        write!(f, "{}", self.lang_b)?;
        if let Some(ref value) = self.meta_b {
            write!(f, " ({})", value)?;
        }
        Ok(())
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
                            map(opt(tag_no_case("Kategorie:")), |value| value.is_some()),
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


fn parse_or_fail(content: &[u8]) -> FileParserResult<OptionalEntry<WikipediaWordEntry<String>>> {
    match base_parser_method(
        content,
        |s| parse_line::<nom::error::Error<&str>>(s)
    ) {
        Ok(value) => {
            Ok(
                value
                    .map(|value|
                        value.map(ToString::to_string)
                    ).into()
            )
        }
        Err(value) => {
            Err(
                value
                    .map(|value|
                        value.map(|value|
                            value.map(ToString::to_string)
                        ).into()
                    )
            )
        }
    }
}

pub fn read_dictionary(file: impl AsRef<Path>) -> io::Result<FunctionBasedLineWiseReader<File, OptionalEntry<WikipediaWordEntry<String>>>> {
    // todo: english    german als letzte zeile filtern
    Ok(LineWiseDictionaryReader::new(
        File::options().read(true).open(file)?,
        parse_or_fail
    ))
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::helper::test::execute_test_read_for;
    use super::read_dictionary;

    #[test]
    fn can_read(){
        let value = read_dictionary(
            "dictionaries/dicts.info/Wikipedia.txt"
        ).unwrap();
        execute_test_read_for(value, 2, 10);
    }

    // #[test]
    // fn read_single(){
    //     const VALUES: &[&str] = &[
    //         "&#945;-Keratin {n}	&#945;-keratin	noun	[biochem.] ",
    //         "(allgemeines) Besäufnis {n} [ugs.]	binge [coll.] [drinking spree]	noun	",
    //         "(Amerikanische) Schnappschildkröte {f}	snapper [coll.] [Chelydra serpentina]	noun	[zool.] [T] ",
    //         "(Echter) Alant {m}	scabwort [Inula helenium] [horse-heal]	noun	[bot.] [T] ",
    //         "NMR-Tomographie {f}	NMR tomography	noun	[MedTech.] ",
    //         "Goethe-Pflanze {f}	donkey ears {pl} [treated as sg.] [Kalanchoe pinnata, syn.: Bryophyllum calycinum, Cotyledon pinnata, Vereia pinnata]	noun	[bot.] [T] "
    //     ];
    //
    //     for s in VALUES.iter().copied() {
    //         match parse_line::<nom::error::VerboseError<&str>>(s) {
    //             Ok((value, data)) => {
    //                 println!("Left: {value}\nData: {data}")
    //             }
    //             Err(value) => {
    //                 println!("{value:?}")
    //             }
    //         }
    //     }
    // }
}