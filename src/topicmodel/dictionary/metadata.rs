use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use once_cell::sync::OnceCell;
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, DictionaryWithVocabulary};
use crate::topicmodel::vocabulary::{Vocabulary, VocabularyImpl, VocabularyMut};
use string_interner::{DefaultStringInterner, DefaultSymbol as InternedString};
use crate::topicmodel::dictionary::direction::{A, AToB, B, BToA, Language};
#[allow(unused_imports)]
use crate::toolkit::once_cell_serializer;
use itertools::Itertools;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MetadataContainer {
    pub(in crate::topicmodel::dictionary) meta_a: Vec<Metadata>,
    pub(in crate::topicmodel::dictionary) meta_b: Vec<Metadata>,
    pub(in crate::topicmodel::dictionary) dictionary_interner: DefaultStringInterner,
    pub(in crate::topicmodel::dictionary) tag_interner: DefaultStringInterner,
    pub(in crate::topicmodel::dictionary) unstemmed_voc: VocabularyImpl<String>,
}

impl MetadataContainer {

    pub fn create() -> Self {
        Self{
            meta_a: Default::default(),
            meta_b: Default::default(),
            dictionary_interner: Default::default(),
            tag_interner: Default::default(),
            unstemmed_voc: Default::default(),
        }
    }

    pub fn get_dictionary_interner(&self) -> &DefaultStringInterner {
        &self.dictionary_interner
    }

    pub fn get_dictionary_interner_mut(&mut self) -> &mut DefaultStringInterner {
        &mut self.dictionary_interner
    }

    pub fn get_tag_interner(&self) -> &DefaultStringInterner {
        &self.tag_interner
    }

    pub fn get_tag_interner_mut(&mut self) -> &mut DefaultStringInterner {
        &mut self.tag_interner
    }

    pub fn get_unstemmed_voc(&self) -> &VocabularyImpl<String> {
        &self.unstemmed_voc
    }

    pub fn get_unstemmed_voc_mut(&mut self) -> &mut VocabularyImpl<String> {
        &mut self.unstemmed_voc
    }

    pub fn set_dictionary_for<L: Language>(&mut self, word_id: usize, dict: &str) {
        self.get_or_init_meta::<L>(word_id).push_associated_dictionary(dict)
    }

    pub fn set_meta_tag_for<L: Language>(&mut self, word_id: usize, tag: &str) {
        self.get_or_init_meta::<L>(word_id).push_meta_tag(tag)
    }

    pub fn set_unstemmed_for<L: Language>(&mut self, word_id: usize, unstemmed: String) {
        self.get_or_init_meta::<L>(word_id).push_unstemmed(unstemmed)
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
            tag_interner: self.tag_interner.clone(),
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
            tag_interner: self.tag_interner.clone(),
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

impl<'a, D, T, V> MetadataContainerWithDict<'a, D, T, V> where D: BasicDictionaryWithMeta + BasicDictionaryWithVocabulary<T, V> {
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

impl<D, T, V> Display for MetadataContainerWithDict<'_, D, T, V> where D: DictionaryWithVocabulary<T, V>, V: Vocabulary<T>, T: Display {
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

impl<'a, D, T, V> MetadataContainerWithDictMut<'a, D, T, V> where D: BasicDictionaryWithMeta + BasicDictionaryWithVocabulary<T, V> {
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


impl<D, T, V> Display for MetadataContainerWithDictMut<'_, D, T, V> where D: DictionaryWithVocabulary<T, V>, V: Vocabulary<T>, T: Display {
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


#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Metadata {
    #[serde(with = "metadata_interned_field_serializer")]
    pub associated_dictionaries: OnceCell<Vec<InternedString>>,
    #[serde(with = "metadata_interned_field_serializer")]
    pub meta_tags: OnceCell<Vec<InternedString>>,
    #[serde(with = "once_cell_serializer")]
    pub unstemmed: OnceCell<Vec<usize>>
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
        self.meta_tags(InternedString) || meta_tag,
        self.unstemmed(usize) || unstemmed
    }
}

mod metadata_interned_field_serializer {
    use std::fmt::Formatter;
    use itertools::Itertools;
    use once_cell::sync::OnceCell;
    use serde::{Deserializer, Serialize, Serializer};
    use serde::de::{Error, Unexpected, Visitor};
    use string_interner::{DefaultSymbol, Symbol};
    use string_interner::symbol::SymbolU32;
    use crate::toolkit::once_cell_serializer::{DeserializeOnceCell, SerializeOnceCell};

    pub(crate) fn serialize<S>(target: &OnceCell<Vec<DefaultSymbol>>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let to_ser = if let Some(value) = target.get() {
            SerializeOnceCell::InitializedOwned(value.iter().map(|value| value.to_usize()).collect_vec())
        } else {
            SerializeOnceCell::Uninitialized
        };
        to_ser.serialize(serializer)
    }

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

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<OnceCell<Vec<DefaultSymbol>>, D::Error> where D: Deserializer<'de> {
        let content: DeserializeOnceCell<Vec<usize>> = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(
            content.map(|content| {
                content.into_iter().map(|value|
                    DefaultSymbol::try_from_usize(value)
                        .ok_or_else(||
                            Error::invalid_value(
                                Unexpected::Unsigned(value as u64),
                                &SymbolU32Visitor
                            )
                        )
                ).collect::<Result<Vec<_>, _>>()
            }).transpose()?.into()
        )
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

    pub fn push_meta_tag(&mut self, tag: impl AsRef<str>) {
        let interned = unsafe{&mut *self.metadata_ref }.get_tag_interner_mut().get_or_intern(tag);
        unsafe {
            self.meta.add_meta_tag(interned);
        }
    }

    pub fn push_unstemmed(&mut self, word: impl Into<String>)  {
        let interned = unsafe{&mut *self.metadata_ref }.get_unstemmed_voc_mut().add(word);
        unsafe {
            self.meta.add_unstemmed(interned);
        }
    }
}


#[derive(Debug, Clone)]
#[pyclass]
pub struct SolvedMetadata {
    associated_dictionaries: Option<Vec<String>>,
    meta_tags: Option<Vec<String>>
}

#[pymethods]
impl SolvedMetadata {
    #[getter]
    pub fn associated_dictionaries(&self) -> Option<Vec<String>> {
        self.associated_dictionaries.clone()
    }

    #[getter]
    pub fn meta_tags(&self) -> Option<Vec<String>> {
        self.meta_tags.clone()
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
        match &self.meta_tags {
            None => {
                write!(f, "meta_tags=[]")
            }
            Some(value) => {
                write!(f, "meta_tags=[{}]", value.join(", "))
            }
        }?;
        write!(f, "}}")
    }
}

pub struct MetadataRef<'a> {
    pub(in super) raw: &'a Metadata,
    pub(in super) metadata_container: &'a MetadataContainer,
    pub(in super) associated_dictionary_cached: Arc<OnceCell<Vec<&'a str>>>,
    pub(in super) meta_tags_cached: Arc<OnceCell<Vec<&'a str>>>,
}

impl<'a> MetadataRef<'a> {

    pub fn new(raw: &'a Metadata, metadata_container: &'a MetadataContainer) -> Self {
        Self {
            raw,
            metadata_container,
            associated_dictionary_cached: Default::default(),
            meta_tags_cached: Default::default()
        }
    }

    pub fn raw(&self) -> &'a Metadata {
        self.raw
    }

    pub fn has_associated_dictionary(&self, q: impl AsRef<str>) -> bool {
        self.metadata_container.get_dictionary_interner().get(q).is_some_and(|value| self.raw.has_associated_dictionary(value))
    }

    pub fn has_meta_tag(&self, q: impl AsRef<str>) -> bool {
        self.metadata_container.get_tag_interner().get(q).is_some_and(|value| self.raw.has_meta_tag(value))
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

    pub fn meta_tags(&self) -> Option<&Vec<&'a str>> {
        if let Some(found) = self.meta_tags_cached.get() {
            Some(found)
        } else {
            if let Some(inner) = self.raw.meta_tags.get() {
                let interner = self.metadata_container.get_tag_interner();
                self.meta_tags_cached.set(
                    inner.iter().map(|value| {
                        interner.resolve(value.clone()).expect("This should be known!")
                    }).collect()
                ).unwrap();
                self.meta_tags_cached.get()
            } else {
                None
            }
        }
    }

    pub fn clone_metadata(self) -> Metadata {
        self.raw.clone()
    }

    pub fn to_solved_metadata(self) -> SolvedMetadata {
        let associated_dictionaries: Option<Vec<String>> = self.associated_dictionaries().map(|value| value.iter().map(|value| value.to_string()).collect());
        let meta_tags: Option<Vec<String>> = self.meta_tags().map(|value| value.iter().map(|value| value.to_string()).collect());
        SolvedMetadata {
            associated_dictionaries,
            meta_tags
        }
    }
}

impl Debug for MetadataRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataRef")
            .field("inner", self.raw)
            .field("associated_dictionary_cached", &self.associated_dictionary_cached.get())
            .field("meta_tags_cached", &self.meta_tags_cached.get())
            .finish_non_exhaustive()
    }
}

impl<'a> Clone for MetadataRef<'a> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            metadata_container: self.metadata_container,
            associated_dictionary_cached: self.associated_dictionary_cached.clone(),
            meta_tags_cached: self.meta_tags_cached.clone()
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

        let b = match self.meta_tags() {
            None => {
                "None".to_string()
            }
            Some(value) => {
                value.join(", ")
            }
        };
        write!(f, "MetadataRef{{[{a}], [{b}]}}")
    }
}