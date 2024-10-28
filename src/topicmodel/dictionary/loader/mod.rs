use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock, RwLockWriteGuard};
use itertools::Itertools;
use crate::topicmodel::dictionary::loader::free_dict::{GramaticHints, Translation};
use crate::topicmodel::dictionary::word_infos::*;
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


#[derive(Debug, Copy, Clone)]
pub enum Origin {
    FreeDict
}

pub struct UnifiedTranslationHelper {
    voc: Arc<RwLock<Vocabulary<String>>>,
    words: Arc<RwLock<HashMap<HashRef<String>, Arc<WordWithMeta>>>>,
    translations: Arc<RwLock<HashMap<HashRef<String>, UnifiedTranslation>>>,
}

impl UnifiedTranslationHelper {

    fn register_word<T: AsRef<str>>(&self, s: T) -> HashRef<String> {
        let r = s.as_ref();
        if let Some(value) =  self.voc.read().unwrap().get_hash_ref(r) {
            return value.clone()
        }
        let mut write = self.voc.write().unwrap();
        let value = write.add(r.to_string());
        write.get_value(value).unwrap().clone()
    }

    fn convert_optional_gram(gram: Option<GramaticHints>) -> (Vec<PartOfSpeech>, Vec<GrammaticalGender>, Vec<GrammaticalNumber>) {
        if let Some(g) = gram {
            (g.pos, g.gender, g.number)
        } else {
            (Vec::with_capacity(0), Vec::with_capacity(0), Vec::with_capacity(0))
        }
    }

    fn get_word<T: AsRef<str>>(
        &self,
        word: T,
    ) -> Option<Arc<WordWithMeta>> {
        self.words.read().unwrap().get(&self.register_word(word)).cloned()
    }

    fn get_or_register_word<T: AsRef<str>>(
        &self,
        word: T,
        language: Language,
        languages: Vec<Language>,
        domains: Vec<Domain>,
        registers: Vec<Register>,
        gender: Vec<GrammaticalGender>,
        pos: Vec<PartOfSpeech>,
        number: Vec<GrammaticalNumber>,
        inflected: Vec<HashRef<String>>,
        abbreviations: Vec<HashRef<String>>
    ) -> Arc<WordWithMeta> {
        let word = self.register_word(word);
        let mut w = self.words.write().unwrap();
        match w.entry(word.clone()) {
            Entry::Occupied(value) => {
                let existing = value.get().clone();
                drop(w);
                {
                    let mut write_meta = existing.meta.write().unwrap();
                    write_meta.update_domains(domains);
                    write_meta.update_languages(languages);
                    write_meta.update_registers(registers);
                    write_meta.update_gender(gender);
                    write_meta.update_pos(pos);
                    write_meta.update_number(number);
                    write_meta.update_inflected(inflected);
                    write_meta.update_abbreviations(abbreviations);
                }
                existing
            }
            Entry::Vacant(value) => {
                let new = Arc::new(
                    WordWithMeta::new(
                        word,
                        language,
                        languages,
                        domains,
                        registers,
                        gender,
                        pos,
                        number,
                        inflected,
                        abbreviations,
                    )
                );
                value.insert(new.clone());
                new
            }
        }
    }

    fn register_translation(&self, word: Arc<WordWithMeta>, translations: Vec<Arc<WordWithMeta>>) -> UnifiedTranslation {
        let mut write = self.translations.write().unwrap();
        match write.entry(word.word.clone()) {
            Entry::Occupied(value) => {
                let value = value.get().clone();
                drop(write);
                value.update_all(translations);
                value
            }
            Entry::Vacant(value) => {
                value.insert(UnifiedTranslation{
                    word,
                    translations: Arc::new(RwLock::new(HashSet::from_iter(translations)))
                }).clone()
            }
        }


    }

    pub fn process_free_dict_entry(&self, entry: free_dict::FreeDictEntry, language: Language) -> UnifiedTranslation {
        let word = {
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

            let (pos, gender, number) =
                Self::convert_optional_gram(gram);

            self.get_or_register_word(
                orth,
                language,
                languages,
                domains,
                registers,
                gender,
                pos,
                number,
                inflected.into_iter().map(|value| self.register_word(value)).collect_vec(),
                abbrev.into_iter().map(|value| self.register_word(value)).collect_vec(),
            )
        };

        let translations = {
            let mut translations = Vec::new();

            for Translation {
                languages,
                word,
                domains,
                gram,
                lang,
                abbrevs,
                registers
            } in entry.translations {

                let (pos, gender, number) =
                    Self::convert_optional_gram(gram);
                translations.push(
                    self.get_or_register_word(
                        word,
                        lang.into(),
                        languages,
                        domains,
                        registers,
                        gender,
                        pos,
                        number,
                        Vec::with_capacity(0),
                        abbrevs.into_iter().map(|value| self.register_word(value)).collect_vec(),
                    )
                );
            }

            translations
        };

        self.register_translation(word, translations)
    }

    fn word_entries_to_words<U: AsRef<str>>(
        &self,
        lang: Language,
        dictcc::WordEntry(lang_cont): dictcc::WordEntry<U>,
        word_types: Option<&dictcc::WordTypes>,
        categories: Option<&dictcc::WordCategories<U>>
    ) {

    }

    pub fn process_dict_cc<T: AsRef<str>>(
        &self,
        lang_a: Language,
        lang_b: Language,
        dictcc::Entry(
            dictcc::WordEntry(lang_a_cont),
            dictcc::WordEntry(lang_b_cont),
            word_types,
            categories
        ): dictcc::Entry<T>
    ) {


    }
}

#[derive(Debug, Clone)]
pub struct UnifiedTranslation {
    word: Arc<WordWithMeta>,
    translations: Arc<RwLock<HashSet<Arc<WordWithMeta>>>>
}

impl UnifiedTranslation {
    pub fn update(&self, translation: Arc<WordWithMeta>) {
        self.translations.write().unwrap().insert(translation);
    }

    pub fn update_all(&self, translation: Vec<Arc<WordWithMeta>>) {
        self.translations.write().unwrap().extend(translation);
    }
}


#[derive(Debug, Clone)]
pub struct WordWithMeta {
    word: HashRef<String>,
    language: Language,
    meta: Arc<RwLock<Meta>>
}

impl Eq for WordWithMeta{}
impl PartialEq for WordWithMeta{
    fn eq(&self, other: &Self) -> bool {
        self.word.eq(&other.word)
    }
}

impl Hash for WordWithMeta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word.hash(state)
    }
}

impl WordWithMeta {
    pub fn new(
        word: HashRef<String>,
        language: Language,
        languages: Vec<Language>,
        domains: Vec<Domain>,
        registers: Vec<Register>,
        gender: Vec<GrammaticalGender>,
        pos: Vec<PartOfSpeech>,
        number: Vec<GrammaticalNumber>,
        inflected: Vec<HashRef<String>>,
        abbreviations: Vec<HashRef<String>>
    ) -> Self {
        Self {
            word,
            language,
            meta: Arc::new(
                RwLock::new(
                    Meta::new(
                        languages, domains, registers, gender, pos, number, inflected, abbreviations
                    )
                )
            )
        }
    }
}

#[derive(Debug)]
pub struct Meta {
    languages: Vec<Language>,
    domains: Vec<Domain>,
    registers: Vec<Register>,
    gender: Vec<GrammaticalGender>,
    pos: Vec<PartOfSpeech>,
    number: Vec<GrammaticalNumber>,
    inflected: Vec<HashRef<String>>,
    abbreviations: Vec<HashRef<String>>
}

macro_rules! impl_update {
    ($self: ident: $($targ: ident: $t: ty),+) => {
        $(
            paste::paste! {
                pub fn [<update_ $targ>](&mut $self, update: Vec<$t>) {
                    $self.$targ.extend(update);
                }

                pub fn [<update_single_ $targ>](&mut $self, update: $t) {
                    $self.$targ.push(update);
                }
            }
        )+
    };
}

impl Meta {
    pub fn new(languages: Vec<Language>, domains: Vec<Domain>, registers: Vec<Register>, gender: Vec<GrammaticalGender>, pos: Vec<PartOfSpeech>, number: Vec<GrammaticalNumber>, inflected: Vec<HashRef<String>>, abbreviations: Vec<HashRef<String>>) -> Self {
        Self { languages, domains, registers, gender, pos, number, inflected, abbreviations }
    }

    impl_update! {
        self:
        languages: Language,
        domains: Domain,
        registers: Register,
        gender: GrammaticalGender,
        pos: PartOfSpeech,
        number: GrammaticalNumber,
        inflected: HashRef<String>,
        abbreviations: HashRef<String>
    }
}

