use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use arcstr::ArcStr;
use itertools::Itertools;
use strum::Display;
use thiserror::Error;
use tinyset::Set64;
use crate::dictionary::word_infos::{Language, PartOfSpeech, Region};
use super::helper::gen_ms_terms_reader::iter::TermEntryElementIter;
use super::helper::gen_ms_terms_reader::*;

pub struct MSTermsReader<R> {
    iter: TermEntryElementIter<R>,
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
                            id_attribute: id
                        },
                        term_note_element: TermNoteElement {
                            content: part_of_speech,
                            type_attribute: term_note_type
                        }
                    } } in ntig_elements {
                    let id = ArcStr::from(id);
                    assert!(matches!(term_note_type, TypeAttribute::PartOfSpeech), "Not a pos! {term_note_type}");
                    if let Some(value) = hash_map.insert(id.clone(), Term {
                        id,
                        term,
                        part_of_speech: part_of_speech.into()
                    }) {
                        panic!("Had a collision with {} in {entry_id}!", value.id)
                    }
                }
                let region = lang.try_into().ok();
                let lang2 = lang.into();
                if let Some(value) = terms.insert(lang, TermDefinition {
                    lang: lang2,
                    region,
                    defintition: defintition.into_iter().collect_vec(),
                    terms: hash_map
                }) {
                    panic!("Has collision for {} in {entry_id}!", value.lang)
                }
            }
            Ok(
                MsTermsEntry {
                    id: entry_id,
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


#[derive(Debug)]
pub struct MsTermsEntry {
    pub id: String,
    pub terms: HashMap<LangAttribute, TermDefinition>
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

    pub fn get_languages(&self) -> HashSet<Language> {
        self.terms.values().map(|v| v.lang).collect()
    }
}

#[derive(Debug)]
pub struct TermDefinition {
    // todo: Sicherstellen, das BE und US korrekt unterschieden werden
    pub lang: Language,
    pub region: Option<Region>,
    pub defintition: Vec<String>,
    pub terms: HashMap<ArcStr, Term>
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
    pub term: String,
    pub id: ArcStr,
    pub part_of_speech: PartOfSpeech
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


#[derive(Debug, Error)]
pub enum MSTermsReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Xml(#[from] MartifReaderError),
    #[error(transparent)]
    Merge(#[from] MsTermsEntryMergeError),
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MergingReaderFinishedMode {
    #[default]
    DoNothing,
    EmitUnfinished,
    EmitWhenAtLeastNLanguages(usize)
}

pub struct MergingReader {
    cache: indexmap::IndexMap<ArcStr, (MsTermsEntry, Set64<usize>)>,
    readers: Vec<Option<Box<dyn Iterator<Item=Result<MsTermsEntry, MSTermsReaderError>>>>>,
    mode: MergingReaderFinishedMode,
    dropped: usize,
    finished: bool,
}

impl MergingReader {
    pub fn new<I: IntoIterator<Item=Box<dyn Iterator<Item=Result<MsTermsEntry, MSTermsReaderError>>>>>(
        readers: I,
        mode: MergingReaderFinishedMode
    ) -> Self {
        let new = Self {
            cache: Default::default(),
            readers: readers.into_iter().map(Some).collect(),
            mode,
            dropped: 0,
            finished: false
        };
        assert!(new.readers.len() > 0);
        new
    }


    pub fn read_from<I: IntoIterator<Item=T>, T: AsRef<Path>>(
        values: I,
        mode: MergingReaderFinishedMode
    ) -> Result<Self, MSTermsReaderError> {
        Ok(
            Self::new(
                values.into_iter().map(|value| {
                    read_ms_terms(value).map(|boxing| {
                        let x: Box<dyn Iterator<Item=Result<MsTermsEntry, MSTermsReaderError>>> = Box::new(boxing);
                        x
                    })
                }).collect::<Result<Vec<_>, _>>()?,
                mode
            )
        )
    }

    pub fn cache(&self) -> &indexmap::IndexMap<ArcStr, (MsTermsEntry, Set64<usize>)> {
        &self.cache
    }

    /// Merges an element in the already cached element.
    /// Pops the element from the ache when it is completed.
    fn merge_element(&mut self, reader_id: usize, entry: MsTermsEntry) -> Result<Option<MsTermsEntry>, MSTermsReaderError> {
        match self.cache.entry(ArcStr::from(entry.id.as_str())) {
            indexmap::map::Entry::Occupied(mut value) => {
                let len = {
                    let (a, b) = value.get_mut();
                    a.merge_in_place(entry)?;
                    b.insert(reader_id);
                    b.len()
                };
                if len == self.readers.len() {
                    Ok(Some(value.swap_remove().0))
                } else {
                    Ok(None)
                }
            }
            indexmap::map::Entry::Vacant(empty) => {
                let (_, b) = empty.insert((entry, Set64::with_capacity(self.readers.len())));
                b.insert(reader_id);
                Ok(None)
            }
        }
    }

    fn read_until_next(&mut self) -> Result<Option<MsTermsEntry>, MSTermsReaderError> {
        let mut cache = Vec::with_capacity(self.readers.len());
        'outer: loop {
            for reader in self.readers.iter_mut() {
                if let Some(reader) = reader {
                    cache.push(reader.next().transpose()?);
                } else {
                    cache.push(None)
                }
            }

            for (id, element) in cache.drain(..).enumerate() {
                match element {
                    None => {
                        self.readers[id] = None;
                    }
                    Some(value) => {
                        if let Some(finished) = self.merge_element(id, value)? {
                            break 'outer Ok(Some(finished))
                        }
                    }
                }
            }

            if self.readers.iter().any(Option::is_some) {
                continue
            }
            break Ok(None)
        }
    }

    pub fn mode(&self) -> MergingReaderFinishedMode {
        self.mode
    }

    pub fn dropped(&self) -> usize {
        self.dropped
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn iter(&mut self) -> MergingReaderIter {
        MergingReaderIter {
            reader: self
        }
    }

    pub fn read_next(&mut self) -> Option<Result<MsTermsEntry, MSTermsReaderError>> {
        loop {
            if self.finished {
                debug_assert!(self.readers.iter().all(Option::is_none), "Not all readers are none!");
                match self.mode {
                    MergingReaderFinishedMode::DoNothing => {
                        if !self.cache.is_empty() {
                            self.dropped = self.cache.len();
                            self.cache.clear();
                        }
                        return None
                    }
                    MergingReaderFinishedMode::EmitUnfinished => {
                        return Some(Ok(self.cache.pop()?.1.0))
                    }
                    MergingReaderFinishedMode::EmitWhenAtLeastNLanguages(n) => {
                        while let Some((_, (value, _))) = self.cache.pop() {
                            if value.get_languages().len() >= n {
                                return Some(Ok(value))
                            } else {
                                self.dropped += 1;
                            }
                        }
                        return None
                    }
                }
            } else if self.readers.len() == 1 {
                match &mut self.readers[0] {
                    Some(val) => {
                        let result = val.next();
                        if result.is_none() {
                            self.finished = true
                        }
                        return result
                    }
                    _ => unreachable!()
                }
            } else {
                let result = self.read_until_next().transpose();
                if result.is_none() {
                    self.finished = true;
                    continue
                }
                return result
            }
        }
    }
}

impl IntoIterator for MergingReader {
    type Item = Result<MsTermsEntry, MSTermsReaderError>;
    type IntoIter = MergingReaderIterator;

    fn into_iter(self) -> Self::IntoIter {
        MergingReaderIterator {
            reader: self
        }
    }
}

pub struct MergingReaderIterator {
    reader: MergingReader
}

impl Iterator for MergingReaderIterator {
    type Item = <MergingReader as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.read_next()
    }
}

pub struct MergingReaderIter<'a> {
    reader: &'a mut MergingReader
}

impl<'a> Iterator for MergingReaderIter<'a> {
    type Item = <MergingReader as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.read_next()
    }
}



#[cfg(test)]
mod test {
    use std::collections::{HashMap};
    use super::{read_ms_terms, MergingReader, MergingReaderFinishedMode, MsTermsEntry};

    #[test]
    fn can_run(){

        let mut en_de: HashMap<String, MsTermsEntry> = HashMap::new();

        for value in read_ms_terms(
            "dictionaries/Microsoft TermCollection/MicrosoftTermCollection_german.tbx"
        ).unwrap() {
            let value = value.unwrap();
            if let Some(value) = en_de.insert(value.id.clone(), value) {
                panic!("Failed for id {}", value.id);
            }
        }

        let mut reader = MergingReader::read_from(
            vec![
                "dictionaries/Microsoft TermCollection/MicrosoftTermCollectio_british_englisch.tbx",
                "dictionaries/Microsoft TermCollection/MicrosoftTermCollection_german.tbx"
            ],
            MergingReaderFinishedMode::EmitWhenAtLeastNLanguages(2)
        ).unwrap();

        println!("{}", reader.iter().count());
        println!("{}", reader.cache().len());
        println!("{}", reader.dropped());
    }
}