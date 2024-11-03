use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::Path;
use std::str::Utf8Error;
use itertools::chain;
use nom::{AsBytes, IResult};
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::{char, space0};
use nom::combinator::{eof, map, opt, recognize};
use nom::error::Error;
use nom::multi::many1;
use nom::sequence::{delimited, pair, preceded};
use quick_xml::events::Event;
use thiserror::Error;
use crate::topicmodel::dictionary::loader::helper::HasLineInfo;

pub struct TbxReader<R> {
    reader: quick_xml::reader::Reader<R>,
    buffer: Vec<u8>,
    in_body: bool,
    finished: bool
}

impl<R> TbxReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: quick_xml::reader::Reader::from_reader(reader),
            buffer: Vec::new(),
            in_body: false,
            finished: false
        }
    }
}

#[derive(Debug, Error)]
pub enum TbxReaderError {
    #[error(transparent)]
    XmlError(#[from] quick_xml::Error),
    #[error(transparent)]
    AttributeError(#[from] quick_xml::events::attributes::AttrError),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error(transparent)]
    NumParse(#[from] ParseIntError),
    #[error(transparent)]
    Parser(#[from] nom::Err<nom::error::Error<String>>),
    #[error("Failed to parse the value '{0}'")]
    ParserFailed(String)
}

impl From<nom::Err<nom::error::Error<&str>>> for TbxReaderError {
    fn from(value: nom::Err<Error<&str>>) -> Self {
        value
            .map(|value| nom::error::Error::new(value.input.to_string(), value.code))
            .into()
    }
}

#[derive(Debug, Clone)]
pub struct TbxEntry {
    term_id: Option<usize>,
    entries: HashMap<String, Vec<String>>
}

impl Display for TbxEntry  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.term_id {
            None => {
                write!(f, "{{Id=-!-, ")?;
            }
            Some(value) => {
                write!(f, "{{Id={value}, ")?;
            }
        }
        for (k, v) in self.entries.iter() {
            write!(f, "[Lang={k}")?;
            for value in v {
                write!(f, ", \"{value}\"")?;
            }
            write!(f, "]")?;
        }
        write!(f, "}}")
    }
}

fn term_parser<'a, E: nom::error::ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, Vec<&'a str>, E> {
    map(
        pair(
            recognize(alt((is_not(";"), eof))),
            opt(
                many1(
                    preceded(
                        delimited(space0, char(':'), space0),
                        recognize(alt((is_not(";"), eof)))
                    )
                )
            )
        ),
        |(a, b)| chain!(std::iter::once(a), b.into_iter().flatten()).collect()
    )(s)
}

impl<R> TbxReader<R> where R: BufRead {
    fn read_event(&mut self) -> Result<Event, TbxReaderError> {
        Ok(self.reader.read_event_into(&mut self.buffer)?)
    }

    fn goto_body(&mut self) -> Result<(), TbxReaderError> {
        if self.in_body {
            return Ok(());
        }
        loop {
            match self.read_event()? {
                Event::Start(e) if matches!(e.name().as_ref(), b"body") => {
                    self.in_body = true;
                    break Ok(())
                }
                Event::Eof => break Ok(()),
                _ => {}
            }
        }
    }


    fn next_impl(&mut self) -> Result<Option<TbxEntry>, TbxReaderError> {
        if self.finished {
            return Ok(None)
        }
        self.goto_body()?;
        let mut term_id = None;
        let mut entries = HashMap::new();
        let mut current_lang_set = None;
        let mut terms = Vec::new();
        let mut in_terms = false;
        loop {
            match self.read_event()? {
                Event::Eof => {
                    self.finished = true;
                    break Ok(None);
                }
                Event::Start(e) => {
                    match e.name().as_ref() {
                        b"termEntry" => {
                            match e.try_get_attribute("id")? {
                                None => {}
                                Some(value) => {
                                    term_id = Some(std::str::from_utf8(value.value.as_ref())?.parse()?)
                                }
                            }
                        }
                        b"langSet" => {
                            match e.try_get_attribute("xml:lang")? {
                                None => {}
                                Some(value) => {
                                    current_lang_set = Some(std::str::from_utf8(value.value.as_ref())?.to_string())
                                }
                            }
                        }
                        b"term" => {
                            in_terms = true;
                        }
                        _ => {}
                    }
                }
                Event::Text(t) if in_terms => {
                    let s = t.unescape()?;
                    let (left, extracted) = term_parser::<nom::error::Error<&str>>(&s)?;
                    if left.is_empty() {
                        terms.extend(extracted.into_iter().map(|value| value.trim().to_string()))
                    } else if extracted.is_empty() {
                        return Err(TbxReaderError::ParserFailed(left.to_string()))
                    } else {
                        log::error!("Failed to completely parse an entry:\n\"{s}\"\n\"{left}\"");
                        terms.extend(extracted.into_iter().map(|value| value.trim().to_string()))
                    }
                }
                Event::End(e) => {
                    match e.name().as_ref() {
                        b"termEntry" => {
                            break Ok(Some(TbxEntry{
                                term_id,
                                entries
                            }));
                        }
                        b"body" => {
                            self.finished = true;
                            break Ok(None);
                        }
                        b"langSet" => {
                            if let Some(current_lang_set) = current_lang_set.take() {
                                match entries.entry(current_lang_set) {
                                    Entry::Occupied(mut value) => {
                                        value.get_mut().extend(std::mem::take(&mut terms));
                                    }
                                    Entry::Vacant(value) => {
                                        value.insert(std::mem::take(&mut terms));
                                    }
                                }
                            }
                        }
                        b"term" => {
                            in_terms = false;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            self.buffer.clear()
        }
    }
}

impl<R> HasLineInfo for TbxReader<R> {
    fn current_buffer(&self) -> Option<&[u8]> {
        None
    }

    fn current_line_number(&self) -> usize {
        0
    }
}

impl<R> Iterator for TbxReader<R> where R: BufRead {
    type Item = Result<TbxEntry, TbxReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        loop {
            match self.next_impl().transpose() {
                None => {
                    if self.finished {
                        break None;
                    }
                }
                Some(value) => {
                    break Some(value)
                }
            }
        }

    }
}

pub fn read_dictionary(path: impl AsRef<Path>) -> std::io::Result<TbxReader<BufReader<File>>> {
    Ok(TbxReader::new(BufReader::new(File::options().read(true).open(path)?)))
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::helper::test::execute_test_read_for;
    use super::read_dictionary;

    #[test]
    fn can_read(){
        let value = read_dictionary(
            "dictionaries/dicts.info/english-german-2020-12-10.tbx"
        ).unwrap();
        execute_test_read_for(value, 5, 30);
    }
}