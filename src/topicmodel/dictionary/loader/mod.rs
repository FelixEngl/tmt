use crate::tokenizer::Tokenizer;
use crate::topicmodel::dictionary::constants::{DICT_CC, DING, FREE_DICT, IATE};
use crate::topicmodel::dictionary::direction::{Direction, Invariant, Language as DirLang, A, B};
use crate::topicmodel::dictionary::loader::free_dict::{read_free_dict, FreeDictReaderError, GramaticHints, Translation};
use crate::topicmodel::dictionary::metadata::loaded::{LoadedMetadataCollectionBuilder, LoadedMetadataManager};
use crate::topicmodel::dictionary::metadata::MetadataManager;
use crate::topicmodel::dictionary::word_infos::*;
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithVocabulary, DictionaryMut, DictionaryWithMeta};
use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};
use itertools::{chain, Either, Itertools, Position};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::path::Path;
use pyo3::{Bound, PyResult};
use pyo3::prelude::{PyModule, PyModuleMethods};
use thiserror::Error;
use crate::topicmodel::dictionary::loader::dictcc::{process_word_entry, ProcessingResult};
use crate::topicmodel::dictionary::loader::file_parser::{DictionaryLineParserError, LineDictionaryReaderError};
use crate::topicmodel::dictionary::loader::iate_reader::{IateElement, IateReader, IateReaderError};
use crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataMutRef;

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

pub(crate) fn register_py_loader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<word_infos::Language>()?;
    m.add_class::<word_infos::Region>()?;
    m.add_class::<word_infos::PartOfSpeech>()?;
    m.add_class::<word_infos::GrammaticalGender>()?;
    m.add_class::<word_infos::GrammaticalNumber>()?;
    m.add_class::<word_infos::Domain>()?;
    m.add_class::<word_infos::Register>()?;
    Ok(())
}


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
    ding_dict_id_provider: u64
}

impl Default for UnifiedTranslationHelper {
    fn default() -> Self {
        Self::new(DefaultPreprocessor)
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Len {
    pub voc_a: usize,
    pub voc_b: usize,
    pub map_a_to_b: usize,
    pub map_b_to_a: usize,
}

impl Len {
    pub fn diff(&self, other: &Len) -> Self {
        Self {
            voc_a: self.voc_a.abs_diff(other.voc_a),
            voc_b: self.voc_b.abs_diff(other.voc_b),
            map_a_to_b: self.map_a_to_b.abs_diff(other.map_a_to_b),
            map_b_to_a: self.map_b_to_a.abs_diff(other.map_b_to_a),
        }
    }
}

impl Display for Len {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<P> UnifiedTranslationHelper<P> {
    pub fn new(preprocessor: P) -> Self {
        Self { dictionary: DictionaryWithMeta::default(), preprocessor, ding_dict_id_provider: 0 }
    }

    fn get_ding_id(&mut self) -> u64 {
        let x = self.ding_dict_id_provider;
        self.ding_dict_id_provider += 1;
        x
    }

    pub fn len(&self) -> Len {
        Len {
            voc_a: self.dictionary.voc_a().len(),
            voc_b: self.dictionary.voc_b().len(),
            map_a_to_b: self.dictionary.map_a_to_b().len(),
            map_b_to_a: self.dictionary.map_b_to_a().len(),
        }
    }
}

pub mod constants {
    pub const FREE_DICT: &'static str = "free_dict";
    pub const DICT_CC: &'static str = "dict_cc";
    pub const DING: &'static str = "ding";
    pub const IATE: &'static str = "iate";
}

impl<P> UnifiedTranslationHelper<P> where P: Preprocessor {

    fn convert_optional_gram(gram: Option<GramaticHints>) -> (Vec<PartOfSpeech>, Vec<GrammaticalGender>, Vec<GrammaticalNumber>) {
        if let Some(g) = gram {
            (g.pos, g.gender, g.number)
        } else {
            (Vec::with_capacity(0), Vec::with_capacity(0), Vec::with_capacity(0))
        }
    }
    fn insert_single_word_inner<'a, D: Direction + DirLang>(&mut self, original: &str, value: Option<Either<Cow<'a, str>, (Cow<'a, str>, Cow<'a, str>)>>) -> usize {
        match value {
            None => {
                self.dictionary.insert_single_word::<D>(original)
            }
            Some(Either::Left(value)) => {
                self.dictionary.insert_single_word::<D>(value.as_ref())
            }
            Some(Either::Right((_, processed))) => {
                self.dictionary.insert_single_word::<D>(processed.as_ref())
            }
        }
    }
    fn insert<'a, D: Direction + DirLang, L: DirLang>(&'a mut self, dict: &'static str, word: &str) -> (usize, LoadedMetadataMutRef<'a>) {
        let preprocessed = if D::LANG.is_a() {
            self.preprocessor.preprocess_word::<L>(dict, word)
        } else {
            self.preprocessor.preprocess_word::<L::OPPOSITE>(dict, word)
        };

        let orth_id = self.insert_single_word_inner::<D>(word, preprocessed);

        (orth_id, if D::DIRECTION.is_a_to_b() {
            self.dictionary.metadata.get_or_create_meta::<L>(orth_id)
        } else {
            self.dictionary.metadata.get_or_create_meta::<L::OPPOSITE>(orth_id)
        })
    }

    pub fn finalize(self) -> DictionaryWithMeta<String, Vocabulary<String>, LoadedMetadataManager> {
        self.dictionary
    }


    pub fn read_free_dict<D: DirLang + Direction>(&mut self, p: impl AsRef<Path>) -> Result<usize, (usize, Vec<FreeDictReaderError>)> {
        let mut ct = 0;
        let mut errors = Vec::new();
        match read_free_dict(p) {
            Ok(reader) => {
                for value in reader {
                    match value {
                        Ok(value) => {
                            self.process_free_dict_entry::<D>(value);
                            ct += 1;
                        }
                        Err(err) => {
                            errors.push(err);
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(ct)
                } else {
                    Err((ct, errors))
                }
            }
            Err(err) => {
                Err((0, vec![err]))
            }
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
            synonyms,
            regions,
            colloc,
            contextual,
            see
        } = entry.word;

        let orth_id = {
            let (orth_id, mut meta) = self.insert::<D, A>(
                FREE_DICT,
                &orth
            );
            meta.add_single_to_unaltered_vocabulary(FREE_DICT, &orth);
            meta.add_single_to_original_entry(FREE_DICT, orth);
            meta.add_all_to_regions(FREE_DICT, regions);
            meta.add_all_to_abbreviations(FREE_DICT, abbrev);
            meta.add_all_to_inflected(FREE_DICT, inflected);
            meta.add_all_to_domains(FREE_DICT, domains);
            meta.add_all_to_languages(FREE_DICT, languages);
            meta.add_all_to_registers(FREE_DICT, registers);
            let (pos, gender, number) =
                Self::convert_optional_gram(gram);
            meta.add_all_to_pos(FREE_DICT, pos);
            meta.add_all_to_genders(FREE_DICT, gender);
            meta.add_all_to_numbers(FREE_DICT, number);

            meta.add_all_to_synonyms(FREE_DICT, synonyms.iter().map(|value| &value.word));
            meta.add_all_to_contextual_informations(
                FREE_DICT,
                chain!(colloc, contextual)
            );
            meta.add_all_to_contextual_informations(
                FREE_DICT,
                see.iter().map(|value| &value.word)
            );

            meta.add_single_to_ids(FREE_DICT, entry.id);

            for free_dict::Synonym {
                target_id,
                word
            } in synonyms.into_iter() {
                meta.add_single_to_synonyms(FREE_DICT, word);
                meta.add_single_to_outgoing_ids(FREE_DICT, target_id);
            }
            for free_dict::See {
                target_id,
                word
            } in see.into_iter() {
                meta.add_single_to_look_at(FREE_DICT, word);
                if let Some(target_id) = target_id {
                    meta.add_single_to_outgoing_ids(FREE_DICT, target_id);
                }
            }

            orth_id
        };

        for Translation {
            languages,
            word,
            domains,
            gram,
            lang,
            abbrevs,
            registers,
            colloc,
            contextual,
            regions
        } in entry.translations {
            let word_id = {
                let (word_id, mut meta) = self.insert::<D::OPPOSITE, B>(
                    FREE_DICT,
                    &word
                );
                meta.add_single_to_languages_default(lang.into());
                meta.add_single_to_unaltered_vocabulary(FREE_DICT, &word);
                meta.add_single_to_original_entry(FREE_DICT, word);
                meta.add_all_to_abbreviations(FREE_DICT, abbrevs);
                meta.add_all_to_domains(FREE_DICT, domains);
                meta.add_all_to_languages(FREE_DICT, languages);
                meta.add_all_to_registers(FREE_DICT, registers);
                let (pos, gender, number) =
                    Self::convert_optional_gram(gram);
                meta.add_all_to_pos(FREE_DICT, pos);
                meta.add_all_to_genders(FREE_DICT, gender);
                meta.add_all_to_numbers(FREE_DICT, number);
                meta.add_all_to_regions(FREE_DICT, regions);
                meta.add_all_to_contextual_informations(
                    FREE_DICT,
                    chain!(colloc, contextual)
                );
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


    pub fn read_dict_cc<D: DirLang + Direction>(&mut self, p: impl AsRef<Path>) -> Result<usize, (usize, Vec<LineReaderError<dictcc::Entry<String>>>)> {
        match dictcc::read_dictionary(p) {
            Ok(reader) => {
                let mut ct = 0;
                let mut errors = Vec::new();
                for value in reader {
                    match value {
                        Ok(value) => {
                            ct += 1;
                            self.process_dictcc::<D, _>(value);
                        }
                        Err(err) => {
                            errors.push(err.into());
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(ct)
                } else {
                    Err((ct, errors))
                }
            }
            Err(err) => {
                Err((0, vec![err.into()]))
            }
        }
    }

    pub fn process_dictcc<D: Direction + DirLang, V: AsRef<str> + Clone + Display>(&mut self, dictcc::Entry(lang_a_cont, lang_b_cont, word_types, categories): dictcc::Entry<V>) {
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

        let a = process_word_entry(lang_a_cont, general_domains.as_slice(), general_pos.as_slice());
        let b = process_word_entry(lang_b_cont, general_domains.as_slice(), general_pos.as_slice());
        let words_a = a.create_all_word_constructs();

        let words_b = b.create_all_word_constructs();
        let mut id_a = Vec::with_capacity(words_a.len());
        let mut id_b = Vec::with_capacity(words_b.len());

        fn extend<V: AsRef<str> + Display>(
            meta: &mut LoadedMetadataMutRef,
            unchanged: &str,
            general_register: &[Register],
            general_pos: &[PartOfSpeech],
            general_domains: &[Domain],
            result: &ProcessingResult<V>
        ) {
            meta.add_single_to_unaltered_vocabulary(DICT_CC, unchanged);
            meta.add_single_to_original_entry(DICT_CC, &result.reconstructed);
            meta.add_all_to_domains(DICT_CC, general_domains.iter().copied().chain(result.domain.iter().copied()));
            meta.add_all_to_pos(DICT_CC, general_pos.iter().copied().chain(result.pos.iter().copied()));
            meta.add_all_to_registers(DICT_CC, general_register.iter().copied().chain(result.register.iter().copied()));
            meta.add_all_to_regions(DICT_CC, result.regions.iter().copied());
            meta.add_all_to_genders(DICT_CC, result.gender.iter().copied());
            meta.add_all_to_synonyms(DICT_CC, result.synonyms.iter());
            meta.add_all_to_abbreviations(DICT_CC, result.abbrev.iter());
            meta.add_all_to_contextual_informations(DICT_CC, result.contextualisation.iter().map(|v| v.to_string()));
            meta.add_all_to_unclassified(DICT_CC, result.latin_names.iter());
        }

        for value in words_a.into_iter() {
            let word = value.into_iter().join(" ");
            let (word_id, mut meta) = self.insert::<D, A>(DICT_CC, &word);
            id_a.push(word_id);
            extend(
                &mut meta,
                word.as_str(),
                &general_register,
                &general_pos,
                &general_domains,
                &a
            )
        }

        for value in words_b.into_iter() {
            let word = value.into_iter().join(" ");
            let (word_id, mut meta) = self.insert::<D::OPPOSITE, B>(DICT_CC, &word);
            id_b.push(word_id);
            extend(
                &mut meta,
                word.as_str(),
                &general_register,
                &general_pos,
                &general_domains,
                &a
            )
        }

        for (a, b) in id_a.into_iter().cartesian_product(id_b) {
            unsafe {
                if D::LANG.is_a() {
                    self.dictionary.insert_raw_values::<Invariant>(a, b);
                } else {
                    self.dictionary.insert_raw_values::<Invariant>(b, a);
                }
            }
        }
    }

    pub fn read_ding_dict<D: DirLang + Direction>(&mut self, p: impl AsRef<Path>) -> Result<usize, (usize, Vec<LineDictionaryReaderError<DictionaryLineParserError<ding::DingEntry<String>>>>)> {
        match ding::read_dictionary(p) {
            Ok(value) => {
                let mut errors = Vec::new();
                let mut ct = 0;
                for r in value {
                    match r {
                        Ok(value) => {
                            if self.process_ding_dict::<D, _>(value).is_ok() {
                                ct += 1usize;
                            }
                        }
                        Err(err) => {
                            errors.push(err);
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(ct)
                } else {
                    Err((ct, errors))
                }
            }
            Err(err) => {
                Err((0, vec![LineDictionaryReaderError::new(0, err.into())]))
            }
        }
    }

    fn process_ding_interchangeable<D: Direction + DirLang, L: DirLang, V: AsRef<str> + Clone + Display>(
        &mut self,
        interchangeables: Vec<Vec<(String, LoadedMetadataCollectionBuilder<V>)>>
    ) -> Vec<Vec<usize>> {
        let mut ids = Vec::new();
        let interchangeable_id = self.get_ding_id();
        for value in interchangeables {
            let mut ids2 = Vec::new();
            let variant_id = self.get_ding_id();
            for (variant, mut meta_data) in value {
                let (word_id, mut meta) = self.insert::<D, L>(DING, &variant);
                meta_data.dictionary_name(Some(DING));
                meta.add_single_to_unaltered_vocabulary(DING, &variant);
                meta_data.extend_internal_ids([interchangeable_id, variant_id]);
                meta_data.build_consuming().unwrap().write_into(&mut meta);
                ids2.push(word_id);
            }
            ids.push(ids2);
        }
        ids
    }

    pub fn process_ding_dict<D: Direction + DirLang, V: AsRef<str> + Clone + Display>(&mut self, entry: ding::DingEntry<V>) -> Result<(), ding::entry_processing::Translation<V>> {
        let value = ding::entry_processing::process_translation_entry(entry);
        let converted = value.create_alternatives();
        if converted.0.len() != converted.1.len() {
            return Err(value);
        }
        drop(value);
        let (alternatives_a, alternatives_b) = converted;
        for (interchangeables_a, interchangeables_b) in alternatives_a.into_iter().zip_eq(alternatives_b) {
            let interchangeables_a = self.process_ding_interchangeable::<D, A, _>(interchangeables_a);
            let interchangeables_b = self.process_ding_interchangeable::<D::OPPOSITE, B, _>(interchangeables_b);
            for (a, b) in interchangeables_a.into_iter().flatten().cartesian_product(interchangeables_b.into_iter().flatten()) {
                unsafe {
                    if D::LANG.is_a() {
                        self.dictionary.insert_raw_values::<Invariant>(a, b);
                    } else {
                        self.dictionary.insert_raw_values::<Invariant>(b, a);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn read_iate_dict<D: DirLang + Direction>(&mut self, p: impl AsRef<Path>, lang_a: Language, lang_b: Language) -> Result<usize, (usize, Vec<IateError>)> {
        match iate_reader::read_iate(p) {
            Ok(reader) => {
                let mut errors = Vec::new();
                let mut ct = 0usize;
                for entry in reader {
                    match entry {
                        Ok(value) => {
                            match self.process_iate::<D>(value, lang_a, lang_b) {
                                Ok(_) => {
                                    ct+=1;
                                }
                                Err(err) => {
                                    errors.push(err);
                                }
                            }
                        }
                        Err(err) => {
                            errors.push(err.into());
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(ct)
                } else {
                    Err((ct, errors))
                }
            }
            Err(err) => {
                Err((0, vec![err.into()]))
            }
        }
    }

    pub fn process_iate<D: Direction + DirLang>(&mut self, element: iate_reader::IateElement, lang_a: Language, lang_b: Language) -> Result<(), IateError> {
        let (id, contextual, domains, registers, mut words) = iate_reader::process_element(element);
        let mut builder = LoadedMetadataCollectionBuilder::with_name(Some(IATE));
        builder.extend_contextual_informations(contextual);
        builder.extend_domains(domains);
        builder.push_internal_ids(id);
        builder.extend_registers(registers);
        if let (Some(a), Some(b)) = (words.remove(&lang_a), words.remove(&lang_b)) {
            let mut a_word = Vec::new();
            let mut b_word = Vec::new();
            let mut a_phrase = Vec::new();
            let mut b_phrase = Vec::new();
            {
                let iate_reader::WordDefinition {
                    words,
                    phrases,
                    abbrev,
                    short_form,
                    registers,
                    // Ignored.
                    realiabilities: _,
                    unknown
                } = a;
                let mut builder = builder.clone();
                builder.extend_registers(registers);
                builder.extend_unclassified(unknown);
                builder.extend_abbreviations(abbrev);
                builder.extend_abbreviations(short_form);
                builder.push_languages(lang_a);


                for word in words {
                    let (id, mut outp) = if D::DIRECTION.is_a_to_b() {
                        self.insert::<D, A>(IATE, &word)
                    } else {
                        self.insert::<D::OPPOSITE, A>(IATE, &word)
                    };
                    a_word.push(id);
                    builder.build().unwrap().write_into(&mut outp);
                }
                builder.push_contextual_informations("phrase".to_string());
                for word in phrases {
                    let (id, mut outp) = if D::DIRECTION.is_a_to_b() {
                        self.insert::<D, A>(IATE, &word)
                    } else {
                        self.insert::<D::OPPOSITE, A>(IATE, &word)
                    };
                    a_phrase.push(id);
                    builder.build().unwrap().write_into(&mut outp);
                }
            }
            {
                let iate_reader::WordDefinition {
                    words,
                    phrases,
                    abbrev,
                    short_form,
                    registers,
                    // Ignored.
                    realiabilities: _,
                    unknown
                } = b;
                let mut builder = builder.clone();
                builder.extend_registers(registers);
                builder.extend_unclassified(unknown);
                builder.extend_abbreviations(abbrev);
                builder.extend_abbreviations(short_form);
                builder.push_languages(lang_b);

                for word in words {
                    let (id, mut outp) = if D::DIRECTION.is_a_to_b() {
                        self.insert::<D::OPPOSITE, B>(IATE, &word)
                    } else {
                        self.insert::<D, B>(IATE, &word)
                    };
                    b_word.push(id);
                    builder.build().unwrap().write_into(&mut outp);
                }
                builder.push_contextual_informations("phrase".to_string());
                for word in phrases {
                    let (id, mut outp) = if D::DIRECTION.is_a_to_b() {
                        self.insert::<D::OPPOSITE, B>(IATE, &word)
                    } else {
                        self.insert::<D, B>(IATE, &word)
                    };
                    b_phrase.push(id);
                    builder.build().unwrap().write_into(&mut outp);
                }
            }

            a_word.extend(a_phrase);
            b_word.extend(b_phrase);

            for (a, b) in a_word.into_iter().cartesian_product(b_word) {
                unsafe {
                    if D::LANG.is_a() {
                        self.dictionary.insert_raw_values::<Invariant>(a, b);
                    } else {
                        self.dictionary.insert_raw_values::<Invariant>(b, a);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum IateError {
    #[error("Encountered unexpected language {0}")]
    IllegalLanguage(Language),
    #[error(transparent)]
    IateReader(#[from] iate_reader::IateReaderError),
}

#[derive(Debug, Error)]
pub enum LineReaderError<T: Debug> {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    LineError(#[from] LineDictionaryReaderError<DictionaryLineParserError<T>>)
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::direction::{AToB, BToA};
    use crate::topicmodel::dictionary::{UnifiedTranslationHelper};
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use crate::topicmodel::dictionary::word_infos::Language;

    #[test]
    pub fn test(){
        let dict_file = File::options().write(true).create(true).truncate(true).open("my_dict.json").unwrap();

        let mut default = UnifiedTranslationHelper::default();

        let x = default.read_free_dict::<AToB>("dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei");
        if let Err((a, b)) = x {
            for value in b {
                println!("{value}")
            }
        }
        let mut current = default.len();
        println!("{}", current);

        let x = default.read_free_dict::<BToA>("dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei");
        if let Err((a, b)) = x {
            for value in b {
                println!("{value}")
            }
        }
        let new = default.len();
        println!("{} delta: {}", new, current.diff(&new));
        current = new;

        let x = default.read_dict_cc::<BToA>("dictionaries/DictCC/dict.txt");
        if let Err((a, b)) = x {
            for value in b {
                println!("{value}")
            }
        }
        let new = default.len();
        println!("{} delta: {}", new, current.diff(&new));
        current = new;

        let x = default.read_iate_dict::<AToB>("dictionaries/IATE/IATE_export.tbx", Language::English, Language::German);
        if let Err((a, b)) = x {
            for value in b {
                println!("{value}")
            }
        }
        let new = default.len();
        println!("{} delta: {}", new, current.diff(&new));
        current = new;



        let data = default.finalize();
        let mut writer = BufWriter::new(dict_file);
        serde_json::to_writer_pretty(&mut writer, &data).unwrap();
        writer.flush().unwrap();

    }
}