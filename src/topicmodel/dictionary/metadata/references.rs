use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};
use itertools::Itertools;
use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, DictionaryWithVocabulary};
use crate::topicmodel::dictionary::direction::{AToB, BToA, A, B};
use crate::topicmodel::dictionary::metadata::container::MetadataContainer;
use crate::topicmodel::dictionary::metadata::Metadata;
use crate::topicmodel::vocabulary::BasicVocabulary;

pub struct MetadataContainerWithDict<'a, D, T, V> {
    dict: *const D,
    meta_data: &'a MetadataContainer,
    _voc_types: PhantomData<fn(T)->V>
}

impl<'a, D, T, V> MetadataContainerWithDict<'a, D, T, V> {
    pub fn new(
        dict: *const D,
        meta_data: &'a MetadataContainer,
    ) -> Self {
        Self {
            dict,
            meta_data,
            _voc_types: PhantomData
        }
    }

    #[inline(always)]
    pub fn dict(&self) -> &'a D {
        unsafe {&*self.dict}
    }
}

impl<'a, D, T, V> MetadataContainerWithDict<'a, D, T, V> where D: BasicDictionaryWithMeta + BasicDictionaryWithVocabulary<V> {
    pub fn wrap(target: &'a D) -> Self {
        let ptr = target as *const D;
        Self::new(
            ptr,
            target.metadata()
        )
    }
}

impl<D, T, V> Deref for MetadataContainerWithDict<'_, D, T, V> {
    type Target = MetadataContainer;

    fn deref(&self) -> &Self::Target {
        self.meta_data
    }
}

impl<D, T, V> Display for MetadataContainerWithDict<'_, D, T, V> where D: DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T>, T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metadata A:\n")?;
        if self.meta_a.is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in 0..self.meta_a.len() {
                if let Some(value) = self.get_meta_ref::<A>(word_id) {
                    write!(f, "    {}: {}\n", self.dict().id_to_word::<AToB>(word_id).unwrap(), value)?;
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
                    write!(f, "    {}: {}\n", self.dict().id_to_word::<BToA>(word_id).unwrap(), value)?;
                }
            }
        }

        Ok(())
    }
}


pub struct MetadataContainerWithDictMut<'a, D, T, V> {
    dict: *mut D,
    meta_data: &'a mut MetadataContainer,
    _voc_types: PhantomData<fn(T)->V>
}

impl<'a, D, T, V> MetadataContainerWithDictMut<'a, D, T, V> {
    pub fn new(
        dict: *mut D,
        meta_data: &'a mut MetadataContainer,
    ) -> Self {
        Self {
            dict,
            meta_data,
            _voc_types: PhantomData
        }
    }

    #[inline(always)]
    pub fn dict(&self) -> &'a mut D {
        unsafe {&mut *self.dict}
    }

    pub fn reserve_meta(&mut self) {

    }
}

impl<'a, D, T, V> MetadataContainerWithDictMut<'a, D, T, V> where D: BasicDictionaryWithMeta + BasicDictionaryWithVocabulary<V> {
    pub fn wrap(target: &'a mut D) -> Self {
        let ptr = target as *mut D;
        Self::new(
            ptr,
            target.metadata_mut()
        )
    }
}

impl<D, T, V> Deref for MetadataContainerWithDictMut<'_, D, T, V> {
    type Target = MetadataContainer;

    fn deref(&self) -> &Self::Target {
        self.meta_data
    }
}

impl<D, T, V> DerefMut for MetadataContainerWithDictMut<'_, D, T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.meta_data
    }
}


impl<D, T, V> Display for MetadataContainerWithDictMut<'_, D, T, V> where D: DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T>, T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metadata A:\n")?;
        if self.meta_a.is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in 0..self.meta_a.len() {
                if let Some(value) = self.get_meta_ref::<A>(word_id) {
                    write!(f, "    {}: {}\n", self.dict().id_to_word::<AToB>(word_id).unwrap(), value)?;
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
                    write!(f, "    {}: {}\n", self.dict().id_to_word::<BToA>(word_id).unwrap(), value)?;
                }
            }
        }

        Ok(())
    }
}


/// Internally used for associating the [MetadataContainer] with the [Metadata].
/// Stores the resolved values instead of the memory saving versions.
pub struct MetadataRef<'a> {
    pub(in super) raw: &'a Metadata,
    pub(in super) metadata_container: &'a MetadataContainer,
    pub(in super) associated_dictionary_cached: Arc<OnceLock<Vec<&'a str>>>,
    pub(in super) subjects_cached: Arc<OnceLock<Vec<&'a str>>>,
    pub(in super) unstemmed_cached: Arc<OnceLock<Vec<(&'a str, Vec<&'a str>)>>>,
}

impl<'a> MetadataRef<'a> {

    pub fn new(raw: &'a Metadata, metadata_container: &'a MetadataContainer) -> Self {
        Self {
            raw,
            metadata_container,
            associated_dictionary_cached: Default::default(),
            subjects_cached: Default::default(),
            unstemmed_cached: Default::default()
        }
    }

    pub fn raw(&self) -> &'a Metadata {
        self.raw
    }

    pub fn has_associated_dictionary(&self, q: impl AsRef<str>) -> bool {
        self.metadata_container.get_dictionary_interner().get(q).is_some_and(|value| self.raw.has_associated_dictionary(value))
    }

    pub fn has_subject(&self, q: impl AsRef<str>) -> bool {
        self.metadata_container.get_subject_interner().get(q).is_some_and(|value| self.raw.has_subject(value))
    }

    pub fn associated_dictionaries(&self) -> Option<&Vec<&'a str>> {
        if let Some(found) = self.associated_dictionary_cached.get() {
            Some(found)
        } else {
            if let Some(inner) = self.raw.associated_dictionaries.get() {
                let interner = self.metadata_container.get_dictionary_interner();
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
                let interner = self.metadata_container.get_subject_interner();
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
            let interner = self.metadata_container.get_dictionary_interner();
            let voc = self.metadata_container.get_unstemmed_voc();
            self.unstemmed_cached.set(
                inner.iter().map(|(k, v)| {
                    (voc.get_value(*k).unwrap().as_str(), v.iter().map(|value| interner.resolve(*value).unwrap()).collect_vec())
                }).collect_vec()
            ).unwrap();
            self.unstemmed_cached.get()
        }
    }

    pub fn cloned_metadata(self) -> Metadata {
        self.raw.clone()
    }
}


impl Debug for MetadataRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataRef")
            .field("inner", self.raw)
            .field("associated_dictionary_cached", &self.associated_dictionary_cached.get())
            .field("subjects_cached", &self.subjects_cached.get())
            .field("unstemmed_cached", &self.unstemmed_cached.get())
            .finish_non_exhaustive()
    }
}

impl<'a> Clone for MetadataRef<'a> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            metadata_container: self.metadata_container,
            associated_dictionary_cached: self.associated_dictionary_cached.clone(),
            subjects_cached: self.subjects_cached.clone(),
            unstemmed_cached: self.unstemmed_cached.clone()
        }
    }
}

impl Display for MetadataRef<'_> {
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
