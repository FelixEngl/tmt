use crate::tokenizer::Tokenizer;
use crate::topicmodel::dictionary::constants::{DICT_CC, DING, FREE_DICT, IATE, MS_TERMS, MUSE, OMEGA, WIKTIONARY};
use crate::topicmodel::dictionary::dicts_info::omega_wiki::OptionalOmegaWikiEntry;
use crate::topicmodel::dictionary::direction::{Invariant, Language as DirLang, LanguageKind, A, B};
use crate::topicmodel::dictionary::loader::dictcc::{process_word_entry, ProcessingResult};
use crate::topicmodel::dictionary::loader::file_parser::{DictionaryLineParserError, LineDictionaryReaderError};
use crate::topicmodel::dictionary::loader::free_dict::{read_free_dict, FreeDictReaderError, GramaticHints, Translation};
use crate::topicmodel::dictionary::loader::iate_reader::IateReaderError;
use crate::topicmodel::dictionary::loader::ms_terms_reader::{MSTermsReaderError, MergingReader, MergingReaderFinishedMode, MsTermsEntry, TermDefinition};
use crate::topicmodel::dictionary::loader::muse::MuseError;
use crate::topicmodel::dictionary::metadata::ex::MetadataMutRefEx;
use crate::topicmodel::dictionary::metadata::ex::{MetadataCollectionBuilder, MetadataManagerEx};
use crate::topicmodel::dictionary::metadata::MetadataManager;
use crate::topicmodel::dictionary::word_infos::*;
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithVocabulary, DictionaryMut, DictionaryWithMeta};
use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary, Vocabulary};
use itertools::{chain, Either, Itertools, Position};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};
use thiserror::Error;
use crate::topicmodel::dictionary::loader::wiktionary_reader::{convert_entry_to_entries, EntryConversionError, ExtractedWord, ExtractedWordValues, WiktionaryReaderError};

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
mod toolkit;
mod muse;
mod wiktionary_reader;

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
        for (pos, (o, p)) in tokenizer.process(word).with_position() {
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



pub struct UnifiedTranslationHelper<P = DefaultPreprocessor> {
    dictionary: DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>,
    preprocessor: P,
    ding_dict_id_provider: u64,
    dir: LanguageDirection,
    enrich_on_finalize: Vec<LoadInstruction<PathBuf>>
}


impl UnifiedTranslationHelper {
    pub fn new(dir: LanguageDirection) -> Self {
        Self::with_preprocessor(dir, DefaultPreprocessor)
    }
}

impl<P> UnifiedTranslationHelper<P> {
    pub fn with_preprocessor(dir: LanguageDirection, preprocessor: P) -> Self {
        Self {
            dir,
            dictionary: DictionaryWithMeta::new_with(
                Some(dir.lang_a().to_string()),
                Some(dir.lang_b().to_string()),
            ),
            preprocessor,
            ding_dict_id_provider: 0,
            enrich_on_finalize: Vec::new()
        }
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum EnrichOption {
    Off,
    All,
    OnFinalize
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadInstruction<T: AsRef<Path>> {
    kind: DictionaryKind,
    direction: LanguageDirection,
    paths: Either<T, Vec<T>>,
    enrich_with_meta: EnrichOption
}

impl<T> LoadInstruction<T> where T: AsRef<Path> {
    pub fn normalize(&self) -> LoadInstruction<PathBuf> {
        LoadInstruction {
            kind: self.kind,
            direction: self.direction,
            paths: self.paths.as_ref().map_either(
                |value| value.as_ref().to_path_buf(),
                |values| values.iter().map(|value| value.as_ref().to_path_buf()).collect_vec()
            ),
            enrich_with_meta: self.enrich_with_meta,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Display, EnumString, IntoStaticStr)]
pub enum DictionaryKind {
    #[strum(to_string = "free_dict")]
    FreeDict,
    #[strum(to_string = "dict_cc")]
    DictCC,
    #[strum(to_string = "ding")]
    Ding,
    #[strum(to_string = "iate")]
    IATE,
    #[strum(to_string = "omega_wiki")]
    Omega,
    #[strum(to_string = "ms_terms")]
    MSTerms,
    #[strum(to_string = "muse")]
    Muse,
    #[strum(to_string = "wiktionary")]
    Wiktionary
}


pub mod constants {
    pub const FREE_DICT: &'static str = "free_dict";
    pub const DICT_CC: &'static str = "dict_cc";
    pub const DING: &'static str = "ding";
    pub const IATE: &'static str = "iate";
    pub const OMEGA: &'static str = "omega_wiki";
    pub const MS_TERMS: &'static str = "ms_terms";
    pub const MUSE: &'static str = "muse";
    pub const WIKTIONARY: &'static str = "wiktionary";
}

#[derive(Debug, Error)]
pub enum LoadByInstructionError {
    #[error("Expected {expected} paths but got {actual}!")]
    WrongNumberOfPaths {
        expected: usize,
        actual: usize
    },
    #[error(transparent)]
    FreeDict(#[from] UnifiedTranslationError<usize, FreeDictReaderError>),
    #[error(transparent)]
    DictCC(#[from] UnifiedTranslationError<usize, LineReaderError<dictcc::Entry<String>>, std::io::Error>),
    #[error(transparent)]
    DingDict(#[from] UnifiedTranslationError<(usize, Vec<ding::entry_processing::Translation<String>>), LineDictionaryReaderError<DictionaryLineParserError<ding::DingEntry<String>>>, std::io::Error>),
    #[error(transparent)]
    IATE(#[from] UnifiedTranslationError<usize, IateError, IateReaderError>),
    #[error(transparent)]
    OmegaDict(#[from] UnifiedTranslationError<usize, LineReaderError<OptionalOmegaWikiEntry>, std::io::Error>),
    #[error(transparent)]
    MSTerms(#[from] UnifiedTranslationError<(usize, usize), MsTermsError, MSTermsReaderError>),
    #[error(transparent)]
    Muse(#[from] UnifiedTranslationError<usize, MuseError>),
    #[error(transparent)]
    Wiktionary(#[from] UnifiedTranslationError<WiktionaryReaderStats, WiktionaryError, std::io::Error>),
}

impl<P> UnifiedTranslationHelper<P> where P: Preprocessor {

    #[inline(always)]
    fn check_dir_lang<L: DirLang>(&self, dir: &LanguageDirection) -> bool {
        self.dir.same_lang::<L>(dir)
    }

    #[inline(always)]
    fn get_lang<L: DirLang>(&self) -> Language {
        self.dir.get::<L>()
    }


    fn contains_processed_word<L: DirLang>(&self, dict: &'static str, word: &str) -> bool {
        let preprocessed = self.preprocessor.preprocess_word::<L>(dict, word);
        let word = match preprocessed {
            None => {
                Cow::Borrowed(word)
            }
            Some(Either::Left(value)) => {
                value
            }
            Some(Either::Right((_, processed))) => {
                processed
            }
        };

        if L::LANG.is_a() {
            self.dictionary.voc_a().contains(word.as_ref())
        } else {
            self.dictionary.voc_b().contains(word.as_ref())
        }
    }

    #[allow(clippy::needless_lifetimes)]
    unsafe fn insert_pipeline<'a, L: DirLang>(&'a mut self, dict: &'static str, word: &str) -> (usize, MetadataMutRefEx<'a>) {
        let preprocessed = self.preprocessor.preprocess_word::<L>(dict, word);
        let orth_id = match preprocessed {
            None => {
                self.dictionary.insert_single_word::<L>(word)
            }
            Some(Either::Left(value)) => {
                self.dictionary.insert_single_word::<L>(value.as_ref())
            }
            Some(Either::Right((_, processed))) => {
                self.dictionary.insert_single_word::<L>(processed.as_ref())
            }
        };
        let lang = self.get_lang::<L>();
        let mut meta = self.dictionary.metadata.get_or_create_meta::<L>(
            match L::LANG {
                LanguageKind::A => {
                    &mut self.dictionary.inner.voc_a
                }
                LanguageKind::B => {
                    &mut self.dictionary.inner.voc_b
                }
            },
            orth_id
        );
        meta.add_single_to_languages_default(lang);
        (orth_id, meta)
    }

    #[allow(clippy::needless_lifetimes)]
    fn insert<'a, L: DirLang>(&'a mut self, dict: &'static str, word: &str, dir: &LanguageDirection) -> (usize, MetadataMutRefEx<'a>) {
        unsafe {
            if self.check_dir_lang::<L>(dir) {
                self.insert_pipeline::<L>(dict, word)
            } else {
                self.insert_pipeline::<L::OPPOSITE>(dict, word)
            }
        }
    }

    fn insert_translation_by_id(&mut self, word_id_a: usize, word_id_b: usize, dir: &LanguageDirection) {
        unsafe {
            if self.dir.eq(dir) {
                self.dictionary.insert_raw_values::<Invariant>(word_id_a, word_id_b);
            } else {
                self.dictionary.insert_raw_values::<Invariant>(word_id_b, word_id_a);
            }
        }
    }

    pub fn finalize(mut self) -> Result<DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>, (DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>, Vec<LoadByInstructionError>)> {
        let mut errors = Vec::new();
        for value in self.enrich_on_finalize.clone() {
            match self.read_by_instruction_impl::<true, _>(&value) {
                Ok(_) => {}
                Err(value) => {
                    errors.push(value);
                }
            }
        }

        if errors.is_empty() {
            Ok(self.dictionary)
        } else {
            Err((self.dictionary, errors))
        }
    }

    fn assert_lang_dir<Payload: Debug, E: Error, InitError: Error>(&self, other: &LanguageDirection) -> Result<(), UnifiedTranslationError<Payload, E, InitError>> {
        if self.dir.contains(&other.lang_a()) && self.dir.contains(&other.lang_b()) {
            Ok(())
        } else {
            Err(UnifiedTranslationError::IllegalLanguageDirection {
                expected: self.dir,
                actual: other.clone()
            })
        }
    }

    pub fn read_all_by_instruction<T: AsRef<Path> + Clone>(&mut self, instructions: &[LoadInstruction<T>]) -> Result<(), Vec<LoadByInstructionError>> {
        let mut errors = Vec::with_capacity(instructions.len());
        for instruction in instructions {
            match self.read_by_instruction(instruction) {
                Ok(_) => {}
                Err(err) => {
                    errors.push(err);
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn read_by_instruction<T: AsRef<Path> + Clone>(&mut self, instruction: &LoadInstruction<T>) -> Result<(), LoadByInstructionError> {
        if matches!(instruction.enrich_with_meta, EnrichOption::OnFinalize) {
            self.enrich_on_finalize.push(instruction.normalize())
        }
        self.read_by_instruction_impl::<false, _>(instruction)
    }

    fn read_by_instruction_impl<const IS_FINALIZE: bool, T: AsRef<Path>>(&mut self, instruction: &LoadInstruction<T>) -> Result<(), LoadByInstructionError> {
        match instruction.kind {
            DictionaryKind::FreeDict => {
                match &instruction.paths {
                    Either::Left(value) => {
                        self.read_free_dict(value, &instruction.direction)?;
                    }
                    Either::Right(values) => {
                        for value in values {
                            self.read_free_dict(value, &instruction.direction)?;
                        }
                    }
                }
            }
            DictionaryKind::DictCC => {
                match &instruction.paths {
                    Either::Left(value) => {
                        self.read_dict_cc(value, &instruction.direction)?;
                    }
                    Either::Right(values) => {
                        for value in values {
                            self.read_dict_cc(value, &instruction.direction)?;
                        }
                    }
                }
            }
            DictionaryKind::Ding => {
                match &instruction.paths {
                    Either::Left(value) => {
                        self.read_ding_dict(value, &instruction.direction)?;
                    }
                    Either::Right(values) => {
                        for value in values {
                            self.read_ding_dict(value, &instruction.direction)?;
                        }
                    }
                }
            }
            DictionaryKind::IATE => {
                match &instruction.paths {
                    Either::Left(value) => {
                        self.read_iate_dict(value, &instruction.direction)?;
                    }
                    Either::Right(values) => {
                        for value in values {
                            self.read_iate_dict(value, &instruction.direction)?;
                        }
                    }
                }
            }
            DictionaryKind::Omega => {
                match &instruction.paths {
                    Either::Left(value) => {
                        self.read_omega_dict(value, &instruction.direction)?;
                    }
                    Either::Right(values) => {
                        for value in values {
                            self.read_omega_dict(value, &instruction.direction)?;
                        }
                    }
                }
            }
            DictionaryKind::MSTerms => {
                match &instruction.paths {
                    Either::Left(_) => {
                        return Err(LoadByInstructionError::WrongNumberOfPaths {
                            actual: 1,
                            expected: 2
                        })
                    }
                    Either::Right(values) => {
                        self.read_ms_terms(values, &instruction.direction)?;
                    }
                }
            }
            DictionaryKind::Muse => {
                let file_name = format!("dictionaries/{}-{}.txt", instruction.direction.lang_a(), instruction.direction.lang_b());
                match &instruction.paths {
                    Either::Left(value) => {
                        self.read_muse(value, file_name, &instruction.direction)?;
                    }
                    Either::Right(values) => {
                        for value in values {
                            self.read_muse(value, &file_name, &instruction.direction)?;
                        }
                    }
                }
            }
            DictionaryKind::Wiktionary => {
                match &instruction.paths {
                    Either::Left(value) => {
                        self.read_wiktionary_impl::<IS_FINALIZE>(value, instruction.enrich_with_meta)?;
                    }
                    Either::Right(values) => {
                        for value in values {
                            self.read_wiktionary_impl::<IS_FINALIZE>(value, instruction.enrich_with_meta)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }


    pub fn read_free_dict(&mut self, p: impl AsRef<Path>, dir: &LanguageDirection) -> Result<usize, UnifiedTranslationError<usize, FreeDictReaderError>> {
        self.assert_lang_dir(dir)?;
        let reader = read_free_dict(p)?;
        let mut ct = 0;
        let mut errors = Vec::new();
        for value in reader {
            match value {
                Ok(value) => {
                    self.process_free_dict_entry(value, dir);
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
            Err(UnifiedTranslationError::reader(ct, errors))
        }
    }

    fn process_free_dict_entry(&mut self, entry: free_dict::FreeDictEntry, dir: &LanguageDirection) {

        fn convert_optional_gram(gram: Option<GramaticHints>) -> (Vec<PartOfSpeech>, Vec<GrammaticalGender>, Vec<GrammaticalNumber>) {
            if let Some(g) = gram {
                (g.pos, g.gender, g.number)
            } else {
                (Vec::with_capacity(0), Vec::with_capacity(0), Vec::with_capacity(0))
            }
        }


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
            let (orth_id, mut meta) = self.insert::<A>(
                FREE_DICT,
                &orth,
                &dir
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
                convert_optional_gram(gram);
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
                meta.add_single_to_synonyms(FREE_DICT, &word);
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
                let (word_id, mut meta) = self.insert::<B>(
                    FREE_DICT,
                    &word,
                    &dir
                );
                meta.add_single_to_languages_default(lang.into());
                meta.add_single_to_unaltered_vocabulary(FREE_DICT, &word);
                meta.add_single_to_original_entry(FREE_DICT, word);
                meta.add_all_to_abbreviations(FREE_DICT, abbrevs);
                meta.add_all_to_domains(FREE_DICT, domains);
                meta.add_all_to_languages(FREE_DICT, languages);
                meta.add_all_to_registers(FREE_DICT, registers);
                let (pos, gender, number) =
                    convert_optional_gram(gram);
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

            self.insert_translation_by_id(orth_id, word_id, &dir);
        }
    }


    pub fn read_dict_cc(&mut self, p: impl AsRef<Path>, dir: &LanguageDirection) -> Result<usize, UnifiedTranslationError<usize, LineReaderError<dictcc::Entry<String>>, std::io::Error>> {
        self.assert_lang_dir(dir)?;
        let reader = dictcc::read_dictionary(p)?;
        let mut ct = 0;
        let mut errors = Vec::new();
        for value in reader {
            match value {
                Ok(value) => {
                    ct += 1;
                    self.process_dictcc(value, dir);
                }
                Err(err) => {
                    errors.push(err.into());
                }
            }
        }
        if errors.is_empty() {
            Ok(ct)
        } else {
            Err(UnifiedTranslationError::reader(ct, errors))
        }
    }

    fn process_dictcc<V: AsRef<str> + Clone + Display>(&mut self, dictcc::Entry(lang_a_cont, lang_b_cont, word_types, categories): dictcc::Entry<V>, dir: &LanguageDirection) {
        use crate::topicmodel::dictionary::loader::dictcc::{SpecialInfo, WordTypeInfo};

        let mut general_register = Vec::new();
        let mut general_pos = Vec::new();
        let mut general_pos_tag = Vec::new();
        if let Some(dictcc::WordTypes(types)) = word_types {
            for WordTypeInfo(a, b, c) in types {
                general_pos.push(b);
                if let Some(c) = c {
                    general_pos_tag.extend_from_slice(c);
                }
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
            meta: &mut MetadataMutRefEx,
            unchanged: &str,
            general_register: &[Register],
            general_pos: &[PartOfSpeech],
            general_pos_tags: &[PartOfSpeechTag],
            general_domains: &[Domain],
            result: &ProcessingResult<V>
        ) {
            meta.add_single_to_unaltered_vocabulary(DICT_CC, unchanged);
            meta.add_single_to_original_entry(DICT_CC, &result.reconstructed);
            meta.add_all_to_domains(DICT_CC, general_domains.iter().copied().chain(result.domain.iter().copied()));
            meta.add_all_to_pos(DICT_CC, general_pos.iter().copied().chain(result.pos.iter().copied()));
            meta.add_all_to_pos_tag(DICT_CC, general_pos_tags.iter().copied().chain(result.pos_tag.iter().copied()));
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
            let (word_id, mut meta) = self.insert::<A>(DICT_CC, &word, dir);
            id_a.push(word_id);
            extend(
                &mut meta,
                word.as_str(),
                &general_register,
                &general_pos,
                &general_pos_tag,
                &general_domains,
                &a
            )
        }

        for value in words_b.into_iter() {
            let word = value.into_iter().join(" ");
            let (word_id, mut meta) = self.insert::<B>(DICT_CC, &word, dir);
            id_b.push(word_id);
            extend(
                &mut meta,
                word.as_str(),
                &general_register,
                &general_pos,
                &general_pos_tag,
                &general_domains,
                &a
            )
        }

        for (a, b) in id_a.into_iter().cartesian_product(id_b) {
            self.insert_translation_by_id(a, b, dir)
        }
    }


    pub fn read_ding_dict(&mut self, p: impl AsRef<Path>, dir: &LanguageDirection) -> Result<(usize, Vec<ding::entry_processing::Translation<String>>), UnifiedTranslationError<(usize, Vec<ding::entry_processing::Translation<String>>), LineDictionaryReaderError<DictionaryLineParserError<ding::DingEntry<String>>>, std::io::Error>> {
        self.assert_lang_dir(dir)?;
        let reader = ding::read_dictionary(p)?;
        let mut errors = Vec::new();
        let mut ct = 0;
        let mut ignored = Vec::new();
        for r in reader {
            match r {
                Ok(value) => {
                    match self.process_ding_dict(value, dir) {
                        Ok(_) => {
                            ct += 1usize;
                        }
                        Err(value) => {
                            ignored.push(value);
                        }
                    }
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        }
        if errors.is_empty() {
            Ok((ct, ignored))
        } else {
            Err(UnifiedTranslationError::reader((ct, ignored), errors))
        }
    }

    fn process_ding_dict<V: AsRef<str> + Clone + Display>(&mut self, entry: ding::DingEntry<V>, dir: &LanguageDirection) -> Result<(), ding::entry_processing::Translation<V>> {
        let value = ding::entry_processing::process_translation_entry(entry);
        let converted = value.create_alternatives();
        if converted.0.len() != converted.1.len() {
            return Err(value);
        }
        drop(value);
        let (alternatives_a, alternatives_b) = converted;
        for (interchangeables_a, interchangeables_b) in alternatives_a.into_iter().zip_eq(alternatives_b) {
            let interchangeables_a = self.process_ding_interchangeable::<A, _>(interchangeables_a, dir);
            let interchangeables_b = self.process_ding_interchangeable::<B, _>(interchangeables_b, dir);
            for (a, b) in interchangeables_a.into_iter().flatten().cartesian_product(interchangeables_b.into_iter().flatten()) {
                self.insert_translation_by_id(a, b, dir);
            }
        }
        Ok(())
    }

    fn process_ding_interchangeable<L: DirLang, V: AsRef<str> + Clone + Display>(&mut self, interchangeables: Vec<Vec<(String, MetadataCollectionBuilder<V>)>>, dir: &LanguageDirection) -> Vec<Vec<usize>> {
        let mut ids = Vec::new();
        let interchangeable_id = self.get_ding_id();
        for value in interchangeables {
            let mut ids2 = Vec::new();
            let variant_id = self.get_ding_id();
            for (variant, mut meta_data) in value {
                let (word_id, mut meta) = self.insert::<L>(DING, &variant, dir);
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



    pub fn read_iate_dict(&mut self, p: impl AsRef<Path>, dir: &LanguageDirection) -> Result<usize, UnifiedTranslationError<usize, IateError, IateReaderError>> {
        self.assert_lang_dir(dir)?;
        let reader = iate_reader::read_iate(p)?;
        let mut errors = Vec::new();
        let mut ct = 0usize;
        for entry in reader {
            match entry {
                Ok(value) => {
                    self.process_iate(value, dir);
                    ct+=1;
                }
                Err(err) => {
                    errors.push(err.into());
                }
            }
        }
        if errors.is_empty() {
            Ok(ct)
        } else {
            Err(UnifiedTranslationError::reader(ct, errors))
        }
    }

    fn process_iate(&mut self, element: iate_reader::IateElement, dir: &LanguageDirection) {
        let (id, contextual, domains, registers, mut words) = iate_reader::process_element(element);
        let mut builder = MetadataCollectionBuilder::with_name(Some(IATE));
        builder.extend_contextual_informations(contextual);
        builder.extend_domains(domains);
        builder.push_internal_ids(id);
        builder.extend_registers(registers);
        if let (Some(a), Some(b)) = (words.remove(&dir.lang_a()), words.remove(&dir.lang_b())) {
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
                builder.push_languages(dir.lang_a());


                for word in words {
                    let (id, mut outp) = self.insert::<A>(IATE, &word, dir);
                    a_word.push(id);
                    builder.build().unwrap().write_into(&mut outp);
                }

                builder.push_contextual_informations("phrase".to_string());
                for word in phrases {
                    let (id, mut outp) = self.insert::<A>(IATE, &word, dir);
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
                builder.push_languages(dir.lang_b());

                for word in words {
                    let (id, mut outp) = self.insert::<B>(IATE, &word, dir);
                    b_word.push(id);
                    builder.build().unwrap().write_into(&mut outp);
                }

                builder.push_contextual_informations("phrase".to_string());
                for word in phrases {
                    let (id, mut outp) = self.insert::<B>(IATE, &word, dir);
                    b_phrase.push(id);
                    builder.build().unwrap().write_into(&mut outp);
                }
            }

            a_word.extend(a_phrase);
            b_word.extend(b_phrase);

            for (a, b) in a_word.into_iter().cartesian_product(b_word) {
                self.insert_translation_by_id(a, b, dir);
            }
        }
    }


    pub fn read_omega_dict(&mut self, p: impl AsRef<Path>, dir: &LanguageDirection) -> Result<usize, UnifiedTranslationError<usize, LineReaderError<OptionalOmegaWikiEntry>, std::io::Error>> {
        self.assert_lang_dir(dir)?;
        let reader = dicts_info::omega_wiki::read_dictionary(p)?;
        let mut counter = 0;
        let mut errors = Vec::new();
        for value in reader {
            match value {
                Ok(value) => {
                    self.process_omega(value, dir);
                    counter+=1;
                }
                Err(err) => {
                    errors.push(err.into())
                }
            }
        }
        if errors.is_empty() {
            Ok(counter)
        } else {
            Err(UnifiedTranslationError::reader(counter, errors))
        }
    }

    fn process_omega(&mut self, entry: OptionalOmegaWikiEntry, dir: &LanguageDirection) {
        if let Some(dicts_info::omega_wiki::OmegaWikiEntry{
            lang_a,
            lang_b
        }) = entry.into_inner() {
            let mut lang_a_ids = Vec::new();
            let mut lang_b_ids = Vec::new();
            for value in lang_a {
                lang_a_ids.push(self.process_omega_impl::<A>(value, dir));
            }
            for value in lang_b {
                lang_b_ids.push(self.process_omega_impl::<B>(value, dir));
            }
            for (a, b) in lang_a_ids.into_iter().cartesian_product(lang_b_ids) {
                self.insert_translation_by_id(a, b, dir);
            }
        }
    }

    fn process_omega_impl<L: DirLang>(&mut self, dicts_info::omega_wiki::OmegaWikiWord { word, meta: meta_info }: dicts_info::omega_wiki::OmegaWikiWord<String>, dir: &LanguageDirection) -> usize {
        let (id, mut meta) = self.insert::<L>(OMEGA, &word, dir);
        if let Some(meta_info) = meta_info {
            if let Ok(value) = meta_info.parse() {
                meta.add_single_to_domains(OMEGA, value);
            } else if let Ok(value) = meta_info.parse() {
                meta.add_single_to_registers(OMEGA, value);
            } else if let Ok(value) = meta_info.parse() {
                meta.add_single_to_regions(OMEGA, value);
            } else {
                meta.add_single_to_contextual_informations(OMEGA, meta_info);
            }
        }
        id
    }


    pub fn read_ms_terms<I: IntoIterator<Item=T>, T: AsRef<Path>>(&mut self, paths: I, dir: &LanguageDirection) -> Result<(usize, usize), UnifiedTranslationError<(usize, usize), MsTermsError, MSTermsReaderError>> {
        let mut reader = MergingReader::read_from(
            paths,
            MergingReaderFinishedMode::EmitWhenAtLeastNLanguages(2)
        )?;
        let mut ct = 0;
        let mut errors = Vec::new();
        for value in reader.iter() {
            match value {
                Ok(result) => {
                    match self.process_ms_terms(result, dir) {
                        Ok(_) => {
                            ct += 1;
                        }
                        Err(err) => {
                            errors.push(err.into());
                        }
                    }
                }
                Err(err) => {
                    errors.push(err.into());
                }
            }
        }
        if errors.is_empty() {
            Ok((ct, reader.dropped()))
        } else {
            Err(UnifiedTranslationError::reader((ct, reader.dropped()), errors))
        }
    }

    fn process_ms_terms(&mut self, MsTermsEntry {terms, id}: MsTermsEntry, dir: &LanguageDirection) -> Result<(), MsTermsError> {
        let mut lang_a_values = Vec::new();
        let mut lang_b_values = Vec::new();
        let mut error = None;

        'outer: for (_, TermDefinition{
            lang,
            terms,
            region,
            defintition
        }) in terms {
            let mut builder =  MetadataCollectionBuilder::with_name(Some(MS_TERMS));
            if let Some(reg) = region {
                builder.push_regions(reg);
            }
            builder.push_ids(id.to_string());
            builder.push_languages(lang);
            builder.extend_unclassified(defintition);
            for (id, ms_terms_reader::Term{
                term,
                id: id2,
                part_of_speech
            }) in terms {
                assert_eq!(id, id2);
                let mut builder2 = builder.clone();
                builder2.push_ids(id.to_string());
                builder2.push_ids(id2.to_string());
                builder2.push_pos(part_of_speech);

                match lang {
                    a if a == dir.lang_a() => {
                        let (id, mut outp) = self.insert::<A>(MS_TERMS, &term, dir);
                        lang_a_values.push(id);
                        builder.build().unwrap().write_into(&mut outp);
                    }
                    b if b == dir.lang_b() => {
                        let (id, mut outp) = self.insert::<B>(MS_TERMS, &term, dir);
                        lang_b_values.push(id);
                        builder.build().unwrap().write_into(&mut outp);
                    }
                    other => {
                        // We go into a safe state. Therefore we can not simply drop out.
                        error = Some(MsTermsError::IllegalLanguage(other));
                        break 'outer;
                    }
                }
            }
        }

        for (a, b) in lang_a_values.into_iter().cartesian_product(lang_b_values) {
            unsafe {
                self.dictionary.insert_raw_values::<Invariant>(a, b);
            }
        }
        match error {
            None => {
                Ok(())
            }
            Some(value) => {
                Err(value)
            }
        }
    }


    pub fn read_muse(&mut self, path: impl AsRef<Path>, target_file_name: impl Into<String>, dir: &LanguageDirection) -> Result<usize, UnifiedTranslationError<usize, MuseError>> {
        let reader = muse::read_single_from_archive(
            path,
            target_file_name
        )?;

        let mut ct = 0;
        let mut errors = Vec::new();
        for a in reader {
            match a {
                Ok(value) => {
                    self.process_muse_term(value, dir);
                    ct += 1;
                }
                Err(value) => {
                    errors.push(value);
                }
            }
        }

        if errors.is_empty() {
            Ok(ct)
        } else {
            Err(UnifiedTranslationError::reader(ct, errors))
        }
    }

    fn process_muse_term(&mut self, (left, right):(String, String), dir: &LanguageDirection) {
        let (a, _) = self.insert::<A>(MUSE, &left, dir);
        let (b, _) = self.insert::<B>(MUSE, &right, dir);
        self.insert_translation_by_id(a, b, dir)
    }

    pub fn read_wiktionary(&mut self, path: impl AsRef<Path>, enrich_with_meta: EnrichOption) -> Result<WiktionaryReaderStats, UnifiedTranslationError<WiktionaryReaderStats, WiktionaryError, std::io::Error>> {
        self.read_wiktionary_impl::<false>(path, enrich_with_meta)
    }

    fn read_wiktionary_impl<const IS_FINALIZE: bool>(
        &mut self,
        path: impl AsRef<Path>,
        enrich_with_meta: EnrichOption
    ) -> Result<WiktionaryReaderStats, UnifiedTranslationError<WiktionaryReaderStats, WiktionaryError, std::io::Error>> {
        let reader = wiktionary_reader::read_wiktionary(path)?;
        let mut errors = Vec::new();
        let mut ct = WiktionaryReaderStats::default();
        for entry in reader {
            match entry {
                Ok(Either::Right(value)) => {
                    match value.lang.parse::<Language>() {
                        Ok(lang) if self.dir.contains(&lang) => {
                            if let Some(other_lang) = value.translations.iter()
                                .filter_map(|value| value.lang.parse::<Language>().ok())
                                .filter(|value| lang.ne(value) && self.dir.contains(value))
                                .next()
                            {
                                if IS_FINALIZE {
                                    continue
                                }
                                match self.process_wikidata_entry(value, lang.to(other_lang)) {
                                    Ok(value) => {
                                        ct.success += value;
                                    }
                                    Err(value) => {
                                        errors.push(value);
                                    }
                                }
                            } else {
                                if self.dir.lang_a() == lang {
                                    ct.no_lang_b += 1;
                                } else {
                                    ct.no_lang_a += 1;
                                }

                                if IS_FINALIZE || matches!(enrich_with_meta, EnrichOption::All) {
                                    match self.register_metadata(value, lang) {
                                        Err(value) => {
                                            errors.push(value);
                                        }
                                        _ => {}
                                    }
                                }

                            }
                        }
                        _ => {
                            ct.not_lang_a_or_b += 1;
                        }
                    }
                }
                Err(err) => {
                    errors.push(err.into());
                }
                _ => {
                    ct.redirects += 1;
                }
            }
        }
        if errors.is_empty() {
            Ok(ct)
        } else {
            Err(UnifiedTranslationError::reader(ct, errors))
        }
    }

    fn register_metadata(
        &mut self,
        word: ExtractedWord,
        language: Language
    ) -> Result<(), WiktionaryError>{
        let ExtractedWordValues {
            main: (word, lang, mut content),
            forms,
            synonyms,
            antonyms: _,
            related:_,
            translations: _
        } = convert_entry_to_entries(word, std::slice::from_ref(&language))?;
        content.dictionary_name(Some(WIKTIONARY));
        content.extend_synonyms(synonyms.into_iter().map(|v| v.0));

        let base_meta = {
            let (_, mut meta) = if lang == self.dir.lang_a() {
                unsafe{
                    self.insert_pipeline::<A>(WIKTIONARY, &word)
                }
            } else {
                unsafe{
                    self.insert_pipeline::<B>(WIKTIONARY, &word)
                }
            };

            let build = content.build().unwrap();
            build.write_to(&mut meta);
            build
        };

        for (w, mut v) in forms {
            v.dictionary_name(Some(WIKTIONARY));
            v.update_with(&base_meta);
            let (_, mut meta) = if lang == self.dir.lang_a() {
                unsafe{
                    self.insert_pipeline::<A>(WIKTIONARY, &w)
                }
            } else {
                unsafe{
                    self.insert_pipeline::<B>(WIKTIONARY, &w)
                }
            };
            v.build_consuming().unwrap().write_into(&mut meta);
        }

        Ok(())
    }

    fn process_wikidata_entry(&mut self, word: ExtractedWord, recognized: LanguageDirection) -> Result<usize, WiktionaryError> {
        let ExtractedWordValues {
            main: (word, lang, mut content),
            forms,
            synonyms,
            antonyms: _,
            related:_,
            translations
        } = convert_entry_to_entries(word, recognized.as_ref())?;
        content.dictionary_name(Some(WIKTIONARY));
        content.extend_synonyms(synonyms.into_iter().map(|v| v.0));

        let mut lang_a_variants = Vec::new();

        let base_meta = {
            let (word, mut meta) = if lang == self.dir.lang_a() {
                unsafe{
                    self.insert_pipeline::<A>(WIKTIONARY, &word)
                }
            } else {
                unsafe{
                    self.insert_pipeline::<B>(WIKTIONARY, &word)
                }
            };

            lang_a_variants.push(word);

            let build = content.build().unwrap();
            build.write_to(&mut meta);
            build
        };

        for (w, mut v) in forms {
            v.dictionary_name(Some(WIKTIONARY));
            v.update_with(&base_meta);
            let (word, mut meta) = if lang == self.dir.lang_a() {
                unsafe{
                    self.insert_pipeline::<A>(WIKTIONARY, &w)
                }
            } else {
                unsafe{
                    self.insert_pipeline::<B>(WIKTIONARY, &w)
                }
            };
            v.build_consuming().unwrap().write_into(&mut meta);
            lang_a_variants.push(word);
        }


        let mut lang_b_variants = Vec::new();
        for (word, lang_b, mut data) in translations {
            data.dictionary_name(Some(WIKTIONARY));
            let data = data.build_consuming().unwrap();
            if lang_b == lang {
                // todo: Do we want to transfer the meta to the translations?
                let (word, mut meta) = if lang == self.dir.lang_a() {
                    unsafe{ self.insert_pipeline::<A>(WIKTIONARY, &word) }
                } else {
                    unsafe{ self.insert_pipeline::<B>(WIKTIONARY, &word) }
                };
                data.write_into(&mut meta);
                lang_a_variants.push(word);
            } else {
                // todo: Do we want to transfer the meta to the translations?
                let (word, mut meta) = if lang == self.dir.lang_a() {
                    unsafe{ self.insert_pipeline::<B>(WIKTIONARY, &word) }
                } else {
                    unsafe{ self.insert_pipeline::<A>(WIKTIONARY, &word) }
                };
                data.write_into(&mut meta);
                lang_b_variants.push(word);
            }
        }

        let mut ct = 0;
        if lang == self.dir.lang_a() {
            for (a, b) in lang_a_variants.into_iter().cartesian_product(lang_b_variants) {
                ct += 1;
                unsafe {
                    self.dictionary.insert_raw_values::<Invariant>(a, b);
                }
            }
        } else {
            for (a, b) in lang_a_variants.into_iter().cartesian_product(lang_b_variants) {
                ct += 1;
                unsafe {
                    self.dictionary.insert_raw_values::<Invariant>(b, a);
                }
            }
        }
        Ok(ct)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct WiktionaryReaderStats {
    success: usize,
    redirects: usize,
    not_lang_a_or_b: usize,
    no_lang_a: usize,
    no_lang_b: usize
}

#[derive(Debug, Error)]
pub enum WiktionaryError {
    #[error(transparent)]
    WiktionaryReaderError(#[from] WiktionaryReaderError),
    #[error(transparent)]
    Conversion(#[from] EntryConversionError)
}


#[derive(Debug, Error)]
pub enum UnifiedTranslationError<P: Debug, E: Error, InitError: Error = E> {
    #[error("Expected any combination of ({ea}, {eb}) got ({aa}, {ab})", ea = expected.lang_a(), eb = expected.lang_b(), aa = actual.lang_a(), ab = actual.lang_b())]
    IllegalLanguageDirection {
        expected: LanguageDirection,
        actual: LanguageDirection
    },
    #[error("Failed the initialisation with: {0:?}")]
    InitialisationError(#[from] InitError),
    #[error("Failed {n} times, returned the payload {payload:?}", n = errors.len())]
    Reader {
        payload: P,
        errors: Vec<E>
    },
}

impl<P: Debug, E: Error, InitError: Error> UnifiedTranslationError<P, E, InitError> {
    pub fn reader(payload: P, errors: Vec<E>) -> Self {
        Self::Reader {payload, errors}
    }
}

#[derive(Error, Debug)]
pub enum IateError {
    #[error("Encountered unexpected language {0}")]
    IllegalLanguage(Language),
    #[error(transparent)]
    IateReader(#[from] iate_reader::IateReaderError),
}

#[derive(Error, Debug)]
pub enum MsTermsError {
    #[error("Encountered unexpected language {0}")]
    IllegalLanguage(Language),
    #[error(transparent)]
    MsTerms(#[from] ms_terms_reader::MSTermsReaderError),
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
    use crate::topicmodel::dictionary::word_infos::LanguageDirection;
    use crate::topicmodel::dictionary::{EnrichOption, LoadInstruction, UnifiedTranslationHelper};
    use either::Either;
    use rayon::prelude::*;
    use crate::topicmodel::dictionary::DictionaryKind::*;
    use crate::topicmodel::dictionary::io::{WriteMode, WriteableDictionary};

    static TARGETS: std::sync::LazyLock<Vec<LoadInstruction<&str>>> = std::sync::LazyLock::new(||vec![
        LoadInstruction {
            direction: LanguageDirection::EN_DE,
            kind: FreeDict,
            paths: Either::Left("dictionaries/freedict/freedict-eng-deu-1.9-fd1.src/eng-deu/eng-deu.tei"),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::DE_EN,
            kind: FreeDict,
            paths: Either::Left("dictionaries/freedict/freedict-deu-eng-1.9-fd1.src/deu-eng/deu-eng.tei"),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::DE_EN,
            kind: DictCC,
            paths: Either::Left("dictionaries/DictCC/dict.txt"),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::DE_EN,
            kind: Ding,
            paths: Either::Left("dictionaries/ding/de-en.txt"),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::EN_DE,
            kind: IATE,
            paths: Either::Left("dictionaries/IATE/IATE_export.tbx"),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::EN_DE,
            kind: Omega,
            paths: Either::Left("dictionaries/dicts.info/OmegaWiki.txt"),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::EN_DE,
            kind: MSTerms,
            paths: Either::Right(
                vec![
                    "dictionaries/Microsoft TermCollection/MicrosoftTermCollectio_british_englisch.tbx",
                    "dictionaries/Microsoft TermCollection/MicrosoftTermCollection_german.tbx"
                ]
            ),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::EN_DE,
            kind: Muse,
            paths: Either::Left("dictionaries/MUSE/dictionaries.tar.gz"),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::DE_EN,
            kind: Muse,
            paths: Either::Left("dictionaries/MUSE/dictionaries.tar.gz"),
            enrich_with_meta: EnrichOption::Off
        },
        LoadInstruction {
            direction: LanguageDirection::EN_DE,
            kind: Wiktionary,
            paths: Either::Left("dictionaries/Wiktionary/raw-wiktextract-data.jsonl.gz"),
            enrich_with_meta: EnrichOption::OnFinalize
        },
        LoadInstruction {
            direction: LanguageDirection::DE_EN,
            kind: Wiktionary,
            paths: Either::Left("dictionaries/Wiktionary/de-extract.jsonl.gz"),
            enrich_with_meta: EnrichOption::OnFinalize
        }
    ]);

    #[test]
    pub fn test(){
        env_logger::init();

        let result = TARGETS.iter().cloned().enumerate().par_bridge().map(|(i, value)| {
            let mut default = UnifiedTranslationHelper::new(LanguageDirection::EN_DE);
            match default.read_by_instruction(&value) {
                Ok(_) => {
                    log::info!("Finished: {}", value.kind);
                }
                Err(err) => {
                    log::info!("####\nHad an error while reading {}:\n{err}\n####", value.kind);
                }
            }
            let data = match default.finalize() {
                Ok(value) => {
                    value
                }
                Err((value, _)) => {
                    value
                }
            };
            data.write_to_path(
                WriteMode::json(false, true),
                format!("dictionary_{}_{}.json", value.kind, i)
            )
        }).collect::<Vec<_>>();

        for value in result {
            println!("{value:?}");
        }
    }

    #[test]
    pub fn test2(){
        let mut default = UnifiedTranslationHelper::new(LanguageDirection::EN_DE);
        match default.read_all_by_instruction(&TARGETS) {
            Ok(_) => {}
            Err(err) => {
                for err in err {
                    log::info!("####\nHad an error:\n{err}\n####");
                }
            }
        }
        let data = match default.finalize() {
            Ok(value) => {
                value
            }
            Err((value, _)) => {
                value
            }
        };
        data.write_to_path(
            WriteMode::binary(true),
            "./dictionary",
        ).unwrap();
    }


}