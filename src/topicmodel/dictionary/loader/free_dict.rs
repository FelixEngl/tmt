use std::collections::VecDeque;
use std::str::Utf8Error;
use aho_corasick::Anchored::No;
use itertools::{Either, Itertools};
use quick_xml::Error;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::Event;
use quick_xml::name::QName;
use serde_json::de::Read;
use strum::{AsRefStr, Display, EnumString, ToString};
use thiserror::Error;
use crate::topicmodel::dictionary::loader::helper::{HasLineInfo, XmlReaderBase};

pub struct TEIReader<R> {
    inner: XmlReaderBase<R>,
    stack: VecDeque<Position>,
    is_in_body: bool
}

impl<R> HasLineInfo for TEIReader<R> {
    delegate::delegate! {
        to self.inner {
            fn current_buffer(&self) -> Option<&[u8]>;

            fn current_line_number(&self) -> usize;
        }
    }
}

#[derive(Debug, Error)]
pub enum TEIReaderError {
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),
    #[error(transparent)]
    Utf8(#[from] Utf8Error)
}


impl<R> TEIReader<R> where R: Read {



    fn next_impl(&mut self) -> Result<Option<()>, TEIReaderError> {
        if self.inner.is_eof() {
            return Ok(None)
        }


        let mut cit_type = None;

        loop {
            if !self.is_in_body {
                return Ok(None)
            }
            match self.inner.read_event()? {
                Event::Start(value) => {
                    let position = match Position::try_from(value.name().as_ref())? {
                        x@ Position::Cit => {
                            if let Some(value) = value.try_get_attribute(b"type")? {
                                cit_type = Some(std::str::from_utf8(value.value.as_ref())?.to_string())
                            }
                            x
                        }
                        unknown @ Position::Unknown => {
                            if self.stack.is_empty() {
                                continue;
                            } else {
                                println!(
                                    "Unknown: {} @ {}",
                                    String::from_utf8_lossy(value.name().as_ref()),
                                    self.stack.iter().join(" >> ")
                                );
                                unknown
                            }
                        }
                        other => {
                            other
                        }
                    };
                    self.stack.push_back(position);
                }
                Event::End(value) => {
                    let current = Position::try_from(value.name().as_ref())?;
                    if let Some(back) = self.stack.pop_back() {
                        if current != back {
                            println!("Current != Back: {current} != {back}")
                        }
                    }
                }
                Event::Text(value) => {

                }
                Event::Eof => {
                    return Ok(None)
                }
                _ => {}
            }
        }

    }
}


#[derive(Debug, AsRefStr, EnumString, Display, Copy, Clone, PartialEq, Eq)]
enum Position {
    #[strum(serialize = "entry")]
    Entry,
    #[strum(serialize = "form")]
    Form,
    #[strum(serialize = "orth")]
    Orth,
    #[strum(serialize = "pron")]
    Pron,
    #[strum(serialize = "sense")]
    Sense,
    #[strum(serialize = "cit")]
    Cit,
    #[strum(serialize = "quote")]
    Quote,
    #[strum(serialize = "gramGrp")]
    GramGrp,
    #[strum(serialize = "pos")]
    Pos,
    #[strum(serialize = "num")]
    Num,
    #[strum(serialize = "body")]
    Body,
    Unknown
}


impl TryFrom<&[u8]> for Position {
    type Error = Utf8Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match std::str::from_utf8(value)?.parse() {
            Ok(value) => Ok(value),
            Err(_) => Ok(Self::Unknown)
        }
    }
}

