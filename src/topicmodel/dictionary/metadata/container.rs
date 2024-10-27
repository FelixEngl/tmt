use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::direction::{Language, A, B};
use crate::topicmodel::dictionary::metadata::{Metadata, MetadataMutRef, MetadataRef};
use crate::toolkit::typesafe_interner::{DefaultDictionaryOriginStringInterner, DefaultTagStringInterner};
use crate::topicmodel::vocabulary::Vocabulary;

/// Contains the metadata for the dictionary
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct MetadataContainer {
    pub(in crate::topicmodel::dictionary) meta_a: Vec<Metadata>,
    pub(in crate::topicmodel::dictionary) meta_b: Vec<Metadata>,
    pub(in crate::topicmodel::dictionary) dictionary_interner: DefaultDictionaryOriginStringInterner,
    #[serde(alias = "tag_interner")]
    pub(in crate::topicmodel::dictionary) subject_interner: DefaultTagStringInterner,
    pub(in crate::topicmodel::dictionary) unstemmed_voc: Vocabulary<String>,
}

impl Default for MetadataContainer {
    fn default() -> Self {
        Self {
            meta_a: Default::default(),
            meta_b: Default::default(),
            dictionary_interner: DefaultDictionaryOriginStringInterner::new(),
            subject_interner: DefaultTagStringInterner::new(),
            unstemmed_voc: Default::default(),
        }
    }
}

impl MetadataContainer {

    pub fn new() -> Self {
        Self{
            meta_a: Default::default(),
            meta_b: Default::default(),
            dictionary_interner: DefaultDictionaryOriginStringInterner::new(),
            subject_interner: DefaultTagStringInterner::new(),
            unstemmed_voc: Default::default()
        }
    }

    pub fn switch_languages(self) -> Self {
        Self {
            meta_a: self.meta_b,
            meta_b: self.meta_a,
            subject_interner: self.subject_interner,
            unstemmed_voc: self.unstemmed_voc,
            dictionary_interner: self.dictionary_interner
        }
    }

    pub fn get_dictionary_interner(&self) -> &DefaultDictionaryOriginStringInterner {
        &self.dictionary_interner
    }

    pub fn get_dictionary_interner_mut(&mut self) -> &mut DefaultDictionaryOriginStringInterner {
        &mut self.dictionary_interner
    }

    pub fn get_subject_interner(&self) -> &DefaultTagStringInterner {
        &self.subject_interner
    }

    pub fn get_tag_interner_mut(&mut self) -> &mut DefaultTagStringInterner {
        &mut self.subject_interner
    }

    pub fn get_unstemmed_voc(&self) -> &Vocabulary<String> {
        &self.unstemmed_voc
    }

    pub fn get_unstemmed_voc_mut(&mut self) -> &mut Vocabulary<String> {
        &mut self.unstemmed_voc
    }

    pub fn set_dictionary_for<L: Language>(&mut self, word_id: usize, dict: &str) {
        self.get_or_init_meta::<L>(word_id).push_associated_dictionary(dict)
    }

    pub fn set_dictionaries_for<L: Language>(&mut self, word_id: usize, dicts: &[impl AsRef<str>]) {
        for dict in dicts {
            self.set_dictionary_for::<L>(word_id, dict.as_ref())
        }
    }

    pub fn set_subject_for<L: Language>(&mut self, word_id: usize, tag: &str) {
        self.get_or_init_meta::<L>(word_id).push_subject(tag)
    }

    pub fn set_subjects_for<L: Language>(&mut self, word_id: usize, tags: &[impl AsRef<str>]) {
        for tag in tags {
            self.set_subject_for::<L>(word_id, tag.as_ref())
        }
    }

    pub fn set_unstemmed_word_for<L: Language>(&mut self, word_id: usize, unstemmed: impl AsRef<str>) {
        self.get_or_init_meta::<L>(word_id).push_unstemmed(unstemmed)
    }

    pub fn set_unstemmed_words_for<L: Language>(&mut self, word_id: usize, unstemmed: &[impl AsRef<str>]) {
        for word in unstemmed {
            self.set_unstemmed_word_for::<L>(word_id, word)
        }
    }

    pub fn set_unstemmed_word_origin<L: Language>(&mut self, word_id: usize, unstemmed: &str, origin: &str) {
        let mut meta =  self.get_or_init_meta::<L>(word_id);
        meta.push_unstemmed_with_origin(unstemmed, origin);
    }

    pub fn set_unstemmed_words_origins_for<L: Language>(&mut self, word_id: usize, unstemmed: &str, origins: &[impl AsRef<str>]) {
        let mut meta =  self.get_or_init_meta::<L>(word_id);
        meta.push_unstemmed_with_origins(unstemmed, origins);
    }

    pub fn get_meta<L: Language>(&self, word_id: usize) -> Option<&Metadata> {
        if L::LANG.is_a() {
            self.meta_a.get(word_id)
        } else {
            self.meta_b.get(word_id)
        }
    }

    pub fn get_meta_mut<L: Language>(&mut self, word_id: usize) -> Option<MetadataMutRef> {
        let ptr = self as *mut Self;
        let value = unsafe{&mut*ptr};
        let result = if L::LANG.is_a() {
            value.meta_a.get_mut(word_id)
        } else {
            value.meta_b.get_mut(word_id)
        }?;
        Some(MetadataMutRef::new(ptr, result))
    }


    pub fn get_or_init_meta<L: Language>(&mut self, word_id: usize) -> MetadataMutRef {
        let ptr = self as *mut Self;

        let targ = if L::LANG.is_a() {
            &mut self.meta_a
        } else {
            &mut self.meta_b
        };

        if word_id >= targ.len() {
            for _ in 0..(word_id - targ.len()) + 1 {
                targ.push(Metadata::default())
            }
        }

        unsafe { MetadataMutRef::new(ptr, targ.get_unchecked_mut(word_id)) }
    }

    pub fn get_meta_ref<L: Language>(&self, word_id: usize) -> Option<MetadataRef> {
        Some(MetadataRef::new(self.get_meta::<L>(word_id)?, self))
    }

    pub fn resize(&mut self, meta_a: usize, meta_b: usize){
        self.meta_a.resize(meta_a, Metadata::default());
        self.meta_b.resize(meta_b, Metadata::default());
    }

    pub fn copy_keep_vocebulary(&self) -> Self {
        Self {
            dictionary_interner: self.dictionary_interner.clone(),
            subject_interner: self.subject_interner.clone(),
            unstemmed_voc: self.unstemmed_voc.clone(),
            meta_b: Default::default(),
            meta_a: Default::default(),
        }
    }

}

impl Clone for MetadataContainer {
    fn clone(&self) -> Self {
        Self {
            meta_a: self.meta_a.clone(),
            meta_b: self.meta_b.clone(),
            dictionary_interner: self.dictionary_interner.clone(),
            subject_interner: self.subject_interner.clone(),
            unstemmed_voc: self.unstemmed_voc.clone(),
        }
    }
}

impl Display for MetadataContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metadata A:\n")?;
        if self.meta_a.is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in 0..self.meta_a.len() {
                if let Some(value) = self.get_meta_ref::<A>(word_id) {
                    write!(f, "    {}: {}\n", word_id, value)?;
                }
            }
        }

        write!(f, "\n------\n")?;
        write!(f, "Metadata B:\n")?;
        if self.meta_b.is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in 0..self.meta_b.len() {
                if let Some(value) = self.get_meta_ref::<B>(word_id) {
                    write!(f, "    {}: {}\n", word_id, value)?;
                }
            }
        }
        Ok(())
    }
}