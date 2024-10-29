use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock, RwLockWriteGuard};
use itertools::{Either, Itertools, Position};
use crate::tokenizer::Tokenizer;
use crate::topicmodel::dictionary::{BasicDictionaryWithVocabulary, Dictionary, DictionaryMut, DictionaryWithMeta};
use crate::topicmodel::dictionary::constants::FREE_DICT;
use crate::topicmodel::dictionary::direction::{AToB, Direction, Invariant, Language as DirLang, A, B};
use crate::topicmodel::dictionary::loader::dictcc::WordEntryElement;
use crate::topicmodel::dictionary::loader::free_dict::{GramaticHints, Translation};
use crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataManager;
use crate::topicmodel::dictionary::metadata::MetadataManager;
use crate::topicmodel::dictionary::word_infos::*;
use crate::topicmodel::dictionary::word_infos::Register::Dialect;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};

mod ding;
mod dictcc;
pub(in crate::topicmodel::dictionary::loader) mod helper;
pub mod word_infos;
mod file_parser;
mod eurovoc;
mod free_dict;
pub mod dicts_info;
mod iate_reader;
mod ms_terms_reader;
mod generalized_data;
mod toolkit;
mod muse;

pub trait Preprocessor {
    fn preprocess_word<'a, D: DirLang>(&self, origin: &'static str, word: &'a str) -> Option<Either<Cow<'a, str>, (Cow<'a, str>, Cow<'a, str>)>>;
}

pub struct DefaultPreprocessor;

impl Preprocessor for DefaultPreprocessor {
    fn preprocess_word<'a, D: DirLang>(&self, _origin: &'static str, _word: &'a str) ->Option<Either<Cow<'a, str>, (Cow<'a, str>, Cow<'a, str>)>> {
        None
    }
}

pub struct SpecialPreprocessor<'a> {
    tokenizer_lang_a: Tokenizer<'a>,
    tokenizer_lang_b: Tokenizer<'a>
}

impl<'b> SpecialPreprocessor<'b> {


    fn select_tokenizer<D: DirLang>(&self) -> &Tokenizer<'b> {
        if D::LANG.is_a() {
            &self.tokenizer_lang_a
        } else {
            &self.tokenizer_lang_b
        }
    }

    fn preprocess_free_dict<'a>(&self, tokenizer: &Tokenizer<'b>, word: &'a str) -> Option<(Cow<'a, str>, Cow<'a, str>)> {
        use std::fmt::Write;
        let mut original = String::new();
        let mut processed = Vec::new();
        for (pos, (o, p)) in tokenizer.phrase(word).with_position() {
            match pos {
                Position::First | Position::Only => {
                    write!(original, "{o}").unwrap();
                    if p.is_word() && !p.is_stopword() {
                        processed.push(p.lemma);
                    } else {
                        return None
                    }
                }
                Position::Middle | Position::Last => {
                    write!(original, " {o}").unwrap();
                    if p.is_word() && !p.is_stopword() && !p.is_separator() {
                        processed.push(p.lemma);
                    }
                }
            }

        }
        Some((Cow::Owned(original), Cow::Owned(processed.join(" "))))
    }

    pub fn new(tokenizer_lang_a: Tokenizer<'b>, tokenizer_lang_b: Tokenizer<'b>) -> Self {
        Self { tokenizer_lang_a, tokenizer_lang_b }
    }
}

impl<'b> Preprocessor for SpecialPreprocessor<'b> {
    fn preprocess_word<'a, D: DirLang>(&self, origin: &'static str, word: &'a str) -> Option<Either<Cow<'a, str>, (Cow<'a, str>, Cow<'a, str>)>> {
        match origin {
            FREE_DICT => {
                Some(Either::Right(self.preprocess_free_dict(self.select_tokenizer::<D>(), word)?))
            }
            _ => None
        }
    }
}


pub struct UnifiedTranslationHelper<P = DefaultPreprocessor> {
    dictionary: DictionaryWithMeta<String, Vocabulary<String>, LoadedMetadataManager>,
    preprocessor: P,
}

impl Default for UnifiedTranslationHelper {
    fn default() -> Self {
        Self::new(DefaultPreprocessor)
    }
}

impl<P> UnifiedTranslationHelper<P> {
    pub fn new(preprocessor: P) -> Self {
        Self { dictionary: DictionaryWithMeta::default(), preprocessor }
    }
}

pub mod constants {
    pub const FREE_DICT: &'static str = "free_dict";
    pub const DICT_CC: &'static str = "dict_cc";
}

impl<P> UnifiedTranslationHelper<P> where P: Preprocessor {

    fn convert_optional_gram(gram: Option<GramaticHints>) -> (Vec<PartOfSpeech>, Vec<GrammaticalGender>, Vec<GrammaticalNumber>) {
        if let Some(g) = gram {
            (g.pos, g.gender, g.number)
        } else {
            (Vec::with_capacity(0), Vec::with_capacity(0), Vec::with_capacity(0))
        }
    }

    pub fn process_free_dict_entry<D: Direction + DirLang>(&mut self, entry: free_dict::FreeDictEntry) {
        let free_dict::Word {
            domains,
            languages,
            registers,
            orth,
            gram,
            inflected,
            abbrev,
            synonyms: _
        } = entry.word;

        let orth_id = {
            let preprocessed = if D::LANG.is_a() {
                self.preprocessor.preprocess_word::<A>(FREE_DICT, &orth)
            } else {
                self.preprocessor.preprocess_word::<B>(FREE_DICT, &orth)
            };

            let orth_id = match preprocessed {
                None => {
                    self.dictionary.insert_single_word::<D>(&orth)
                }
                Some(Either::Left(value)) => {
                    self.dictionary.insert_single_word::<D>(value.as_ref())
                }
                Some(Either::Right((_, processed))) => {
                    self.dictionary.insert_single_word::<D>(processed.as_ref())
                }
            };


            let mut meta_dict = self.dictionary.metadata_with_dict_mut();
            let mut meta = if D::DIRECTION.is_a_to_b() {
                meta_dict.get_or_create_meta::<A>(orth_id)
            } else {
                meta_dict.get_or_create_meta::<B>(orth_id)
            };
            meta.add_single_to_unaltered_vocabulary(FREE_DICT, orth);
            meta.add_all_to_abbreviations(FREE_DICT, abbrev);
            meta.add_all_to_inflected(FREE_DICT, inflected);
            meta.add_all_to_domains(FREE_DICT, domains);
            meta.add_all_to_languages(FREE_DICT, languages);
            meta.add_all_to_registers(FREE_DICT, registers);
            let (pos, gender, number) =
                Self::convert_optional_gram(gram);
            meta.add_all_to_pos(FREE_DICT, pos);
            meta.add_all_to_gender(FREE_DICT, gender);
            meta.add_all_to_number(FREE_DICT, number);
            orth_id
        };

        for Translation {
            languages,
            word,
            domains,
            gram,
            lang,
            abbrevs,
            registers
        } in entry.translations {
            let word_id = {
                let preprocessed = if D::DIRECTION.is_a_to_b() {
                    self.preprocessor.preprocess_word::<B>(FREE_DICT, &word)
                } else {
                    self.preprocessor.preprocess_word::<A>(FREE_DICT, &word)
                };

                let word_id = match preprocessed {
                    None => {
                        self.dictionary.insert_single_word::<D::OPPOSITE>(&word)
                    }
                    Some(Either::Left(value)) => {
                        self.dictionary.insert_single_word::<D::OPPOSITE>(value.as_ref())
                    }
                    Some(Either::Right((_, processed))) => {
                        self.dictionary.insert_single_word::<D::OPPOSITE>(processed.as_ref())
                    }
                };
                let mut meta_dict = self.dictionary.metadata_with_dict_mut();

                let mut meta = if D::DIRECTION.is_a_to_b() {
                    meta_dict.get_or_create_meta::<B>(word_id)
                } else {
                    meta_dict.get_or_create_meta::<A>(word_id)
                };

                meta.add_single_to_languages_default(lang.into());
                meta.add_single_to_unaltered_vocabulary(FREE_DICT, word);
                meta.add_all_to_abbreviations(FREE_DICT, abbrevs);
                meta.add_all_to_domains(FREE_DICT, domains);
                meta.add_all_to_languages(FREE_DICT, languages);
                meta.add_all_to_registers(FREE_DICT, registers);
                let (pos, gender, number) =
                    Self::convert_optional_gram(gram);
                meta.add_all_to_pos(FREE_DICT, pos);
                meta.add_all_to_gender(FREE_DICT, gender);
                meta.add_all_to_number(FREE_DICT, number);
                word_id
            };

            unsafe {
                if D::LANG.is_a() {
                    self.dictionary.insert_raw_values::<Invariant>(orth_id, word_id);
                } else {
                    self.dictionary.insert_raw_values::<Invariant>(word_id, orth_id);
                }
            }
        }
    }

    pub fn process_dictcc<D: Direction + DirLang, V: AsRef<str>>(&mut self, dictcc::Entry(
        dictcc::WordEntry(lang_a_cont),
        dictcc::WordEntry(lang_b_cont),
        word_types,
        categories
    ): dictcc::Entry<V>) {
        use crate::topicmodel::dictionary::loader::dictcc::{SpecialInfo, WordTypeInfo};

        let mut general_register = Vec::new();
        let mut general_pos = Vec::new();
        if let Some(dictcc::WordTypes(types)) = word_types {
            for WordTypeInfo(a, b) in types {
                general_pos.push(b);
                match a {
                    None => {}
                    Some(SpecialInfo::Archaic) => {
                        general_register.push(Register::Archaic);
                    }
                    Some(SpecialInfo::Rare) => {
                        general_register.push(Register::Rare);
                    }
                }
            }
        }

        let general_domains = if let Some(dictcc::WordCategories(types)) = categories {
            types.into_iter().map(|value| value.as_ref().parse::<Domain>().unwrap()).collect_vec()
        } else {
            vec![]
        };


    }


    pub fn finalize(self) -> DictionaryWithMeta<String, Vocabulary<String>, LoadedMetadataManager> {
        self.dictionary
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use fst::Set;
    use itertools::Itertools;
    use rust_stemmers::Algorithm;
    use crate::tokenizer::TokenizerBuilder;
    use crate::topicmodel::dictionary::direction::{AToB, BToA, DirectionTuple};
    use crate::topicmodel::dictionary::loader::free_dict::read_free_dict;
    use crate::topicmodel::dictionary::{BasicDictionary, SpecialPreprocessor, UnifiedTranslationHelper};

    #[test]
    pub fn test(){
        let dict_file = File::options().write(true).create(true).truncate(true).open("my_dict.json").unwrap();
        let stopwords = serde_json::from_reader::<_, HashMap<String, Vec<String>>>(
            File::open(r#"D:\Downloads\stopwords-iso.json"#).unwrap()
        ).unwrap();
        let mut words_en = stopwords.get("en").unwrap().clone();
        words_en.sort();

        let mut words_de = stopwords.get("de").unwrap().clone();
        words_de.sort();



        let mut builder1 = TokenizerBuilder::default();
        builder1.stemmer(Some((Algorithm::English, false)));
        const SEPARATORS: [&str;5] = [" ", ", ", ". ", "?", "!"];
        builder1.separators(&SEPARATORS);
        let words_en = match Set::from_iter(words_en) {
            Ok(words) => {words}
            Err(value) => {panic!("No stopwords")}
        };
        builder1.stop_words(&words_en);
        builder1.unicode(true);
        builder1.lossy_normalization(true);
        let tokenizer1 = builder1.build();


        let mut builder2 = TokenizerBuilder::default();
        builder2.stemmer(Some((Algorithm::German, false)));
        builder2.separators(&SEPARATORS);
        let words_de = match Set::from_iter(words_de) {
            Ok(words) => {words}
            Err(value) => {panic!("No stopwords")}
        };
        builder2.stop_words(&words_de);
        builder2.unicode(true);
        builder2.lossy_normalization(true);
        let tokenizer2 = builder2.build();

        let spec = SpecialPreprocessor::new(tokenizer1, tokenizer2);

        let mut default = UnifiedTranslationHelper::new(spec);
        for value in read_free_dict("dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei").unwrap().take(200) {
            default.process_free_dict_entry::<AToB>(value.unwrap());
        }
        for value in read_free_dict("dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei").unwrap().take(200) {
            default.process_free_dict_entry::<BToA>(value.unwrap());
        }
        let data = default.finalize();
        let mut writer = BufWriter::new(dict_file);
        serde_json::to_writer_pretty(&mut writer, &data).unwrap();
        writer.flush().unwrap();

        let number_of_entries = data.iter().unique_by(|value|{
            (value.a, value.b)
        }).count();

        let x = number_of_entries / 100;

        for a in data.into_iter().chunks(x).into_iter() {
            let DirectionTuple {
                a:(a_id, a_word, a_meta),
                b:(b_id, b_word, b_meta),
                direction
            } = a.last().unwrap();
            println!("{direction}: {a_word}");
            if let Some(meta) = a_meta {
                println!("{meta}");
            }
            println!("{direction}: {b_word}");
            if let Some(meta) = b_meta {
                println!("{meta}");
            }
            println!("\n----\n");
        }
    }
}