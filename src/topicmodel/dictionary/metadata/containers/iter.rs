use crate::topicmodel::dictionary::direction::{DirectionKind, DirectionTuple, A, B};
use crate::topicmodel::dictionary::iterators::DictIter;
use crate::topicmodel::dictionary::{BasicDictionaryWithMeta};
use std::marker::PhantomData;
use super::MetadataManager;

pub struct DictionaryWithMetaIter<'a, D, DNest, M> where D: BasicDictionaryWithMeta<DNest, M> + ?Sized, M: MetadataManager<DNest>
{
    dictionary_with_meta: &'a D,
    iter: DictIter<'a>,
    _phantom: PhantomData<(M, D, DNest)>
}

impl<'a, D, DNest, M> DictionaryWithMetaIter<'a, D, DNest, M> where D: BasicDictionaryWithMeta<DNest, M> + ?Sized, M: MetadataManager<DNest>
{
    pub fn new(dictionary_with_meta: &'a D) -> Self {
        Self {
            iter: dictionary_with_meta.iter(),
            dictionary_with_meta,
            _phantom: PhantomData
        }
    }
}

impl<'a, D, DNest, M> Iterator for DictionaryWithMetaIter<'a, D, DNest, M> where
    D: BasicDictionaryWithMeta<DNest, M>,
    M: MetadataManager<DNest> + 'a
{
    type Item = DirectionTuple<(usize, Option<M::Reference<'a>>), (usize, Option<M::Reference<'a>>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let DirectionTuple{a, b, direction} = self.iter.next()?;
        Some(
            match direction {
                DirectionKind::AToB => {
                    DirectionTuple::a_to_b(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(self.dictionary_with_meta.underlying_dict(), a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(self.dictionary_with_meta.underlying_dict(), b))
                    )
                }
                DirectionKind::BToA => {
                    DirectionTuple::b_to_a(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(self.dictionary_with_meta.underlying_dict(),a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(self.dictionary_with_meta.underlying_dict(),b))
                    )
                }
                DirectionKind::Invariant => {
                    DirectionTuple::invariant(
                        (a, self.dictionary_with_meta.metadata().get_meta_ref::<A>(self.dictionary_with_meta.underlying_dict(),a)),
                        (b, self.dictionary_with_meta.metadata().get_meta_ref::<B>(self.dictionary_with_meta.underlying_dict(),b))
                    )
                }
            }
        )
    }
}