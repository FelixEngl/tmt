// use std::borrow::Borrow;
// use std::marker::PhantomData;
// use bitvec::prelude::*;
// use sealed::sealed;
// use serde::{Deserialize, Deserializer, Serialize, Serializer};
// use strum::EnumCount;
// use tinyset::{Fits64, Set64};
// use crate::topicmodel::dictionary::word_infos::Domain;
//
// pub trait MetadataValueFieldType where Self: Sized {
//     type FieldType: MetadataValueField<Self>;
// }
//
// impl MetadataValueFieldType for Domain {
//     type FieldType = BitMetadataField<Domain, [u32; bitvec::mem::elts::<u32>(Domain::COUNT)]>;
// }
//
// macro_rules! metadata_value_field_type_impl {
//     (
//         $ty: ty => Set64
//     ) => {
//
//     };
//
//     (
//         $ty: ty => Bits
//     ) => {
//
//     };
// }
//
// pub(crate) use metadata_value_field_type_impl;
//
//
// #[sealed]
// pub trait MetadataValueField<T>: Extend<T> + Clone + Default + Eq + PartialEq {
//
//     type BitStore: BitStore;
//
//     fn insert(&mut self, value: T);
//
//     fn len(&self) -> usize;
//
//     fn contains<R: Borrow<T>>(&self, value: R) -> bool;
//
//     fn is_empty(&self) -> bool;
//
//     fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a;
//
//     fn as_set64(&self) -> Set64<T> where T: Fits64;
//
//     fn as_bit_vec(&self) -> BitVec<Self::BitStore> where T: Fits64;
// }
//
// #[sealed]
// impl<T> MetadataValueField<T> for Set64<T> where T: Fits64 {
//     type BitStore = usize;
//
//     #[inline(always)]
//     fn insert(&mut self, value: T) {
//         Set64::insert(self, value);
//     }
//
//     #[inline(always)]
//     fn len(&self) -> usize {
//         Set64::len(self)
//     }
//
//     #[inline(always)]
//     fn contains<R: Borrow<T>>(&self, value: R) -> bool {
//         Set64::contains(self, value)
//     }
//
//     #[inline(always)]
//     fn is_empty(&self) -> bool {
//         Set64::is_empty(self)
//     }
//
//     #[inline(always)]
//     fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a {
//         Set64::iter(self)
//     }
//
//     #[inline(always)]
//     fn as_set64(&self) -> Set64<T> where T: Fits64 {
//         Set64::clone(self)
//     }
//
//     #[inline(always)]
//     fn as_bit_vec(&self) -> BitVec<Self::BitStore>
//     where
//         T: Fits64,
//     {
//         Set64::iter(self).map(|value| value.to_u64() as usize).fold(BitVec::new(), |mut bits, i| {
//             if bits.len() <= i {
//                 bits.resize(i, false);
//                 bits.push(true);
//             } else {
//                 bits.set(i, true);
//             }
//             bits
//         })
//     }
// }
//
//
// #[derive(Debug)]
// pub struct BitMetadataField<T, A> where A: BitStore {
//     inner: BitArray<A>,
//     _type: PhantomData<T>
// }
//
// impl<T, A> Serialize for BitMetadataField<T, A>
// where
//     A: BitStore,
//     A::Mem: Serialize,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         self.inner.serialize(serializer)
//     }
// }
//
// impl<'de, T, A> Deserialize<'de> for BitMetadataField<T, A>
// where
//     A: BitStore,
//     A::Mem: Deserialize<'de>,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>
//     {
//         Ok(
//             Self {
//                 inner: BitArray::<A>::deserialize(deserializer)?,
//                 _type: PhantomData
//             }
//         )
//     }
// }
//
// impl<T, A> Extend<T> for BitMetadataField<T, A>
// where
//     A: BitStore,
//     T: Fits64,
// {
//     fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
//         iter.into_iter().for_each(|value| self.insert(value))
//     }
// }
//
// impl<T, A> Clone for BitMetadataField<T, A>
// where
//     A: BitStore
// {
//     fn clone(&self) -> Self {
//         Self {
//             inner: self.inner.clone(),
//             _type: PhantomData
//         }
//     }
// }
//
// impl<T, A> Default for BitMetadataField<T, A>
// where
//     A: BitStore
// {
//     fn default() -> Self {
//         Self {
//             inner: Default::default(),
//             _type: PhantomData,
//         }
//     }
// }
//
// impl<T, A> Eq for BitMetadataField<T, A>
// where
//     A: BitStore
// {}
//
// impl<T, A> PartialEq<Self> for BitMetadataField<T, A>
// where
//     A: BitStore
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.inner.eq(&other.inner)
//     }
// }
//
//
// #[sealed]
// impl<T, A> MetadataValueField<T> for  BitMetadataField<T, A>
// where
//     A: BitStore,
//     T: Fits64
// {
//     type BitStore = <A as BitStore>::Unalias;
//
//     fn insert(&mut self, value: T) {
//         self.inner.set(value.to_u64() as usize, true);
//     }
//
//     fn len(&self) -> usize {
//         self.inner.count_ones()
//     }
//
//     fn contains<R: Borrow<T>>(&self, value: R) -> bool {
//         self.inner.get(value.borrow().clone().to_u64() as usize).is_some_and(|value| *value)
//     }
//
//     fn is_empty(&self) -> bool {
//         self.inner.is_empty()
//     }
//
//     fn iter<'a>(&'a self) -> impl Iterator<Item=T> + 'a {
//         self.inner.iter_ones().map(|value| unsafe{T::from_u64(value as u64)})
//     }
//
//     fn as_set64(&self) -> Set64<T>
//     where
//         T: Fits64
//     {
//         self.iter().collect()
//     }
//
//     fn as_bit_vec(&self) -> BitVec<Self::BitStore>
//     where
//         T: Fits64,
//     {
//         self.inner.to_bitvec()
//     }
// }
//


