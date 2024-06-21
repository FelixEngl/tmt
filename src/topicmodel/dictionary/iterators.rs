use std::collections::HashMap;
use std::hash::Hash;
use std::iter::{Chain, Cloned, Enumerate, FlatMap, Map};
use std::marker::PhantomData;
use std::slice::Iter;
use itertools::{Itertools, Unique};
use strum::EnumIs;
use crate::toolkit::tupler::{SupportsTupling, TupleFirst, TupleLast};
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, Dictionary, DictionaryWithVocabulary};
use crate::topicmodel::dictionary::direction::{A, B, DirectionKind, DirectionTuple, Language};
use crate::topicmodel::dictionary::metadata::{MetadataRef, SolvedMetadata};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::BasicVocabulary;



pub struct DictLangIter<'a, T, L, D: ?Sized, V> where L: Language {
    iter: Enumerate<Iter<'a, HashRef<T>>>,
    dict: &'a D,
    _language: PhantomData<(L, V)>
}

impl<'a, T, L, D: ?Sized, V> DictLangIter<'a, T, L, D, V> where L: Language, V: BasicVocabulary<T> + 'a, D: DictionaryWithVocabulary<T, V> {
    pub(in crate::topicmodel::dictionary) fn new(dict: &'a D) -> Self {
        Self {
            iter: if L::DIRECTION.is_a_to_b() {
                dict.voc_a().iter().enumerate()
            } else {
                dict.voc_b().iter().enumerate()
            },
            dict,
            _language: PhantomData
        }
    }
}

impl<'a, T, L, D, V> Iterator for DictLangIter<'a, T, L, D, V> where L: Language, V: BasicVocabulary<T> + 'a, D: DictionaryWithVocabulary<T, V> {
    type Item = (usize, &'a HashRef<T>, Option<Vec<(usize, &'a HashRef<T>)>>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, next) = self.iter.next()?;
        let translation = self.dict.translate_id::<L>(id);
        Some((id, next, translation))
    }
}

pub type DictIter<'a> = DictIterImpl<'a>;


/// Iterates over all mappings (a to b and b to a), does not filter for uniqueness.
pub struct DictIterImpl<'a> {
    a_to_b: &'a Vec<Vec<usize>>,
    b_to_a: &'a Vec<Vec<usize>>,
    used: HashMap<(usize, usize), ()>,
    iter: Chain<ABIter<'a>, BAIter<'a>>,
}

type ABIter<'a> = FlatMap<Enumerate<Iter<'a, Vec<usize>>>, Map<TupleFirst<Cloned<Iter<'a, usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>, fn((usize, &Vec<usize>)) -> Map<TupleFirst<Cloned<Iter<usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>>;
type BAIter<'a> = FlatMap<Enumerate<Iter<'a, Vec<usize>>>, Map<TupleLast<Cloned<Iter<'a, usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>, fn((usize, &Vec<usize>)) -> Map<TupleLast<Cloned<Iter<usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>>;
impl<'a> DictIterImpl<'a> {
    pub(in crate::topicmodel::dictionary) fn new(dict: &'a (impl BasicDictionary + ?Sized)) -> Self {
        let a_to_b: ABIter = dict.map_a_to_b().iter().enumerate().flat_map(|(a, value)| value.iter().cloned().tuple_first(a).map(|(a, b) | DirectionTuple::a_to_b(a, b)));
        let b_to_a: BAIter = dict.map_b_to_a().iter().enumerate().flat_map(|(b, value)| value.iter().cloned().tuple_last(b).map(|(a, b) | DirectionTuple::b_to_a(a, b)));
        let iter: Chain<ABIter, BAIter> = a_to_b.chain(b_to_a);
        Self {
            a_to_b: dict.map_a_to_b(),
            b_to_a: dict.map_b_to_a(),
            used: HashMap::new(),
            iter
        }
    }
}
impl<'a> Iterator for DictIterImpl<'a> {
    type Item = DirectionTuple<usize, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tuple = self.iter.next()?;
        while self.used.insert(tuple.value_tuple(), ()).is_some() {
            tuple = self.iter.next()?;
        }
        match &tuple.direction {
            DirectionKind::AToB => {
                if self.b_to_a[tuple.b].contains(&tuple.a) {
                    tuple.direction = DirectionKind::Invariant
                }
            }
            DirectionKind::BToA => {
                if self.a_to_b[tuple.a].contains(&tuple.b) {
                    tuple.direction = DirectionKind::Invariant
                }
            }
            _ => unreachable!()
        }
        Some(tuple)
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
pub struct DictionaryIteratorImpl<T, V, D> where D: DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T> {
    pos: usize,
    index: usize,
    state: DictionaryIteratorPointerState,
    pub(in crate::topicmodel::dictionary) inner: D,
    used: HashMap<(usize, usize), ()>,
    _types: PhantomData<fn(T, V)->()>
}

impl<T, V, D> DictionaryIteratorImpl<T, V, D> where D: DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T> {
    pub(in crate::topicmodel::dictionary) fn new(inner: D) -> Self {
        let mut new = Self {
            pos: 0,
            index: 0,
            state: DictionaryIteratorPointerState::NextAB,
            inner,
            used: HashMap::new(),
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
                    DirectionTuple::invariant(a_value, b_value)
                } else {
                    DirectionTuple::a_to_b(a_value, b_value)
                })
            }
            DictionaryIteratorPointerState::NextBA => {
                let a = *self.inner.map_b_to_a().get(self.pos)?.get(self.index)?;
                let a_value = (a, self.inner.id_to_word::<A>(a).unwrap().clone());
                let b_value = (self.pos, self.inner.id_to_word::<B>(self.pos).unwrap().clone());
                Some(if self.inner.map_a_to_b()[a].contains(&self.pos) {
                    DirectionTuple::invariant(a_value, b_value)
                } else {
                    DirectionTuple::b_to_a(a_value, b_value)
                })
            }
            DictionaryIteratorPointerState::Finished => unreachable!()
        }
    }
}

impl<T, V, D> Iterator for DictionaryIteratorImpl<T, V, D> where D: DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T> {
    type Item = DirectionTuple<(usize, HashRef<T>), (usize, HashRef<T>)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.state.is_finished() {
                break None;
            }
            match self.get_current() {
                None => {
                    if !self.increment_pos_and_idx() {
                        break None;
                    }
                }
                Some(value) => {
                    self.increment_pos_and_idx();
                    if self.used.insert((value.a.0, value.b.0), ()).is_some() {
                        continue
                    }
                    break Some(value);
                }
            }
        }
    }
}

pub type DictionaryIterator<T, V> = Unique<DictionaryIteratorImpl<T, V, Dictionary<T, V>>>;

impl<T, V> IntoIterator for Dictionary<T, V> where V: BasicVocabulary<T>, T: Eq + Hash {
    type Item = DirectionTuple<(usize, HashRef<T>), (usize, HashRef<T>)>;
    type IntoIter = DictionaryIterator<T, V>;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryIteratorImpl::new(self).unique()
    }
}


pub struct DictionaryWithMetaIterator<D, T, V> where D: BasicDictionaryWithMeta + DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T> {
    inner: DictionaryIteratorImpl<T, V, D>
}

impl<D, T, V> DictionaryWithMetaIterator<D, T, V> where D: BasicDictionaryWithMeta + DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T>  {
    pub fn new(inner: D) -> Self {
        Self {
            inner: DictionaryIteratorImpl::new(inner)
        }
    }

    pub fn into_inner(self) -> D {
        self.inner.inner
    }
}

impl<D, T, V> Iterator for DictionaryWithMetaIterator<D, T, V> where D: BasicDictionaryWithMeta + DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T> {
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
