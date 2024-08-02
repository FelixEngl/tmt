use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};
use pyo3::{Bound, FromPyObject, IntoPy, pyclass, pymethods, PyObject, PyResult, Python};
use serde::{Deserialize, Deserializer, Serialize};
use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, DictionaryWithVocabulary};
use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};
use string_interner::{DefaultStringInterner, DefaultSymbol as InternedString, DefaultSymbol, Symbol};
use crate::topicmodel::dictionary::direction::{A, AToB, B, BToA, Language};
#[allow(unused_imports)]
use crate::toolkit::once_lock_serializer;
use itertools::Itertools;
use pyo3::prelude::{PyModule, PyModuleMethods};
use serde::de::{Error, Unexpected, Visitor};
use string_interner::symbol::SymbolU32;
use crate::topicmodel::dictionary::metadata::MetadataPyStateValues::{InternedVec, UnstemmedMapping};

/// Contains the metadata for the dictionary
#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq)]
pub struct MetadataContainer {
    pub(in crate::topicmodel::dictionary) meta_a: Vec<Metadata>,
    pub(in crate::topicmodel::dictionary) meta_b: Vec<Metadata>,
    pub(in crate::topicmodel::dictionary) dictionary_interner: DefaultStringInterner,
    #[serde(alias = "tag_interner")]
    pub(in crate::topicmodel::dictionary) subject_interner: DefaultStringInterner,
    pub(in crate::topicmodel::dictionary) unstemmed_voc: Vocabulary<String>,
}

impl MetadataContainer {

    pub fn new() -> Self {
        Self{
            meta_a: Default::default(),
            meta_b: Default::default(),
            dictionary_interner: Default::default(),
            subject_interner: Default::default(),
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

    pub fn get_dictionary_interner(&self) -> &DefaultStringInterner {
        &self.dictionary_interner
    }

    pub fn get_dictionary_interner_mut(&mut self) -> &mut DefaultStringInterner {
        &mut self.dictionary_interner
    }

    pub fn get_subject_interner(&self) -> &DefaultStringInterner {
        &self.subject_interner
    }

    pub fn get_tag_interner_mut(&mut self) -> &mut DefaultStringInterner {
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

#[derive(Debug, FromPyObject, Clone)]
pub enum MetadataPyStateValues {
    InternedVec(Vec<usize>),
    UnstemmedMapping(HashMap<usize, Vec<usize>>)
}

impl IntoPy<PyObject> for MetadataPyStateValues {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            InternedVec(value) => {
                value.into_py(py)
            }
            UnstemmedMapping(value) => {
                value.into_py(py)
            }
        }
    }
}

/// The container for the metadata
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq)]
pub struct Metadata {
    #[serde(with = "metadata_interned_field_serializer")]
    pub associated_dictionaries: OnceLock<Vec<InternedString>>,
    #[serde(with = "metadata_interned_field_serializer")]
    #[serde(alias = "meta_tags")]
    pub subjects: OnceLock<Vec<InternedString>>,
    #[serde(with = "metadata_unstemmed_serializer")]
    pub unstemmed: OnceLock<HashMap<usize, Vec<InternedString>>>,
}

macro_rules! create_methods {
    ($($self: ident.$target:ident($_type: ty) || $single_target: ident),+) => {
        $(
           paste::paste! {
            pub fn [<has_ $single_target>](&$self, q: $_type) -> bool {
                $self.$target.get().is_some_and(|value| value.contains(&q))
            }

            pub unsafe fn [<add_ $single_target>](&mut self, q: $_type) {
                if let Some(to_edit) = self.$target.get_mut() {
                    if to_edit.is_empty() || !to_edit.contains(&q) {
                        to_edit.push(q)
                    }
                } else {
                    let mut new = Vec::with_capacity(1);
                    new.push(q);
                    self.$target.set(new).expect("This should be unset!");
                }
            }

            pub unsafe fn [<add_all_ $target>](&mut self, q: &[$_type]) {
                if let Some(to_edit) = self.$target.get_mut() {
                    let mut set = std::collections::HashSet::with_capacity(q.len() + to_edit.len());
                    set.extend(to_edit.drain(..));
                    set.extend(q);
                    to_edit.extend(set);
                } else {
                    let mut new = Vec::with_capacity(q.len());
                    new.extend(q.into_iter().unique());
                    self.$target.set(new).expect("This should be unset!");
                }
            }
        }
        )+

    };
}

impl Metadata {
    create_methods! {
        self.associated_dictionaries(InternedString) || associated_dictionary,
        self.subjects(InternedString) || subject
    }


    pub fn add_all_unstemmed(&mut self, unstemmed_words: &HashMap<usize, Vec<DefaultSymbol>>) {
        if let Some(found) = self.unstemmed.get_mut() {
            for (word, v) in unstemmed_words.iter() {
                match found.entry(*word) {
                    Entry::Vacant(value) => {
                        value.insert(v.clone());
                    }
                    Entry::Occupied(mut value) => {
                        let mutable = value.get_mut();
                        let mut set = HashSet::with_capacity(mutable.len() + v.len());
                        set.extend(mutable.drain(..));
                        set.extend(v);
                        mutable.extend(set);
                    }
                }
            }
        } else {
            self.unstemmed.set(unstemmed_words.clone()).unwrap()
        }
    }

    pub fn has_unstemmed(&self, unstemmed_word: usize) -> bool {
        self.unstemmed
            .get()
            .is_some_and(|value| value.contains_key(&unstemmed_word))
    }

    pub fn add_unstemmed(&mut self, unstemmed_word: usize) {
        if let Some(found) = self.unstemmed.get_mut() {
            match found.entry(unstemmed_word) {
                Entry::Vacant(value) => {
                    value.insert(Vec::with_capacity(0));
                }
                _ => {}
            }
        } else {
            let mut new = HashMap::with_capacity(1);
            new.insert(unstemmed_word, Vec::<_>::with_capacity(0));
            self.unstemmed.set(new).unwrap();
        }
    }


    pub fn add_all_unstemmed_words(&mut self, unstemmed_words: &[usize]) {
        if let Some(found) = self.unstemmed.get_mut() {
            for word in unstemmed_words {
                match found.entry(*word) {
                    Entry::Vacant(value) => {
                        value.insert(Vec::with_capacity(0));
                    }
                    _ => {}
                }
            }

        } else {
            let mut new = HashMap::with_capacity(unstemmed_words.len());
            for word in unstemmed_words {
                new.insert(*word, Vec::with_capacity(0));
            }
            self.unstemmed.set(new).unwrap();
        }
    }

    pub fn has_unstemmed_origin(&self, unstemmed_word: usize, origin: DefaultSymbol) -> bool {
        self.unstemmed
            .get()
            .is_some_and(|value|
                value.get(&unstemmed_word).is_some_and(|value| value.contains(&origin))
            )
    }

    pub unsafe fn add_unstemmed_origin(&mut self, unstemmed_word: usize, origin: DefaultSymbol) {
        if let Some(found) = self.unstemmed.get_mut() {
            match found.entry(unstemmed_word) {
                Entry::Vacant(value) => {
                    let mut new = Vec::with_capacity(1);
                    new.push(origin);
                    value.insert(new);
                }
                Entry::Occupied(mut value) => {
                    let mutable = value.get_mut();
                    if !mutable.contains(&origin) {
                        mutable.push(origin);
                    }
                }
            }
        } else {
            let mut new = HashMap::with_capacity(1);
            let mut new_vec = Vec::with_capacity(1);
            new_vec.push(origin);
            new.insert(unstemmed_word, new_vec);
            self.unstemmed.set(new).unwrap();
        }
    }

    pub unsafe fn add_all_unstemmed_origins(&mut self, unstemmed_word: usize, origins: &[DefaultSymbol]) {
        if let Some(found) = self.unstemmed.get_mut() {
            match found.entry(unstemmed_word) {
                Entry::Vacant(value) => {
                    value.insert(origins.to_vec());
                }
                Entry::Occupied(mut value) => {
                    let inner = value.get_mut();
                    let mut set = std::collections::HashSet::with_capacity(origins.len() + inner.len());
                    set.extend(inner.drain(..));
                    set.extend(origins);
                    inner.extend(set);
                }
            }
        } else {
            let mut new = HashMap::with_capacity(1);
            new.insert(unstemmed_word, origins.to_vec());
            self.unstemmed.set(new).unwrap();
        }
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        if let Some(associated_dictionaries) = self.associated_dictionaries.get() {
            if let Some(other_associated_dictionaries) = other.associated_dictionaries.get() {
                if associated_dictionaries != other_associated_dictionaries {
                    return false;
                }
            } else {
                return false;
            }
        } else if other.associated_dictionaries.get().is_some() {
            return false;
        }

        if let Some(subjectsgs) = self.subjects.get() {
            if let Some(other_subjects) = other.subjects.get() {
                if subjectsgs != other_subjects {
                    return false;
                }
            } else {
                return false;
            }
        } else if other.subjects.get().is_some() {
            return false;
        }

        if let Some(unstemmed) = self.unstemmed.get() {
            if let Some(other_unstemmed) = other.unstemmed.get() {
                if unstemmed != other_unstemmed {
                    return false;
                }
            } else {
                return false;
            }
        } else if other.unstemmed.get().is_some() {
            return false;
        }

        return true;
    }
}

/// Helper for SymbolU32 serialisation
struct SymbolU32Visitor;

impl<'de> Visitor<'de> for SymbolU32Visitor {
    type Value = SymbolU32;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("The default symbols are between 0 and u32::MAX-1.")
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> where E: Error {
        match DefaultSymbol::try_from_usize(v as usize) {
            None => {
                Err(E::invalid_value(
                    Unexpected::Unsigned(v as u64),
                    &self
                ))
            }
            Some(value) => {
                Ok(value)
            }
        }
    }
}

/// Converter for a vec filled with [DefaultSymbol]
fn convert_vec_defaultsymbol(value: &Vec<DefaultSymbol>) -> Vec<usize> {
    value.iter().map(|value| value.to_usize()).collect_vec()
}

/// The inverse of [convert_vec_defaultsymbol]
fn convert_vec_usize<'de, D>(value: Vec<usize>) -> Result<Vec<DefaultSymbol>, D::Error> where D: Deserializer<'de>  {
    value.into_iter().map(|value|
        DefaultSymbol::try_from_usize(value)
            .ok_or_else(||
                Error::invalid_value(
                    Unexpected::Unsigned(value as u64),
                    &SymbolU32Visitor
                )
            )
    ).collect::<Result<Vec<_>, _>>()
}

mod metadata_interned_field_serializer {
    use std::sync::OnceLock;
    use serde::{Deserializer, Serialize, Serializer};
    use string_interner::{DefaultSymbol};
    use crate::toolkit::once_lock_serializer::{DeserializeOnceLock, SerializeOnceLock};
    use crate::topicmodel::dictionary::metadata::{convert_vec_defaultsymbol, convert_vec_usize};

    pub(crate) fn serialize<S>(target: &OnceLock<Vec<DefaultSymbol>>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let to_ser = if let Some(value) = target.get() {
            SerializeOnceLock::InitializedOwned(convert_vec_defaultsymbol(value))
        } else {
            SerializeOnceLock::Uninitialized
        };
        to_ser.serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<OnceLock<Vec<DefaultSymbol>>, D::Error> where D: Deserializer<'de> {
        let content: DeserializeOnceLock<Vec<usize>> = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(
            content.map(convert_vec_usize::<D>).transpose()?.into()
        )
    }
}

mod metadata_unstemmed_serializer {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    use serde::{Deserializer, Serializer, Serialize};
    use string_interner::{DefaultSymbol};
    use crate::toolkit::once_lock_serializer::{DeserializeOnceLock, SerializeOnceLock};
    use crate::topicmodel::dictionary::metadata::{convert_vec_defaultsymbol, convert_vec_usize};

    pub(crate) fn serialize<S>(target: &OnceLock<HashMap<usize, Vec<DefaultSymbol>>>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let to_ser = if let Some(value) = target.get() {
            SerializeOnceLock::InitializedOwned(
                value
                    .iter()
                    .map(|(k, v)| (*k, convert_vec_defaultsymbol(v)))
                    .collect::<HashMap<_, _>>()
            )
        } else {
            SerializeOnceLock::Uninitialized
        };
        to_ser.serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<OnceLock<HashMap<usize, Vec<DefaultSymbol>>>, D::Error> where D: Deserializer<'de> {
        let content: DeserializeOnceLock<HashMap<usize, Vec<usize>>> = serde::de::Deserialize::deserialize(deserializer)?;
        let value = content.map(|value| {
            value.into_iter().map(|(k, v)|
                match convert_vec_usize::<D>(v) {
                    Ok(value) => {
                        Ok((k, value))
                    }
                    Err(err) => {
                        Err(err)
                    }
                }
            ).collect::<Result<HashMap<_, _>, D::Error>>()
        }).transpose()?.map(|value| value.into());
        Ok(value.into())
    }
}

pub struct MetadataMutRef<'a> {
    pub(in crate::topicmodel::dictionary) meta: &'a mut Metadata,
    // always outlifes meta
    metadata_ref: *mut MetadataContainer
}

impl<'a> MetadataMutRef<'a> {
    fn new(dict_ref: *mut MetadataContainer, meta: &'a mut Metadata) -> Self {
        Self { meta, metadata_ref: dict_ref }
    }

    pub fn push_associated_dictionary(&mut self, dictionary: impl AsRef<str>) {
        let interned = unsafe{&mut *self.metadata_ref }.get_dictionary_interner_mut().get_or_intern(dictionary);
        unsafe {
            self.meta.add_associated_dictionary(interned);
        }
    }

    pub fn get_or_push_associated_dictionary(&mut self, dictionary: impl AsRef<str>) -> DefaultSymbol {
        let interned = unsafe{&mut *self.metadata_ref }.get_dictionary_interner_mut().get_or_intern(dictionary);
        if self.meta.has_associated_dictionary(interned) {
            return interned
        }
        unsafe{self.meta.add_associated_dictionary(interned)};
        return interned;
    }

    pub fn push_subject(&mut self, tag: impl AsRef<str>) {
        let interned = unsafe{&mut *self.metadata_ref }.get_tag_interner_mut().get_or_intern(tag);
        unsafe {
            self.meta.add_subject(interned);
        }
    }

    pub fn push_unstemmed(&mut self, word: impl AsRef<str>)  {
        let interned = unsafe{&mut *self.metadata_ref }.get_unstemmed_voc_mut().add(word.as_ref());
        self.meta.add_unstemmed(interned);
    }


    pub fn get_or_push_unstemmed(&mut self, word: impl AsRef<str>) -> usize {
        let reference = unsafe{&mut *self.metadata_ref }.get_unstemmed_voc_mut();
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

/// A completely memory save copy of some [Metadata]
#[derive(Debug, Clone, Eq, PartialEq)]
#[pyclass]
pub struct SolvedMetadata {
    associated_dictionaries: Option<Vec<String>>,
    subjects: Option<Vec<String>>,
    unstemmed: Option<HashMap<String, Vec<String>>>
}

impl SolvedMetadata {
    pub fn new(associated_dictionaries: Option<Vec<String>>, subjects: Option<Vec<String>>, unstemmed: Option<HashMap<String, Vec<String>>>) -> Self {
        Self { associated_dictionaries, subjects, unstemmed }
    }
}

#[pymethods]
impl SolvedMetadata {
    #[getter]
    pub fn associated_dictionaries(&self) -> Option<Vec<String>> {
        self.associated_dictionaries.clone()
    }

    #[getter]
    pub fn subjects(&self) -> Option<Vec<String>> {
        self.subjects.clone()
    }

    #[getter]
    pub fn unstemmed(&self) -> Option<HashMap<String, Vec<String>>> {
        self.unstemmed.clone()
    }

    pub fn __repr__(&self) -> String {
        self.to_string()
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

impl Display for SolvedMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Meta{{")?;
        match &self.associated_dictionaries {
            None => {
                write!(f, "associated_dictionaries=[], ")
            }
            Some(value) => {
                write!(f, "associated_dictionaries=[{}], ", value.join(", "))
            }
        }?;
        match &self.subjects {
            None => {
                write!(f, "subjects=[]")
            }
            Some(value) => {
                write!(f, "subjects=[{}]", value.join(", "))
            }
        }?;
        match &self.unstemmed {
            None => {
                write!(f, ", unstemmed=[]")
            }
            Some(value) => {
                write!(
                    f,
                    ", unstemmed=[{}]",
                    value.iter().map(|(k, v)| {
                        format!("({k}, {{{}}})", v.iter().join(", "))
                    }).join(", "))
            }
        }?;
        write!(f, "}}")
    }
}

impl<'a> From<MetadataRef<'a>> for SolvedMetadata {
    fn from(value: MetadataRef<'a>) -> Self {
        let associated_dictionaries: Option<Vec<String>> = value.associated_dictionaries().map(|value| value.iter().map(|value| value.to_string()).collect());
        let subjects: Option<Vec<String>> = value.subjects().map(|value| value.iter().map(|value| value.to_string()).collect());
        let unstemmed: Option<HashMap<String, Vec<String>>> = value.unstemmed().map(|value| value.iter().map(|(a, b)| (a.to_string(), b.iter().map(|v|v.to_string()).collect_vec())).collect());
        SolvedMetadata::new(
            associated_dictionaries,
            subjects,
            unstemmed
        )
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
            .field("meta_tags_cached", &self.subjects_cached.get())
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

pub(crate) fn register_py_metadata(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SolvedMetadata>()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::topicmodel::dictionary::direction::{A, B};
    use crate::topicmodel::dictionary::metadata::{MetadataContainer, SolvedMetadata};

    #[test]
    fn test_if_it_works(){
        let mut container = MetadataContainer::new();
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
        let k: MetadataContainer = serde_json::from_str(&x).unwrap();
        assert_eq!(container, k);
    }
}