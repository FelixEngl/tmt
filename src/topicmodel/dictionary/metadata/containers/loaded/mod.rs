pub mod metadata;
pub mod reference;
pub mod reference_mut;
pub mod python;

pub use metadata::*;
pub use reference::*;
pub use reference_mut::*;
pub use python::*;

use serde::{Deserialize, Serialize};
use crate::toolkit::typesafe_interner::{DefaultAbbreviation, DefaultAbbreviationStringInterner, DefaultDictionaryOrigin, DefaultDictionaryOriginStringInterner, DefaultInflected, DefaultInflectedStringInterner, DefaultSynonym, DefaultSynonymStringInterner, DefaultUnalteredVoc, DefaultUnalteredVocStringInterner};
use crate::topicmodel::dictionary::direction::Language;
use crate::topicmodel::dictionary::metadata::containers::loaded::metadata::LoadedMetadata;
use crate::topicmodel::dictionary::metadata::containers::loaded::reference::LoadedMetadataRef;
use crate::topicmodel::dictionary::metadata::containers::loaded::reference_mut::LoadedMetadataMutRef;
use crate::topicmodel::dictionary::metadata::loaded::python::SolvedLoadedMetadata;
use crate::topicmodel::dictionary::metadata::MetadataManager;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoadedMetadataManager {
    pub(in crate::topicmodel::dictionary) meta_a: Vec<LoadedMetadata>,
    pub(in crate::topicmodel::dictionary) meta_b: Vec<LoadedMetadata>,
    pub(in crate::topicmodel::dictionary) dictionary_interner: DefaultDictionaryOriginStringInterner,
    pub(in crate::topicmodel::dictionary) inflected_interner: DefaultInflectedStringInterner,
    pub(in crate::topicmodel::dictionary) abbrevitation_interner: DefaultAbbreviationStringInterner,
    pub(in crate::topicmodel::dictionary) unaltered_voc_interner: DefaultUnalteredVocStringInterner,
    pub(in crate::topicmodel::dictionary) synonyms_interner: DefaultSynonymStringInterner
}

impl LoadedMetadataManager {
    pub fn intern_dictionary_origin_static(&mut self, voc_entry: &'static str) -> DefaultDictionaryOrigin {
        self.dictionary_interner.get_or_intern_static(voc_entry)
    }
    pub fn intern_dictionary_origin(&mut self, voc_entry: impl AsRef<str>) -> DefaultDictionaryOrigin {
        self.dictionary_interner.get_or_intern(voc_entry)
    }
    pub fn intern_unaltered_vocabulary(&mut self, voc_entry: impl AsRef<str>) -> DefaultUnalteredVoc {
        self.unaltered_voc_interner.get_or_intern(voc_entry)
    }
    pub fn intern_inflected(&mut self, voc_entry: impl AsRef<str>) -> DefaultInflected {
        self.inflected_interner.get_or_intern(voc_entry)
    }
    pub fn intern_abbreviations(&mut self, voc_entry: impl AsRef<str>) -> DefaultAbbreviation {
        self.abbrevitation_interner.get_or_intern(voc_entry)
    }
    pub fn intern_synonyms(&mut self, voc_entry: impl AsRef<str>) -> DefaultSynonym {
        self.synonyms_interner.get_or_intern(voc_entry)
    }
}

impl Default for LoadedMetadataManager {
    fn default() -> Self {
        Self {
            meta_a: Vec::new(),
            meta_b: Vec::new(),
            dictionary_interner: DefaultDictionaryOriginStringInterner::new(),
            inflected_interner: DefaultInflectedStringInterner::new(),
            abbrevitation_interner: DefaultAbbreviationStringInterner::new(),
            unaltered_voc_interner: DefaultUnalteredVocStringInterner::new(),
            synonyms_interner: DefaultSynonymStringInterner::new(),
        }
    }
}

impl MetadataManager for LoadedMetadataManager {
    type Metadata = LoadedMetadata;
    type ResolvedMetadata = SolvedLoadedMetadata;
    type Reference<'a> = LoadedMetadataRef<'a> where Self: 'a;
    type MutReference<'a> = LoadedMetadataMutRef<'a> where Self: 'a;

    fn meta_a(&self) -> &[Self::Metadata] {
        self.meta_a.as_slice()
    }

    fn meta_b(&self) -> &[Self::Metadata] {
        self.meta_b.as_slice()
    }

    fn switch_languages(self) -> Self {
        Self {
            meta_a: self.meta_b,
            meta_b: self.meta_a,
            abbrevitation_interner: self.abbrevitation_interner,
            inflected_interner: self.inflected_interner,
            dictionary_interner: self.dictionary_interner,
            unaltered_voc_interner: self.unaltered_voc_interner,
            synonyms_interner: self.synonyms_interner,
        }
    }

    fn get_meta<L: Language>(&self, word_id: usize) -> Option<&Self::Metadata> {
        if L::LANG.is_a() {
            self.meta_a.get(word_id)
        } else {
            self.meta_b.get(word_id)
        }
    }

    fn get_meta_mut<'a, L: Language>(&'a mut self, word_id: usize) -> Option<Self::MutReference<'a>> {
        let ptr = self as *mut Self;
        let value = unsafe{&mut*ptr};
        let result = if L::LANG.is_a() {
            value.meta_a.get_mut(word_id)
        } else {
            value.meta_b.get_mut(word_id)
        }?;
        Some(LoadedMetadataMutRef::new(ptr, result))
    }

    fn get_or_create_meta<'a, L: Language>(&'a mut self, word_id: usize) -> Self::MutReference<'a> {
        let ptr = self as *mut Self;

        let targ = if L::LANG.is_a() {
            &mut self.meta_a
        } else {
            &mut self.meta_b
        };

        if word_id >= targ.len() {
            targ.resize(word_id + 1, LoadedMetadata::default())
        }

        unsafe{
            LoadedMetadataMutRef::new(ptr, targ.get_unchecked_mut(word_id))
        }
    }

    fn get_meta_ref<'a, L: Language>(&'a self, word_id: usize) -> Option<Self::Reference<'a>> {
        Some(LoadedMetadataRef::new(self.get_meta::<L>(word_id)?, self))
    }

    fn resize(&mut self, meta_a: usize, meta_b: usize) {
        if meta_a > self.meta_a.len() {
            self.meta_a.resize(meta_a, LoadedMetadata::default());
        }

        if meta_b > self.meta_a.len() {
            self.meta_b.resize(meta_b, LoadedMetadata::default());
        }
    }

    fn copy_keep_vocabulary(&self) -> Self {
        Self {
            abbrevitation_interner: self.abbrevitation_interner.clone(),
            inflected_interner: self.inflected_interner.clone(),
            dictionary_interner: self.dictionary_interner.clone(),
            meta_a: Default::default(),
            meta_b: Default::default(),
            unaltered_voc_interner: self.unaltered_voc_interner.clone(),
            synonyms_interner: self.synonyms_interner.clone()
        }
    }
}