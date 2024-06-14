#![allow(dead_code)]

pub mod metadata;

use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::iter::{Chain, Cloned, Enumerate, FlatMap, Map};
use std::marker::PhantomData;
use std::slice::Iter;
use itertools::{Itertools, Position, Unique};
use serde::{Deserialize, Serialize};
use crate::toolkit::tupler::{SupportsTupling, TupleFirst, TupleLast};
use crate::topicmodel::dictionary::direction::{A, AToB, B, BToA, Direction, Invariant, Language, Translation};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{MappableVocabulary, Vocabulary, VocabularyMut};
use strum::{EnumIs};

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
        if D::A2B {
            self.map_a_to_b().get(word_id)
        } else {
            self.map_b_to_a().get(word_id)
        }
    }


    /// Iterates over all mappings (a to b and b to a), does not filter for uniqueness.
    fn iter(&self) -> DictIter {
        DictIterImpl::new(self).unique()
    }
}

#[derive(Debug, Copy, Clone, EnumIs, Eq, PartialEq, Hash)]
pub enum DirectionTuple<Ta, Tb> {
    AToB{
        a: Ta,
        b: Tb
    },
    BToA {
        a: Ta,
        b: Tb
    },
    Invariant {
        a: Ta,
        b: Tb
    }
}

impl<Ta, Tb> DirectionTuple<Ta, Tb> {
    pub fn a(&self) -> &Ta {
        match self {
            DirectionTuple::AToB { a, .. } => {a}
            DirectionTuple::BToA { a, .. } => {a}
            DirectionTuple::Invariant { a, .. } => {a}
        }
    }
    pub fn b(&self) -> &Tb {
        match self {
            DirectionTuple::AToB { b, .. } => {b}
            DirectionTuple::BToA { b, .. } => {b}
            DirectionTuple::Invariant { b, .. } => {b}
        }
    }

    pub fn to_ab_tuple(self) -> (Ta, Tb) {
        match self {
            DirectionTuple::AToB { a, b } => {(a, b)}
            DirectionTuple::BToA { a, b } => {(a, b)}
            DirectionTuple::Invariant { a, b } => {(a, b)}
        }
    }

    pub fn map<Ra, Rb, F1: FnOnce(Ta) -> Ra, F2: FnOnce(Tb) -> Rb>(self, map_a: F1, map_b: F2) -> DirectionTuple<Ra, Rb> {
        match self {
            DirectionTuple::AToB { a, b } => {
                DirectionTuple::AToB { a: map_a(a), b: map_b(b) }
            }
            DirectionTuple::BToA { a, b } => {
                DirectionTuple::BToA { a: map_a(a), b: map_b(b) }
            }
            DirectionTuple::Invariant { a, b } => {
                DirectionTuple::Invariant { a: map_a(a), b: map_b(b) }
            }
        }
    }
}

impl<T> DirectionTuple<T, T>  {
    pub fn map_both<R, F: Fn(T) -> R>(self, mapping: F) -> DirectionTuple<R, R> {
        match self {
            DirectionTuple::AToB { a, b } => {
                DirectionTuple::AToB { a: mapping(a), b: mapping(b) }
            }
            DirectionTuple::BToA { a, b } => {
                DirectionTuple::BToA { a: mapping(a), b: mapping(b) }
            }
            DirectionTuple::Invariant { a, b } => {
                DirectionTuple::Invariant { a: mapping(a), b: mapping(b) }
            }
        }
    }
}

impl<Ta: Display, Tb: Display> Display for  DirectionTuple<Ta, Tb>  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectionTuple::AToB { a, b } => {
                write!(f, "AToB{{{a}, {b}}}")
            }
            DirectionTuple::BToA { a, b } => {
                write!(f, "BToA{{{a}, {b}}}")
            }
            DirectionTuple::Invariant { a, b } => {
                write!(f, "Invariant{{{a}, {b}}}")
            }
        }
    }
}

pub type DictIter<'a> = Unique<DictIterImpl<'a>>;
/// Iterates over all mappings (a to b and b to a), does not filter for uniqueness.
pub struct DictIterImpl<'a> {
    a_to_b: &'a Vec<Vec<usize>>,
    b_to_a: &'a Vec<Vec<usize>>,
    iter: Chain<ABIter<'a>, BAIter<'a>>,
}
type ABIter<'a> = FlatMap<Enumerate<Iter<'a, Vec<usize>>>, Map<TupleFirst<Cloned<Iter<'a, usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>, fn((usize, &Vec<usize>)) -> Map<TupleFirst<Cloned<Iter<usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>>;
type BAIter<'a> = FlatMap<Enumerate<Iter<'a, Vec<usize>>>, Map<TupleLast<Cloned<Iter<'a, usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>, fn((usize, &Vec<usize>)) -> Map<TupleLast<Cloned<Iter<usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>>;
impl<'a> DictIterImpl<'a> {
    fn new(dict: &'a (impl BasicDictionary + ?Sized)) -> Self {
        let a_to_b: ABIter = dict.map_a_to_b().iter().enumerate().flat_map(|(a, value)| value.iter().cloned().tuple_first(a).map(|(a, b) | DirectionTuple::AToB {a, b}));
        let b_to_a: BAIter = dict.map_b_to_a().iter().enumerate().flat_map(|(b, value)| value.iter().cloned().tuple_last(b).map(|(a, b) | DirectionTuple::AToB {a, b}));
        let iter: Chain<ABIter, BAIter> = a_to_b.chain(b_to_a);
        Self {
            a_to_b: dict.map_a_to_b(),
            b_to_a: dict.map_b_to_a(),
            iter
        }
    }
}
impl<'a> Iterator for DictIterImpl<'a> {
    type Item = DirectionTuple<usize, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.iter.next()?;
        Some(
            match current {
                all @ DirectionTuple::AToB { a, b } => {
                    if self.b_to_a[b].contains(&a) {
                        DirectionTuple::Invariant {a, b}
                    } else {
                        all
                    }
                }
                all @ DirectionTuple::BToA { a, b } => {
                    if self.a_to_b[a].contains(&b) {
                        DirectionTuple::Invariant {a, b}
                    } else {
                        all
                    }
                }
                _ => unreachable!()
            }
        )
    }
}


pub trait BasicDictionaryWithVocabulary<T, V>: BasicDictionary {
    fn voc_a(&self) -> &V;

    fn voc_b(&self) -> &V;
}

pub trait DictionaryWithVocabulary<T, V>: BasicDictionaryWithVocabulary<T, V> where V: Vocabulary<T> {

    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        if D::A2B {
            self.voc_a().contains_id(id) && self.map_a_to_b().get(id).is_some_and(|value| !value.is_empty())
        } else {
            self.voc_b().contains_id(id) && self.map_b_to_a().get(id).is_some_and(|value| !value.is_empty())
        }
    }

    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        if D::A2B {
            self.voc_a().get_value(id)
        } else {
            self.voc_b().get_value(id)
        }
    }

    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        if D::A2B {
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
        if D::A2B {
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

pub struct DictLangIter<'a, T, L, D: ?Sized, V> where L: Language {
    iter: Enumerate<Iter<'a, HashRef<T>>>,
    dict: &'a D,
    _language: PhantomData<(L, V)>
}

impl<'a, T, L, D: ?Sized, V> DictLangIter<'a, T, L, D, V> where L: Language, V: Vocabulary<T> + 'a, D: BasicDictionaryWithVocabulary<T, V> {
    fn new(dict: &'a D) -> Self {
        Self {
            iter: if L::A2B {
                dict.voc_a().iter().enumerate()
            } else {
                dict.voc_b().iter().enumerate()
            },
            dict,
            _language: PhantomData
        }
    }
}

impl<'a, T, L, D, V> Iterator for DictLangIter<'a, T, L, D, V> where L: Language, V: Vocabulary<T> + 'a, D: DictionaryWithVocabulary<T, V> {
    type Item = (usize, &'a HashRef<T>, Option<Vec<(usize, &'a HashRef<T>)>>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, next) = self.iter.next()?;
        let translation = self.dict.translate_id::<L>(id);
        Some((id, next, translation))
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
        let id = if D::A2B {
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
        if D::A2B {
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
        if D::A2B {
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
        if D::A2B {
            self.voc_a.contains_id(id) && self.map_a_to_b.get(id).is_some_and(|value| !value.is_empty())
        } else {
            self.voc_b.contains_id(id) && self.map_b_to_a.get(id).is_some_and(|value| !value.is_empty())
        }
    }

    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        if D::A2B {
            self.voc_a.get_value(id)
        } else {
            self.voc_b.get_value(id)
        }
    }

    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        if D::A2B {
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
        if D::A2B {
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
        if D::A2B {
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
            if !D::B2A {
                return DirectionTuple::AToB {a: id_a, b: id_b};
            }
        }
        if D::B2A {
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
            if !D::A2B {
                return DirectionTuple::BToA {a: id_a, b: id_b};
            }
        }

        DirectionTuple::Invariant {a: id_a, b: id_b}
    }

    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        let id = if D::A2B {
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
        if D::A2B {
            self.voc_a.get_id(id)
        } else {
            self.voc_b.get_id(id)
        }
    }
}
impl<T, V> DictionaryFilterable<T, V>  for Dictionary<T, V> where T: Eq + Hash, V: VocabularyMut<T> + Default{
    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized {
        let mut new_dict = Dictionary::new();

        for value in self.iter() {
            match value {
                DirectionTuple::AToB { a, b } => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionTuple::BToA { a, b } => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionTuple::Invariant { a, b } => {
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
        for value in self.iter() {
            let a = self.id_to_word::<A>(*value.a()).unwrap();
            let b = self.id_to_word::<B>(*value.b()).unwrap();
            match value {
                DirectionTuple::AToB { .. } => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionTuple::BToA { .. } => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionTuple::Invariant { .. } => {
                    if filter_a(a) && filter_b(b) {
                        new_dict.insert_hash_ref::<Invariant>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_b(b) {
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
            write!(f, "{}:\n", L::LANGUAGE_NAME)?;
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

#[derive(Debug, Copy, Clone, EnumIs)]
pub enum DictionaryIteratorPointerState {
    NextAB,
    NextBA,
    Finished
}

impl DictionaryIteratorPointerState {
    pub fn next(&self) -> DictionaryIteratorPointerState {
        match self {
            DictionaryIteratorPointerState::NextAB => {DictionaryIteratorPointerState::NextBA}
            DictionaryIteratorPointerState::NextBA => {DictionaryIteratorPointerState::Finished}
            DictionaryIteratorPointerState::Finished => {DictionaryIteratorPointerState::Finished}
        }
    }
}

/// Iterates over all mappings (a to b and b to a).
pub struct DictionaryIteratorImpl<T, V, D> where D: DictionaryWithVocabulary<T, V>, V: Vocabulary<T> {
    pos: usize,
    index: usize,
    state: DictionaryIteratorPointerState,
    inner: D,
    _types: PhantomData<fn(T, V)->()>
}

impl<T, V, D> DictionaryIteratorImpl<T, V, D> where D: DictionaryWithVocabulary<T, V>, V: Vocabulary<T> {
    pub fn new(inner: D) -> Self {
        let mut new = Self {
            pos: 0,
            index: 0,
            state: DictionaryIteratorPointerState::NextAB,
            inner,
            _types: PhantomData
        };
        if !new.inner.map_a_to_b().get(new.pos).is_some_and(|found| !found.is_empty()) {
            new.increment_pos_and_idx();
        }
        return new
    }

    pub fn into_inner(self) -> D {
        self.inner
    }

    fn increment_pos_and_idx(&mut self) -> bool {
        let targ = match self.state {
            DictionaryIteratorPointerState::NextAB => {
                self.inner.map_a_to_b()
            }
            DictionaryIteratorPointerState::NextBA => {
                self.inner.map_b_to_a()
            }
            DictionaryIteratorPointerState::Finished => {return false}
        };
        if self.pos < targ.len() {
            let new_index = self.index + 1;
            if new_index < unsafe{targ.get_unchecked(self.pos)}.len() {
                self.index = new_index;
                return true
            }
        }
        let new_pos = self.pos + 1;
        if new_pos < targ.len() {
            self.index = 0;
            if unsafe{targ.get_unchecked(self.pos)}.is_empty() {
                self.pos = 0;
                self.state = self.state.next();
                self.increment_pos_and_idx()
            } else {
                self.pos = new_pos;
                true
            }
        } else {
            self.state = self.state.next();
            self.increment_pos_and_idx()
        }
    }

    /// This one should only be called when `self.state` is not finished!
    fn get_current(&self) -> Option<DirectionTuple<(usize, HashRef<T>), (usize, HashRef<T>)>> {
        match self.state {
            DictionaryIteratorPointerState::NextAB => {
                let b = *self.inner.map_a_to_b().get(self.pos)?.get(self.index)?;
                let a_value = (self.pos, self.inner.id_to_word::<A>(self.pos).unwrap().clone());
                let b_value = (b, self.inner.id_to_word::<B>(b).unwrap().clone());
                Some(if self.inner.map_b_to_a()[b].contains(&self.pos) {
                    DirectionTuple::Invariant {
                        a: a_value,
                        b: b_value
                    }
                } else {
                    DirectionTuple::AToB {
                        a: a_value,
                        b: b_value
                    }
                })
            }
            DictionaryIteratorPointerState::NextBA => {
                let a = *self.inner.map_b_to_a().get(self.pos)?.get(self.index)?;
                let a_value = (a, self.inner.id_to_word::<A>(a).unwrap().clone());
                let b_value = (self.pos, self.inner.id_to_word::<B>(self.pos).unwrap().clone());
                Some(if self.inner.map_a_to_b()[a].contains(&self.pos) {
                    DirectionTuple::Invariant {
                        a: a_value,
                        b: b_value
                    }
                } else {
                    DirectionTuple::BToA {
                        a: a_value,
                        b: b_value
                    }
                })
            }
            DictionaryIteratorPointerState::Finished => unreachable!()
        }
    }
}

impl<T, V, D> Iterator for DictionaryIteratorImpl<T, V, D> where D: DictionaryWithVocabulary<T, V>, V: Vocabulary<T> {
    type Item = DirectionTuple<(usize, HashRef<T>), (usize, HashRef<T>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.is_finished() {
            return None
        }
        loop {
            match self.get_current() {
                None => {
                    if !self.increment_pos_and_idx() {
                        break None;
                    }
                }
                Some(value) => {
                    self.increment_pos_and_idx();
                    break Some(value);
                }
            }
        }
    }
}

pub type DictionaryIterator<T, V> = Unique<DictionaryIteratorImpl<T, V, Dictionary<T, V>>>;

impl<T, V> IntoIterator for Dictionary<T, V> where V: Vocabulary<T>, T: Eq + Hash {
    type Item = DirectionTuple<(usize, HashRef<T>), (usize, HashRef<T>)>;
    type IntoIter = DictionaryIterator<T, V>;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryIteratorImpl::new(self).unique()
    }
}


pub mod direction {

    mod private {
        pub(crate) trait Sealed{}
    }

    /// A direction for a translation
    #[allow(private_bounds)]
    pub trait Direction: private::Sealed {
        const A2B: bool;
        const B2A: bool;
        const DIRECTION_NAME: &'static str;
    }

    ///
    #[allow(private_bounds)]
    pub trait Translation: Direction + private::Sealed {}

    #[allow(private_bounds)]
    pub trait Language: Translation + Direction + private::Sealed{
        const A: bool;
        const B: bool;
        const LANGUAGE_NAME: &'static str;
    }

    pub struct A;
    impl private::Sealed for A{}
    impl Language for A{
        const A: bool = true;
        const B: bool = false;
        const LANGUAGE_NAME: &'static str = "A";
    }

    pub type AToB = A;
    impl Direction for AToB {
        const A2B: bool = true;
        const B2A: bool = false;
        const DIRECTION_NAME: &'static str = "AToB";
    }
    impl Translation for AToB {}


    pub struct B;
    impl private::Sealed for B{}
    impl Language for B {
        const A: bool = false;
        const B: bool = true;
        const LANGUAGE_NAME: &'static str = "B";
    }
    pub type BToA = B;
    impl Direction for BToA {
        const A2B: bool = false;
        const B2A: bool = true;
        const DIRECTION_NAME: &'static str = "BToA";
    }
    impl Translation for BToA {}

    pub struct Invariant;
    impl private::Sealed for Invariant {}
    impl Direction for Invariant {
        const A2B: bool = true;
        const B2A: bool = true;
        const DIRECTION_NAME: &'static str = "Invariant";
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
        let tuple = self.iter.next()?;
        let a = *tuple.a();
        let b = *tuple.b();
        Some(
            match tuple {
                DirectionTuple::AToB { .. } => {
                    DirectionTuple::AToB {
                        a: (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        b: (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    }
                }
                DirectionTuple::BToA { .. } => {
                    DirectionTuple::BToA {
                        a: (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        b: (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    }
                }
                DirectionTuple::Invariant { .. } => {
                    DirectionTuple::Invariant {
                        a: (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        b: (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    }
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
            unsafe { meta.add_all_unstemmed(&unstemmed) }
        }
    }

    pub fn create_subset_with_filters<F1, F2>(&self, filter_a: F1, filter_b: F2) -> DictionaryWithMeta<T, V> where F1: Fn(&DictionaryWithMeta<T, V>, usize, Option<&MetadataRef>) -> bool, F2: Fn(&DictionaryWithMeta<T, V>, usize, Option<&MetadataRef>) -> bool {
        let mut new = Self {
            inner: Dictionary::new(),
            metadata: self.metadata.copy_keep_vocebulary()
        };
        for value in self.iter_with_meta() {
            match value {
                DirectionTuple::AToB { a:(word_id_a, meta_a), b:(word_id_b, meta_b) } => {
                    if filter_a(self, word_id_a, meta_a.as_ref()) {
                        if filter_b(self, word_id_b, meta_b.as_ref()) {
                            let word_a = self.inner.voc_a.get_value(word_id_a).unwrap();
                            let word_b = self.inner.voc_b.get_value(word_id_b).unwrap();
                            let (word_a, word_b) = new.insert_hash_ref::<AToB>(word_a.clone(), word_b.clone()).to_ab_tuple();
                            if let Some(meta_a) = meta_a {
                                new.insert_meta_for_create_subset::<A>(word_a, meta_a);
                            }
                            if let Some(meta_b) = meta_b {
                                new.insert_meta_for_create_subset::<B>(word_b, meta_b);
                            }
                        }
                    }
                }
                DirectionTuple::BToA { a:(word_id_a, meta_a), b:(word_id_b, meta_b) } => {
                    if filter_a(self, word_id_a, meta_a.as_ref()) {
                        if filter_b(self, word_id_b, meta_b.as_ref()) {
                            let word_a = self.inner.voc_a.get_value(word_id_a).unwrap();
                            let word_b = self.inner.voc_b.get_value(word_id_b).unwrap();
                            let (word_a, word_b) = new.insert_hash_ref::<BToA>(word_a.clone(), word_b.clone()).to_ab_tuple();
                            if let Some(meta_a) = meta_a {
                                new.insert_meta_for_create_subset::<A>(word_a, meta_a);
                            }
                            if let Some(meta_b) = meta_b {
                                new.insert_meta_for_create_subset::<B>(word_b, meta_b);
                            }
                        }
                    }
                },
                DirectionTuple::Invariant { a:(word_id_a, meta_a), b:(word_id_b, meta_b) } => {
                    if filter_a(self, word_id_a, meta_a.as_ref()) {
                        if filter_b(self, word_id_b, meta_b.as_ref()) {
                            let word_a = self.inner.voc_a.get_value(word_id_a).unwrap();
                            let word_b = self.inner.voc_b.get_value(word_id_b).unwrap();
                            let (word_a, word_b) = new.insert_hash_ref::<Invariant>(word_a.clone(), word_b.clone()).to_ab_tuple();
                            if let Some(meta_a) = meta_a {
                                new.insert_meta_for_create_subset::<A>(word_a, meta_a);
                            }
                            if let Some(meta_b) = meta_b {
                                new.insert_meta_for_create_subset::<B>(word_b, meta_b);
                            }
                        }
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

        for value in self.iter() {
            match value {
                DirectionTuple::AToB { a, b } => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionTuple::BToA { a, b } => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionTuple::Invariant { a, b } => {
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
        let mut new_dict = DictionaryWithMeta::new(
            Default::default(),
            self.metadata.copy_keep_vocebulary()
        );
        for value in self.iter() {
            let a = self.id_to_word::<A>(*value.a()).unwrap();
            let b = self.id_to_word::<B>(*value.b()).unwrap();
            match value {
                DirectionTuple::AToB { .. } => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionTuple::BToA { .. } => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionTuple::Invariant { .. } => {
                    if filter_a(a) && filter_b(b) {
                        new_dict.insert_hash_ref::<Invariant>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_b(b) {
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


pub struct DictionaryWithMetaIterator<D, T, V> where D: BasicDictionaryWithMeta + DictionaryWithVocabulary<T, V>, V: Vocabulary<T> {
    inner: DictionaryIteratorImpl<T, V, D>
}

impl<D, T, V> DictionaryWithMetaIterator<D, T, V> where D: BasicDictionaryWithMeta + DictionaryWithVocabulary<T, V>, V: Vocabulary<T>  {
    pub fn new(inner: D) -> Self {
        Self {
            inner: DictionaryIteratorImpl::new(inner)
        }
    }

    pub fn into_inner(self) -> D {
        self.inner.inner
    }
}

impl<D, T, V> Iterator for DictionaryWithMetaIterator<D, T, V> where D: BasicDictionaryWithMeta + DictionaryWithVocabulary<T, V>, V: Vocabulary<T> {
    type Item = DirectionTuple<(usize, HashRef<T>, Option<SolvedMetadata>), (usize, HashRef<T>, Option<SolvedMetadata>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;

        Some(
            next.map(
                |(id, href)| {
                    let value = self.inner.inner.metadata().get_meta_ref::<A>(id).map(MetadataRef::to_solved_metadata);
                    (id, href, value)
                },
                |(id, href)| {
                    let value = self.inner.inner.metadata().get_meta_ref::<B>(id).map(MetadataRef::to_solved_metadata);
                    (id, href, value)
                }
            )
        )
    }
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, DictionaryMut, DictionaryWithMeta, DictionaryWithVocabulary};
    use crate::topicmodel::dictionary::direction::{A, B, Invariant};
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
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airfoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),).to_ab_tuple();
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictE");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictC");
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("wing").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("deck").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),).to_ab_tuple();
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("hydrofoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),).to_ab_tuple();
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictC");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("foil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),).to_ab_tuple();
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictB");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictB");
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("bearing surface").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),).to_ab_tuple();
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
                     value.a().1, value.a().0, value.a().clone().2.map_or("NONE".to_string(), |value| value.to_string()),
                     value.b().1, value.b().0, value.b().clone().2.map_or("NONE".to_string(), |value| value.to_string())
            )
        }
    }
}
