use std::ops::Deref;
use rayon::prelude::IntoParallelIterator;
use crate::translate::{TopicLike, TopicMeta, TopicMetas, TopicModelLikeMatrix};


impl<T, L> TopicMetas for T
where
    T: Deref<Target=[L]>,
    L: TopicMeta + Send + Sync + 'static
{
    type TopicMeta = L;
    type Iter<'a> = std::slice::Iter<'a, Self::TopicMeta> where Self: 'a;
    type ParIter<'a> = rayon::slice::Iter<'a, Self::TopicMeta> where Self: 'a;

    fn get(&self, topic_id: usize) -> Option<&Self::TopicMeta> {
        <[_]>::get(self, topic_id)
    }

    unsafe fn get_unchecked(&self, topic_id: usize) -> &Self::TopicMeta {
        <[_]>::get_unchecked(self, topic_id)
    }

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        <&[_]>::into_iter(self)
    }

    fn par_iter<'a>(&'a self) -> Self::ParIter<'a> {
        <&[_]>::into_par_iter(self)
    }
}

impl<T, L> TopicModelLikeMatrix for T
where
    T: Deref<Target=[L]>,
    L: TopicLike + Send + Sync + 'static
{
    type Iter<'a> = std::slice::Iter<'a, Self::TopicLike> where Self: 'a;

    type ParIter<'a> = rayon::slice::Iter<'a, Self::TopicLike> where Self: 'a;

    type TopicLike = L;

    #[inline(always)]
    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, topic_id: usize) -> Option<&Self::TopicLike> {
        <[_]>::get(self, topic_id)
    }

    unsafe fn get_unchecked(&self, topic_id: usize) -> &Self::TopicLike {
        <[_]>::get_unchecked(self, topic_id)
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        <&[_]>::into_iter(self)
    }

    fn par_iter<'a>(&'a self) -> Self::ParIter<'a> {
        <&[_]>::into_par_iter(self)
    }
}

impl<T> TopicLike for T
where
    T: Deref<Target=[f64]>
{
    type Iter<'a> = std::slice::Iter<'a, f64> where Self: 'a;
    type ParIter<'a> = rayon::slice::Iter<'a, f64> where Self: 'a;

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn get(&self, voter_id: usize) -> Option<&f64> {
        <[_]>::get(self, voter_id)
    }

    unsafe fn get_unchecked(&self, voter_id: usize) -> &f64 {
        <[_]>::get_unchecked(self, voter_id)
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        <&[_]>::into_iter(self)
    }

    fn par_iter<'a>(&'a self) -> Self::ParIter<'a> {
        <&[_]>::into_par_iter(self)
    }
}