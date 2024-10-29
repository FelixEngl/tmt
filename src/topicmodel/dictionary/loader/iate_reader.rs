use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;
use crate::topicmodel::dictionary::loader::toolkit::replace_none_or_panic;
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
    reliability: Reliability,
    term_type: Option<TermType>,
    administrative_status: Option<AdministrativeStatus>,
}

#[derive(Debug, Error)]
#[error("The value {1} is not valid for {0}.")]
pub struct NotAValidTermNoteValueError(&'static str, ETermNoteElement);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumString)]
pub enum TermType {
    #[strum(serialize="fullForm")]
    FullForm,
    #[strum(serialize="abbreviation")]
    Abbreviation,
    #[strum(serialize="phrase")]
    Phrase,
    #[strum(serialize="shortForm")]
    ShortForm,
    #[strum(serialize="appellation")]
    Appellation,
    #[strum(serialize="formula")]
    Formula,
}

impl TryFrom<ETermNoteElement> for TermType {
    type Error = NotAValidTermNoteValueError;

    fn try_from(value: ETermNoteElement) -> Result<Self, Self::Error> {
        match value {
            ETermNoteElement::FullForm => Ok(Self::FullForm),
            ETermNoteElement::Abbreviation => Ok(Self::Abbreviation),
            ETermNoteElement::Phrase => Ok(Self::Phrase),
            ETermNoteElement::ShortForm => Ok(Self::ShortForm),
            ETermNoteElement::Appellation => Ok(Self::Appellation),
            ETermNoteElement::Formula => Ok(Self::Formula),
            invalid => Err(NotAValidTermNoteValueError("TermType", invalid))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumString)]
pub enum AdministrativeStatus {
    /// A term which was previously used to denote a concept, but is no longer in use.
    #[strum(to_string = "Obsolete", serialize="supersededTerm-admn-sts")]
    Obsolete,
    /// A term which is widely used, and is therefore likely to appear in documents, but
    /// which should not be used, and should be changed when editing a text.
    #[strum(to_string = "Deprecated", serialize="deprecatedTerm-admn-sts")]
    Deprecated,
    /// A term which is correct, but for which better synonyms exist.
    #[strum(to_string = "Admitted", serialize="admittedTerm-admn-sts")]
    Admitted,
    /// A term may be marked as ‘preferred’ because it is intrinsically better than other terms,
    /// or simply to ensure consistency in EU texts.
    #[strum(to_string = "Preferred", serialize="preferredTerm-admn-sts")]
    Preferred,
    /// A term or denomination which has been proposed but not yet fully adopted.
    #[strum(to_string = "Proposed", serialize="proposedTerm-admn-sts")]
    Proposed,
}

impl TryFrom<ETermNoteElement> for AdministrativeStatus {
    type Error = NotAValidTermNoteValueError;

    fn try_from(value: ETermNoteElement) -> Result<Self, Self::Error> {
        match value {
            ETermNoteElement::SupersededTermAdmnSts => Ok(Self::Obsolete),
            ETermNoteElement::DeprecatedTermAdmnSts => Ok(Self::Deprecated),
            ETermNoteElement::AdmittedTermAdmnSts => Ok(Self::Admitted),
            ETermNoteElement::PreferredTermAdmnSts => Ok(Self::Preferred),
            ETermNoteElement::ProposedTermAdmnSts => Ok(Self::Proposed),
            invalid => Err(NotAValidTermNoteValueError("AdministrativeStatus", invalid))
        }
    }
}


#[derive(Debug, Copy, Clone, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u8)]
pub enum Reliability {
    /// Automatically assigned to terms entered by non-native speakers. Also, all lookup forms have
    /// a reliability of one.
    NotVerifiedReliability = 1,
    /// Automatically assigned to terms entered or updated by native speakers.
    MinimumReliability = 6,
    /// Manually assigned by a terminologist following a reliability assessment. Reliable terms should
    /// satisfy at least one of the following criteria:
    /// having been obtained from a trusted source;
    /// having been agreed on by a representative body of same-language terminologists;
    /// being the common designation of the concept in its  eld.
    Reliable = 9,
    /// Manually assigned following a reliability assessment. Very reliable terms are:
    /// well-established and widely accepted by experts as the correct designation, or
    /// con rmed by a trusted and authoritative source, in particular a reliable written source.
    VeryReliable = 10,
}


impl<R> Iterator for IateReader<R> where R: BufRead {
    type Item = Result<IateElement, IateReaderError>;

    fn next(&mut self) -> Option<Self::Item> {

        fn process(
            ConceptEntryElement {
                id_attribute: id,
                lang_sec_elements,
                descrip_element: DescripElement {
                    content: concept_desc,
                    type_attribute: concept_desc_type
                }
            }: ConceptEntryElement
        ) -> Result<IateElement, IateReaderError> {
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

                    let mut term_type: Option<TermType> = None;
                    let mut administrative_status: Option<AdministrativeStatus> = None;

                    for TermNoteElement {
                        type_attribute: term_note_type,
                        content: term_note_value
                    } in term_note_elements {
                        match term_note_type {
                            TypeAttribute::TermType => {
                                replace_none_or_panic!(term_type, term_note_value.try_into()?, "The term_type was already set!");
                            }
                            TypeAttribute::AdministrativeStatus => {
                                replace_none_or_panic!(administrative_status, term_note_value.try_into()?, "The administrative_status was already set!");
                            }
                            _ => unreachable!()
                        }
                    }

                    terms.push(
                        Term {
                            word: term,
                            reliability: Reliability::try_from(term_reliability.parse::<u8>()?)?,
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
            Ok(
                IateElement {
                    id,
                    subject: concept_desc,
                    terms: language_elements
                }
            )
        }

        match self.iter.next()?.map_err(IateReaderError::Xml) {
            Ok(element) => {
                Some(process(element))
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
    #[error(transparent)]
    NumParser(#[from] std::num::ParseIntError),
    #[error(transparent)]
    TermNoteParser(#[from] NotAValidTermNoteValueError),
    #[error(transparent)]
    ReliabilityParser(#[from] num_enum::TryFromPrimitiveError<Reliability>)
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
    use std::collections::HashSet;
    use itertools::Itertools;
    use super::read_iate;

    #[test]
    fn can_run(){
        let reader = read_iate(
            "dictionaries/IATE/IATE_export.tbx"
        ).unwrap();
        let mut ct_1 = 0;
        let mut ct_2 = 0;
        let mut one_langs = HashSet::new();
        for value in reader {
            let value = value.unwrap();
            let lang = value.terms.iter().map(|value| value.language).collect::<HashSet<_>>();
            if lang.len() != 2 {
                if lang.len() != 1 {
                    println!("Found one with {}", lang.len());
                    break
                } else {
                    ct_1 += 1;
                    one_langs.extend(lang);
                }
            } else {
                ct_2 += 1;
            }
        }
        println!("Found with only one: {}", ct_1);
        println!("Found with only one: {}", one_langs.into_iter().join(", "));
        println!("Found with two: {}", ct_2);

       // let x =  reader.process_results(|value| {
       //      value.into_grouping_map_by(|value| { value.id }).fold_with(
       //          |_, _| { None },
       //          |acc, _key, value| {
       //              match acc {
       //                  None => {
       //                      value.terms.into_iter()
       //                  },
       //                  Some(acc) => {
       //
       //                  }
       //              }
       //          }
       //      ).collect::<Vec<_>>()
       //  }).unwrap();




    }
}

