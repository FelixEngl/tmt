use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use itertools::Itertools;
use thiserror::Error;
use crate::topicmodel::dictionary::word_infos::Language;
use super::helper::gen_iate_tbx_reader::iter::ConceptEntryElementIter;
use super::helper::gen_iate_tbx_reader::*;
pub struct IateReader<R> {
    iter: ConceptEntryElementIter<R>
}

#[derive(Debug, Clone)]
pub struct IateElement {
    id: u64,
    subject: String,
    terms: Vec<TermCollection>
}

#[derive(Debug, Clone)]
pub struct TermCollection {
    language: Language,
    terms: Vec<Term>
}

#[derive(Debug, Clone)]
pub struct Term {
    word: String,
    reliability: String,
    term_type: HashSet<ETermNoteElement>,
    administrative_status: HashSet<ETermNoteElement>,
}

impl<R> Iterator for IateReader<R> where R: BufRead {
    type Item = Result<IateElement, IateReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next()?.map_err(IateReaderError::Xml) {
            Ok(
                ConceptEntryElement {
                    id_attribute: id,
                    lang_sec_elements,
                    descrip_element: DescripElement {
                        content: concept_desc,
                        type_attribute: concept_desc_type
                    }
                }
            ) => {
                assert!(matches!(concept_desc_type, TypeAttribute::SubjectField));
                let mut language_elements = Vec::new();
                for LangSecElement{
                    lang_attribute: sec_lang,
                    term_sec_elements
                } in lang_sec_elements {
                    let sec_lang = Language::from(sec_lang);
                    let mut terms = Vec::new();
                    for TermSecElement {
                        term_element: TermElement{ content: term},
                        descrip_element: DescripElement {
                            content: term_reliability,
                            type_attribute: term_reliability_type
                        },
                        term_note_elements
                    } in term_sec_elements {
                        assert!(matches!(term_reliability_type, TypeAttribute::ReliabilityCode));

                        let mut term_type: Option<ETermNoteElement> = HashSet::new();
                        let mut administrative_status = HashSet::new();

                        for TermNoteElement {
                            type_attribute: term_note_type,
                            content: term_note
                        } in term_note_elements {
                            match term_note_type {
                                TypeAttribute::TermType => {
                                    term_type.insert(term_note);
                                }
                                TypeAttribute::AdministrativeStatus => {
                                    administrative_status.insert(term_note);
                                }
                                _ => unreachable!()
                            }
                        }

                        terms.push(
                            Term {
                                word: term,
                                reliability: term_reliability,
                                term_type,
                                administrative_status
                            }
                        )
                    }
                    language_elements.push(
                        TermCollection {
                            language: sec_lang,
                            terms
                        }
                    )
                }
                Some(
                    Ok(
                        IateElement {
                            id,
                            subject: concept_desc,
                            terms: language_elements
                        }
                    )
                )
            }
            Err(err) => {
                Some(Err(err))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum IateReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Xml(#[from] TbxReaderError),
}

pub fn read_iate(path: impl AsRef<Path>) -> Result<IateReader<BufReader<File>>, IateReaderError> {
    let r = quick_xml::reader::Reader::from_reader(
        BufReader::with_capacity(
            128*1024,
            File::options().read(true).open(path)?
        )
    );
    let iter = iter_for_concept_entry_element(r);
    Ok(
        IateReader {
            iter,
        }
    )
}

#[cfg(test)]
mod test {
    use super::read_iate;

    #[test]
    fn can_run(){
        let mut reader = read_iate(
            "dictionaries/IATE/IATE_export.tbx"
        ).unwrap();

        println!("{}", reader.count());

        let mut reader = read_iate(
            "dictionaries/IATE/IATE_export.tbx"
        ).unwrap();

        println!("{:?}", reader.next())
    }
}