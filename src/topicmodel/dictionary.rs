#![allow(dead_code)]

use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::iter::Enumerate;
use std::marker::PhantomData;
use itertools::{Itertools, Position};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::direction::{A, B, Direction, Language, Translation};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{MappableVocabulary, Vocabulary, VocabularyMut};

#[macro_export]
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
    () => {DictionaryImpl::<_, $crate::topicmodel::vocabulary::VocabularyImpl<_>>::new()};
    ($($a:tt $op:tt $b:tt)+) => {
        {
            let mut __dict = DictionaryImpl::<_, $crate::topicmodel::vocabulary::VocabularyImpl<_>>::new();
            $(
                $crate::dict_insert!(__dict, $a $op $b);
            )+
            __dict
        }
    }
}

pub trait Dictionary<T, V>: Send + Sync {
    fn voc_a(&self) -> &V;

    fn voc_b(&self) -> &V;

    fn map_a_to_b(&self) -> &Vec<Vec<usize>>;

    fn map_b_to_a(&self) -> &Vec<Vec<usize>>;
}

pub trait DictionaryWithVoc<T, V>: Dictionary<T, V> where V: Vocabulary<T> {
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        if D::A2B {
            self.voc_a().contains_id(id) && self.map_a_to_b().get(id).is_some_and(|value| !value.is_empty())
        } else {
            self.voc_b().contains_id(id) && self.map_b_to_a().get(id).is_some_and(|value| !value.is_empty())
        }
    }

    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        if D::A2B {
            self.map_a_to_b().get(word_id)
        } else {
            self.map_b_to_a().get(word_id)
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

    fn iter<'a, L: Language>(&'a self) -> DictIter<'a, T, L, Self, V> where V: 'a {
        DictIter::<T, L, Self, V>::new(self)
    }
}

pub trait DictionaryMut<T, V>: DictionaryWithVoc<T, V> where T: Eq + Hash, V: VocabularyMut<T> {
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>);

    fn insert_value<D: Direction>(&mut self, word_a: T, word_b: T) {
        self.insert_hash_ref::<D>(HashRef::new(word_a), HashRef::new(word_b))
    }

    fn insert<D: Direction>(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) {
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
pub struct DictionaryImpl<T, V> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    voc_a: V,
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    voc_b: V,
    map_a_to_b: Vec<Vec<usize>>,
    map_b_to_a: Vec<Vec<usize>>,
    _word_type: PhantomData<T>
}

unsafe impl<T, V> Send for DictionaryImpl<T, V>{}
unsafe impl<T, V> Sync for DictionaryImpl<T, V>{}

impl<T, V> Clone for DictionaryImpl<T, V> where V: Clone {
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

impl<T, V> Dictionary<T, V> for DictionaryImpl<T, V> {
    fn voc_a(&self) -> &V {
        &self.voc_a
    }

    fn voc_b(&self) -> &V {
        &self.voc_b
    }

    fn map_a_to_b(&self) -> &Vec<Vec<usize>> {
        &self.map_a_to_b
    }

    fn map_b_to_a(&self) -> &Vec<Vec<usize>> {
        &self.map_b_to_a
    }
}

impl<T, V> Default for DictionaryImpl<T, V> where V: Default {
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

impl<T, V> DictionaryImpl<T, V> where V: Vocabulary<T> + Default {
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

impl<T, V> DictionaryWithVoc<T, V> for  DictionaryImpl<T, V> where V: Vocabulary<T> {
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        if D::A2B {
            self.voc_a.contains_id(id) && self.map_a_to_b.get(id).is_some_and(|value| !value.is_empty())
        } else {
            self.voc_b.contains_id(id) && self.map_b_to_a.get(id).is_some_and(|value| !value.is_empty())
        }
    }

    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        if D::A2B {
            &self.map_a_to_b
        } else {
            &self.map_b_to_a
        }.get(word_id)
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

impl<T, V> DictionaryImpl<T, V> where V: Default {
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

pub struct DictIter<'a, T, L, D: ?Sized, V> where L: Language {
    iter: Enumerate<std::slice::Iter<'a, HashRef<T>>>,
    dict: &'a D,
    _language: PhantomData<(L, V)>
}

impl<'a, T, L, D: ?Sized, V> DictIter<'a, T, L, D, V> where L: Language, V: Vocabulary<T> + 'a, D: Dictionary<T, V> {
    fn new(dict: &'a D) -> Self {
        Self {
            iter: if L::TranslationDirection::A2B {
                dict.voc_a().iter().enumerate()
            } else {
                dict.voc_b().iter().enumerate()
            },
            dict,
            _language: PhantomData
        }
    }
}

impl<'a, T, L, D, V> Iterator for DictIter<'a, T, L, D, V> where L: Language, V: Vocabulary<T> + 'a, D: DictionaryWithVoc<T, V> {
    type Item = (usize, &'a HashRef<T>, Option<Vec<(usize, &'a HashRef<T>)>>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, next) = self.iter.next()?;
        let translation = self.dict.translate_id::<L::TranslationDirection>(id);
        Some((id, next, translation))
    }
}

impl<T, V> DictionaryMut<T, V> for  DictionaryImpl<T, V> where T: Eq + Hash, V: VocabularyMut<T> {
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) {
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

impl<T, V> DictionaryImpl<T, V> where T: Eq + Hash, V: MappableVocabulary<T> {
    pub fn map<Q: Eq + Hash, Voc, F>(self, f: F) -> DictionaryImpl<Q, Voc> where F: for<'a> Fn(&'a T)-> Q, Voc: From<Vec<Q>> {
        DictionaryImpl {
            map_a_to_b: self.map_a_to_b,
            map_b_to_a: self.map_b_to_a,
            voc_a: self.voc_a.map(&f),
            voc_b: self.voc_b.map(f),
            _word_type: PhantomData
        }
    }
}

impl<T: Display, V: Vocabulary<T>> Display for DictionaryImpl<T, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn write_language<L: Language, T: Display, V: Vocabulary<T>>(dictionary: &DictionaryImpl<T, V>, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:\n", L::NAME)?;
            for (position_a, (id_a, value_a, translations)) in dictionary.iter::<L>().with_position() {
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

pub mod direction {
    mod private {
        pub(crate) trait Sealed{}
    }

    pub trait Language: private::Sealed{
        type TranslationDirection: Translation;
        const NAME: &'static str;
    }

    pub struct A;
    impl private::Sealed for A{}
    impl Language for A{
        type TranslationDirection = AToB;
        const NAME: &'static str = "A";
    }


    pub struct B;
    impl private::Sealed for B{}
    impl Language for B{
        type TranslationDirection = BToA;
        const NAME: &'static str = "B";
    }


    pub trait Direction: private::Sealed{
        const A2B: bool;
        const B2A: bool;
        const NAME: &'static str;
    }

    pub trait Translation: Direction + private::Sealed {}

    pub struct AToB;
    impl private::Sealed for AToB{}
    impl Direction for AToB {
        const A2B: bool = true;
        const B2A: bool = false;
        const NAME: &'static str = "AToB";
    }

    impl Translation for AToB {}

    pub struct BToA;
    impl private::Sealed for BToA{}
    impl Direction for BToA {
        const A2B: bool = false;
        const B2A: bool = true;
        const NAME: &'static str = "BToA";
    }
    impl Translation for BToA {}

    pub struct Invariant;
    impl private::Sealed for Invariant {}
    impl Direction for Invariant {
        const A2B: bool = true;
        const B2A: bool = true;
        const NAME: &'static str = "Invariant";
    }

}

