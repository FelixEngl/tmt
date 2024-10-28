use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use itertools::Itertools;
use strum::Display;
use thiserror::Error;
use crate::topicmodel::dictionary::word_infos::{Language, PartOfSpeech};
use crate::topicmodel::reference::HashRef;
use super::helper::gen_ms_terms_reader::iter::TermEntryElementIter;
use super::helper::gen_ms_terms_reader::*;

pub struct MSTermsReader<R> {
    iter: TermEntryElementIter<R>,
}

#[derive(Debug)]
pub struct MsTermsEntry {
    id: HashRef<String>,
    terms: HashMap<LangAttribute, TermDefinition>
}

#[derive(Debug, Error)]
pub enum MsTermsEntryMergeError {
    #[error("Failed on top level!")]
    EntryLevel(MsTermsEntry),
    #[error("Failed on definition level!")]
    DefinitionLevel(TermDefinition),
    #[error("Failed on term level {2}!")]
    TermLevel(Term, Term, MsTermsEntryMergeTermErrorKind),
}

#[derive(Debug, Copy, Clone, Display)]
pub enum MsTermsEntryMergeTermErrorKind {
    Id,
    Term,
    POS
}

impl MsTermsEntry {
    pub fn merge_in_place(&mut self, other: MsTermsEntry) -> Result<(), MsTermsEntryMergeError> {
        if self.id == other.id {
            for (k, v) in other.terms {
                match self.terms.entry(k) {
                    Entry::Occupied(mut value) => {
                        value.get_mut().merge_in_place(v)?;
                    }
                    Entry::Vacant(value) => {
                        value.insert(v);
                    }
                }
            }
            Ok(())
        } else {
            Err(MsTermsEntryMergeError::EntryLevel(other))
        }
    }
}

#[derive(Debug)]
pub struct TermDefinition {
    lang: LangAttribute,
    defintition: Vec<String>,
    terms: HashMap<HashRef<String>, Term>
}

impl TermDefinition {
    pub fn merge_in_place(&mut self, other: TermDefinition) -> Result<(), MsTermsEntryMergeError> {
        if self.lang == other.lang {
            self.defintition.extend(other.defintition);
            for (k, v) in other.terms.into_iter() {
                match self.terms.entry(k) {
                    Entry::Occupied(mut value) => {
                        value.get_mut().merge_in_place(v)?;
                    }
                    Entry::Vacant(value) => {
                        value.insert(v);
                    }
                }
            }
            Ok(())
        } else {
            Err(MsTermsEntryMergeError::DefinitionLevel(other))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Term {
    term: String,
    id: HashRef<String>,
    part_of_speech: PartOfSpeech
}

impl Term {
    pub fn merge_in_place(&mut self, other: Term) -> Result<(), MsTermsEntryMergeError> {
        if self.id != other.id {
            return Err(MsTermsEntryMergeError::TermLevel(self.clone(), other, MsTermsEntryMergeTermErrorKind::Id))
        }
        if self.term != other.term {
            return Err(MsTermsEntryMergeError::TermLevel(self.clone(), other, MsTermsEntryMergeTermErrorKind::Term))
        }
        if self.part_of_speech != other.part_of_speech {
            return Err(MsTermsEntryMergeError::TermLevel(self.clone(), other, MsTermsEntryMergeTermErrorKind::POS))
        }
        Ok(())
    }
}

impl<R> Iterator for MSTermsReader<R> where R: BufRead {
    type Item = Result<MsTermsEntry, MSTermsReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        fn process(
            TermEntryElement {
                id_attribute: entry_id,
                lang_set_elements,
            }: TermEntryElement
        ) -> Result<MsTermsEntry, MSTermsReaderError> {

            let mut terms = HashMap::new();

            for LangSetElement {
                lang_attribute: lang,
                ntig_elements,
                descrip_grp_element: group_descriptor
            } in lang_set_elements {
                let defintition = if let Some(DescripGrpElement{ descrip_element: DescripElement {
                    content,
                    type_attribute
                } }) = group_descriptor {
                    assert!(matches!(type_attribute, TypeAttribute::Definition), "Not a definition buf expected one! {type_attribute}");
                    Some(content)
                } else {
                    None
                };

                let mut hash_map = HashMap::new();
                for NtigElement {
                    term_grp_element: TermGrpElement {
                        term_element: TermElement {
                            content: term,
                            id_attribute: term_id
                        },
                        term_note_element: TermNoteElement {
                            content: part_of_speech,
                            type_attribute: term_note_type
                        }
                    } } in ntig_elements {
                    assert!(matches!(term_note_type, TypeAttribute::PartOfSpeech), "Not a pos! {term_note_type}");
                    let id = HashRef::new(term_id);
                    if let Some(value) = hash_map.insert(id.clone(), Term {
                        id,
                        term,
                        part_of_speech: part_of_speech.into()
                    }) {
                        panic!("Had a collision with {} in {entry_id}!", value.id)
                    }
                }
                if let Some(value) = terms.insert(lang, TermDefinition {
                    lang,
                    defintition: defintition.into_iter().collect_vec(),
                    terms: hash_map
                }) {
                    panic!("Has collision for {} in {entry_id}!", value.lang)
                }
            }
            Ok(
                MsTermsEntry {
                    id: HashRef::new(entry_id),
                    terms
                }
            )
        }

        match self.iter.next()? {
            Ok(value) => {
                Some(process(value))
            }
            Err(err) => {
                Some(Err(err.into()))
            }
        }

    }
}

#[derive(Debug, Error)]
pub enum MSTermsReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Xml(#[from] MartifReaderError),
}

pub fn read_ms_terms(path: impl AsRef<Path>) -> Result<MSTermsReader<BufReader<File>>, MSTermsReaderError> {
    let r = quick_xml::reader::Reader::from_reader(
        BufReader::with_capacity(
            128*1024,
            File::options().read(true).open(path)?
        )
    );
    let iter = iter_for_term_entry_element(r);
    Ok(
        MSTermsReader {
            iter,
        }
    )
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap};
    use crate::topicmodel::reference::HashRef;
    use super::{read_ms_terms, MsTermsEntry};

    #[test]
    fn can_run(){

        let mut en_de: HashMap<HashRef<String>, MsTermsEntry> = HashMap::new();

        for value in read_ms_terms(
            "dictionaries/Microsoft TermCollection/MicrosoftTermCollection_german.tbx"
        ).unwrap() {
            let value = value.unwrap();
            if let Some(value) = en_de.insert(value.id.clone(), value) {
                panic!("Failed for id {}", value.id);
            }
        }

        let mut en_en: HashMap<HashRef<String>, MsTermsEntry> = HashMap::new();

        for value in read_ms_terms(
            "dictionaries/Microsoft TermCollection/MicrosoftTermCollectio_british_englisch.tbx"
        ).unwrap() {
            let value = value.unwrap();
            if let Some(value) = en_en.insert(value.id.clone(), value) {
                panic!("Failed for id {}", value.id);
            }
        }


        let mut merge_ct = 0usize;

        for (k, v) in en_de.iter_mut() {
            if let Some(value) = en_en.remove(k) {
                merge_ct += 1;
                v.merge_in_place(value).unwrap()
            }
        }

        println!("{merge_ct}");
    }
}