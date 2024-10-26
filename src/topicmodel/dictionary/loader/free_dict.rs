use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::vec::IntoIter;
use itertools::{ExactlyOneError, Itertools};
use thiserror::Error;
use crate::topicmodel::dictionary::loader::helper::gen_freedict_tei_reader::*;

pub struct FreeDictReader<R> {
    iter: iter::EntryElementIter<R>
}

pub struct FreeDictEntry {
    pub id: String,
    pub entry: LangEntry
}


pub struct LangEntry {
    pub word: Word,
    pub translations: Vec<Translation>
}

pub struct Word {
    pub orth: String,
    pub abbrev: Vec<String>,
}

pub struct Translation {
    pub word: String,
    pub lang: LangAttribute,
    pub gender: Option<EGenElement>,
    pub pos: Option<EPosElement>,
    pub number: Option<ENumberElement>,
    /// Kategorie der wörter
    pub categories: Vec<String>,
    /// Gemeinsames auftreten mit diesen wörtern
    pub collocations: Vec<String>
}

fn vec_to_option_or_panic<T>(value: Vec<T>) -> Option<T> where T: Debug {
    match value.len() {
        0 => None,
        1 => Some(value.into_iter().exactly_one().expect("This should never happen!")),
        other => panic!("A value contains {other} not the expected one!")
    }
}

impl<R> Iterator for FreeDictReader<R> where R: BufRead {
    type Item = Result<FreeDictEntry, FreeDictReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next()?.map_err(FreeDictReaderError::Xml) {
            Ok(EntryElement{
                   id_attribute,
                   form_element,
                   gram_grp_element,
                   sense_element
               }) => {

                let form_element = form_element.expect("The top level form has to exist!");
                let FormElement {
                    orth_element,
                    gram_grp_element,
                    form_elements,
                    usg_element,
                    type_attribute
                } = form_element;
                let orth_element = orth_element.unwrap().content;
                assert!(gram_grp_element.is_none(), "{id_attribute}: An lvl 1 orth has a gram_grp_element!");
                assert!(type_attribute.is_none(), "{id_attribute}: An lvl 1 orth has a type_attribute!");
                assert!(usg_element.is_none(), "{id_attribute}: An lvl 1 orth has a usg_element!");

                let mut abbrev = Vec::new();
                for form in form_elements {
                    match form.type_attribute.expect("Nested form needs a type!") {
                        TypeAttribute::Abbrev => {
                            abbrev.push(form.orth_element.unwrap().content);
                        }
                        other => panic!("{id_attribute}: The type attribute is {} but expected abbrev!", other)
                    }
                }

                let word = Word {
                    orth: orth_element,
                    abbrev
                };

                let mut translations = Vec::new();

                if let Some(sense_element) = sense_element {
                    for CitElement {
                        gram_grp_element,
                        usg_elements,
                        type_attribute,
                        orth_element,
                        cit_elements,
                        quote_element,
                        ..
                        //note_elements
                    } in sense_element.cit_elements {
                        let type_attribute = type_attribute.expect("A top level cit requires a type!");
                        match type_attribute {
                            TypeAttribute::Trans => {
                                if !cit_elements.is_empty() {
                                    panic!("{id_attribute}: Hat a nested cit");
                                }

                                assert!(orth_element.is_none(), "{id_attribute}: Got an orth on level 1 cit!");

                                let quote  = quote_element.expect("Expect a quote at top level!");

                                // let mut casual_usg = Vec::new();
                                let mut categories = Vec::new();
                                let mut collocations = Vec::new();

                                for usg in usg_elements {
                                    if let Some(typ) = usg.type_attribute {
                                        match typ {
                                            TypeAttribute::Reg => {
                                                categories.push(usg.content);
                                            }
                                            TypeAttribute::Hint => {
                                                // Gives some kind of hint about the translation.
                                            }
                                            TypeAttribute::Geo => {
                                                // Things like british and american english.
                                            }
                                            TypeAttribute::Colloc => {
                                                collocations.push(usg.content)
                                            }
                                            other => panic!("{id_attribute}: Did not expect an usg type with type {other} - {}!", usg.content)
                                        }
                                    }  else {
                                        panic!("{id_attribute}: Has casual usg");
                                    }
                                }

                                let mut gender = None;
                                let mut pos = None;
                                let mut number = None;

                                if let Some(
                                    GramGrpElement{
                                        gen_elements,
                                        colloc_elements,
                                        pos_elements,
                                        subc_elements,
                                        number_elements,
                                        tns_element ,
                                        ..
                                        // mood_element
                                    }
                                ) = gram_grp_element {
                                    assert!(tns_element.is_none(), "{id_attribute}: has a tns!?");
                                    assert!(subc_elements.is_empty(), "{id_attribute}: has a subc elements!?");
                                    assert!(colloc_elements.is_empty(), "{id_attribute}: has a colloc elements: {}", colloc_elements.into_iter().map(|value| value.content).join(", "));
                                    if !gen_elements.is_empty() {
                                        match gen_elements.into_iter().exactly_one() {
                                            Ok(value) => {
                                                let _ = gender.insert(value.content);
                                            }
                                            Err(err) => {
                                                panic!("{id_attribute}: Why should there be more genders?? {}", err.count())
                                            }
                                        }
                                    };
                                    if !pos_elements.is_empty() {
                                        match pos_elements.into_iter().exactly_one() {
                                            Ok(value) => {
                                                let _ = pos.insert(value.content);
                                            }
                                            Err(err) => {
                                                panic!("{id_attribute}: Why should there be more positions?? {}", err.count())
                                            }
                                        }
                                    }
                                    if !number_elements.is_empty() {
                                        match number_elements.into_iter().exactly_one() {
                                            Ok(value) => {
                                                let _ = number.insert(value.content);
                                            }
                                            Err(err) => {
                                                panic!("{id_attribute}: Why should there be more numbers?? {}", err.count())
                                            }
                                        }
                                    }
                                };



                                translations.push(
                                    Translation {
                                        word: quote.content,
                                        lang: quote.lang_attribute.unwrap(),
                                        gender,
                                        categories,
                                        pos,
                                        number,
                                        collocations
                                    }
                                )
                            }
                            TypeAttribute::Example => {}
                            other => {
                                panic!("{id_attribute}: Unknown syn on top level{}", other)
                            }
                        }
                    }
                }


                Some(Ok(FreeDictEntry {
                    id: id_attribute,
                    entry: LangEntry {
                        word,
                        translations
                    }
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
            iter
        }
    )
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::loader::free_dict::read_free_dict;

    #[test]
    fn can_run(){


        let mut reader = read_free_dict(
            "dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei"
        ).unwrap();

        println!("{}", reader.count());

        let mut reader = read_free_dict(
            "dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei"
        ).unwrap();

        println!("{}", reader.count());
    }
}