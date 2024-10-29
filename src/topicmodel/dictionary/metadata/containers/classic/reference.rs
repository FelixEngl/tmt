use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use itertools::Itertools;
use crate::topicmodel::dictionary::metadata::containers::classic::metadata::ClassicMetadata;
use crate::topicmodel::dictionary::metadata::containers::classic::ClassicMetadataManager;
use crate::topicmodel::dictionary::metadata::{MetadataManager, MetadataReference};
use crate::topicmodel::vocabulary::BasicVocabulary;

/// Internally used for associating the [ClassicMetadataManager] with the [ClassicMetadata].
/// Stores the resolved values instead of the memory saving versions.
pub struct ClassicMetadataRef<'a> {
    pub(in super) raw: &'a ClassicMetadata,
    pub(in super) manager_ref: &'a ClassicMetadataManager,
    pub(in super) associated_dictionary_cached: Arc<OnceLock<Vec<&'a str>>>,
    pub(in super) subjects_cached: Arc<OnceLock<Vec<&'a str>>>,
    pub(in super) unstemmed_cached: Arc<OnceLock<Vec<(&'a str, Vec<&'a str>)>>>,
}

impl<'a> Deref for ClassicMetadataRef<'a> {
    type Target = ClassicMetadata;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}

impl<'a> MetadataReference<'a, ClassicMetadataManager> for ClassicMetadataRef<'a> {
    fn raw(&self) -> &'a ClassicMetadata {
        self.raw
    }

    fn meta_manager(&self) -> &'a ClassicMetadataManager {
        self.manager_ref
    }

    fn into_owned(self) -> ClassicMetadata {
        self.raw.clone()
    }

    fn into_solved(self) -> <ClassicMetadataManager as MetadataManager>::SolvedMetadata {
        self.into()
    }
}

impl<'a> ClassicMetadataRef<'a> {

    pub fn new(raw: &'a ClassicMetadata, metadata_container: &'a ClassicMetadataManager) -> Self {
        Self {
            raw,
            manager_ref: metadata_container,
            associated_dictionary_cached: Default::default(),
            subjects_cached: Default::default(),
            unstemmed_cached: Default::default()
        }
    }

    pub fn has_associated_dictionary(&self, q: impl AsRef<str>) -> bool {
        self.manager_ref.get_dictionary_interner().get(q).is_some_and(|value| self.raw.has_associated_dictionary(value))
    }

    pub fn has_subject(&self, q: impl AsRef<str>) -> bool {
        self.manager_ref.get_subject_interner().get(q).is_some_and(|value| self.raw.has_subject(value))
    }

    pub fn associated_dictionaries(&self) -> Option<&Vec<&'a str>> {
        if let Some(found) = self.associated_dictionary_cached.get() {
            Some(found)
        } else {
            if let Some(inner) = self.raw.associated_dictionaries.get() {
                let interner = self.manager_ref.get_dictionary_interner();
                self.associated_dictionary_cached.set(
                    inner.iter().map(|value| {
                        interner.resolve(value.clone()).expect("This should be known!")
                    }).collect()
                ).unwrap();
                self.associated_dictionary_cached.get()
            } else {
                None
            }
        }
    }

    pub fn subjects(&self) -> Option<&Vec<&'a str>> {
        if let Some(found) = self.subjects_cached.get() {
            Some(found)
        } else {
            if let Some(inner) = self.raw.subjects.get() {
                let interner = self.manager_ref.get_subject_interner();
                self.subjects_cached.set(
                    inner.iter().map(|value| {
                        interner.resolve(value.clone()).expect("This should be known!")
                    }).collect()
                ).unwrap();
                self.subjects_cached.get()
            } else {
                None
            }
        }
    }

    pub fn unstemmed(&self) -> Option<&Vec<(&'a str, Vec<&'a str>)>> {
        if let Some(found) = self.unstemmed_cached.get() {
            Some(found)
        } else {
            let inner = self.raw.unstemmed.get()?;
            let interner = self.manager_ref.get_dictionary_interner();
            let voc = self.manager_ref.get_unstemmed_voc();
            self.unstemmed_cached.set(
                inner.iter().map(|(k, v)| {
                    (voc.get_value(*k).unwrap().as_str(), v.iter().map(|value| interner.resolve(*value).unwrap()).collect_vec())
                }).collect_vec()
            ).unwrap();
            self.unstemmed_cached.get()
        }
    }
}


impl Debug for ClassicMetadataRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataRef")
            .field("inner", self.raw)
            .field("associated_dictionary_cached", &self.associated_dictionary_cached.get())
            .field("subjects_cached", &self.subjects_cached.get())
            .field("unstemmed_cached", &self.unstemmed_cached.get())
            .finish_non_exhaustive()
    }
}

impl<'a> Clone for ClassicMetadataRef<'a> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            manager_ref: self.manager_ref,
            associated_dictionary_cached: self.associated_dictionary_cached.clone(),
            subjects_cached: self.subjects_cached.clone(),
            unstemmed_cached: self.unstemmed_cached.clone()
        }
    }
}

impl Display for ClassicMetadataRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let a = match self.associated_dictionaries() {
            None => {
                "None".to_string()
            }
            Some(value) => {
                value.join(", ")
            }
        };

        let b = match self.subjects() {
            None => {
                "None".to_string()
            }
            Some(value) => {
                value.join(", ")
            }
        };

        let c = match self.unstemmed() {
            None => {
                "None".to_string()
            }
            Some(value) => {
                value.iter().map(|(k, v)| {
                    format!("{k} {{{}}}", v.join(", "))
                }).join(", ")
            }
        };
        write!(f, "MetadataRef{{[{a}], [{b}], [{c}]}}")
    }
}
