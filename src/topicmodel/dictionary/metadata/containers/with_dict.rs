use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, DictionaryWithVocabulary};
use crate::topicmodel::dictionary::direction::{AToB, BToA, A, B};
use crate::topicmodel::dictionary::metadata::containers::MetadataManager;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary};

pub struct MetadataContainerWithDict<'a, D, T, V, M: MetadataManager> {
    dict: *const D,
    meta_data: &'a M,
    _voc_types: PhantomData<fn(T)->V>
}

impl<'a, D, T, V, M: MetadataManager> MetadataContainerWithDict<'a, D, T, V, M> {
    pub fn new(
        dict: *const D,
        meta_data: &'a M,
    ) -> Self {
        Self {
            dict,
            meta_data,
            _voc_types: PhantomData
        }
    }

    #[inline(always)]
    pub fn dict(&self) -> &'a D {
        unsafe {&*self.dict}
    }
}

impl<'a, D, T, V, M: MetadataManager> MetadataContainerWithDict<'a, D, T, V, M>
where
    D: BasicDictionaryWithMeta<M, V> + BasicDictionaryWithVocabulary<V>,
    V: AnonymousVocabulary
{
    pub fn wrap(target: &'a D) -> Self {
        let ptr = target as *const D;
        Self::new(
            ptr,
            target.metadata()
        )
    }
}

impl<D, T, V, M: MetadataManager> Deref for MetadataContainerWithDict<'_, D, T, V, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        self.meta_data
    }
}

impl<D, T, V, M> Display for MetadataContainerWithDict<'_, D, T, V, M>
where
    D: DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T> + AnonymousVocabulary,
    T: Display,
    M: MetadataManager,
    for<'a> <M as MetadataManager>::Reference<'a>: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metadata A:\n")?;
        if self.meta_a().is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in 0..self.meta_a().len() {
                if let Some(value) = self.get_meta_ref::<A>(
                    self.dict().voc_a(),
                    word_id
                ) {
                    write!(f, "    {}: {}\n", self.dict().id_to_word::<AToB>(word_id).unwrap(), value)?;
                }
            }
        }

        write!(f, "\n------\n")?;
        write!(f, "Metadata B:\n")?;
        if self.meta_b().is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in 0..self.meta_b().len() {
                if let Some(value) = self.get_meta_ref::<B>(
                    self.dict().voc_b(),
                    word_id
                ) {
                    write!(f, "    {}: {}\n", self.dict().id_to_word::<BToA>(word_id).unwrap(), value)?;
                }
            }
        }

        Ok(())
    }
}


pub struct MetadataContainerWithDictMut<'a, D, T, V, M: MetadataManager> {
    dict: *mut D,
    meta_data: &'a mut M,
    _voc_types: PhantomData<fn(T)->V>
}

impl<'a, D, T, V, M: MetadataManager> MetadataContainerWithDictMut<'a, D, T, V, M> {
    pub fn new(
        dict: *mut D,
        meta_data: &'a mut M,
    ) -> Self {
        Self {
            dict,
            meta_data,
            _voc_types: PhantomData
        }
    }

    #[inline(always)]
    pub fn dict(&self) -> &'a mut D {
        unsafe {&mut *self.dict}
    }
}

impl<'a, D, T, V, M: MetadataManager> MetadataContainerWithDictMut<'a, D, T, V, M>
where
    D: BasicDictionaryWithMeta<M, V> + BasicDictionaryWithVocabulary<V>,
    V: AnonymousVocabulary
{
    pub fn wrap(target: &'a mut D) -> Self {
        let ptr = target as *mut D;
        Self::new(
            ptr,
            target.metadata_mut()
        )
    }
}

impl<D, T, V, M: MetadataManager> Deref for MetadataContainerWithDictMut<'_, D, T, V, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        self.meta_data
    }
}

impl<D, T, V, M: MetadataManager> DerefMut for MetadataContainerWithDictMut<'_, D, T, V, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.meta_data
    }
}


impl<D, T, V, M: MetadataManager> Display for MetadataContainerWithDictMut<'_, D, T, V, M>
where
    D: DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T> + AnonymousVocabulary,
    T: Display,
    M: MetadataManager,
    for<'a> <M as MetadataManager>::Reference<'a>: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metadata A:\n")?;
        if self.meta_a().is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in 0..self.meta_a().len() {
                if let Some(value) = self.get_meta_ref::<A>(
                    self.dict().voc_a(),
                    word_id
                ) {
                    write!(f, "    {}: {}\n", self.dict().id_to_word::<AToB>(word_id).unwrap(), value)?;
                }
            }
        }

        write!(f, "\n------\n")?;
        write!(f, "Metadata B:\n")?;
        if self.meta_b().is_empty() {
            write!(f, "  ==UNSET==\n")?;
        } else {
            for word_id in 0..self.meta_b().len() {
                if let Some(value) = self.get_meta_ref::<B>(
                    self.dict().voc_b(),
                    word_id
                ) {
                    write!(f, "    {}: {}\n", self.dict().id_to_word::<BToA>(word_id).unwrap(), value)?;
                }
            }
        }

        Ok(())
    }
}