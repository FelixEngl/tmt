use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use tinyset::Set64;
use crate::toolkit::typesafe_interner::{DefaultAbbreviation, DefaultInflected, DefaultSynonym, DefaultUnalteredVoc};
use crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager;
use crate::topicmodel::dictionary::metadata::containers::loaded::metadata::LoadedMetadata;
use crate::topicmodel::dictionary::metadata::{MetadataManager, MetadataReference};
use crate::topicmodel::dictionary::word_infos::*;

#[derive(Clone)]
pub struct LoadedMetadataRef<'a> {
    pub(in super) raw: &'a LoadedMetadata,
    pub(in super) manager_ref: &'a LoadedMetadataManager,
    pub(in super) synonyms_cache: Arc<OnceLock<(Set64<DefaultSynonym>, Vec<&'a str>)>>,
    pub(in super) unaltered_vocabulary_cache: Arc<OnceLock<(Set64<DefaultUnalteredVoc>, Vec<&'a str>)>>,
    pub(in super) inflected_cache: Arc<OnceLock<(Set64<DefaultInflected>, Vec<&'a str>)>>,
    pub(in super) abbreviation_cache: Arc<OnceLock<(Set64<DefaultAbbreviation>, Vec<&'a str>)>>,
    pub(in super) languages_cache: Arc<OnceLock<Set64<Language>>>,
    pub(in super) domains_cache: Arc<OnceLock<Set64<Domain>>>,
    pub(in super) registers_cache: Arc<OnceLock<Set64<Register>>>,
    pub(in super) gender_cache: Arc<OnceLock<Set64<GrammaticalGender>>>,
    pub(in super) pos_cache: Arc<OnceLock<Set64<PartOfSpeech>>>,
    pub(in super) number_cache: Arc<OnceLock<Set64<GrammaticalNumber>>>,
}

impl<'a> LoadedMetadataRef<'a> {
    pub fn new(raw: &'a LoadedMetadata, manager_ref: &'a LoadedMetadataManager) -> Self {
        Self {
            raw,
            manager_ref,
            synonyms_cache: Arc::new(OnceLock::new()),
            unaltered_vocabulary_cache: Arc::new(OnceLock::new()),
            inflected_cache: Arc::new(OnceLock::new()),
            abbreviation_cache: Arc::new(OnceLock::new()),
            languages_cache: Arc::new(OnceLock::new()),
            domains_cache: Arc::new(OnceLock::new()),
            registers_cache: Arc::new(OnceLock::new()),
            gender_cache: Arc::new(OnceLock::new()),
            pos_cache: Arc::new(OnceLock::new()),
            number_cache: Arc::new(OnceLock::new()),
        }
    }
}

impl<'a> Deref for LoadedMetadataRef<'a>  {
    type Target = LoadedMetadata;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl<'a> MetadataReference<'a, LoadedMetadataManager> for LoadedMetadataRef<'a> {
    fn raw(&self) -> &'a <LoadedMetadataManager as MetadataManager>::Metadata {
        self.raw
    }

    fn meta_manager(&self) -> &'a LoadedMetadataManager {
        self.manager_ref
    }

    fn into_owned(self) -> <LoadedMetadataManager as MetadataManager>::Metadata {
        self.raw.clone()
    }

    fn into_resolved(self) -> <LoadedMetadataManager as MetadataManager>::ResolvedMetadata {
        self.into()
    }
}



impl<'a> LoadedMetadataRef<'a> {

    pub fn get_unaltered_vocabulary_impl(&self) -> &(Set64<DefaultUnalteredVoc>, Vec<&'a str>) {
        self.unaltered_vocabulary_cache.get_or_init(|| {
            let set = self.raw.collect_all_unaltered_vocabulary();
            let mut resolved: Vec<&'a str> = Vec::with_capacity(set.len());
            for value in set.iter() {
                resolved.push(
                    self.manager_ref
                        .unaltered_voc_interner
                        .resolve(value)
                        .expect("Encountered an unknown inflection!")
                )
            }
            (set, resolved)
        })
    }
    pub fn get_synonyms(&self) -> &Vec<&'a str> {
        &self.get_synonyms_impl().1
    }

    pub fn get_synonyms_raw(&self) -> &Set64<DefaultUnalteredVoc> {
        &self.get_unaltered_vocabulary_impl().0
    }

    pub fn get_synonyms_impl(&self) -> &(Set64<DefaultSynonym>, Vec<&'a str>) {
        self.synonyms_cache.get_or_init(|| {
            let set = self.raw.collect_all_synonyms();
            let mut resolved: Vec<&'a str> = Vec::with_capacity(set.len());
            for value in set.iter() {
                resolved.push(
                    self.manager_ref
                        .synonyms_interner
                        .resolve(value)
                        .expect("Encountered an unknown inflection!")
                )
            }
            (set, resolved)
        })
    }
    pub fn get_unaltered_vocabulary(&self) -> &Vec<&'a str> {
        &self.get_unaltered_vocabulary_impl().1
    }

    pub fn get_unaltered_vocabulary_raw(&self) -> &Set64<DefaultUnalteredVoc> {
        &self.get_unaltered_vocabulary_impl().0
    }

    pub fn get_inflected_impl(&self) -> &(Set64<DefaultInflected>, Vec<&'a str>) {
        self.inflected_cache.get_or_init(|| {
            let set = self.raw.collect_all_inflected();
            let mut resolved: Vec<&'a str> = Vec::with_capacity(set.len());
            for value in set.iter() {
                resolved.push(
                    self.manager_ref
                        .inflected_interner
                        .resolve(value)
                        .expect("Encountered an unknown inflection!")
                )
            }
            (set, resolved)
        })
    }
    pub fn get_inflected(&self) -> &Vec<&'a str> {
        &self.get_inflected_impl().1
    }

    pub fn get_inflected_raw(&self) -> &Set64<DefaultInflected> {
        &self.get_inflected_impl().0
    }

    pub fn get_abbreviation_impl(&self) -> &(Set64<DefaultAbbreviation>, Vec<&'a str>) {
        self.abbreviation_cache.get_or_init(|| {
            let set = self.raw.collect_all_abbreviations();
            let mut resolved: Vec<&'a str> = Vec::with_capacity(set.len());
            for value in set.iter() {
                resolved.push(
                    self.manager_ref
                        .abbrevitation_interner
                        .resolve(value)
                        .expect("Encountered an unknown inflection!")
                )
            }
            (set, resolved)
        })
    }
    pub fn get_abbreviation(&self) -> &Vec<&'a str> {
        &self.get_abbreviation_impl().1
    }

    pub fn get_abbreviation_raw(&self) -> &Set64<DefaultAbbreviation> {
        &self.get_abbreviation_impl().0
    }
}

macro_rules! create_cached_getter {
    (no_postprocessing_needed: $($ident:ident: $ty:ty),+) => {
        impl<'a> LoadedMetadataRef<'a> {
            $(
                paste::paste! {
                    pub fn [<get_ $ident>](&self) -> &tinyset::Set64<$ty> {
                        self.[<$ident _cache>].get_or_init(|| self.raw.[<collect_all_ $ident>]())
                    }
                }
            )+
        }
    };
}

create_cached_getter! {
    no_postprocessing_needed:
    languages: Language,
    domains: Domain,
    registers: Register,
    gender: GrammaticalGender,
    pos: PartOfSpeech,
    number: GrammaticalNumber
}