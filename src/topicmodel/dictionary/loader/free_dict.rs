use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;
use crate::topicmodel::dictionary::loader::helper::gen_freedict_tei_reader::*;
use crate::topicmodel::dictionary::word_infos::{Domain, GrammaticalGender, GrammaticalNumber, Language, PartOfSpeech, Register};
// see https://tei-c.org/release/doc/tei-p5-doc/en/html/DI.html

pub struct FreeDictReader<R> {
    iter: iter::EntryElementIter<R>,
}

pub struct FreeDictEntry {
    pub id: String,
    pub word: Word,
    pub translations: Vec<Translation>
}


pub struct Word {
    pub orth: String,
    /// <abbr> (abbreviation) contains an abbreviation of any sort. [3.6.5 Abbreviations and Their Expansions]
    pub abbrev: Vec<String>,
    /// An inflected form of a word has a changed spelling or ending that shows the way it is used in sentences: "Finds" and "found" are inflected forms of "find".
    pub inflected: Vec<String>,
    /// <domain> (domain of use) describes the most important social context in which the text was realized or for which it is intended, for example private vs. public, education, religion, etc. [16.2.1 The Text Description]
    pub domains: Vec<Domain>,
    pub gram: Option<GramaticHints>,
    pub registers: Vec<Register>,
    pub languages: Vec<Language>,
    pub synonyms: Vec<Synonym>,
}


pub struct Synonym {
    pub target_id: String,
    pub word: String
}

pub struct Translation {
    pub word: String,
    pub lang: LangAttribute,
    pub gram: Option<GramaticHints>,
    /// In sociolinguistics, a register is a variety of language used for a particular purpose or particular communicative situation
    pub registers: Vec<Register>,
    pub abbrevs: Vec<String>,
    pub domains: Vec<Domain>,
    pub languages: Vec<Language>
}

pub struct GramaticHints {
    // (gender) identifies the morphological gender of a lexical item, as given in the dictionary.
    pub gender: Vec<GrammaticalGender>,
    // (part of speech) indicates the part of speech assigned to a dictionary headword such as noun, verb, or adjective.
    pub pos: Vec<PartOfSpeech>,
    // (number) indicates grammatical number associated with a form, as given in a dictionary.
    pub number: Vec<GrammaticalNumber>,
    // (collocate) contains any sequence of words that co-occur with the headword with significant frequency.
    pub collocations: Vec<String>
}

impl GramaticHints {
    pub fn read_from(_id: &str, GramGrpElement{
        // (gender) identifies the morphological gender of a lexical item, as given in the dictionary.
        gen_elements,
        // (collocate) contains any sequence of words that co-occur with the headword with significant frequency.
        colloc_elements,
        // (part of speech) indicates the part of speech assigned to a dictionary headword such as noun, verb, or adjective.
        pos_elements,
        // (subcategorization) contains subcategorization information (transitive/intransitive, countable/non-countable, etc.) [10.3.2 Grammatical Information]
        subc_elements:_,
        // (number) indicates grammatical number associated with a form, as given in a dictionary.
        number_elements,
        // (tense) indicates the grammatical tense associated with a given inflected form in a dictionary.
        tns_element: _,
        // (mood) contains information about the grammatical mood of verbs (e.g. indicative, subjunctive, imperative).
        mood_element: _
    }: GramGrpElement) -> Self {
        Self {
            gender: gen_elements.into_iter().map(|value| value.content.into()).collect(),
            pos: pos_elements.into_iter().map(|value| value.content.into()).collect(),
            number: number_elements.into_iter().map(|value| value.content.into()).collect(),
            collocations: colloc_elements.into_iter().map(|value| value.content).collect()
        }
    }
}


impl<R> Iterator for FreeDictReader<R> where R: BufRead {
    type Item = Result<FreeDictEntry, FreeDictReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next()?.map_err(FreeDictReaderError::Xml) {
            // (entry) contains a single structured entry in any kind of lexical resource, such as a dictionary or lexicon.
            Ok(EntryElement{
                   id_attribute,
                   // (form information group) groups all the information on the written and spoken
                   // forms of one headword.
                   form_element,
                   // (grammatical information) within an entry in a dictionary or a
                   // terminological data file, contains grammatical information
                   // relating to a term, word, or form.
                   gram_grp_element,
                   // groups together all information relating to one word sense in a dictionary
                   // entry, for example definitions, examples, and translation equivalents.
                   sense_element
               }) => {
                let gram = if let Some(gram) = gram_grp_element {
                    Some(GramaticHints::read_from(&id_attribute, gram))
                } else {
                    None
                };

                let FormElement {
                    // (orthographic form) gives the orthographic form of a dictionary headword.
                    orth_element,
                    // (grammatical information group) groups morpho-syntactic information about a
                    // lexical item, e.g. pos, gen, number, case, or iType (inflectional class).
                    gram_grp_element,
                    // (form information group) groups all the information on the written and spoken
                    // forms of one headword.
                    form_elements,
                    // (usage) contains usage information in a dictionary entry.
                    usg_element,
                    // classifies form as simple, compound, etc.
                    type_attribute
                } = form_element;
                let orth_element = orth_element.content;
                assert!(type_attribute.is_none(), "{id_attribute}: A top level form has a type_attribute!");
                assert!(gram_grp_element.is_none(), "{id_attribute}: A top level form has a gram_grp_element!");

                let mut abbrev = Vec::new();
                let mut inflected = Vec::new();
                for form in form_elements {
                    match form.type_attribute.expect("Nested form needs a type!") {
                        TypeAttribute::Abbrev => {
                            abbrev.push(form.orth_element.content);
                        }
                        TypeAttribute::Infl => {
                            inflected.push(form.orth_element.content);
                        }
                        _ => unreachable!()
                    }
                }

                let mut domains = Vec::new();
                let mut languages: Vec<Language> = Vec::new();
                let mut registers = Vec::new();
                if let Some(UsgElement {
                    content,
                    type_attribute
                }) = usg_element {
                    match type_attribute {
                        TypeAttribute::Colloc => {
                            // collocation given to show usage
                        }
                        TypeAttribute::Dom => {
                            match content.parse() {
                                Ok(value) => {
                                    domains.push(value);
                                }
                                Err(value) => {
                                    return Some(Err(FreeDictReaderError::Strum(value)))
                                }
                            }
                        }
                        TypeAttribute::Geo => {
                            // geographic area r.g. schw., br. am.
                        }
                        TypeAttribute::Hint => {
                            // unclassifiable piece of information to guide sense choice
                        }
                        TypeAttribute::Lang => {
                            match content.parse() {
                                Ok(value) => {
                                    languages.push(value);
                                }
                                Err(value) => {
                                    return Some(Err(FreeDictReaderError::Strum(value)))
                                }
                            }

                        }
                        TypeAttribute::Reg => {
                            match content.parse() {
                                Ok(value) => {
                                    registers.push(value);
                                }
                                Err(value) => {
                                    return Some(Err(FreeDictReaderError::Strum(value)))
                                }
                            }
                            // Register is defined as the level of formality in language that's determined by the context in which it is spoken or written.
                        }
                        TypeAttribute::Style => {
                            // style (figurative, literal, etc.)
                        }
                        TypeAttribute::Time => {
                            // temporal, historical era (‘archaic’, ‘old’, etc.)
                        }
                        _ => unreachable!()
                    }
                }
                if languages.len() > 1 {
                    panic!("{id_attribute}: hHas multiple langs {languages:?}");
                }
                let mut synonyms = Vec::new();
                let mut translations = Vec::new();
                if let Some(
                    SenseElement{
                        // (note) contains a note or annotation.
                        note_elements: _,
                        // (usage) contains usage information in a dictionary entry.
                        usg_elements,
                        // (cited quotation) contains a quotation from some other document,
                        // together with a bibliographic reference to its source.
                        //
                        // In a dictionary it may contain an example text with at least one
                        // occurrence of the word form, used in the sense being described,
                        // or a translation of the headword, or an example.
                        cit_elements,
                        // (cross-reference phrase) contains a phrase, sentence, or icon referring the reader to some other location in this or another text.
                        xr_elements
                    }) = sense_element
                {
                    // Usage infos
                    for UsgElement {
                        content,
                        type_attribute
                    } in usg_elements {
                        match type_attribute {
                            TypeAttribute::Colloc => {
                                // collocation given to show usage
                            }
                            TypeAttribute::Dom => {
                                match content.parse() {
                                    Ok(value) => {
                                        domains.push(value);
                                    }
                                    Err(value) => {
                                        return Some(Err(FreeDictReaderError::Strum(value)))
                                    }
                                }
                            }
                            TypeAttribute::Geo => {
                                // geographic area r.g. schw., br. am.
                            }
                            TypeAttribute::Hint => {
                                // unclassifiable piece of information to guide sense choice
                            }
                            TypeAttribute::Lang => {
                                match content.parse() {
                                    Ok(value) => {
                                        languages.push(value);
                                    }
                                    Err(value) => {
                                        return Some(Err(FreeDictReaderError::Strum(value)))
                                    }
                                }
                            }
                            TypeAttribute::Reg => {
                                // Register is defined as the level of formality in language that's determined by the context in which it is spoken or written.
                                match content.parse() {
                                    Ok(value) => {
                                        registers.push(value);
                                    }
                                    Err(value) => {
                                        return Some(Err(FreeDictReaderError::Strum(value)))
                                    }
                                }
                            }
                            TypeAttribute::Style => {
                                // style (figurative, literal, etc.)
                            }
                            TypeAttribute::Time => {
                                // temporal, historical era (‘archaic’, ‘old’, etc.)
                            }
                            other => panic!("{id_attribute}: An usg_elements has the type {other}!")
                        }
                    }


                    for element in cit_elements {
                        match element.type_attribute {
                            TypeAttribute::Trans => {
                                fn handle_translation(
                                    id_attribute: &str,
                                    CitElement {
                                        // (grammatical information) within an entry in a dictionary or a
                                        // terminological data file, contains grammatical information
                                        // relating to a term, word, or form.
                                        gram_grp_element,
                                        // (usage) contains usage information in a dictionary entry.
                                        usg_elements,
                                        // Already used by the outer layer
                                        type_attribute: _,
                                        // (orthographic form) gives the orthographic form of a dictionary headword.
                                        orth_element: _,
                                        // (cited quotation) contains a quotation from some other document,
                                        // together with a bibliographic reference to its source.
                                        //
                                        // In a dictionary it may contain an example text with at least one
                                        // occurrence of the word form, used in the sense being described,
                                        // or a translation of the headword, or an example.
                                        cit_elements,
                                        // (quotation) contains a phrase or passage attributed by the narrator or
                                        // author to some agency external to the text.
                                        quote_element,
                                        note_elements: _
                                    }: CitElement,
                                ) -> Result<Translation, FreeDictReaderError> {
                                    let mut abbrevs = Vec::new();
                                    {
                                        for CitElement {
                                            orth_element,
                                            type_attribute,
                                            ..
                                        } in cit_elements {
                                            match type_attribute {
                                                TypeAttribute::Abbrev => {
                                                    assert!(orth_element.is_some(), "{id_attribute}: Nested cit abbrev has no orth!");
                                                    abbrevs.push(orth_element.unwrap().content);
                                                }
                                                other => {
                                                    panic!("{id_attribute}: Nested cit has unexpected type {other}")
                                                }
                                            }
                                        }
                                    }

                                    let quote  = quote_element.expect("Expect a quote at top level!");

                                    let mut registers = Vec::new();
                                    let mut domains = Vec::new();
                                    let mut languages = Vec::new();

                                    for UsgElement {
                                        content,
                                        type_attribute
                                    } in usg_elements {
                                        match type_attribute {
                                            TypeAttribute::Reg => {
                                                registers.push(content.parse()?);
                                            }
                                            TypeAttribute::Colloc => {
                                                // collocation given to show usage
                                            }
                                            TypeAttribute::Dom => {
                                                domains.push(content.parse()?);
                                            }
                                            TypeAttribute::Geo => {
                                                // geographic area r.g. schw., br. am.
                                            }
                                            TypeAttribute::Hint => {
                                                // unclassifiable piece of information to guide sense choice
                                            }
                                            TypeAttribute::Lang => {
                                                languages.push(content.parse()?);
                                            }
                                            TypeAttribute::Style => {
                                                // style (figurative, literal, etc.)
                                            }
                                            TypeAttribute::Time => {
                                                // temporal, historical era (‘archaic’, ‘old’, etc.)
                                            }
                                            other => panic!("{id_attribute}: Did not expect an usg type with type {other} - {}!", content)
                                        }
                                    }

                                    Ok(
                                        Translation {
                                            word: quote.content,
                                            lang: quote.lang_attribute,
                                            registers,
                                            gram: gram_grp_element.map(|value| GramaticHints::read_from(&id_attribute, value)),
                                            abbrevs,
                                            domains,
                                            languages
                                        }
                                    )
                                }
                                match handle_translation(&id_attribute, element) {
                                    Ok(value) => {
                                        translations.push(value)
                                    }
                                    Err(value) => {
                                        return Some(Err(value))
                                    }
                                }

                            }
                            TypeAttribute::Example => {
                                // We ignore examples.
                            }
                            other => {
                                panic!("{id_attribute}: Unknown syn on top level{}", other)
                            }
                        }
                    }

                    for XrElement{
                        type_attribute,
                        ref_elements
                    } in xr_elements {
                        match type_attribute {
                            TypeAttribute::Syn => {
                                for RefElement{
                                    content,
                                    target_attribute
                                } in ref_elements {
                                    if let Some(target_attribute) = target_attribute {
                                        synonyms.push(
                                            Synonym {
                                                target_id: target_attribute.trim_start_matches('#').to_string(),
                                                word: content
                                            }
                                        )
                                    } else {
                                        panic!("{id_attribute}: A synonym xr element is missing a reference!")
                                    }
                                }
                            }
                            TypeAttribute::See => {
                                // We ignore the see hint!
                            }
                            other => {
                                panic!("{id_attribute}: Top level xr element has unknown type attribuet: {other}")
                            }
                        }
                    }

                }



                let word = Word {
                    orth: orth_element,
                    abbrev,
                    inflected,
                    domains,
                    gram,
                    registers,
                    languages,
                    synonyms
                };


                Some(Ok(FreeDictEntry {
                    id: id_attribute,
                    word,
                    translations
                }))
            }
            Err(err) => {
                Some(Err(err))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum FreeDictReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Xml(#[from] TeiReaderError),
    #[error(transparent)]
    Strum(#[from] strum::ParseError),
}

pub fn read_free_dict(path: impl AsRef<Path>) -> Result<FreeDictReader<BufReader<File>>, FreeDictReaderError> {
    let r = quick_xml::reader::Reader::from_reader(
        BufReader::with_capacity(
            128*1024,
            File::options().read(true).open(path)?
        )
    );
    let iter = iter_for_entry_element(r);
    Ok(
        FreeDictReader {
            iter,
        }
    )
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::free_dict::{read_free_dict};

    #[test]
    fn can_run(){
        println!("{}", read_free_dict("dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei").unwrap().map(|value| value.unwrap()).count());
        println!("{}", read_free_dict("dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei").unwrap().map(|value| value.unwrap()).count());
    }
}