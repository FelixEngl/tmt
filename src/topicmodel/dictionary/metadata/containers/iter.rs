use crate::topicmodel::dictionary::direction::{DirectionKind, DirectionTuple};
use crate::topicmodel::dictionary::iterators::DictIter;
use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithVocabulary};
use std::marker::PhantomData;
use crate::topicmodel::vocabulary::AnonymousVocabulary;
use super::MetadataManager;



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
    pub fn new(dictionary_with_meta: &'a D) -> Self {
        Self {
            iter: dictionary_with_meta.iter(),
            dictionary_with_meta,
            _phantom: PhantomData
        }
    }
}

impl<'a, D, M, V> Iterator for DictionaryWithMetaIter<'a, D, M, V>
where
    D: BasicDictionaryWithMeta<M, V> + BasicDictionaryWithVocabulary<V>,
    M: MetadataManager + 'a,
    V: AnonymousVocabulary + 'a
{
    type Item = DirectionTuple<(usize, Option<M::Reference<'a>>), (usize, Option<M::Reference<'a>>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let DirectionTuple{a, b, direction} = self.iter.next()?;
        Some(
            match direction {
                DirectionKind::AToB => {
                    DirectionTuple::a_to_b(
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
                DirectionKind::BToA => {
                    DirectionTuple::b_to_a(
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
                DirectionKind::Invariant => {
                    DirectionTuple::invariant(
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