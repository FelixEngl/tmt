use std::io::{BufReader, Read};
use quick_xml::events::Event;
use thiserror::Error;
use crate::topicmodel::dictionary::loader::helper::HasLineInfo;

pub struct XmlReaderBase<R> {
    reader: quick_xml::reader::Reader<BufReader<R>>,
    buf: Vec<u8>,
    eof: bool
}

impl<R> XmlReaderBase<R> {
    pub fn new(reader: BufReader<R>) -> Self {
        Self { reader: quick_xml::reader::Reader::from_reader(reader), buf: Vec::new(), eof: false }
    }

    pub fn is_eof(&self) -> bool {
        self.eof
    }
}

#[derive(Debug, Error)]
pub enum ReadTagContentError {
    #[error("Unexpected event while reading: {0}")]
    UnexpectedEventError(String),
    #[error("Unexpected event while reading: {0}")]
    TagClosedBeforeOpening(String)
}

impl<R> XmlReaderBase<R> where R: Read {
    pub fn from_reader(reader: R) -> Self {
        Self::new(BufReader::new(reader))
    }

    pub fn read_event(&mut self) -> Result<Event, quick_xml::Error> {
        if self.eof {
            return Ok(Event::Eof);
        }
        self.buf.clear();
        match self.reader.read_event_into(&mut self.buf)? {
            x @ Event::Eof => {
                self.eof = true;
                Ok(x)
            }
            other => Ok(other)
        }
    }
}


impl<R>  HasLineInfo for XmlReaderBase<R> {
    fn current_buffer(&self) -> Option<&[u8]> {
        Some(self.buf.as_slice())
    }

    fn current_line_number(&self) -> usize {
        0
    }
}


