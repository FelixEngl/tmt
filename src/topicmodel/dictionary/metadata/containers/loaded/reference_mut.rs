use std::ops::{Deref, DerefMut};
use crate::toolkit::typesafe_interner::DefaultDictionaryOrigin;
use crate::topicmodel::dictionary::direction::Language;
use crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager;
use crate::topicmodel::dictionary::metadata::containers::loaded::metadata::LoadedMetadata;
use crate::topicmodel::dictionary::metadata::{MetadataManager, MetadataMutReference};
use crate::topicmodel::dictionary::word_infos::{Domain, GrammaticalGender, GrammaticalNumber, Language as TargLang, PartOfSpeech, Register};

pub struct LoadedMetadataMutRef<'a> {
    pub(in crate::topicmodel::dictionary) meta: &'a mut LoadedMetadata,
    // always outlifes meta
    manager_ref: *mut LoadedMetadataManager
}

impl<'a> Deref for LoadedMetadataMutRef<'a> {
    type Target = LoadedMetadata;

    fn deref(&self) -> &Self::Target {
        self.meta
    }
}

impl<'a> DerefMut for LoadedMetadataMutRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.meta
    }
}

impl<'a> MetadataMutReference<'a, LoadedMetadataManager> for LoadedMetadataMutRef<'a> {
    fn update_with<'b, L: Language>(&mut self, associated: <LoadedMetadataManager as MetadataManager>::Reference<'b>) {
        self.meta.update_with(associated.raw)
    }

    fn raw_mut<'b: 'a>(&'b mut self) -> &'a mut <LoadedMetadataManager as MetadataManager>::Metadata {
        self.meta
    }

    fn meta_container_mut<'b: 'a>(&'b self) -> &'a mut LoadedMetadataManager {
        unsafe { &mut *self.manager_ref }
    }
}

macro_rules! impl_adders {
    (inderned: $($ident: ident),+) => {
        impl<'a> LoadedMetadataMutRef<'a> {
        $(
            paste::paste! {
                pub fn [<add_single_to_ $ident _default>](&mut self, value: impl AsRef<str>) {
                    let interned = unsafe { &mut *self.manager_ref }.[<intern_ $ident>](value);
                    self.meta
                        .get_mut_general_metadata()
                        .[<add_single_to_ $ident>](interned)
                }

                pub fn [<add_single_to_ $ident _by_dict>](&mut self, dictionary_name: DefaultDictionaryOrigin, value: impl AsRef<str>) {
                    let interned = unsafe { &mut *self.manager_ref }.[<intern_ $ident>](value);
                    self.meta.get_or_create(dictionary_name).[<add_single_to_ $ident>](interned)
                }

                pub fn [<add_single_to_ $ident>](&mut self, dictionary_name: impl AsRef<str>, value: impl AsRef<str>) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_single_to_ $ident _by_dict>](name, value)
                }

                pub fn [<add_all_to_ $ident _default>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, values: I) {
                    let converted = values.into_iter().map(|value| self.meta_container_mut().[<intern_ $ident>](value)).collect::<Vec<_>>();
                    self.meta
                        .get_mut_general_metadata()
                        .[<add_all_to_ $ident>](converted)
                }

                pub fn [<add_all_to_ $ident _by_dict>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, dictionary_name: DefaultDictionaryOrigin, values: I) {
                    let converted = values.into_iter().map(|value| self.meta_container_mut().[<intern_ $ident>](value)).collect::<Vec<_>>();
                    self.meta.get_or_create(dictionary_name)
                        .[<add_all_to_ $ident>](converted)
                }

                pub fn [<add_all_to_ $ident>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, dictionary_name: impl AsRef<str>, values: I) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_all_to_ $ident _by_dict>](name, values)
                }
            }
            )+
        }
    };

    (typed: $($ident: ident: $ty:ty),+) => {
        impl<'a> LoadedMetadataMutRef<'a> {
        $(
            paste::paste! {
                pub fn [<add_single_to_ $ident _default>](&mut self, value: $ty) {
                    self.meta
                        .get_mut_general_metadata()
                        .[<add_single_to_ $ident>](value)
                }

                pub fn [<add_single_to_ $ident _by_dict>](&mut self, dictionary_name: DefaultDictionaryOrigin, value: $ty) {
                    self.meta.get_or_create(dictionary_name).[<add_single_to_ $ident>](value)
                }

                pub fn [<add_single_to_ $ident>](&mut self, dictionary_name: impl AsRef<str>, value: $ty) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_single_to_ $ident _by_dict>](name, value)
                }

                pub fn [<add_all_to_ $ident _default>]<I: IntoIterator<Item=$ty>>(&mut self, values: I) {
                    self.meta
                        .get_mut_general_metadata()
                        .[<add_all_to_ $ident>](values)
                }

                pub fn [<add_all_to_ $ident _by_dict>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: DefaultDictionaryOrigin, values: I) {
                    self.meta.get_or_create(dictionary_name)
                        .[<add_all_to_ $ident>](values)
                }

                pub fn [<add_all_to_ $ident>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: impl AsRef<str>, values: I) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_all_to_ $ident _by_dict>](name, values)
                }
            }
        )+
    }
    }
}

impl_adders! {
    inderned:
    inflected,
    abbreviations,
    unaltered_vocabulary
}


impl_adders! {
    typed:
    languages: TargLang,
    domains: Domain,
    registers: Register,
    gender: GrammaticalGender,
    pos: PartOfSpeech,
    number: GrammaticalNumber
}

impl<'a> LoadedMetadataMutRef<'a> {
    pub(in crate::topicmodel::dictionary) fn new(dict_ref: *mut LoadedMetadataManager, meta: &'a mut LoadedMetadata) -> Self {
        Self { meta, manager_ref: dict_ref }
    }

    pub fn add_dictionary_static(&mut self, name: &'static str) -> DefaultDictionaryOrigin {
        self.meta_container_mut().intern_dictionary_origin_static(name)
    }

    pub fn add_dictionary(&mut self, name: impl AsRef<str>) -> DefaultDictionaryOrigin {
        self.meta_container_mut().intern_dictionary_origin(name)
    }
}