use std::borrow::Borrow;
use std::hash::Hash;
use serde::{Deserialize, Serialize};
use crate::topicmodel::vocabulary::Vocabulary;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary<T> {
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de> + Hash + Eq"))]
    voc_a: Vocabulary<T>,
    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de> + Hash + Eq"))]
    voc_b: Vocabulary<T>,
    map_a_to_b: Vec<Vec<usize>>,
    map_b_to_a: Vec<Vec<usize>>
}

impl<T: Eq + Hash> Dictionary<T> {
    pub fn insert<D: Direction>(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) {
        let id_a = self.voc_a.add(word_a);
        let id_b = self.voc_a.add(word_b);
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
            if let Some(found) = self.map_b_to_a.get_mut(id_a) {
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

    pub fn translate_word_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        let word = if D::A2B {
            self.voc_a.get_word_id(word)
        } else {
            self.voc_b.get_word_id(word)
        }?;
        self.translate_id_to_ids::<D>(word)
    }

    pub fn translate_word_to_words<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<Vec<&T>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        Some(self.ids_to_words::<D>(self.translate_word_to_ids::<D, Q>(word)?))
    }
}

impl<T> Dictionary<T> {

    fn ids_to_words<D: Translation>(&self, ids: &Vec<usize>) -> Vec<&T> {
        if D::A2B {
            ids.iter().map(|value| unsafe {
                self.voc_b.get_word(*value).unwrap_unchecked()
            }).collect()
        } else {
            ids.iter().map(|value| unsafe {
                self.voc_a.get_word(*value).unwrap_unchecked()
            }).collect()
        }
    }

    pub fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        if D::A2B {
            &self.map_a_to_b
        } else {
            &self.map_b_to_a
        }.get(word_id)
    }

    pub fn translate_id_to_words<D: Translation>(&self, word_id: usize) -> Option<Vec<&T>> {
        Some(self.ids_to_words::<D>(self.translate_id_to_ids::<D>(word_id)?))
    }
}






mod private {
    pub(crate) trait Sealed{}
}

pub trait Direction: private::Sealed{
    const A2B: bool;
    const B2A: bool;
}

pub trait Translation: Direction + private::Sealed {}


pub struct AToB;
impl private::Sealed for AToB{}
impl Direction for AToB{
    const A2B: bool = true;
    const B2A: bool = false;
}

impl Translation for AToB {}

pub struct BToA;
impl private::Sealed for BToA{}
impl Direction for BToA{
    const A2B: bool = false;
    const B2A: bool = true;
}
impl Translation for BToA {}

pub struct Indifferent;
impl private::Sealed for Indifferent{}
impl Direction for Indifferent{
    const A2B: bool = true;
    const B2A: bool = true;
}
