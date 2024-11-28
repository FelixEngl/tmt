use crate::dictionary::direction::{DirectionMarker, DirectedElement};
use crate::dictionary::iterators::DictIter;
use crate::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, BasicDictionaryWithVocabulary, DictionaryWithVocabulary};
use std::marker::PhantomData;
use std::ops::{Range};
use ldatranslate_toolkit::sync_ext::OwnedOrArcRw;
use crate::dictionary::metadata::MetadataReference;
use crate::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut, BasicVocabulary};
use super::MetadataManager;


pub struct MetaIter<'a, D, M, V>
where
    D: BasicDictionaryWithMeta<M, V> + ?Sized,
    M: MetadataManager,
    V: AnonymousVocabulary
{
    dictionary_with_meta: &'a D,
    direction: DirectionMarker,
    range: Range<usize>,
    _phantom: PhantomData<fn(M, V) -> ()>
}

impl<'a, D, M, V> MetaIter<'a, D, M, V>
where
    D: BasicDictionaryWithMeta<M, V> + ?Sized,
    M: MetadataManager,
    V: AnonymousVocabulary
{
    pub fn new(dictionary_with_meta: &'a D, direction: DirectionMarker) -> Self {
        let range = match direction {
            DirectionMarker::AToB | DirectionMarker::Invariant => {
                0..dictionary_with_meta.metadata().meta_a().len()
            }
            DirectionMarker::BToA => {
                0..dictionary_with_meta.metadata().meta_b().len()
            }
        };
        Self {
            dictionary_with_meta,
            direction,
            range,
            _phantom: PhantomData
        }
    }
}

impl<'a, D, M, V> Iterator for MetaIter<'a, D, M, V>
where
    D: BasicDictionaryWithMeta<M, V> + ?Sized,
    M: MetadataManager +'a,
    V: AnonymousVocabulary
{
    type Item = (usize, Option<<M as MetadataManager>::Reference<'a>>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.direction {
            DirectionMarker::AToB => {
                let word_id = self.range.next()?;
                return Some((word_id, self.dictionary_with_meta.get_meta_for_a(word_id)));
            }
            DirectionMarker::Invariant => {
                if let Some(word_id) = self.range.next() {
                    return Some((word_id, self.dictionary_with_meta.get_meta_for_a(word_id)));
                }
                self.direction = DirectionMarker::BToA;
                self.range = 0..self.dictionary_with_meta.metadata().meta_b().len();
            }
            DirectionMarker::BToA => {}
        }
        let id = self.range.next()?;
        Some((id, self.dictionary_with_meta.get_meta_for_b(id)))
    }
}



pub struct MetaMutIter<'a, D, M, V>
where
    D: BasicDictionaryWithMutMeta<M, V> + ?Sized,
    M: MetadataManager,
    V: AnonymousVocabularyMut + AnonymousVocabulary
{
    dictionary_with_meta: &'a mut D,
    direction: DirectionMarker,
    range: Range<usize>,
    _phantom: PhantomData<fn(M, V) -> ()>
}

impl<'a, D, M, V> MetaMutIter<'a, D, M, V>
where
    D: BasicDictionaryWithMutMeta<M, V> + ?Sized,
    M: MetadataManager,
    V: AnonymousVocabularyMut + AnonymousVocabulary
{
    pub fn new(dictionary_with_meta: &'a mut D, direction: DirectionMarker) -> Self {
        let range = match direction {
            DirectionMarker::AToB | DirectionMarker::Invariant => {
                0..dictionary_with_meta.metadata().meta_a().len()
            }
            DirectionMarker::BToA => {
                0..dictionary_with_meta.metadata().meta_b().len()
            }
        };
        Self {
            dictionary_with_meta,
            direction,
            range,
            _phantom: PhantomData
        }
    }
}

impl<'a, D, M, V> Iterator for MetaMutIter<'a, D, M, V>
where
    D: BasicDictionaryWithMutMeta<M, V> + ?Sized,
    M: MetadataManager +'a,
    V: AnonymousVocabularyMut + AnonymousVocabulary
{
    type Item = (usize, Option<<M as MetadataManager>::MutReference<'a>>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.direction {
            DirectionMarker::AToB => {
                let word_id = self.range.next()?;
                return Some((word_id, unsafe{std::mem::transmute(self.dictionary_with_meta.get_mut_meta_a(word_id))}));
            }
            DirectionMarker::Invariant => {
                if let Some(word_id) = self.range.next() {
                    return Some((word_id, unsafe{std::mem::transmute(self.dictionary_with_meta.get_mut_meta_a(word_id))}));
                }
                self.direction = DirectionMarker::BToA;
                self.range = 0..self.dictionary_with_meta.metadata().meta_b().len();
            }
            DirectionMarker::BToA => {}
        }
        let id = self.range.next()?;
        Some((id, unsafe{std::mem::transmute(self.dictionary_with_meta.get_mut_meta_b(id))}))
    }
}


pub struct MetaIterOwned<D, T, V, M> {
    inner: OwnedOrArcRw<D>,
    direction: DirectionMarker,
    range: Range<usize>,
    _phantom: PhantomData<fn(M, V) -> T>
}

impl<D, T, V, M> MetaIterOwned<D, T, V, M>
where
    D: BasicDictionaryWithMeta<M, V> + DictionaryWithVocabulary<T, V>,
    M: MetadataManager,
    V: BasicVocabulary<T> + AnonymousVocabulary,
{
    pub fn new(inner: impl Into<OwnedOrArcRw<D>>, direction: DirectionMarker) -> Self {
        let inner = inner.into();
        let range = match direction {
            DirectionMarker::AToB | DirectionMarker::Invariant => {
                0..inner.get().metadata().meta_a().len()
            }
            DirectionMarker::BToA => {
                0..inner.get().metadata().meta_b().len()
            }
        };
        Self {
            inner,
            direction,
            range,
            _phantom: PhantomData
        }
    }
}

impl<D, T, V, M> Iterator for MetaIterOwned<D, T, V, M>
where
    D: BasicDictionaryWithMeta<M, V> + DictionaryWithVocabulary<T, V>,
    M: MetadataManager,
    V: BasicVocabulary<T> + AnonymousVocabulary,
    T: Clone
{
    type Item = (usize, T, Option<M::ResolvedMetadata>);

    fn next(&mut self) -> Option<Self::Item> {
        let dict = self.inner.get();
        match self.direction {
            DirectionMarker::AToB => {
                let word_id = self.range.next()?;
                let word = dict.convert_id_a_to_word(word_id).expect("This should never fail!");
                return Some((word_id, word.clone(), dict.get_meta_for_a(word_id).map(|v| v.into_resolved())));
            }
            DirectionMarker::Invariant => {
                if let Some(word_id) = self.range.next() {
                    let word = dict.convert_id_a_to_word(word_id).expect("This should never fail!");
                    return Some((word_id, word.clone(), dict.get_meta_for_a(word_id).map(|v| v.into_resolved())));
                }
                self.direction = DirectionMarker::BToA;
                self.range = 0..dict.metadata().meta_b().len();
            }
            DirectionMarker::BToA => {}
        }
        let word_id = self.range.next()?;
        let word = dict.convert_id_b_to_word(word_id).expect("This should never fail!");
        Some((word_id, word.clone(), dict.get_meta_for_b(word_id).map(|v| v.into_resolved())))
    }
}

pub struct DictionaryWithMetaIter<'a, D, M, V>
where
    D: BasicDictionaryWithMeta<M, V> + ?Sized,
    M: MetadataManager,
    V: AnonymousVocabulary
{
    dictionary_with_meta: &'a D,
    iter: DictIter<'a>,
    _phantom: PhantomData<(M, V)>
}

impl<'a, D, M, V> DictionaryWithMetaIter<'a, D, M, V>
where
    D: BasicDictionaryWithMeta<M, V> + ?Sized,
    M: MetadataManager,
    V: AnonymousVocabulary
{
    pub fn new(dictionary_with_meta: &'a D, dir: DirectionMarker) -> Self {
        Self {
            iter: dictionary_with_meta.iter_dir(dir),
            dictionary_with_meta,
            _phantom: PhantomData
        }
    }

    #[inline(always)]
    pub fn direction(&self) -> DirectionMarker {
        self.iter.direction()
    }
}

impl<'a, D, M, V> Iterator for DictionaryWithMetaIter<'a, D, M, V>
where
    D: BasicDictionaryWithMeta<M, V> + BasicDictionaryWithVocabulary<V>,
    M: MetadataManager + 'a,
    V: AnonymousVocabulary + 'a
{
    type Item = DirectedElement<(usize, Option<M::Reference<'a>>), (usize, Option<M::Reference<'a>>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let DirectedElement {a, b, direction} = self.iter.next()?;
        Some(
            match direction {
                DirectionMarker::AToB => {
                    DirectedElement::a_to_b(
                        (
                            a,
                            self.dictionary_with_meta.metadata().get_meta_ref_a(
                                self.dictionary_with_meta.voc_a(),
                                a
                            )
                        ),
                        (
                            b,
                            self.dictionary_with_meta.metadata().get_meta_ref_b(
                                self.dictionary_with_meta.voc_b(),
                                b
                            )
                        )
                    )
                }
                DirectionMarker::BToA => {
                    DirectedElement::b_to_a(
                        (
                            a,
                            self.dictionary_with_meta.metadata().get_meta_ref_a(
                                self.dictionary_with_meta.voc_a(),
                                a
                            )
                        ),
                        (
                            b,
                            self.dictionary_with_meta.metadata().get_meta_ref_b(
                                self.dictionary_with_meta.voc_b(),
                                b
                            )
                        )
                    )
                }
                DirectionMarker::Invariant => {
                    DirectedElement::invariant(
                        (
                            a,
                            self.dictionary_with_meta.metadata().get_meta_ref_a(
                                self.dictionary_with_meta.voc_a(),
                                a
                            )
                        ),
                        (
                            b,
                            self.dictionary_with_meta.metadata().get_meta_ref_b(
                                self.dictionary_with_meta.voc_b(),
                                b
                            )
                        )
                    )
                }
            }
        )
    }
}