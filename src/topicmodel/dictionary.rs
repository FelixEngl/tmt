#![allow(dead_code)]

use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::iter::{Chain, Cloned, Enumerate, FlatMap, Map};
use std::marker::PhantomData;
use std::slice::Iter;
use std::sync::Arc;
use itertools::{Itertools, Position};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use crate::toolkit::tupler::{SupportsTupling, TupleFirst, TupleLast};
use crate::topicmodel::dictionary::direction::{A, AToB, B, BToA, Direction, Language, Translation};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{MappableVocabulary, Vocabulary, VocabularyMut};
use string_interner::{DefaultStringInterner, DefaultSymbol as InternedString, DefaultSymbol};


use strum::EnumIs;#[macro_export]
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
        DictIter::new(self)
    }
}

#[derive(Debug, Copy, Clone, EnumIs)]
pub enum DirectionTuple<Ta, Tb> {
    AToB{
        a: Ta,
        b: Tb
    },
    BToA {
        a: Ta,
        b: Tb
    }
}

impl<Ta, Tb> DirectionTuple<Ta, Tb> {
    pub fn a(&self) -> &Ta {
        match self {
            DirectionTuple::AToB { a, .. } => {a}
            DirectionTuple::BToA { a, .. } => {a}
        }
    }
    pub fn b(&self) -> &Tb {
        match self {
            DirectionTuple::AToB { b, .. } => {b}
            DirectionTuple::BToA { b, .. } => {b}
        }
    }

    pub fn to_ab_tuple(self) -> (Ta, Tb) {
        match self {
            DirectionTuple::AToB { a, b } => {(a, b)}
            DirectionTuple::BToA { a, b } => {(a, b)}
        }
    }
}

/// Iterates over all mappings (a to b and b to a), does not filter for uniqueness.
pub struct DictIter<'a> {
    iter: Chain<ABIter<'a>, BAIter<'a>>,
}

type ABIter<'a> = FlatMap<Enumerate<Iter<'a, Vec<usize>>>, Map<TupleFirst<Cloned<Iter<'a, usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>, fn((usize, &Vec<usize>)) -> Map<TupleFirst<Cloned<Iter<usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>>;
type BAIter<'a> = FlatMap<Enumerate<Iter<'a, Vec<usize>>>, Map<TupleLast<Cloned<Iter<'a, usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>, fn((usize, &Vec<usize>)) -> Map<TupleLast<Cloned<Iter<usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>>;

impl<'a> DictIter<'a> {
    fn new(dict: &'a (impl BasicDictionary + ?Sized)) -> Self {
        let a_to_b: ABIter = dict.map_a_to_b().iter().enumerate().flat_map(|(a, value)| value.iter().cloned().tuple_first(a).map(|(a, b) | DirectionTuple::AToB {a, b}));
        let b_to_a: BAIter = dict.map_b_to_a().iter().enumerate().flat_map(|(b, value)| value.iter().cloned().tuple_last(b).map(|(a, b) | DirectionTuple::AToB {a, b}));
        let iter: Chain<ABIter, BAIter> = a_to_b.chain(b_to_a);
        Self { iter }
    }
}

impl<'a> Iterator for DictIter<'a> {
    type Item = DirectionTuple<usize, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
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
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> (usize, usize);

    fn insert_value<D: Direction>(&mut self, word_a: T, word_b: T) -> (usize, usize) {
        self.insert_hash_ref::<D>(HashRef::new(word_a), HashRef::new(word_b))
    }

    fn insert<D: Direction>(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) -> (usize, usize) {
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
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> (usize, usize) {
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
        }
        (id_a, id_b)
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
                    write!(f, "    - None -")?;
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

/// Iterates over all mappings (a to b and b to a), does not filter for uniqueness.
pub struct DictionaryIterator<T, V, D> where D: BasicDictionaryWithVocabulary<T, V> {
    pos: usize,
    index: usize,
    state: DictionaryIteratorPointerState,
    inner: D,
    _types: PhantomData<fn(T, V)->()>
}

impl<T, V, D> DictionaryIterator<T, V, D> where D: BasicDictionaryWithVocabulary<T, V> {
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
    fn get_current(&self) -> Option<(usize, usize)> {
        match self.state {
            DictionaryIteratorPointerState::NextAB => {
                Some((self.pos, *self.inner.map_a_to_b().get(self.pos)?.get(self.index)?))
            }
            DictionaryIteratorPointerState::NextBA => {
                Some((*self.inner.map_b_to_a().get(self.pos)?.get(self.index)?, self.pos))
            }
            DictionaryIteratorPointerState::Finished => unreachable!()
        }
    }
}

impl<T, V, D> Iterator for DictionaryIterator<T, V, D> where D: BasicDictionaryWithVocabulary<T, V> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.is_finished() {
            return None
        }
        loop {
            match self.get_current() {
                None => {
                    if self.increment_pos_and_idx() {

                    } else {
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

impl<T, V> IntoIterator for Dictionary<T, V> {
    type Item = (usize, usize);
    type IntoIter = DictionaryIterator<T, V, Dictionary<T, V>>;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryIterator::new(self)
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


#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Metadata {
    #[serde(with = "metadata_interned_field_serializer")]
    pub associated_dictionaries: OnceCell<Vec<InternedString>>,
    #[serde(with = "metadata_interned_field_serializer")]
    pub meta_tags: OnceCell<Vec<InternedString>>,
}

impl Metadata {
    pub fn has_associated_dictionary(&self, q: DefaultSymbol) -> bool {
        self.associated_dictionaries.get().is_some_and(|value| value.contains(&q))
    }

    pub fn has_meta_tag(&self, q: DefaultSymbol) -> bool {
        self.meta_tags.get().is_some_and(|value| value.contains(&q))
    }

    pub unsafe fn add_dictionary(&mut self, dictionary: InternedString) {
        if let Some(to_edit) = self.associated_dictionaries.get_mut() {
            if to_edit.is_empty() || !to_edit.contains(&dictionary) {
                to_edit.push(dictionary)
            }
        } else {
            let mut new = Vec::with_capacity(1);
            new.push(dictionary);
            self.associated_dictionaries.set(new).expect("This should be unset!");
        }
    }

    pub unsafe fn add_tag(&mut self, tag: InternedString) {
        if let Some(to_edit) = self.meta_tags.get_mut() {
            if to_edit.is_empty() || !to_edit.contains(&tag) {
                to_edit.push(tag)
            }
        } else {
            let mut new = Vec::with_capacity(1);
            new.push(tag);
            self.meta_tags.set(new).expect("This should be unset!");
        }
    }

    pub unsafe fn add_all_dictionary(&mut self, dictionaries: &[InternedString]) {
        if let Some(to_edit) = self.associated_dictionaries.get_mut() {
            let mut set = HashSet::with_capacity(dictionaries.len() + to_edit.len());
            set.extend(to_edit.drain(..));
            set.extend(dictionaries);
            to_edit.extend(set);
        } else {
            let mut new = Vec::with_capacity(dictionaries.len());
            new.extend(dictionaries.into_iter().unique());
            self.associated_dictionaries.set(new).expect("This should be unset!");
        }
    }

    pub unsafe fn add_all_tag(&mut self, tags: &[InternedString]) {
        if let Some(to_edit) = self.meta_tags.get_mut() {
            let mut set = HashSet::with_capacity(tags.len() + to_edit.len());
            set.extend(to_edit.drain(..));
            set.extend(tags);
            to_edit.extend(set);
        } else {
            let mut new = Vec::with_capacity(tags.len());
            new.extend(tags.into_iter().unique());
            self.meta_tags.set(new).expect("This should be unset!");
        }
    }
}

mod metadata_interned_field_serializer { use std::fmt::Formatter;
    use itertools::Itertools;
    use once_cell::sync::OnceCell;
    use serde::{Deserializer, Serialize, Serializer};
    use serde::de::{Error, Unexpected, Visitor};
    use string_interner::{DefaultSymbol, Symbol};
    use string_interner::symbol::SymbolU32;

    pub(crate) fn serialize<S>(target: &OnceCell<Vec<DefaultSymbol>>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let to_ser = if let Some(value) = target.get() {
            Some(value.iter().map(|value| value.to_usize()).collect_vec())
        } else {
            None
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
        let cell = OnceCell::new();
        let content: Option<Vec<usize>> = serde::de::Deserialize::deserialize(deserializer)?;
        if let Some(content) = content {
            cell.set(
                content.into_iter().map(|value|
                    DefaultSymbol::try_from_usize(value)
                        .ok_or_else(||
                            Error::invalid_value(
                                Unexpected::Unsigned(value as u64),
                                &SymbolU32Visitor
                            )
                        )
                ).collect::<Result<Vec<_>, _>>()?
            ).unwrap();
        }
        Ok(cell)
    }
}

pub struct MetadataMutRef<'a, D> where D: BasicDictionaryWithMeta + ?Sized {
    meta: &'a mut Metadata,
    // always outlifes meta
    dict_ref: *mut D
}

impl<'a, D> MetadataMutRef<'a, D>  where D: BasicDictionaryWithMeta + ?Sized {
    fn new(dict_ref: *mut D, meta: &'a mut Metadata) -> Self {
        Self { meta, dict_ref }
    }

    pub fn push_associated_dictionary(&mut self, dictionary: impl AsRef<str>) {
        let interned = unsafe{&mut *self.dict_ref}.get_dictionary_interner_mut().get_or_intern(dictionary);
        unsafe {
            self.meta.add_dictionary(interned);
        }
    }

    pub fn push_meta_tag(&mut self, tag: impl AsRef<str>) {
        let interned = unsafe{&mut *self.dict_ref}.get_tag_interner_mut().get_or_intern(tag);
        unsafe {
            self.meta.add_tag(interned);
        }
    }
}



pub struct MetadataRef<'a, D> where D: BasicDictionaryWithMeta + ?Sized {
    raw: &'a Metadata,
    dict: &'a D,
    associated_dictionary_cached: Arc<OnceCell<Vec<&'a str>>>,
    meta_tags_cached: Arc<OnceCell<Vec<&'a str>>>,
}

impl<'a, D: BasicDictionaryWithMeta> MetadataRef<'a, D> where D: BasicDictionaryWithMeta + ?Sized {

    pub fn new(raw: &'a Metadata, dict: &'a D) -> Self {
        Self {
            raw,
            dict,
            associated_dictionary_cached: Default::default(),
            meta_tags_cached: Default::default()
        }
    }

    pub fn raw(&self) -> &'a Metadata {
        self.raw
    }

    pub fn dict(&self) -> &'a D {
        self.dict
    }

    pub fn has_associated_dictionary(&self, q: impl AsRef<str>) -> bool {
        self.dict.get_dictionary_interner().get(q).is_some_and(|value| self.raw.has_associated_dictionary(value))
    }

    pub fn has_meta_tag(&self, q: impl AsRef<str>) -> bool {
        self.dict.get_tag_interner().get(q).is_some_and(|value| self.raw.has_meta_tag(value))
    }

    pub fn associated_dictionaries(&self) -> Option<&Vec<&'a str>> {
        if let Some(found) = self.associated_dictionary_cached.get() {
            Some(found)
        } else {
            if let Some(inner) = self.raw.associated_dictionaries.get() {
                let interner = self.dict.get_dictionary_interner();
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
                let interner = self.dict.get_tag_interner();
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
}

impl<D> Debug for MetadataRef<'_, D> where D: BasicDictionaryWithMeta + Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataRef")
            .field("inner", self.raw)
            .field("associated_dictionary_cached", &self.associated_dictionary_cached.get())
            .field("meta_tags_cached", &self.meta_tags_cached.get())
            .finish_non_exhaustive()
    }
}

impl<'a, D> Clone for MetadataRef<'a, D>  where D: BasicDictionaryWithMeta {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            dict: self.dict,
            associated_dictionary_cached: self.associated_dictionary_cached.clone(),
            meta_tags_cached: self.meta_tags_cached.clone()
        }
    }
}

impl<D> Display for MetadataRef<'_, D> where D: BasicDictionaryWithMeta {
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

pub trait BasicDictionaryWithMeta: BasicDictionary {
    fn get_dictionary_interner(&self) -> &DefaultStringInterner;
    fn get_dictionary_interner_mut(&mut self) -> &mut DefaultStringInterner;

    fn get_tag_interner(&self) -> &DefaultStringInterner;
    fn get_tag_interner_mut(&mut self) -> &mut DefaultStringInterner;

    fn set_dictionary_for<L: Language>(&mut self, word_id: usize, dict: &str) {
        self.get_or_init_meta::<L>(word_id).push_associated_dictionary(dict)
    }

    fn set_meta_tag_for<L: Language>(&mut self, word_id: usize, tag: &str) {
        self.get_or_init_meta::<L>(word_id).push_meta_tag(tag)
    }


    fn get_meta<L: Language>(&self, word_id: usize) -> Option<&Metadata>;
    fn get_meta_mut<L: Language>(&mut self, word_id: usize) -> Option<MetadataMutRef<Self>>;
    fn get_or_init_meta<L: Language>(&mut self, word_id: usize) -> MetadataMutRef<Self>;

    fn get_meta_ref<L: Language>(&self, word_id: usize) -> Option<MetadataRef<Self>> {
        Some(MetadataRef::new(self.get_meta::<L>(word_id)?, self))
    }

    fn iter_with_meta(&self) -> DictionaryWithMetaIter<Self> {
        DictionaryWithMetaIter::new(self)
    }

    fn reserve_meta(&mut self);
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
    type Item = DirectionTuple<(usize, Option<MetadataRef<'a, D>>), (usize, Option<MetadataRef<'a, D>>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let tuple = self.iter.next()?;
        let a = *tuple.a();
        let b = *tuple.b();
        Some(
            match tuple {
                DirectionTuple::AToB { .. } => {
                    DirectionTuple::AToB {
                        a: (a, self.dictionary_with_meta.get_meta_ref::<A>(a)),
                        b: (b, self.dictionary_with_meta.get_meta_ref::<B>(b))
                    }
                }
                DirectionTuple::BToA { .. } => {
                    DirectionTuple::BToA {
                        a: (a, self.dictionary_with_meta.get_meta_ref::<A>(a)),
                        b: (b, self.dictionary_with_meta.get_meta_ref::<B>(b))
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
    dictionary_interner: DefaultStringInterner,
    tag_interner: DefaultStringInterner,
    meta_a: Vec<Metadata>,
    meta_b: Vec<Metadata>,
}
unsafe impl<T, V> Send for DictionaryWithMeta<T, V>{}
unsafe impl<T, V> Sync for DictionaryWithMeta<T, V>{}
impl<T, V> DictionaryWithMeta<T, V> where V: VocabularyMut<T> + Default, T: Hash + Eq  {

    fn insert_meta_for_create_subset<'a, L: Language>(&mut self, word_id: usize, metadata_ref: MetadataRef<'a, DictionaryWithMeta<T, V>>) {
        let tags = metadata_ref.raw.meta_tags.get();
        let dics = metadata_ref.raw.associated_dictionaries.get();

        if tags.is_none() && dics.is_none() {
            return;
        }

        let meta = self.get_or_init_meta::<L>(word_id).meta;

        if let Some(dics) = dics {
            unsafe { meta.add_all_dictionary(&dics) }
        }
        if let Some(tags) = tags {
            unsafe { meta.add_all_dictionary(&tags) }
        }
    }

    pub fn create_subset_with_filters<F1, F2>(&self, filter_a: F1, filter_b: F2) -> DictionaryWithMeta<T, V> where F1: Fn(&DictionaryWithMeta<T, V>, usize, Option<&MetadataRef<DictionaryWithMeta<T, V>>>) -> bool, F2: Fn(&DictionaryWithMeta<T, V>, usize, Option<&MetadataRef<DictionaryWithMeta<T, V>>>) -> bool {
        let mut new = Self {
            inner: Dictionary::new(),
            dictionary_interner: self.dictionary_interner.clone(),
            tag_interner: self.tag_interner.clone(),
            meta_a: Default::default(),
            meta_b: Default::default(),
        };
        for value in self.iter_with_meta() {


            match value {
                DirectionTuple::AToB { a:(word_id_a, meta_a), b:(word_id_b, meta_b) } => {
                    if filter_a(self, word_id_a, meta_a.as_ref()) {
                        if filter_b(self, word_id_b, meta_b.as_ref()) {
                            let word_a = self.inner.voc_a.get_value(word_id_a).unwrap();
                            let word_b = self.inner.voc_b.get_value(word_id_b).unwrap();
                            let (word_a, word_b) = new.insert_hash_ref::<AToB>(word_a.clone(), word_b.clone());
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
                            let (word_a, word_b) = new.insert_hash_ref::<BToA>(word_a.clone(), word_b.clone());
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
        Self {
            inner: Dictionary::from_voc_a(voc_a),
            dictionary_interner: Default::default(),
            tag_interner: Default::default(),
            meta_a: Default::default(),
            meta_b: Default::default(),
        }
    }
}

impl<T, V> DictionaryWithMeta<T, V> where V: Vocabulary<T> {
    pub fn from_voc(voc_a: V, voc_b: V) -> Self {
        Self {
            inner: Dictionary::from_voc(voc_a, voc_b),
            dictionary_interner: Default::default(),
            tag_interner: Default::default(),
            meta_a: Default::default(),
            meta_b: Default::default(),
        }
    }
}

impl<T, V> DictionaryWithMeta<T, V> where V: Default {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            dictionary_interner: Default::default(),
            tag_interner: Default::default(),
            meta_a: Default::default(),
            meta_b: Default::default(),
        }
    }
}
impl<T, V> Clone for DictionaryWithMeta<T, V> where V: Clone {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            dictionary_interner: self.dictionary_interner.clone(),
            tag_interner: self.tag_interner.clone(),
            meta_a: self.meta_a.clone(),
            meta_b: self.meta_b.clone()
        }
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
    fn get_dictionary_interner(&self) -> &DefaultStringInterner {
        &self.dictionary_interner
    }

    fn get_dictionary_interner_mut(&mut self) -> &mut DefaultStringInterner {
        &mut self.dictionary_interner
    }

    fn get_tag_interner(&self) -> &DefaultStringInterner {
        &self.tag_interner
    }

    fn get_tag_interner_mut(&mut self) -> &mut DefaultStringInterner {
        &mut self.tag_interner
    }

    fn get_meta<L: Language>(&self, word_id: usize) -> Option<&Metadata> {
        if L::A2B {
            self.meta_a.get(word_id)
        } else {
            self.meta_b.get(word_id)
        }
    }

    fn get_meta_mut<L: Language>(&mut self, word_id: usize) -> Option<MetadataMutRef<Self>> {
        let ptr = self as *mut Self;
        let value = unsafe{&mut*ptr};
        let result = if L::A2B {
            value.meta_a.get_mut(word_id)
        } else {
            value.meta_b.get_mut(word_id)
        }?;
        Some(MetadataMutRef::new(ptr, result))
    }

    fn get_or_init_meta<L: Language>(&mut self, word_id: usize) -> MetadataMutRef<Self> {
        let ptr = self as *mut Self;

        let targ = if L::A2B {
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

    fn reserve_meta(&mut self) {
        self.meta_a.resize(self.inner.voc_a.len(), Metadata::default());
        self.meta_b.resize(self.inner.voc_b.len(), Metadata::default());
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
        DictionaryWithMeta::<Q, Voc> {
            inner: self.inner.map(f),
            dictionary_interner: self.dictionary_interner,
            tag_interner: self.tag_interner,
            meta_a: self.meta_a,
            meta_b: self.meta_b,
        }
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
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> (usize, usize) {
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
impl<T: Display, V: Vocabulary<T>> Display for DictionaryWithMeta<T, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)?;
        write!(f, "\n------\n")?;
        write!(f, "Metadata A:\n")?;
        if self.meta_a.is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in self.inner.voc_a.ids() {
                if let Some(value) = self.get_meta_ref::<A>(word_id) {
                    write!(f, "    {}: {}\n", self.id_to_word::<AToB>(word_id).unwrap(), value)?;
                }
            }
        }

        write!(f, "\n------\n")?;
        write!(f, "Metadata B:\n")?;
        if self.meta_b.is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in self.inner.voc_b.ids() {
                if let Some(value) = self.get_meta_ref::<B>(word_id) {
                    write!(f, "    {}: {}\n", self.id_to_word::<BToA>(word_id).unwrap(), value)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, DictionaryMut, DictionaryWithMeta};
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
            dict.reserve_meta();
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
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airfoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictE");
            drop(meta_a);
            let mut meta_b = dict.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictC");
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("wing").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("deck").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("hydrofoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictC");
            drop(meta_a);
            let mut meta_b = dict.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("foil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictB");
            drop(meta_a);
            let mut meta_b = dict.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictB");
            let (a, b) = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("bearing surface").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");

            drop(meta_b);
            let mut meta_a = dict.get_or_init_meta::<A>(0);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictB");
        }

        println!("{}", dict);

        let mut result = dict.create_subset_with_filters(
            |dict, v, meaning| {
                if let Some(found) = meaning {
                    found.has_associated_dictionary("DictA") || found.has_associated_dictionary("DictB")
                } else {
                    false
                }
            },
            |dict, v, meaning| { true }
        );
        result.reserve_meta();
        println!(".=======.");
        println!("{}", result);
    }
}
