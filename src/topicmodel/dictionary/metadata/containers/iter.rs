use crate::topicmodel::dictionary::direction::{DirectionKind, DirectionTuple, A, B};
use crate::topicmodel::dictionary::iterators::DictIter;
use crate::topicmodel::dictionary::{BasicDictionaryWithMeta};
use std::marker::PhantomData;
use super::MetadataManager;

pub struct DictionaryWithMetaIter<'a, D, M> where D: BasicDictionaryWithMeta<M> + ?Sized, M: MetadataManager
{
    dictionary_with_meta: &'a D,
    iter: DictIter<'a>,
    _phantom: PhantomData<M>
}

impl<'a, D, M> DictionaryWithMetaIter<'a, D, M> where D: BasicDictionaryWithMeta<M> + ?Sized, M: MetadataManager
{
    pub fn new(dictionary_with_meta: &'a D) -> Self {
        Self {
            iter: dictionary_with_meta.iter(),
            dictionary_with_meta,
            _phantom: PhantomData
        }
    }
}

impl<'a, D, M> Iterator for DictionaryWithMetaIter<'a, D, M> where
    D: BasicDictionaryWithMeta<M>,
    M: MetadataManager + 'a
{
    type Item = DirectionTuple<(usize, Option<M::Reference<'a>>), (usize, Option<M::Reference<'a>>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let DirectionTuple{a, b, direction} = self.iter.next()?;
        Some(
            match direction {
                DirectionKind::AToB => {
                    DirectionTuple::a_to_b(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    )
                }
                DirectionKind::BToA => {
                    DirectionTuple::b_to_a(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    )
                }
                DirectionKind::Invariant => {
                    DirectionTuple::invariant(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(b))
                    )
                }
            }
        )
    }
}