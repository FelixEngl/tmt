#![allow(dead_code)]

pub mod metadata;
pub mod direction;
pub mod iterators;

use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use itertools::{Itertools, Position};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::direction::{A, AToB, B, BToA, Direction, DirectionKind, DirectionTuple, Invariant, Language, Translation};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{MappableVocabulary, Vocabulary, VocabularyMut};
use crate::topicmodel::dictionary::iterators::{DictionaryWithMetaIterator, DictIter, DictIterImpl, DictLangIter};

use crate::topicmodel::dictionary::metadata::{MetadataContainer, MetadataContainerWithDict, MetadataContainerWithDictMut, MetadataRef, SolvedMetadata};#[macro_export]

macro_rules! dict_insert {
    ($d: ident, $a: tt : $b: tt) => {
        $crate::topicmodel::dictionary::DictionaryMut::insert_value::<$crate::topicmodel::dictionary::direction::Invariant>(&mut $d, $a, $b)
    };
    ($d: ident, $a: tt :: $b: tt) => {
        dict_insert!($d, $a : $b);
    };
    ($d: ident, $a: tt :=: $b: tt) => {
        dict_insert!($d, $a : $b);
    };
    ($d: ident, $a: tt :>: $b: tt) => {
        $crate::topicmodel::dictionary::DictionaryMut::insert_value::<$crate::topicmodel::dictionary::direction::AToB>(&mut $d, $a, $b)
    };
    ($d: ident, $a: tt :<: $b: tt) => {
        $crate::topicmodel::dictionary::DictionaryMut::insert_value::<$crate::topicmodel::dictionary::direction::BToA>(&mut $d, $a, $b)
    };
}

#[macro_export]
macro_rules! dict {
    () => {Dictionary::<_, $crate::topicmodel::vocabulary::VocabularyImpl<_>>::new()};
    ($($a:tt $op:tt $b:tt)+) => {
        {
            let mut __dict = Dictionary::<_, $crate::topicmodel::vocabulary::VocabularyImpl<_>>::new();
            $(
                $crate::dict_insert!(__dict, $a $op $b);
            )+
            __dict
        }
    }
}

pub trait BasicDictionary: Send + Sync {
    fn map_a_to_b(&self) -> &Vec<Vec<usize>>;

    fn map_b_to_a(&self) -> &Vec<Vec<usize>>;

    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        if D::DIRECTION.is_a_to_b() {
            self.map_a_to_b().get(word_id)
        } else {
            self.map_b_to_a().get(word_id)
        }
    }


    /// Iterates over all mappings (a to b and b to a), does not filter for uniqueness.
    fn iter(&self) -> DictIter {
        DictIterImpl::new(self)
    }
}

pub trait BasicDictionaryWithVocabulary<T, V>: BasicDictionary {
    fn voc_a(&self) -> &V;

    fn voc_b(&self) -> &V;
}

pub trait DictionaryWithVocabulary<T, V>: BasicDictionaryWithVocabulary<T, V> where V: Vocabulary<T> {

    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a().contains_id(id) && self.map_a_to_b().get(id).is_some_and(|value| !value.is_empty())
        } else {
            self.voc_b().contains_id(id) && self.map_b_to_a().get(id).is_some_and(|value| !value.is_empty())
        }
    }

    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a().get_value(id)
        } else {
            self.voc_b().get_value(id)
        }
    }

    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            ids.iter().map(|value| unsafe {
                self.voc_b().get_id_entry(*value).unwrap_unchecked()
            }).collect()
        } else {
            ids.iter().map(|value| unsafe {
                self.voc_a().get_id_entry(*value).unwrap_unchecked()
            }).collect()
        }
    }

    fn ids_to_values<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<&'a HashRef<T>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            ids.iter().map(|value| unsafe {
                self.voc_b().get_value(*value).unwrap_unchecked()
            }).collect()
        } else {
            ids.iter().map(|value| unsafe {
                self.voc_a().get_value(*value).unwrap_unchecked()
            }).collect()
        }
    }

    fn translate_id<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<(usize, &'a HashRef<T>)>> where V: 'a {
        Some(self.ids_to_id_entry::<D>(self.translate_id_to_ids::<D>(word_id)?))
    }

    fn translate_id_to_values<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<&'a HashRef<T>>> where V: 'a {
        Some(self.ids_to_values::<D>(self.translate_id_to_ids::<D>(word_id)?))
    }

    fn iter_language<'a, L: Language>(&'a self) -> DictLangIter<'a, T, L, Self, V> where V: 'a {
        DictLangIter::<T, L, Self, V>::new(self)
    }
}

pub trait DictionaryMut<T, V>: DictionaryWithVocabulary<T, V> where T: Eq + Hash, V: VocabularyMut<T> {
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize>;

    fn insert_value<D: Direction>(&mut self, word_a: T, word_b: T) -> DirectionTuple<usize, usize> {
        self.insert_hash_ref::<D>(HashRef::new(word_a), HashRef::new(word_b))
    }

    fn insert<D: Direction>(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) -> DirectionTuple<usize, usize> {
        self.insert_value::<D>(word_a.into(), word_b.into())
    }

    fn translate_value<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<(usize, &'a HashRef<T>)>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq, V: 'a
    {
        Some(self.ids_to_id_entry::<D>(self.translate_value_to_ids::<D, Q>(word)?))
    }

    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        let id = if D::DIRECTION.is_a_to_b() {
            self.voc_a().get_id(word)
        } else {
            self.voc_b().get_id(word)
        }?;
        self.translate_id_to_ids::<D>(id)
    }

    fn word_to_id<D: Translation, Q: ?Sized>(&self, id: &Q) -> Option<usize>
        where
            T: Borrow<Q>,
            Q: Hash + Eq {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a().get_id(id)
        } else {
            self.voc_b().get_id(id)
        }
    }

    fn can_translate_word<D: Translation, Q: ?Sized>(&self, word: &Q) -> bool
        where
            T: Borrow<Q>,
            Q: Hash + Eq {
        self.word_to_id::<D, _>(word).is_some_and(|value| self.can_translate_id::<D>(value))
    }

    fn translate_value_to_values<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<&'a HashRef<T>>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq,
            V: 'a
    {
        Some(self.ids_to_values::<D>(self.translate_value_to_ids::<D, Q>(word)?))
    }
}

pub trait DictionaryFilterable<T, V>: DictionaryMut<T, V> where T: Eq + Hash, V: VocabularyMut<T> + Default {
    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized;

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<T>) -> bool, Fb: Fn(&'a HashRef<T>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, T: 'a;
}



#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary<T, V> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    voc_a: V,
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    voc_b: V,
    map_a_to_b: Vec<Vec<usize>>,
    map_b_to_a: Vec<Vec<usize>>,
    _word_type: PhantomData<T>
}

unsafe impl<T, V> Send for Dictionary<T, V>{}
unsafe impl<T, V> Sync for Dictionary<T, V>{}

impl<T, V> Dictionary<T, V> where V: Vocabulary<T> {
    pub fn from_voc(voc_a: V, voc_b: V) -> Self {
        let mut map_a_to_b = Vec::new();
        map_a_to_b.resize_with(voc_a.len(), || Vec::with_capacity(1));
        let mut map_b_to_a = Vec::new();
        map_b_to_a.resize_with(voc_b.len(), || Vec::with_capacity(1));

        Self {
            voc_a,
            voc_b,
            map_a_to_b,
            map_b_to_a,
            _word_type: PhantomData
        }
    }
}

impl<T, V> Dictionary<T, V> where V: Vocabulary<T> + Default {
    pub fn from_voc_a(voc_a: V) -> Self {
        let mut map_a_to_b = Vec::new();
        map_a_to_b.resize_with(voc_a.len(), || Vec::with_capacity(1));

        Self {
            voc_a,
            voc_b: Default::default(),
            map_a_to_b,
            map_b_to_a: Default::default(),
            _word_type: PhantomData
        }
    }
}

impl<T, V> Dictionary<T, V> where V: Default {
    pub fn new() -> Self {
        Self {
            voc_a: Default::default(),
            voc_b: Default::default(),
            map_a_to_b: Default::default(),
            map_b_to_a: Default::default(),
            _word_type: PhantomData
        }
    }
}

impl<T, V> Clone for Dictionary<T, V> where V: Clone {
    fn clone(&self) -> Self {
        Self {
            voc_a: self.voc_a.clone(),
            voc_b: self.voc_b.clone(),
            map_a_to_b: self.map_a_to_b.clone(),
            map_b_to_a: self.map_b_to_a.clone(),
            _word_type: PhantomData
        }
    }
}

impl<T, V> Default for Dictionary<T, V> where V: Default {
    fn default() -> Self {
        Self {
            voc_a: Default::default(),
            voc_b: Default::default(),
            map_a_to_b: Default::default(),
            map_b_to_a: Default::default(),
            _word_type: PhantomData
        }
    }
}

impl<T, V> BasicDictionary for Dictionary<T, V> {
    fn map_a_to_b(&self) -> &Vec<Vec<usize>> {
        &self.map_a_to_b
    }

    fn map_b_to_a(&self) -> &Vec<Vec<usize>> {
        &self.map_b_to_a
    }

    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        if D::DIRECTION.is_a_to_b() {
            &self.map_a_to_b
        } else {
            &self.map_b_to_a
        }.get(word_id)
    }
}

impl<T, V> BasicDictionaryWithVocabulary<T, V> for Dictionary<T, V> {
    fn voc_a(&self) -> &V {
        &self.voc_a
    }

    fn voc_b(&self) -> &V {
        &self.voc_b
    }
}

impl<T, V> Dictionary<T, V> where T: Eq + Hash, V: MappableVocabulary<T> {
    pub fn map<Q: Eq + Hash, Voc, F>(self, f: F) -> Dictionary<Q, Voc> where F: for<'a> Fn(&'a T)-> Q, Voc: From<Vec<Q>> {
        Dictionary {
            voc_a: self.voc_a.map(&f),
            voc_b: self.voc_b.map(f),
            map_a_to_b: self.map_a_to_b,
            map_b_to_a: self.map_b_to_a,
            _word_type: PhantomData
        }
    }
}

impl<T, V> DictionaryWithVocabulary<T, V> for  Dictionary<T, V> where V: Vocabulary<T> {
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a.contains_id(id) && self.map_a_to_b.get(id).is_some_and(|value| !value.is_empty())
        } else {
            self.voc_b.contains_id(id) && self.map_b_to_a.get(id).is_some_and(|value| !value.is_empty())
        }
    }

    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a.get_value(id)
        } else {
            self.voc_b.get_value(id)
        }
    }

    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            ids.iter().map(|value| unsafe {
                self.voc_b.get_id_entry(*value).unwrap_unchecked()
            }).collect()
        } else {
            ids.iter().map(|value| unsafe {
                self.voc_a.get_id_entry(*value).unwrap_unchecked()
            }).collect()
        }
    }

    fn ids_to_values<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<&'a HashRef<T>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            ids.iter().map(|value| unsafe {
                self.voc_b.get_value(*value).unwrap_unchecked()
            }).collect()
        } else {
            ids.iter().map(|value| unsafe {
                self.voc_a.get_value(*value).unwrap_unchecked()
            }).collect()
        }
    }
}

impl<T, V> DictionaryMut<T, V> for  Dictionary<T, V> where T: Eq + Hash, V: VocabularyMut<T> {
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        let id_a = self.voc_a.add_hash_ref(word_a);
        let id_b = self.voc_b.add_hash_ref(word_b);
        if D::DIRECTION.is_a_to_b() {
            if let Some(found) = self.map_a_to_b.get_mut(id_a) {
                if !found.contains(&id_b) {
                    found.push(id_b)
                }
            } else {
                while self.map_a_to_b.len() <= id_a {
                    self.map_a_to_b.push(Vec::with_capacity(1));
                }
                unsafe {
                    self.map_a_to_b.get_unchecked_mut(id_a).push(id_b);
                }
            }
            if !D::DIRECTION.is_b_to_a() {
                return DirectionTuple::a_to_b(id_a, id_b);
            }
        }
        if D::DIRECTION.is_b_to_a() {
            if let Some(found) = self.map_b_to_a.get_mut(id_b) {
                if !found.contains(&id_a) {
                    found.push(id_a)
                }
            } else {
                while self.map_b_to_a.len() <= id_b {
                    self.map_b_to_a.push(Vec::with_capacity(1));
                }
                unsafe {
                    self.map_b_to_a.get_unchecked_mut(id_b).push(id_a);
                }
            }
            if !D::DIRECTION.is_a_to_b() {
                return DirectionTuple::b_to_a(id_a, id_b);
            }
        }

        DirectionTuple::invariant(id_a, id_b)
    }

    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        let id = if D::DIRECTION.is_a_to_b() {
            self.voc_a.get_id(word)
        } else {
            self.voc_b.get_id(word)
        }?;
        self.translate_id_to_ids::<D>(id)
    }

    fn word_to_id<D: Translation, Q: ?Sized>(&self, id: &Q) -> Option<usize>
        where
            T: Borrow<Q>,
            Q: Hash + Eq {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a.get_id(id)
        } else {
            self.voc_b.get_id(id)
        }
    }
}
impl<T, V> DictionaryFilterable<T, V>  for Dictionary<T, V> where T: Eq + Hash, V: VocabularyMut<T> + Default{
    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized {
        let mut new_dict = Dictionary::new();

        for DirectionTuple{a, b, direction} in self.iter() {
            match direction {
                DirectionKind::AToB => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionKind::BToA => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionKind::Invariant => {
                    if filter_a(a) && filter_b(b) {
                        new_dict.insert_hash_ref::<Invariant>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    } else if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    } else if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
            }
        }

        new_dict
    }

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<T>) -> bool, Fb: Fn(&'a HashRef<T>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, T: 'a {
        let mut new_dict = Dictionary::new();
        for DirectionTuple{a, b, direction} in self.iter() {
            let a = self.id_to_word::<A>(a).unwrap();
            let b = self.id_to_word::<B>(b).unwrap();
            match direction {
                DirectionKind::AToB => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionKind::BToA => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionKind::Invariant => {
                    let filter_a = filter_a(a);
                    let filter_b = filter_b(b);
                    if filter_a && filter_b {
                        new_dict.insert_hash_ref::<Invariant>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_a {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_b {
                        new_dict.insert_hash_ref::<BToA>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
            }
        }

        new_dict
    }
}

impl<T: Display, V: Vocabulary<T>> Display for Dictionary<T, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn write_language<L: Language, T: Display, V: Vocabulary<T>>(dictionary: &Dictionary<T, V>, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:\n", L::LANG)?;
            for (position_a, (id_a, value_a, translations)) in dictionary.iter_language::<L>().with_position() {
                write!(f, "  {value_a}({id_a}):\n")?;
                if let Some(translations) = translations {
                    for (position_b, (id_b, value_b)) in translations.iter().with_position() {
                        match position_b {
                            Position::First | Position::Middle => {
                                write!(f, "    {value_b}({id_b})\n")?
                            }
                            Position::Last | Position::Only => {
                                match position_a {
                                    Position::First | Position::Middle => {
                                        write!(f, "    {value_b}({id_b})\n")?
                                    }
                                    Position::Last | Position::Only => {
                                        write!(f, "    {value_b}({id_b})")?
                                    }
                                }
                            }
                        }

                    }
                } else {
                    write!(f, "    - None -\n")?;
                }
            }
            Ok(())
        }

        write_language::<A, _, V>(self, f)?;
        write!(f, "\n------\n")?;
        write_language::<B, _, V>(self, f)
    }
}



pub trait BasicDictionaryWithMeta: BasicDictionary {
    fn metadata(&self) -> &MetadataContainer;
    fn metadata_mut(&mut self) -> &mut MetadataContainer;

    fn iter_with_meta(&self) -> DictionaryWithMetaIter<Self> {
        DictionaryWithMetaIter::new(self)
    }
}

pub struct DictionaryWithMetaIter<'a, D> where D: BasicDictionaryWithMeta + ?Sized {
    dictionary_with_meta: &'a D,
    iter: DictIter<'a>
}

impl<'a, D> DictionaryWithMetaIter<'a, D> where D: BasicDictionaryWithMeta + ?Sized {
    pub fn new(dictionary_with_meta: &'a D) -> Self {
        Self {
            iter: dictionary_with_meta.iter(),
            dictionary_with_meta
        }
    }
}

impl<'a, D> Iterator for DictionaryWithMetaIter<'a, D> where D: BasicDictionaryWithMeta {
    type Item = DirectionTuple<(usize, Option<MetadataRef<'a>>), (usize, Option<MetadataRef<'a>>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let DirectionTuple{a, b, direction} = self.iter.next()?;
        Some(
            match direction {
                DirectionKind::AToB => {
                    DirectionTuple::a_to_b(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    )
                }
                DirectionKind::BToA => {
                    DirectionTuple::b_to_a(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    )
                }
                DirectionKind::Invariant => {
                    DirectionTuple::invariant(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    )
                }
            }
        )
    }
}


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DictionaryWithMeta<T, V> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    inner: Dictionary<T, V>,
    metadata: MetadataContainer
}

impl<T, V> DictionaryWithMeta<T, V> {
    fn new(inner: Dictionary<T, V>, metadata: MetadataContainer) -> Self {
        Self { inner, metadata }
    }
}

impl<T, V> DictionaryWithMeta<T, V> where V: Vocabulary<T> {
    fn metadata_with_dict(&self) -> MetadataContainerWithDict<Self, T, V> where Self: Sized {
        MetadataContainerWithDict::wrap(self)
    }

    fn metadata_with_dict_mut(&mut self) -> MetadataContainerWithDictMut<Self, T, V> where Self: Sized {
        MetadataContainerWithDictMut::wrap(self)
    }
}
unsafe impl<T, V> Send for DictionaryWithMeta<T, V>{}
unsafe impl<T, V> Sync for DictionaryWithMeta<T, V>{}
impl<T, V> DictionaryWithMeta<T, V> where V: VocabularyMut<T> + Default, T: Hash + Eq  {

    fn insert_meta_for_create_subset<'a, L: Language>(&mut self, word_id: usize, metadata_ref: MetadataRef<'a>) {
        let tags = metadata_ref.raw.meta_tags.get();
        let dics = metadata_ref.raw.associated_dictionaries.get();
        let unstemmed = metadata_ref.raw.unstemmed.get();

        if tags.is_none() && dics.is_none() {
            return;
        }

        let meta = self.metadata.get_or_init_meta::<L>(word_id).meta;

        if let Some(dics) = dics {
            unsafe { meta.add_all_associated_dictionaries(&dics) }
        }
        if let Some(tags) = tags {
            unsafe { meta.add_all_meta_tags(&tags) }
        }
        if let Some(unstemmed) = unstemmed {
            meta.add_all_unstemmed(unstemmed)
        }
    }

    pub fn create_subset_with_filters<F1, F2>(&self, filter_a: F1, filter_b: F2) -> DictionaryWithMeta<T, V> where F1: Fn(&DictionaryWithMeta<T, V>, usize, Option<&MetadataRef>) -> bool, F2: Fn(&DictionaryWithMeta<T, V>, usize, Option<&MetadataRef>) -> bool {
        let mut new = Self {
            inner: Dictionary::new(),
            metadata: self.metadata.copy_keep_vocebulary()
        };
        for DirectionTuple{
            a: (word_id_a, meta_a),
            b: (word_id_b, meta_b),
            direction
        } in self.iter_with_meta() {
            if filter_a(self, word_id_a, meta_a.as_ref()) {
                if filter_b(self, word_id_b, meta_b.as_ref()) {
                    let word_a = self.inner.voc_a.get_value(word_id_a).unwrap();
                    let word_b = self.inner.voc_b.get_value(word_id_b).unwrap();
                    let DirectionTuple{ a: word_a, b: word_b, direction: _ } = match direction {
                        DirectionKind::AToB => {
                            new.insert_hash_ref::<AToB>(word_a.clone(), word_b.clone())
                        }
                        DirectionKind::BToA => {
                            new.insert_hash_ref::<BToA>(word_a.clone(), word_b.clone())
                        },
                        DirectionKind::Invariant => {
                            new.insert_hash_ref::<Invariant>(word_a.clone(), word_b.clone())
                        }
                    };
                    if let Some(meta_a) = meta_a {
                        new.insert_meta_for_create_subset::<A>(word_a, meta_a);
                    }
                    if let Some(meta_b) = meta_b {
                        new.insert_meta_for_create_subset::<B>(word_b, meta_b);
                    }
                }
            }
        }
        new
    }
}

impl<T, V> DictionaryWithMeta<T, V> where V: Vocabulary<T> + Default {
    pub fn from_voc_a(voc_a: V) -> Self {
        Self::new(
            Dictionary::from_voc_a(voc_a),
            Default::default()
        )
    }
}
impl<T, V> DictionaryWithMeta<T, V> where V: Vocabulary<T> {
    pub fn from_voc(voc_a: V, voc_b: V) -> Self {
        Self::new(
            Dictionary::from_voc(voc_a, voc_b),
            Default::default()
        )
    }
}
impl<T, V> Clone for DictionaryWithMeta<T, V> where V: Clone {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone(), self.metadata.clone())
    }
}
impl<T, V> BasicDictionary for DictionaryWithMeta<T, V> {
    delegate::delegate! {
        to self.inner {
            fn map_a_to_b(&self) -> &Vec<Vec<usize>>;
            fn map_b_to_a(&self) -> &Vec<Vec<usize>>;
        }
    }

    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        self.inner.translate_id_to_ids::<D>(word_id)
    }
}
impl<T, V> BasicDictionaryWithMeta for DictionaryWithMeta<T, V> where V: Vocabulary<T> {
    fn metadata(&self) -> &MetadataContainer {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut MetadataContainer {
        &mut self.metadata
    }
}
impl<T, V> BasicDictionaryWithVocabulary<T, V> for DictionaryWithMeta<T, V> {
    delegate::delegate! {
        to self.inner {
            fn voc_a(&self) -> &V;
            fn voc_b(&self) -> &V;
        }
    }
}
impl<T, V> DictionaryWithMeta<T, V> where T: Eq + Hash, V: MappableVocabulary<T> {
    pub fn map<Q: Eq + Hash, Voc, F>(self, f: F) -> DictionaryWithMeta<Q, Voc> where F: for<'a> Fn(&'a T)-> Q, Voc: From<Vec<Q>> {
        DictionaryWithMeta::<Q, Voc>::new(
            self.inner.map(&f),
            self.metadata.clone()
        )
    }
}
impl<T, V> DictionaryWithVocabulary<T, V> for  DictionaryWithMeta<T, V> where V: Vocabulary<T> {

    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        self.inner.can_translate_id::<D>(id)
    }

    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        self.inner.id_to_word::<D>(id)
    }

    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        self.inner.ids_to_id_entry::<D>(ids)
    }

    fn ids_to_values<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<&'a HashRef<T>> where V: 'a {
        self.inner.ids_to_values::<D>(ids)
    }
}
impl<T, V> DictionaryMut<T, V> for  DictionaryWithMeta<T, V> where T: Eq + Hash, V: VocabularyMut<T> {
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        self.inner.insert_hash_ref::<D>(word_a, word_b)
    }

    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        self.inner.translate_value_to_ids::<D, _>(word)
    }

    fn word_to_id<D: Translation, Q: ?Sized>(&self, id: &Q) -> Option<usize>
        where
            T: Borrow<Q>,
            Q: Hash + Eq {
        self.inner.word_to_id::<D, _>(id)
    }

}
impl<T, V> DictionaryFilterable<T, V>  for DictionaryWithMeta<T, V> where T: Eq + Hash, V: VocabularyMut<T> + Default{
    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized {
        let mut new_dict = DictionaryWithMeta::new(
            Default::default(),
            self.metadata.copy_keep_vocebulary()
        );

        for DirectionTuple{a, b, direction} in self.iter() {
            match direction {
                DirectionKind::AToB => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionKind::BToA => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionKind::Invariant => {
                    let filter_a = filter_a(a);
                    let filter_b = filter_b(b);
                    if filter_a && filter_b {
                        new_dict.insert_hash_ref::<Invariant>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    } else if filter_a {
                        new_dict.insert_hash_ref::<AToB>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    } else if filter_b {
                        new_dict.insert_hash_ref::<BToA>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
            }
        }

        new_dict
    }

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<T>) -> bool, Fb: Fn(&'a HashRef<T>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, T: 'a {
        let mut new_dict = DictionaryWithMeta::new(
            Default::default(),
            self.metadata.copy_keep_vocebulary()
        );
        for DirectionTuple{a, b, direction} in self.iter() {
            let a = self.id_to_word::<A>(a).unwrap();
            let b = self.id_to_word::<B>(b).unwrap();
            match direction {
                DirectionKind::AToB => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionKind::BToA => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionKind::Invariant => {
                    let filter_a = filter_a(a);
                    let filter_b = filter_b(a);
                    if filter_a && filter_b {
                        new_dict.insert_hash_ref::<Invariant>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_a {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_b {
                        new_dict.insert_hash_ref::<BToA>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
            }
        }

        new_dict
    }
}

impl<T: Display, V: Vocabulary<T>> Display for DictionaryWithMeta<T, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)?;
        write!(f, "\n------\n")?;
        write!(f, "{}", self.metadata_with_dict())?;
        Ok(())
    }
}

impl<T, V> From<Dictionary<T, V>> for DictionaryWithMeta<T, V> {
    fn from(value: Dictionary<T, V>) -> Self {
        Self::new(
            value,
            MetadataContainer::create()
        )
    }
}

impl<T, V> IntoIterator for DictionaryWithMeta<T, V> where V: Vocabulary<T>, T: Hash + Eq {
    type Item = DirectionTuple<(usize, HashRef<T>, Option<SolvedMetadata>), (usize, HashRef<T>, Option<SolvedMetadata>)>;
    type IntoIter = DictionaryWithMetaIterator<DictionaryWithMeta<T, V>, T, V>;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryWithMetaIterator::new(self)
    }
}


#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, DictionaryMut, DictionaryWithMeta, DictionaryWithVocabulary};
    use crate::topicmodel::dictionary::direction::{A, B, DirectionTuple, Invariant};
    use crate::topicmodel::vocabulary::{VocabularyImpl, VocabularyMut};

    #[test]
    fn can_create_with_meta(){
        let mut voc_a = VocabularyImpl::<String>::new();
        voc_a.extend(vec![
            "plane".to_string(),
            "aircraft".to_string(),
            "airplane".to_string(),
            "flyer".to_string(),
            "airman".to_string(),
            "airfoil".to_string(),
            "wing".to_string(),
            "deck".to_string(),
            "hydrofoil".to_string(),
            "foil".to_string(),
            "bearing surface".to_string()
        ]);
        let mut voc_b = VocabularyImpl::<String>::new();
        voc_b.extend(vec![
            "Flugzeug".to_string(),
            "Flieger".to_string(),
            "Tragfläche".to_string(),
            "Ebene".to_string(),
            "Planum".to_string(),
            "Platane".to_string(),
            "Maschine".to_string(),
            "Bremsberg".to_string(),
            "Berg".to_string(),
            "Fläche".to_string(),
            "Luftfahrzeug".to_string(),
            "Fluggerät".to_string(),
            "Flugsystem".to_string(),
            "Motorflugzeug".to_string(),
        ]);

        let mut dict = DictionaryWithMeta::from_voc(voc_a.clone(), voc_b.clone());
        {
            dict.metadata_with_dict_mut().reserve_meta();
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Ebene").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Planum").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Platane").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Maschine").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Bremsberg").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Berg").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Fläche").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Luftfahrzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Fluggerät").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flugsystem").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Motorflugzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("flyer").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airman").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airfoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictE");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictC");
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("wing").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("deck").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("hydrofoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictC");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("foil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictB");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictB");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("bearing surface").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");

            drop(meta_b);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(0);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictB");
        }

        println!("{}", dict);

        let result = dict.create_subset_with_filters(
            |_, _, meaning| {
                if let Some(found) = meaning {
                    found.has_associated_dictionary("DictA") || found.has_associated_dictionary("DictB")
                } else {
                    false
                }
            },
            |_,_,_| { true }
        );
        println!(".=======.");
        println!("{}", result);
        println!("--==========--");

        for value in dict.iter_with_meta() {
            println!("{}", value.map(
                |(id, meta)| {
                    format!("'{}({id})': {}", dict.id_to_word::<A>(id).unwrap().to_string(), meta.map_or("NONE".to_string(), |value| value.to_solved_metadata().to_string()))
                },
                |(id, meta)| {
                    format!("'{}({id})': {}", dict.id_to_word::<B>(id).unwrap().to_string(), meta.map_or("NONE".to_string(), |value| value.to_solved_metadata().to_string()))
                }
            ))
        }
        println!("--==========--");
        for value in dict.into_iter() {
            println!("'{}({})': {}, '{}({})': {}",
                     value.a.1, value.a.0, value.a.clone().2.map_or("NONE".to_string(), |value| value.to_string()),
                     value.b.1, value.b.0, value.b.clone().2.map_or("NONE".to_string(), |value| value.to_string())
            )
        }
    }
}
