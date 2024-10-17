use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::str::Utf8Error;
use itertools::Itertools;
use quick_xml::events::Event;
use thiserror::Error;
use crate::topicmodel::dictionary::loader::helper::HasLineInfo;

#[derive(Debug, Error)]
pub enum EuroVocError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}


#[derive(Debug, Clone)]
pub struct EuroVocEntry {
    id: String,
    entries: Vec<String>
}

impl Display for EuroVocEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(id=\"{}\", entries=[\"{}\"])", self.id, self.entries.iter().join("\", \""))
    }
}


pub struct EuroVocReader<R> {
    reader: quick_xml::reader::Reader<BufReader<R>>,
    buf: Vec<u8>,
    eof: bool
}

impl<R> EuroVocReader<R> {
    pub fn new(reader: BufReader<R>) -> Self {
        Self { reader: quick_xml::reader::Reader::from_reader(reader), buf: Vec::new(), eof: false }
    }
}

impl<R> EuroVocReader<R> where R: Read {
    pub fn from_reader(reader: R) -> Self {
        Self::new(BufReader::new(reader))
    }

    fn next_impl(&mut self) -> Result<Option<EuroVocEntry>, EuroVocError>  {
        let mut in_record = false;
        let mut in_descriptor_id = false;
        let mut in_uf = false;
        let mut in_uf_el = false;
        let mut descriptor = None;
        let mut uf = Vec::new();
        let result = loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf)? {
                Event::Start(value) => {
                    match value.name().as_ref() {
                        b"RECORD" => {
                            in_record = true;
                        }
                        b"DESCRIPTEUR_ID" => {
                            in_descriptor_id = true
                        }
                        b"UF" => {
                            in_uf = true
                        }
                        b"UF_EL" => {
                            println!("UF_EL");
                            in_uf_el = true
                        }
                        unknown if in_uf_el || in_uf || in_descriptor_id => {
                            log::warn!("{} was found where is was not expected!", std::str::from_utf8(unknown).unwrap());
                        }
                        _ => {}
                    }
                }
                Event::End(value) => {
                    match value.name().as_ref() {
                        b"RECORD" => {
                            in_record = false;
                            break Some(
                                EuroVocEntry {
                                    id: descriptor.unwrap_or_default(),
                                    entries: uf
                                }
                            )
                        }
                        b"DESCRIPTEUR_ID" => {
                            in_descriptor_id = false
                        }
                        b"UF" => {
                            in_uf = false
                        }
                        b"UF_EL" => {
                            in_uf_el = false
                        }
                        _ => {}
                    }
                }
                Event::Text(value) => {
                    if in_descriptor_id {
                        descriptor = Some(std::str::from_utf8(value.as_ref())?.to_string());
                    } else if in_uf_el {
                        uf.push(
                            std::str::from_utf8(value.as_ref())?.to_string()
                        )
                    }
                }
                Event::Eof => {
                    self.eof = true;
                    break if descriptor.is_none() {
                        None
                    } else {
                        Some(
                            EuroVocEntry {
                                id: descriptor.unwrap(),
                                entries: uf
                            }
                        )
                    }
                }
                _ => {}
            }
        };

        Ok(result)
    }
}

impl<R> HasLineInfo for EuroVocReader<R> {
    fn current_buffer(&self) -> Option<&[u8]> {
        Some(self.buf.as_ref())
    }

    fn current_line_number(&self) -> usize {
        0
    }
}

impl<R> Iterator for EuroVocReader<R> where R: Read {
    type Item = Result<EuroVocEntry, EuroVocError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eof {
            return None
        }
        self.next_impl().transpose()
    }
}

pub fn read_dictionary(path: impl AsRef<Path>) -> std::io::Result<EuroVocReader<File>> {
    Ok(EuroVocReader::from_reader(File::options().read(true).open(path)?))
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::helper::test::execute_test_read_for;
    use super::read_dictionary;

    #[test]
    fn can_read(){
        let value1 = read_dictionary(
            "dictionaries/eurovoc/desc_en.xml"
        ).unwrap();
        let value2 = read_dictionary(
            "dictionaries/eurovoc/desc_de.xml"
        ).unwrap();
        execute_test_read_for(value1, 5, 30);
        println!("-----");
        execute_test_read_for(value2, 5, 30);
    }
}