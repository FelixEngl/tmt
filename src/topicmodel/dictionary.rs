use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::iter::Enumerate;
use std::marker::PhantomData;
use itertools::{Itertools, Position};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::direction::{A, B, Direction, Language, Translation};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{Vocabulary};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary<T> {
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de> + Hash + Eq"))]
    voc_a: Vocabulary<T>,
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de> + Hash + Eq"))]
    voc_b: Vocabulary<T>,
    map_a_to_b: Vec<Vec<usize>>,
    map_b_to_a: Vec<Vec<usize>>
}

impl<T> Dictionary<T> {
    pub fn new() -> Self {
        Self {
            voc_a: Default::default(),
            voc_b: Default::default(),
            map_a_to_b: Default::default(),
            map_b_to_a: Default::default(),
        }
    }

    pub fn from_voc_a(voc_a: Vocabulary<T>) -> Self {
        let mut map_a_to_b = Vec::new();
        map_a_to_b.resize_with(voc_a.len(), || Vec::with_capacity(1));

        Self {
            voc_a,
            voc_b: Default::default(),
            map_a_to_b,
            map_b_to_a: Default::default(),
        }
    }

    pub fn voc_a(&self) -> &Vocabulary<T> {
        &self.voc_a
    }

    pub fn voc_b(&self) -> &Vocabulary<T> {
        &self.voc_b
    }

    pub fn map_a_to_b(&self) -> &Vec<Vec<usize>> {
        &self.map_a_to_b
    }

    pub fn map_b_to_a(&self) -> &Vec<Vec<usize>> {
        &self.map_b_to_a
    }

    pub fn iter<L: Language>(&self) -> DictIter<T, L> {
        DictIter::<T, L>::new(self)
    }
}

pub struct DictIter<'a, T, L> where L: Language {
    iter: Enumerate<std::slice::Iter<'a, HashRef<T>>>,
    dict: &'a Dictionary<T>,
    _language: PhantomData<L>
}

impl<'a, T, L> DictIter<'a, T, L> where L: Language {
    fn new(dict: &'a Dictionary<T>) -> Self {
        Self {
            iter: if L::TranslationDirection::A2B {
                dict.voc_a.iter().enumerate()
            } else {
                dict.voc_b.iter().enumerate()
            },
            dict,
            _language: PhantomData
        }
    }
}

impl<'a, T, L> Iterator for DictIter<'a, T, L> where L: Language {
    type Item = (usize, &'a HashRef<T>, Option<Vec<(usize, &'a HashRef<T>)>>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, next) = self.iter.next()?;
        let translation = self.dict.translate_id::<L::TranslationDirection>(id);
        Some((id, next, translation))
    }
}


impl<T: Eq + Hash> Dictionary<T> {
    pub fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) {
        let id_a = self.voc_a.add_hash_ref(word_a);
        let id_b = self.voc_b.add_hash_ref(word_b);
        if D::A2B {
            if let Some(found) = self.map_a_to_b.get_mut(id_a) {
                found.push(id_b)
            } else {
                while self.map_a_to_b.len() < id_a {
                    self.map_a_to_b.push(Vec::with_capacity(1));
                }
                unsafe {
                    self.map_a_to_b.get_unchecked_mut(id_a).push(id_b);
                }
            }
        }
        if D::B2A {
            if let Some(found) = self.map_b_to_a.get_mut(id_b) {
                found.push(id_a)
            } else {
                while self.map_b_to_a.len() < id_b {
                    self.map_b_to_a.push(Vec::with_capacity(1));
                }
                unsafe {
                    self.map_b_to_a.get_unchecked_mut(id_a).push(id_b);
                }
            }
        }
    }

    pub fn insert_value<D: Direction>(&mut self, word_a: T, word_b: T) {
        self.insert_hash_ref::<D>(HashRef::new(word_a), HashRef::new(word_b))
    }

    pub fn insert<D: Direction>(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) {
        self.insert_value::<D>(word_a.into(), word_b.into())
    }

    pub fn translate_value<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<Vec<(usize, &HashRef<T>)>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        Some(self.ids_to_id_entry::<D>(self.translate_value_to_ids::<D, Q>(word)?))
    }

    pub fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
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

    pub fn translate_value_to_values<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<Vec<&HashRef<T>>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        Some(self.ids_to_values::<D>(self.translate_value_to_ids::<D, Q>(word)?))
    }
}

impl<T: Display> Display for Dictionary<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn write_language<L: Language, T: Display>(dictionary: &Dictionary<T>, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:\n", L::NAME)?;
            for (id_a, value_a, translations) in dictionary.iter::<L>() {
                write!(f, "  {value_a}({id_a}):\n")?;
                if let Some(translations) = translations {
                    for (position, (id_b, value_b)) in translations.iter().with_position() {
                        match position {
                            Position::First | Position::Middle => {
                                write!(f, "    {value_b}({id_b})\n")?
                            }
                            Position::Last | Position::Only => {
                                write!(f, "    {value_b}({id_b})")?
                            }
                        }

                    }
                } else {
                    write!(f, "    - None -")?;
                }
            }
            Ok(())
        }

        write_language::<A, _>(self, f)?;
        write!(f, "\n------\n")?;
        write_language::<B, _>(self, f)
    }
}

impl<T> Dictionary<T> {

    fn ids_to_id_entry<D: Translation>(&self, ids: &Vec<usize>) -> Vec<(usize, &HashRef<T>)> {
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

    fn ids_to_values<D: Translation>(&self, ids: &Vec<usize>) -> Vec<&HashRef<T>> {
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

    pub fn translate_id<D: Translation>(&self, word_id: usize) -> Option<Vec<(usize, &HashRef<T>)>> {
        Some(self.ids_to_id_entry::<D>(self.translate_id_to_ids::<D>(word_id)?))
    }

    pub fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        if D::A2B {
            &self.map_a_to_b
        } else {
            &self.map_b_to_a
        }.get(word_id)
    }

    pub fn translate_id_to_values<D: Translation>(&self, word_id: usize) -> Option<Vec<&HashRef<T>>> {
        Some(self.ids_to_values::<D>(self.translate_id_to_ids::<D>(word_id)?))
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

