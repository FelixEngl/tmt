use std::ops::{Deref, DerefMut};
use itertools::Itertools;
use crate::toolkit::typesafe_interner::DictionaryOriginSymbol;
use crate::topicmodel::dictionary::metadata::containers::classic::metadata::ClassicMetadata;
use crate::topicmodel::dictionary::metadata::containers::classic::ClassicMetadataManager;
use crate::topicmodel::dictionary::metadata::{MetadataManager, MetadataMutReference};
use crate::topicmodel::vocabulary::{SearchableVocabulary, VocabularyMut};

pub struct ClassicMetadataMutRef<'a> {
    pub(in crate::topicmodel::dictionary) meta: &'a mut ClassicMetadata,
    // always outlifes meta
    manager_ref: *mut ClassicMetadataManager
}


impl<'a> Deref for ClassicMetadataMutRef<'a> {
    type Target = ClassicMetadata;

    fn deref(&self) -> &Self::Target {
        self.meta
    }
}

impl<'a> DerefMut for ClassicMetadataMutRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.meta
    }
}

impl<'a> MetadataMutReference<'a, ClassicMetadataManager> for ClassicMetadataMutRef<'a> {
    fn update_with_reference<'b>(&mut self, associated: <ClassicMetadataManager as MetadataManager>::Reference<'b>) {
        let tags = associated.raw.subjects.get();
        let dics = associated.raw.associated_dictionaries.get();
        let unstemmed = associated.raw.unstemmed.get();

        if tags.is_none() && dics.is_none() {
            return;
        }

        if let Some(dics) = dics {
            unsafe { self.meta.add_all_associated_dictionaries(&dics) }
        }
        if let Some(tags) = tags {
            unsafe { self.meta.add_all_subjects(&tags) }
        }
        if let Some(unstemmed) = unstemmed {
            self.meta.add_all_unstemmed(unstemmed)
        }
    }

    fn update_with_resolved(&mut self, _: &<ClassicMetadataManager as MetadataManager>::ResolvedMetadata) -> Result<(), ()> {
        todo!()
    }

    fn raw_mut<'b: 'a>(&'b mut self) -> &'a mut <ClassicMetadataManager as MetadataManager>::Metadata {
        self.meta
    }

    fn meta_container_mut<'b: 'a>(&'b self) -> &'a mut ClassicMetadataManager {
        unsafe { &mut *self.manager_ref }
    }
}

impl<'a> ClassicMetadataMutRef<'a> {
    pub(in crate::topicmodel::dictionary) fn new(dict_ref: *mut ClassicMetadataManager, meta: &'a mut ClassicMetadata) -> Self {
        Self { meta, manager_ref: dict_ref }
    }

    pub fn push_associated_dictionary(&mut self, dictionary: impl AsRef<str>) {
        let interned = unsafe{&mut *self.manager_ref }.get_dictionary_interner_mut().get_or_intern(dictionary);
        unsafe {
            self.meta.add_associated_dictionary(interned);
        }
    }

    pub fn get_or_push_associated_dictionary(&mut self, dictionary: impl AsRef<str>) -> DictionaryOriginSymbol {
        let interned = unsafe{&mut *self.manager_ref }.get_dictionary_interner_mut().get_or_intern(dictionary);
        if self.meta.has_associated_dictionary(interned) {
            return interned
        }
        unsafe{self.meta.add_associated_dictionary(interned)};
        interned
    }

    pub fn push_subject(&mut self, tag: impl AsRef<str>) {
        let interned = unsafe{&mut *self.manager_ref }.get_tag_interner_mut().get_or_intern(tag);
        unsafe {
            self.meta.add_subject(interned);
        }
    }

    pub fn push_unstemmed(&mut self, word: impl AsRef<str>)  {
        let interned = unsafe{&mut *self.manager_ref }.get_unstemmed_voc_mut().add(word.as_ref());
        self.meta.add_unstemmed(interned);
    }


    pub fn get_or_push_unstemmed(&mut self, word: impl AsRef<str>) -> usize {
        let reference = unsafe{&mut *self.manager_ref }.get_unstemmed_voc_mut();
        let word = word.as_ref();
        match reference.get_id(word) {
            None => {
                let interned = reference.add(word.to_string());
                self.meta.add_unstemmed(interned);
                interned
            }
            Some(value) => {
                value
            }
        }
    }

    pub fn push_unstemmed_with_origin(&mut self, word: impl AsRef<str>, origin: impl AsRef<str>) {
        let word = self.get_or_push_unstemmed(word);
        let origin = self.get_or_push_associated_dictionary(origin);
        unsafe { self.meta.add_unstemmed_origin(word, origin) }
    }

    pub fn push_unstemmed_with_origins(&mut self, word: impl AsRef<str>, origins: &[impl AsRef<str>]) {
        let word = self.get_or_push_unstemmed(word);
        let origins = origins.iter().map(|value| self.get_or_push_associated_dictionary(value)).collect_vec();
        unsafe { self.meta.add_all_unstemmed_origins(word, &origins) }
    }
}