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

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::{Cloned, Enumerate, FlatMap, Map};
use std::marker::PhantomData;
use std::ops::Deref;
use std::slice::Iter;
use itertools::{Itertools, Unique};
use strum::EnumIs;
use crate::toolkit::sync_ext::OwnedOrArcRw;
use crate::toolkit::tupler::{SupportsTupling, TupleFirst, TupleLast};
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, Dictionary, DictionaryWithVocabulary};
use crate::topicmodel::dictionary::direction::{DirectionKind, DirectionTuple, Language, LanguageKind};
use crate::topicmodel::dictionary::metadata::{MetadataManager, MetadataReference};
use crate::topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary};

/// Iterator for a dictionary
pub struct DictLangIter<'a, T, D: ?Sized, V> {
    iter: Enumerate<Iter<'a, T>>,
    dict: &'a D,
    direction: LanguageKind,
    _phantom: PhantomData<V>
}

impl<'a, T, D: ?Sized, V> DictLangIter<'a, T, D, V> where V: BasicVocabulary<T> + 'a, D: DictionaryWithVocabulary<T, V> {
    pub(in crate::topicmodel::dictionary) fn new<L: Language>(dict: &'a D) -> Self {
        Self {
            iter: if L::LANG.is_a() {
                dict.voc_a().iter().enumerate()
            } else {
                dict.voc_b().iter().enumerate()
            },
            dict,
            direction: L::LANG,
            _phantom: PhantomData
        }
    }
}

impl<'a, T, D, V> Iterator for DictLangIter<'a, T, D, V> where V: BasicVocabulary<T> + 'a, D: DictionaryWithVocabulary<T, V> {
    type Item = (usize, &'a T, Option<Vec<(usize, &'a T)>>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, next) = self.iter.next()?;
        let translation = if self.direction.is_a() {
            self.dict.translate_id_a_to_entries_b(id)
        } else {
            self.dict.translate_id_b_to_entries_a(id)
        };
        Some((id, next, translation))
    }
}

pub type DictIter<'a> = DictIterImpl<'a>;


/// Iterates over all mappings (a to b and b to a), does not filter for uniqueness.
pub struct DictIterImpl<'a> {
    a_to_b: &'a Vec<Vec<usize>>,
    b_to_a: &'a Vec<Vec<usize>>,
    used: HashSet<(usize, usize)>,
    iter_a_b: ABIter<'a>,
    iter_b_a: BAIter<'a>,
    direction: DirectionKind
}

type ABIter<'a> = FlatMap<Enumerate<Iter<'a, Vec<usize>>>, Map<TupleFirst<Cloned<Iter<'a, usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>, fn((usize, &Vec<usize>)) -> Map<TupleFirst<Cloned<Iter<usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>>;
type BAIter<'a> = FlatMap<Enumerate<Iter<'a, Vec<usize>>>, Map<TupleLast<Cloned<Iter<'a, usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>, fn((usize, &Vec<usize>)) -> Map<TupleLast<Cloned<Iter<usize>>, usize>, fn((usize, usize)) -> DirectionTuple<usize, usize>>>;
impl<'a> DictIterImpl<'a> {

    pub fn direction(&self) -> DirectionKind {
        self.direction
    }

    pub(in crate::topicmodel::dictionary) fn new(
        dict: &'a (impl BasicDictionary + ?Sized),
        direction: DirectionKind
    ) -> Self {
        let a_to_b: ABIter = dict.map_a_to_b().iter().enumerate().flat_map(|(a, value)| value.iter().cloned().tuple_first(a).map(|(a, b) | DirectionTuple::a_to_b(a, b)));
        let b_to_a: BAIter = dict.map_b_to_a().iter().enumerate().flat_map(|(b, value)| value.iter().cloned().tuple_last(b).map(|(a, b) | DirectionTuple::b_to_a(a, b)));
        // let iter: Chain<ABIter, BAIter> = a_to_b.chain(b_to_a);
        Self {
            a_to_b: dict.map_a_to_b(),
            b_to_a: dict.map_b_to_a(),
            used: HashSet::new(),
            iter_a_b: a_to_b,
            iter_b_a: b_to_a,
            direction
        }
    }

    fn next_impl(&mut self) -> Option<DirectionTuple<usize, usize>> {
        match self.direction {
            DirectionKind::AToB => {
                self.iter_a_b.next()
            }
            DirectionKind::BToA => {
                self.iter_b_a.next()
            }
            DirectionKind::Invariant => {
                self.iter_a_b.next().or_else(|| self.iter_b_a.next())
            }
        }
    }
}
impl<'a> Iterator for DictIterImpl<'a> {
    type Item = DirectionTuple<usize, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tuple = self.next_impl()?;
        while !self.used.insert(tuple.value_tuple()) {
            tuple = self.next_impl()?;
        }
        match &tuple.direction {
            DirectionKind::AToB => {
                if self.b_to_a[tuple.b].contains(&tuple.a) {
                    tuple.direction = self.direction
                }
            }
            DirectionKind::BToA => {
                if self.a_to_b[tuple.a].contains(&tuple.b) {
                    tuple.direction = self.direction
                }
            }
            _ => unreachable!()
        }
        Some(tuple)
    }
}





/// The state of the dict iterator
#[derive(Debug, Copy, Clone, EnumIs)]
enum DictionaryIteratorPointerState {
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
    inner: OwnedOrArcRw<D>,
    used: HashMap<(usize, usize), ()>,
    _types: PhantomData<fn(T, V)->()>
}

impl<T, V, D> DictionaryIteratorImpl<T, V, D> where D: DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T>, T: Clone {
    pub(in crate::topicmodel::dictionary) fn new(inner: impl Into<OwnedOrArcRw<D>>) -> Self {
        let mut new = Self {
            pos: 0,
            index: 0,
            state: DictionaryIteratorPointerState::NextAB,
            inner: inner.into(),
            used: HashMap::new(),
            _types: PhantomData
        };
        if !new.inner.get().map_a_to_b().get(new.pos).is_some_and(|found| !found.is_empty()) {
            new.increment_pos_and_idx();
        }
        new
    }



    fn increment_pos_and_idx(&mut self) -> bool {
        let read = self.inner.get();
        let targ = match self.state {
            DictionaryIteratorPointerState::NextAB => {
                read.map_a_to_b()
            }
            DictionaryIteratorPointerState::NextBA => {
                read.map_b_to_a()
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
                drop(read);
                self.pos = 0;
                self.state = self.state.next();
                self.increment_pos_and_idx()
            } else {
                self.pos = new_pos;
                true
            }
        } else {
            drop(read);
            self.state = self.state.next();
            self.increment_pos_and_idx()
        }
    }

    /// This one should only be called when `self.state` is not finished!
    fn get_current(&self) -> Option<DirectionTuple<(usize, T), (usize, T)>> {
        let read = self.inner.get();
        match self.state {
            DictionaryIteratorPointerState::NextAB => {
                let b = *read.map_a_to_b().get(self.pos)?.get(self.index)?;
                let a_value = (self.pos, read.convert_id_a_to_word(self.pos).unwrap().clone());
                let b_value = (b, read.convert_id_b_to_word(b).unwrap().clone());
                Some(if read.map_b_to_a()[b].contains(&self.pos) {
                    DirectionTuple::invariant(a_value, b_value)
                } else {
                    DirectionTuple::a_to_b(a_value, b_value)
                })
            }
            DictionaryIteratorPointerState::NextBA => {
                let a = *read.map_b_to_a().get(self.pos)?.get(self.index)?;
                let a_value = (a, read.convert_id_a_to_word(a).unwrap().clone());
                let b_value = (self.pos, read.convert_id_b_to_word(self.pos).unwrap().clone());
                Some(if read.map_a_to_b()[a].contains(&self.pos) {
                    DirectionTuple::invariant(a_value, b_value)
                } else {
                    DirectionTuple::b_to_a(a_value, b_value)
                })
            }
            DictionaryIteratorPointerState::Finished => unreachable!()
        }
    }
}

impl<T, V, D> Iterator for DictionaryIteratorImpl<T, V, D> where D: DictionaryWithVocabulary<T, V>, V: BasicVocabulary<T>, T: Clone {
    type Item = DirectionTuple<(usize, T), (usize, T)>;

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

/// A dict iterator
pub type DictionaryIterator<T, V> = Unique<DictionaryIteratorImpl<T, V, Dictionary<T, V>>>;

impl<T, V> IntoIterator for Dictionary<T, V> where V: BasicVocabulary<T>, T: Eq + Hash + Clone {
    type Item = DirectionTuple<(usize, T), (usize, T)>;
    type IntoIter = DictionaryIterator<T, V>;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryIteratorImpl::new(self).unique()
    }
}


/// A dict iterator with metadata
pub struct DictionaryWithMetaIterator<D, T, V, M>
where
    D: DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T> + AnonymousVocabulary,
    T: Clone
{
    inner: DictionaryIteratorImpl<T, V, D>,
    _meta: PhantomData<M>
}

impl<D, T, V, M> DictionaryWithMetaIterator<D, T, V, M>
where
    D: BasicDictionaryWithMeta<M, V> + DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T> + AnonymousVocabulary,
    M: MetadataManager,
    T: Clone
{
    pub fn new(inner: impl Into<OwnedOrArcRw<D>>) -> Self {
        Self {
            inner: DictionaryIteratorImpl::new(inner),
            _meta: PhantomData
        }
    }
}

impl<D, T, V, M> Iterator for DictionaryWithMetaIterator<D, T, V, M>
where
    D: BasicDictionaryWithMeta<M, V> + DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T> + AnonymousVocabulary,
    M: MetadataManager,
    T: Clone
{
    type Item = DirectionTuple<
        (usize, T, Option<M::ResolvedMetadata>),
        (usize, T, Option<M::ResolvedMetadata>)
    >;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        let read_owned = self.inner.inner.get();
        let red = read_owned.deref();
        Some(
            next.map(
                |(id, href)| {
                    let value = red
                        .get_meta_for_a(id)
                        .map(|value| value.into_resolved());
                    (id, href, value)
                },
                |(id, href)| {
                    let value = red
                        .get_meta_for_b(id)
                        .map(|value| value.into_resolved());
                    (id, href, value)
                }
            )
        )
    }
}
