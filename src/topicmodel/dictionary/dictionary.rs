use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use itertools::{Itertools, Position};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::direction::{AToB, BToA, Direction, DirectionKind, DirectionTuple, Invariant, Language, LanguageKind, Translation, A, B};
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithVocabulary, DictionaryFilterable, DictionaryMut, DictionaryWithVocabulary, FromVoc};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{BasicVocabulary, MappableVocabulary, VocabularyMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary<T, V> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    pub(crate) voc_a: V,
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    pub(crate) voc_b: V,
    pub(crate) map_a_to_b: Vec<Vec<usize>>,
    pub(crate) map_b_to_a: Vec<Vec<usize>>,
    _word_type: PhantomData<T>
}

unsafe impl<T, V> Send for Dictionary<T, V>{}
unsafe impl<T, V> Sync for Dictionary<T, V>{}

impl<T, V> FromVoc<T, V> for Dictionary<T, V> where V: BasicVocabulary<T> + Default, T: Hash + Eq  {
    fn from_voc(voc_a: V, voc_b: V) -> Self {
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

    fn from_voc_lang<L: Language>(voc: V, other_lang: Option<LanguageHint>) -> Self {
        match L::LANG {
            LanguageKind::A => {
                let mut map_a_to_b = Vec::new();
                map_a_to_b.resize_with(voc.len(), || Vec::with_capacity(1));

                Self {
                    voc_a: voc,
                    voc_b: V::create(other_lang),
                    map_a_to_b,
                    map_b_to_a: Default::default(),
                    _word_type: PhantomData
                }
            }
            LanguageKind::B => {
                let mut map_b_to_a = Vec::new();
                map_b_to_a.resize_with(voc.len(), || Vec::with_capacity(1));

                Self {
                    voc_a: V::create(other_lang),
                    voc_b: voc,
                    map_a_to_b: Default::default(),
                    map_b_to_a,
                    _word_type: PhantomData
                }
            }
        }
    }
}

impl<T, V> Dictionary<T, V> where V: From<Option<LanguageHint>>  {
    pub fn new_with(language_a: Option<impl Into<LanguageHint>>, language_b: Option<impl Into<LanguageHint>>) -> Self {

        Self {
            voc_a: language_a.map(|value| value.into()).into(),
            voc_b: language_b.map(|value| value.into()).into(),
            map_a_to_b: Default::default(),
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

    fn switch_languages(self) -> Self where Self: Sized {
        Self {
            voc_a: self.voc_b,
            voc_b: self.voc_a,
            map_a_to_b: self.map_b_to_a,
            map_b_to_a: self.map_a_to_b,
            _word_type: PhantomData
        }
    }
}

impl<T, V> BasicDictionaryWithVocabulary<V> for Dictionary<T, V> {
    fn voc_a(&self) -> &V {
        &self.voc_a
    }

    fn voc_b(&self) -> &V {
        &self.voc_b
    }

}

impl<T, V> Dictionary<T, V> where T: Eq + Hash, V: MappableVocabulary<T> {
    pub fn map<Q: Eq + Hash, Voc, F>(self, f: F) -> Dictionary<Q, Voc> where F: for<'a> Fn(&'a T)-> Q, Voc: BasicVocabulary<Q> {
        Dictionary {
            voc_a: self.voc_a.map(&f),
            voc_b: self.voc_b.map(f),
            map_a_to_b: self.map_a_to_b,
            map_b_to_a: self.map_b_to_a,
            _word_type: PhantomData
        }
    }
}

impl<T, V> DictionaryWithVocabulary<T, V> for Dictionary<T, V> where V: BasicVocabulary<T> {
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
    fn set_language<L: Language>(&mut self, value: Option<LanguageHint>) -> Option<LanguageHint> {
        if L::LANG.is_a() {
            self.voc_a.set_language(value)
        } else {
            self.voc_b.set_language(value)
        }
    }

    fn insert_single_ref<L: Language>(&mut self, word: HashRef<T>) -> usize {
        let word_id = if L::LANG.is_a() {
            self.voc_a.add_hash_ref(word)
        } else {
            self.voc_b.add_hash_ref(word)
        };
        unsafe{self.reserve_for_single_value::<L>(word_id);}
        word_id
    }


    unsafe fn reserve_for_single_value<L: Language>(&mut self, word_id: usize) {
        if L::LANG.is_a() {
            if self.map_a_to_b.len() <= word_id {
                self.map_a_to_b.resize_with(word_id+1, || Vec::with_capacity(1));
            }
        } else {
            if self.map_b_to_a.len() <= word_id {
                self.map_b_to_a.resize_with(word_id+1, || Vec::with_capacity(1));
            }
        }
    }

    unsafe fn insert_raw_values<D: Direction>(&mut self, id_a: usize, id_b: usize) {
        if D::DIRECTION.is_a_to_b() {
            if let Some(found) = self.map_a_to_b.get_mut(id_a) {
                if !found.contains(&id_b) {
                    found.push(id_b)
                }
            } else {
                if self.map_a_to_b.len() <= id_a {
                    self.map_a_to_b.resize_with(id_a+1, || Vec::with_capacity(1));
                }
                unsafe {
                    self.map_a_to_b.get_unchecked_mut(id_a).push(id_b);
                }
            }
        }
        if D::DIRECTION.is_b_to_a() {
            if let Some(found) = self.map_b_to_a.get_mut(id_b) {
                if !found.contains(&id_a) {
                    found.push(id_a)
                }
            } else {
                if self.map_b_to_a.len() <= id_b {
                    self.map_b_to_a.resize_with(id_b+1, || Vec::with_capacity(1));
                }
                unsafe {
                    self.map_b_to_a.get_unchecked_mut(id_b).push(id_a);
                }
            }
        }
    }

    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        let id_a = self.voc_a.add_hash_ref(word_a);
        let id_b = self.voc_b.add_hash_ref(word_b);
        unsafe { self.insert_raw_values::<D>(id_a, id_b); }
        DirectionTuple::new(id_a, id_b, D::DIRECTION)
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