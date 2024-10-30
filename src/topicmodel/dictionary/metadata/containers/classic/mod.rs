mod reference;
mod metadata;
mod reference_mut;
pub mod python;

pub use reference::{
    ClassicMetadataRef
};
pub use metadata::{
    ClassicMetadata
};
pub use reference_mut::{
    ClassicMetadataMutRef
};

use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::direction::{Language, A, B};
use crate::toolkit::typesafe_interner::{DefaultDictionaryOriginStringInterner, DefaultTagStringInterner};
use crate::topicmodel::dictionary::metadata::classic::python::SolvedMetadata;
use crate::topicmodel::dictionary::metadata::containers::MetadataManager;
use crate::topicmodel::vocabulary::Vocabulary;

/// Contains the metadata for the dictionary
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ClassicMetadataManager {
    pub(in crate::topicmodel::dictionary) meta_a: Vec<ClassicMetadata>,
    pub(in crate::topicmodel::dictionary) meta_b: Vec<ClassicMetadata>,
    pub(in crate::topicmodel::dictionary) dictionary_interner: DefaultDictionaryOriginStringInterner,
    #[serde(alias = "tag_interner")]
    pub(in crate::topicmodel::dictionary) subject_interner: DefaultTagStringInterner,
    pub(in crate::topicmodel::dictionary) unstemmed_voc: Vocabulary<String>,
}

impl Default for ClassicMetadataManager {
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

impl MetadataManager for ClassicMetadataManager {
    type Metadata = ClassicMetadata;
    type ResolvedMetadata = SolvedMetadata;
    type Reference<'a> = ClassicMetadataRef<'a>;
    type MutReference<'a> = ClassicMetadataMutRef<'a>;

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
            subject_interner: self.subject_interner,
            unstemmed_voc: self.unstemmed_voc,
            dictionary_interner: self.dictionary_interner
        }
    }

    fn get_meta<L: Language>(&self, word_id: usize) -> Option<&ClassicMetadata> {
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
        Some(ClassicMetadataMutRef::new(ptr, result))
    }


    fn get_or_create_meta<'a, L: Language>(&'a mut self, word_id: usize) -> Self::MutReference<'a> {
        let ptr = self as *mut Self;

        let targ = if L::LANG.is_a() {
            &mut self.meta_a
        } else {
            &mut self.meta_b
        };

        if word_id >= targ.len() {
            for _ in 0..(word_id - targ.len()) + 1 {
                targ.push(ClassicMetadata::default())
            }
        }

        unsafe { ClassicMetadataMutRef::new(ptr, targ.get_unchecked_mut(word_id)) }
    }

    fn get_meta_ref<'a, L: Language>(&'a self, word_id: usize) -> Option<Self::Reference<'a>> {
        Some(ClassicMetadataRef::new(self.get_meta::<L>(word_id)?, self))
    }

    fn resize(&mut self, meta_a: usize, meta_b: usize){
        if meta_a > self.meta_a.len() {
            self.meta_a.resize(meta_a, ClassicMetadata::default());
        }

        if meta_b > self.meta_a.len() {
            self.meta_b.resize(meta_b, ClassicMetadata::default());
        }
    }

    fn copy_keep_vocabulary(&self) -> Self {
        Self {
            dictionary_interner: self.dictionary_interner.clone(),
            subject_interner: self.subject_interner.clone(),
            unstemmed_voc: self.unstemmed_voc.clone(),
            meta_b: Default::default(),
            meta_a: Default::default(),
        }
    }
}

impl ClassicMetadataManager {

    pub fn new() -> Self {
        Self{
            meta_a: Default::default(),
            meta_b: Default::default(),
            dictionary_interner: DefaultDictionaryOriginStringInterner::new(),
            subject_interner: DefaultTagStringInterner::new(),
            unstemmed_voc: Default::default()
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
        self.get_or_create_meta::<L>(word_id).push_associated_dictionary(dict)
    }

    pub fn set_dictionaries_for<L: Language>(&mut self, word_id: usize, dicts: &[impl AsRef<str>]) {
        for dict in dicts {
            self.set_dictionary_for::<L>(word_id, dict.as_ref())
        }
    }

    pub fn set_subject_for<L: Language>(&mut self, word_id: usize, tag: &str) {
        self.get_or_create_meta::<L>(word_id).push_subject(tag)
    }

    pub fn set_subjects_for<L: Language>(&mut self, word_id: usize, tags: &[impl AsRef<str>]) {
        for tag in tags {
            self.set_subject_for::<L>(word_id, tag.as_ref())
        }
    }

    pub fn set_unstemmed_word_for<L: Language>(&mut self, word_id: usize, unstemmed: impl AsRef<str>) {
        self.get_or_create_meta::<L>(word_id).push_unstemmed(unstemmed)
    }

    pub fn set_unstemmed_words_for<L: Language>(&mut self, word_id: usize, unstemmed: &[impl AsRef<str>]) {
        for word in unstemmed {
            self.set_unstemmed_word_for::<L>(word_id, word)
        }
    }

    pub fn set_unstemmed_word_origin<L: Language>(&mut self, word_id: usize, unstemmed: &str, origin: &str) {
        let mut meta =  self.get_or_create_meta::<L>(word_id);
        meta.push_unstemmed_with_origin(unstemmed, origin);
    }

    pub fn set_unstemmed_words_origins_for<L: Language>(&mut self, word_id: usize, unstemmed: &str, origins: &[impl AsRef<str>]) {
        let mut meta =  self.get_or_create_meta::<L>(word_id);
        meta.push_unstemmed_with_origins(unstemmed, origins);
    }

}

impl Clone for ClassicMetadataManager {
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

impl Display for ClassicMetadataManager {
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::topicmodel::dictionary::direction::{A, B};
    use crate::topicmodel::dictionary::metadata::classic::ClassicMetadataManager;
    use crate::topicmodel::dictionary::metadata::classic::python::SolvedMetadata;
    use crate::topicmodel::dictionary::metadata::MetadataManager;

    #[test]
    fn test_if_it_works(){
        let mut container = ClassicMetadataManager::default();
        container.set_dictionary_for::<A>(0, "dict0");
        container.set_dictionary_for::<B>(0, "dict3");
        container.set_unstemmed_word_for::<A>(0, "test_word");
        container.set_unstemmed_word_origin::<A>(0, "test_word", "dict1");
        container.set_subject_for::<A>(0, "geo");
        let data_a = container.get_meta_ref::<A>(0).expect("There sould be something!");
        assert_eq!(SolvedMetadata::new(
            Some(vec!["dict0".to_string(), "dict1".to_string()]),
            Some(vec!["geo".to_string()]),
            Some(HashMap::from([("test_word".to_string(), vec!["dict1".to_string()])]))
        ) , SolvedMetadata::from(data_a));

        let data_b = container.get_meta_ref::<B>(0).expect("There sould be something!");
        assert_eq!(SolvedMetadata::new(
            Some(vec!["dict3".to_string()]),
            None,
            None
        ) , SolvedMetadata::from(data_b));

        let x = serde_json::to_string(&container).unwrap();
        let k: ClassicMetadataManager = serde_json::from_str(&x).unwrap();
        assert_eq!(container, k);
    }
}