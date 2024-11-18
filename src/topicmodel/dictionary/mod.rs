//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

#![allow(dead_code)]

pub mod metadata;
pub mod direction;
pub mod iterators;
mod traits;
mod loader;
pub mod io;
pub mod len;
pub mod hacks;
pub mod search;

use std::borrow::Borrow;
pub use loader::*;
pub use traits::*;

pub use metadata::dictionary::*;
use crate::toolkit::once_serializer::OnceLockDef;


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
    () => {$crate::topicmodel::Dictionary::<_, $crate::topicmodel::vocabulary::Vocabulary<_>>::new()};
    (for $lang_a: tt, $lang_b: tt;) => {$crate::topicmodel::dictionary::Dictionary::<_, $crate::topicmodel::vocabulary::Vocabulary<_>>::new_with(Some($lang_a), Some($lang_b))};

    (for $lang_a: tt, $lang_b: tt: $($a:tt $op:tt $b:tt)+) => {
        {
            let mut __dict = $crate::topicmodel::dictionary::Dictionary::<_, $crate::topicmodel::vocabulary::Vocabulary<_>>::new_with(Some($lang_a), Some($lang_b));
            $(
                $crate::dict_insert!(__dict, $a $op $b);
            )+
            __dict
        }
    };

    ($($a:tt $op:tt $b:tt)+) => {
        {
            let mut __dict = $crate::topicmodel::dictionary::Dictionary::<_, $crate::topicmodel::vocabulary::Vocabulary<_>>::default();
            $(
                $crate::dict_insert!(__dict, $a $op $b);
            )+
            __dict
        }
    }
}



use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::OnceLock;
use itertools::{Itertools, Position};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::direction::{AToB, BToA, DirectionKind, DirectionTuple, Invariant, Language, A, B};
use crate::topicmodel::dictionary::search::{DictionarySearcher, SearchIndex};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{BasicVocabulary, MappableVocabulary, SearchableVocabulary, VocabularyMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary<T, V> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    pub(crate) voc_a: V,
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    pub(crate) voc_b: V,
    pub(crate) map_a_to_b: Vec<Vec<usize>>,
    pub(crate) map_b_to_a: Vec<Vec<usize>>,
    #[serde(with = "OnceLockDef", skip_serializing_if = "index_not_initialized", default)]
    pub(crate) search_index: OnceLock<SearchIndex>,
    _word_type: PhantomData<T>
}

fn index_not_initialized(cell: &OnceLock<SearchIndex>) -> bool {
    cell.get().is_none()
}

unsafe impl<T, V> Send for Dictionary<T, V>{}
unsafe impl<T, V> Sync for Dictionary<T, V>{}

impl<T, V> FromVoc<T, V> for Dictionary<T, V> where V: BasicVocabulary<T> + Default, T: Hash + Eq  {
    fn from_voc(voc_a: V, voc_b: V) -> Self {
        let mut map_a_to_b = Vec::new();
        map_a_to_b.resize_with(voc_a.len(), || Vec::with_capacity(1));
        let mut map_b_to_a = Vec::new();
        map_b_to_a.resize_with(voc_b.len(), || Vec::with_capacity(1));

        Self::new(
            voc_a,
            map_a_to_b,
            voc_b,
            map_b_to_a,
        )
    }

    fn from_voc_lang_a(voc: V, other_lang: Option<LanguageHint>) -> Self {
        let mut map_a_to_b = Vec::new();
        map_a_to_b.resize_with(voc.len(), || Vec::with_capacity(1));
        Self::new(
            voc,
            map_a_to_b,
            V::create(other_lang),
            Default::default(),
        )
    }

    fn from_voc_lang_b(other_lang: Option<LanguageHint>, voc: V) -> Self {
        let mut map_b_to_a = Vec::new();
        map_b_to_a.resize_with(voc.len(), || Vec::with_capacity(1));
        Self::new(
            V::create(other_lang),
            Default::default(),
            voc,
            map_b_to_a,
        )
    }
}

impl<T, V> Dictionary<T, V> where V: From<Option<LanguageHint>>  {
    pub fn new_with(language_a: Option<impl Into<LanguageHint>>, language_b: Option<impl Into<LanguageHint>>) -> Self {
        Self::new(
            language_a.map(|value| value.into()).into(),
            Default::default(),
            language_b.map(|value| value.into()).into(),
            Default::default()
        )
    }
}

impl<T, V> Dictionary<T, V> {
    pub fn new(voc_a: V, map_a_to_b: Vec<Vec<usize>>, voc_b: V, map_b_to_a: Vec<Vec<usize>>) -> Self {
        Self { voc_a, voc_b, map_a_to_b, map_b_to_a, search_index: Default::default(), _word_type: PhantomData }
    }
}

impl<T, V> Clone for Dictionary<T, V> where V: Clone {
    fn clone(&self) -> Self {
        Self::new(
            self.voc_a.clone(),
            self.map_a_to_b.clone(),
            self.voc_b.clone(),
            self.map_b_to_a.clone()
        )
    }
}

impl<T, V> Default for Dictionary<T, V> where V: Default {
    fn default() -> Self {
        Self::new(
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        )
    }
}

impl<T, V> BasicDictionary for Dictionary<T, V> {
    fn map_a_to_b(&self) -> &Vec<Vec<usize>> {
        &self.map_a_to_b
    }

    fn map_b_to_a(&self) -> &Vec<Vec<usize>> {
        &self.map_b_to_a
    }

    fn switch_languages(self) -> Self where Self: Sized {
        Self::new(
            self.voc_b,
            self.map_b_to_a,
            self.voc_a,
            self.map_a_to_b
        )
    }
}

impl<T, V> BasicDictionaryWithVocabulary<V> for Dictionary<T, V> {
    fn voc_a(&self) -> &V {
        &self.voc_a
    }

    fn voc_b(&self) -> &V {
        &self.voc_b
    }

    fn voc_a_mut(&mut self) -> &mut V {
        &mut self.voc_a
    }

    fn voc_b_mut(&mut self) -> &mut V {
        &mut self.voc_b
    }
}

impl<T, V> Dictionary<T, V> where T: Eq + Hash, V: MappableVocabulary<T> {
    pub fn map<Q: Eq + Hash, Voc, F>(self, f: F) -> Dictionary<Q, Voc> where F: for<'a> Fn(&'a T)-> Q, Voc: BasicVocabulary<Q> {
        Dictionary::new(
            self.voc_a.map(&f),
            self.map_a_to_b,
            self.voc_b.map(f),
            self.map_b_to_a
        )
    }
}

impl<T, V> DictionaryWithVocabulary<T, V> for Dictionary<T, V> where V: BasicVocabulary<T> {}

impl<T, V> MergingDictionary<T, V> for Dictionary<T, V> where T: Eq + Hash, V: VocabularyMut<T> + Extend<T> {
    fn merge(mut self, other: impl Into<Self>) -> Self
    where
        Self: Sized
    {
        let other = other.into();

        for DirectionTuple{a, b, direction} in other.iter() {
            unsafe {
                match direction {
                    DirectionKind::AToB => {
                        self.insert_hash_ref::<AToB>(
                            other.voc_a().get_value_unchecked(a).clone(),
                            other.voc_b().get_value_unchecked(b).clone(),
                        );
                    }
                    DirectionKind::BToA => {
                        self.insert_hash_ref::<BToA>(
                            other.voc_a().get_value_unchecked(a).clone(),
                            other.voc_b().get_value_unchecked(b).clone(),
                        );
                    }
                    DirectionKind::Invariant => {
                        self.insert_hash_ref::<Invariant>(
                            other.voc_a().get_value_unchecked(a).clone(),
                            other.voc_b().get_value_unchecked(b).clone(),
                        );
                    }
                }
            }
        }

        self.voc_a.add_all_hash_ref(other.voc_a);
        self.voc_b.add_all_hash_ref(other.voc_b);

        self
    }
}

impl<T, V> DictionaryMut<T, V> for  Dictionary<T, V> where T: Eq + Hash, V: VocabularyMut<T> {

    unsafe fn reserve_for_single_value_a(&mut self, word_id: usize) {
        if self.map_a_to_b.len() <= word_id {
            self.map_a_to_b.resize_with(word_id+1, || Vec::with_capacity(1));
        }
    }

    unsafe fn reserve_for_single_value_b(&mut self, word_id: usize) {
        if self.map_b_to_a.len() <= word_id {
            self.map_b_to_a.resize_with(word_id+1, || Vec::with_capacity(1));
        }
    }

    unsafe fn insert_raw_values_a_to_b(&mut self, id_a: usize, id_b: usize) {
        if let Some(found) = self.map_a_to_b.get_mut(id_a) {
            if !found.contains(&id_b) {
                found.push(id_b)
            }
        } else {
            self.reserve_for_single_value_a(id_a);
            unsafe { self.map_a_to_b.get_unchecked_mut(id_a).push(id_b); }
        }
    }

    unsafe fn insert_raw_values_b_to_a(&mut self, id_a: usize, id_b: usize) {
        if let Some(found) = self.map_b_to_a.get_mut(id_b) {
            if !found.contains(&id_a) {
                found.push(id_a)
            }
        } else {
            self.reserve_for_single_value_b(id_b);
            unsafe { self.map_b_to_a.get_unchecked_mut(id_b).push(id_a); }
        }
    }

    fn delete_translations_of_word_a<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>,
    {
        match self.voc_a.get_id(value) {
            None => false,
            Some(id) => {
                if let Some(target) = self.map_a_to_b.get_mut(id) {
                    target.clear();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn delete_translations_of_word_b<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        match self.voc_b.get_id(value) {
            None => false,
            Some(id) => {
                if let Some(target) = self.map_b_to_a.get_mut(id) {
                    target.clear();
                    true
                } else {
                    false
                }
            }
        }
    }
}
impl<T, V> DictionaryFilterable<T, V>  for Dictionary<T, V> where T: Eq + Hash, V: VocabularyMut<T> + Default  {
    fn filter_and_process<'a, Fa, Fb, E>(&'a self, f_a: Fa, f_b: Fb) -> Result<Self, E>
    where
        Self: Sized,
        T: 'a,
        Fa: Fn(&'a HashRef<T>) -> Result<Option<HashRef<T>>, E>,
        Fb: Fn(&'a HashRef<T>) -> Result<Option<HashRef<T>>, E>
    {
        let mut new_dict = Dictionary::default();
        for DirectionTuple{a, b, direction} in self.iter() {
            if let Some(a) = f_a(self.convert_id_a_to_word(a).unwrap())? {
                if let Some(b) = f_b(self.convert_id_b_to_word(b).unwrap())? {
                    match direction {
                        DirectionKind::AToB => {
                            new_dict.insert_hash_ref::<AToB>(
                                a,
                                b
                            );
                        }
                        DirectionKind::BToA => {
                            new_dict.insert_hash_ref::<BToA>(
                                a,
                                b
                            );
                        }
                        DirectionKind::Invariant => {
                            new_dict.insert_hash_ref::<Invariant>(
                                a,
                                b
                            );
                        }
                    }
                }
            }
        }

        Ok(new_dict)
    }

    //noinspection DuplicatedCode
    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized {
        let mut new_dict = Dictionary::default();

        for DirectionTuple{a, b, direction} in self.iter() {
            match direction {
                DirectionKind::AToB => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            self.convert_id_a_to_word(a).unwrap().clone(),
                            self.convert_id_b_to_word(b).unwrap().clone()
                        );
                    }
                }
                DirectionKind::BToA => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            self.convert_id_a_to_word(a).unwrap().clone(),
                            self.convert_id_b_to_word(b).unwrap().clone()
                        );
                    }
                }
                DirectionKind::Invariant => {
                    if filter_a(a) && filter_b(b) {
                        new_dict.insert_hash_ref::<Invariant>(
                            self.convert_id_a_to_word(a).unwrap().clone(),
                            self.convert_id_b_to_word(b).unwrap().clone()
                        );
                    } else if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            self.convert_id_a_to_word(a).unwrap().clone(),
                            self.convert_id_b_to_word(b).unwrap().clone()
                        );
                    } else if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            self.convert_id_a_to_word(a).unwrap().clone(),
                            self.convert_id_b_to_word(b).unwrap().clone()
                        );
                    }
                }
            }
        }

        new_dict
    }

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<T>) -> bool, Fb: Fn(&'a HashRef<T>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, T: 'a {
        let mut new_dict = Dictionary::default();
        for DirectionTuple{a, b, direction} in self.iter() {
            let a = self.convert_id_a_to_word(a).unwrap();
            let b = self.convert_id_b_to_word(b).unwrap();
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

impl<T: Display, V: BasicVocabulary<T>> Display for Dictionary<T, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn write_language<L: Language, T: Display, V: BasicVocabulary<T>>(dictionary: &Dictionary<T, V>, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:\n", dictionary.language::<L>().map_or_else(|| L::LANG.to_string(), |value| value.to_string()))?;
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


impl<V> DictionaryWithSearch<String, V> for Dictionary<String, V>
where
    V: BasicVocabulary<String>
{
    fn get_searcher(&self) -> DictionarySearcher<Self, V> {
        let index = self.search_index.get_or_init(SearchIndex::new);
        DictionarySearcher::new(self, index)
    }
}







#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMetaGen, DictionaryMutGen, DictionaryWithMeta, DictionaryWithVocabulary, FromVoc};
    use crate::topicmodel::dictionary::direction::{DirectionTuple, Invariant, A, B};
    use crate::topicmodel::dictionary::metadata::classic::ClassicMetadataManager;
    use crate::topicmodel::dictionary::metadata::classic::python::SolvedMetadata;
    use crate::topicmodel::vocabulary::{SearchableVocabulary, Vocabulary};

    #[test]
    fn can_create_with_meta(){
        let mut voc_a = Vocabulary::<String>::default();
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
        let mut voc_b = Vocabulary::<String>::default();
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

        let mut dict: DictionaryWithMeta<_, _, ClassicMetadataManager> = DictionaryWithMeta::from_voc(voc_a.clone(), voc_b.clone());
        {
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
            let mut meta_a = dict.get_or_create_meta_for::<A>(a);
            meta_a.push_associated_dictionary("DictE");
            drop(meta_a);
            let mut meta_b = dict.get_or_create_meta_for::<B>(b);
            meta_b.push_associated_dictionary("DictC");
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("wing").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("deck").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_create_meta_for::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.get_or_create_meta_for::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("hydrofoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_create_meta_for::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictC");
            drop(meta_a);
            let mut meta_b = dict.get_or_create_meta_for::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("foil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_create_meta_for::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictB");
            drop(meta_a);
            let mut meta_b = dict.get_or_create_meta_for::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictB");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("bearing surface").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.get_or_create_meta_for::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.get_or_create_meta_for::<B>(b);
            meta_b.push_associated_dictionary("DictA");

            drop(meta_b);
            let mut meta_a = dict.get_or_create_meta_for::<A>(0);
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
                    format!("'{}({id})': {}", dict.convert_id_a_to_word(id).unwrap().to_string(), meta.map_or("NONE".to_string(), |value| SolvedMetadata::from(value).to_string()))
                },
                |(id, meta)| {
                    format!("'{}({id})': {}", dict.convert_id_b_to_word(id).unwrap().to_string(), meta.map_or("NONE".to_string(), |value| SolvedMetadata::from(value).to_string()))
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
