use std::error::Error;
use std::fmt::Debug;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::str::Utf8Error;
use nom::error::ParseError;
use nom::IResult;
use thiserror::Error;

pub type FunctionBasedLineWiseReader<R, O, E = DictionaryLineParserError<O>> =  LineWiseDictionaryReader<R, fn(&[u8]) -> Result<O, E>>;

pub type FileParserResult<R> = Result<R, DictionaryLineParserError<R>>;

pub trait DictionaryLineParser {
    type LineEntry;

    type Error: Error + From<std::io::Error>;

    fn parse_line(&self, line: &[u8]) -> Result<Self::LineEntry, Self::Error>;
}

impl<O, E, F> DictionaryLineParser for F
where
    E: Error + From<std::io::Error>,
    F: Fn(&[u8]) -> Result<O, E>
{
    type LineEntry = O;
    type Error = E;

    fn parse_line(&self, line: &[u8]) -> Result<Self::LineEntry, Self::Error> {
        self(line)
    }
}

/// An error that is created when some kind of data is left over after the parse and it didn't fail.
pub trait DataLeftError<R> {
    fn create(result: R, left: &str) -> Self;
}

#[derive(Error, Debug)]
pub enum DictionaryLineParserError<O> {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error(transparent)]
    Parser(#[from] nom::Err<nom::error::Error<String>>),
    #[error("Failed to properly parse the data!")]
    Lost(O, String)
}

impl<O> DictionaryLineParserError<O> {
    pub fn map<O2, F: FnOnce(O) -> O2>(self, mapper: F) -> DictionaryLineParserError<O2> {
        match self {
            DictionaryLineParserError::Io(value) => {
                DictionaryLineParserError::Io(value)
            }
            DictionaryLineParserError::Utf8(value) => {
                DictionaryLineParserError::Utf8(value)
            }
            DictionaryLineParserError::Parser(value) => {
                DictionaryLineParserError::Parser(value)
            }
            DictionaryLineParserError::Lost(target, value) => {
                DictionaryLineParserError::Lost(mapper(target), value)
            }
        }
    }
}

impl<O> DataLeftError<O> for DictionaryLineParserError<O> {
    fn create(result: O, left: &str) -> Self {
        Self::Lost(
            result,
            left.to_string()
        )
    }
}

pub fn base_parser_method<O, F>(
    content: &[u8],
    parser_function: F
) -> Result<O, DictionaryLineParserError<O>>
where
    O: Debug,
    F: FnOnce(&str) -> IResult<&str, O, nom::error::Error<&str>>
{
    generic_base_parser_method(content, parser_function)
}

/// A base function to create a content parser from a normal nom parser method.
#[inline(always)]
pub fn generic_base_parser_method<O, E, F>(
    content: &[u8],
    parser_function: F
) -> Result<O, E>
where
    E: From<nom::Err<nom::error::Error<String>>> + DataLeftError<O> + From<Utf8Error>,
    F: FnOnce(&str) -> IResult<&str, O, nom::error::Error<&str>>
{
    let content = std::str::from_utf8(content)?;
    let (left, entry) = parser_function(content).map_err(|err| {
        match err {
            nom::Err::Error(err) => {
                nom::Err::Error(nom::error::Error::from_error_kind(err.input.to_string(), err.code))
            },
            nom::Err::Incomplete(err) => {
                nom::Err::Incomplete(err)
            },
            nom::Err::Failure(err) => {
                nom::Err::Failure(nom::error::Error::from_error_kind(err.input.to_string(), err.code))
            }
        }
    })?;
    if !left.is_empty() {
        Err(E::create(entry, left))
    } else {
        Ok(entry)
    }
}



pub struct LineWiseDictionaryReader<R, P: DictionaryLineParser> {
    reader: BufReader<R>,
    parser: P,
    line_number: usize,
    eof: bool,
    buffer: Option<Vec<u8>>
}

impl<R: Read, P: DictionaryLineParser> LineWiseDictionaryReader<R, P> {
    pub fn new(reader: R, parser: P) -> Self {
        Self {
            reader: BufReader::new(reader),
            parser,
            line_number: 0,
            eof: false,
            buffer: None
        }
    }
}


impl<R: Read, P: DictionaryLineParser> LineWiseDictionaryReader<R, P> {

    pub fn current_line_number(&self) -> usize {
        self.line_number
    }

    pub fn current_buffer(&self) -> Option<&Vec<u8>> {
        self.buffer.as_ref()
    }

    fn next_impl(&mut self) -> Option<Result<P::LineEntry, P::Error>> {
        if self.eof {
            return None
        }
        let mut content = Vec::new();
        loop {
            match self.reader.read_until(b'\n', &mut content) {
                Ok(value) => {
                    if value == 0 {
                        self.eof = true;
                        break None
                    } else {
                        self.line_number += 1;
                        if let Some(first) = content.first() {
                            // Comment or empty line
                            if b'#'.eq(first) || b'\r'.eq(first) || b'\n'.eq(first) {
                                content.clear();
                                continue
                            }
                        }
                        let result = Some(self.parser.parse_line(content.as_slice()));
                        self.buffer = Some(content);
                        break result
                    }
                }
                Err(value) => {
                    break Some(Err(value.into()))
                }
            }
        }
    }
}

#[derive(Error, Debug)]
#[error("{line_number}: {source}")]
pub struct LineDictionaryReaderError<E: Error>{
    line_number: usize,
    source: E
}

impl<E: Error> LineDictionaryReaderError<E> {
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn source(&self) -> &E {
        &self.source
    }

    pub fn new(line_number: usize, source: E) -> Self {
        Self { line_number, source }
    }
}

impl<R: Read, P: DictionaryLineParser> Iterator for LineWiseDictionaryReader<R, P> {
    type Item = Result<P::LineEntry, LineDictionaryReaderError<P::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().map(|value| {
            value.map_err(|err|
                LineDictionaryReaderError::new(self.line_number, err)
            )
        })
    }
}