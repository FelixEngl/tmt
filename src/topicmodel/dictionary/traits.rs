mod generic_traits;

pub use generic_traits::*;

use std::borrow::Borrow;
use std::hash::Hash;
use crate::topicmodel::dictionary::direction::{DirectionKind, DirectionTuple, A, B};
use crate::topicmodel::dictionary::iterators::{DictIter, DictIterImpl, DictLangIter};
use crate::topicmodel::dictionary::len::Len;
use crate::topicmodel::dictionary::metadata::containers::MetadataManager;
use crate::topicmodel::dictionary::metadata::{DictionaryWithMetaIter, MetadataMutReference};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut, BasicVocabulary, SearchableVocabulary, VocabularyMut};

/// A basic dictionary that can translate IDs
pub trait BasicDictionary: Send + Sync {

    fn map_a_to_b(&self) -> &Vec<Vec<usize>>;

    fn map_b_to_a(&self) -> &Vec<Vec<usize>>;

    fn translate_id_a_to_id_b(&self, word_id: usize) -> Option<&Vec<usize>> {
        self.map_a_to_b().get(word_id)
    }
    fn translate_id_b_to_id_a(&self, word_id: usize) -> Option<&Vec<usize>> {
        self.map_b_to_a().get(word_id)
    }

    /// Switches language a and b
    fn switch_languages(self) -> Self where Self: Sized;

    /// Iterates over all mappings (a to b and b to a), does not filter for uniqueness.
    fn iter(&self) -> DictIter {
        DictIterImpl::new(self)
    }
}



/// A basic dictionary with a vocabulary
pub trait BasicDictionaryWithVocabulary<V>: BasicDictionary {
    fn voc_a(&self) -> &V;

    fn voc_b(&self) -> &V;

    fn voc_a_mut(&mut self) -> &mut V;

    fn voc_b_mut(&mut self) -> &mut V;
}

/// A dictionary with known vocabulary types.
pub trait DictionaryWithVocabulary<T, V>: BasicDictionaryWithVocabulary<V> where V: BasicVocabulary<T> {

    /// returns the length informations about a dictionary
    fn len(&self) -> Len {
        Len {
            voc_a: self.voc_a().len(),
            voc_b: self.voc_b().len(),
            map_a_to_b: self.map_a_to_b().len(),
            map_b_to_a: self.map_b_to_a().len(),
        }
    }


    /// The typed access based on the language
    fn language_a<'a>(&'a self) -> Option<&'a LanguageHint> where V: 'a {
        self.voc_a().language()
    }

    /// The typed access based on the language
    fn language_b<'a>(&'a self) -> Option<&'a LanguageHint> where V: 'a {
        self.voc_b().language()
    }

    /// Returns the direction of the dictionary
    fn language_direction_a_to_b<'a>(&'a self) -> (Option<&'a LanguageHint>, Option<&'a LanguageHint>) where V: 'a {
        (self.language_a(), self.language_b())
    }

    /// Returns the direction of the dictionary
    fn language_direction_b_to_a<'a>(&'a self) -> (Option<&'a LanguageHint>, Option<&'a LanguageHint>) where V: 'a {
        (self.language_b(), self.language_a())
    }


    /// Check if the translation is possible
    fn can_translate_a_to_b(&self, id: usize) -> bool {
        self.voc_a().contains_id(id) && self.map_a_to_b().get(id).is_some_and(|value| !value.is_empty())
    }

    /// Check if the translation is possible
    fn can_translate_b_to_a(&self, id: usize) -> bool {
        self.voc_b().contains_id(id) && self.map_b_to_a().get(id).is_some_and(|value| !value.is_empty())
    }

    /// Convert an ID to a word
    fn convert_id_a_to_word<'a>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        self.voc_a().get_value(id)
    }

    /// Convert an ID to a word
    fn convert_id_b_to_word<'a>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        self.voc_b().get_value(id)
    }

    /// Convert ids to ids with entries
    fn convert_ids_a_to_id_entries<'a, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        ids.into_iter().map(|value| unsafe {
            self.voc_a().get_id_entry(value).unwrap_unchecked()
        }).collect()
    }

    /// Convert ids to ids with entries
    fn convert_ids_b_to_id_entries<'a, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        ids.into_iter().map(|value| unsafe {
            self.voc_b().get_id_entry(value).unwrap_unchecked()
        }).collect()
    }

    /// Convert ids to values
    fn convert_ids_a_to_values<'a, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<&'a HashRef<T>> where V: 'a {
        ids.into_iter().map(|value| unsafe {
            self.voc_a().get_value(value).unwrap_unchecked()
        }).collect()
    }

    /// Convert ids to values
    fn convert_ids_b_to_values<'a, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<&'a HashRef<T>> where V: 'a {
        ids.into_iter().map(|value| unsafe {
            self.voc_b().get_value(value).unwrap_unchecked()
        }).collect()
    }

    /// Translates [word_id] to the entries.
    fn translate_id_a_to_entries_b<'a>(&'a self, word_id: usize) -> Option<Vec<(usize, &'a HashRef<T>)>> where V: 'a {
        Some(self.convert_ids_b_to_id_entries(self.translate_id_a_to_id_b(word_id)?.into_iter().copied()))
    }

    /// Translates [word_id] to the entries.
    fn translate_id_b_to_entries_a<'a>(&'a self, word_id: usize) -> Option<Vec<(usize, &'a HashRef<T>)>> where V: 'a {
        Some(self.convert_ids_a_to_id_entries(self.translate_id_b_to_id_a(word_id)?.into_iter().copied()))
    }

    /// Translates a single [word_id]
    fn translate_id_a_to_words_b<'a>(&'a self, word_id: usize) -> Option<Vec<&'a HashRef<T>>> where V: 'a {
        Some(self.convert_ids_b_to_values(self.translate_id_a_to_id_b(word_id)?.iter().copied()))
    }

    /// Translates a single [word_id]
    fn translate_id_b_to_words_a<'a>(&'a self, word_id: usize) -> Option<Vec<&'a HashRef<T>>> where V: 'a {
        Some(self.convert_ids_a_to_values(self.translate_id_b_to_id_a(word_id)?.iter().copied()))
    }

    fn iter_language_a<'a>(&'a self) -> DictLangIter<'a, T, Self, V> where V: 'a, Self: Sized {
        DictLangIter::<T, Self, V>::new::<A>(self)
    }

    fn iter_language_b<'a>(&'a self) -> DictLangIter<'a, T, Self, V> where V: 'a, Self: Sized {
        DictLangIter::<T, Self, V>::new::<B>(self)
    }

    fn translate_word_a_to_ids_b<Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.translate_id_a_to_id_b(self.voc_a().get_id(word)?)
    }

    fn translate_word_b_to_ids_a<Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.translate_id_b_to_id_a(self.voc_a().get_id(word)?)
    }


    fn translate_word_a_to_entries_b<'a, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<(usize, &'a HashRef<T>)>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T> + 'a
    {
        Some(self.convert_ids_b_to_id_entries(self.translate_word_a_to_ids_b::<Q>(word)?.into_iter().copied()))
    }

    fn translate_word_b_to_entries_a<'a, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<(usize, &'a HashRef<T>)>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T> + 'a
    {
        Some(self.convert_ids_a_to_id_entries(self.translate_word_b_to_ids_a::<Q>(word)?.into_iter().copied()))
    }

    fn word_to_id_a<Q: ?Sized>(&self, word: &Q) -> Option<usize>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.voc_a().get_id(word)
    }

    fn word_to_id_b<Q: ?Sized>(&self, id: &Q) -> Option<usize>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.voc_b().get_id(id)
    }

    fn can_translate_word_a<Q: ?Sized>(&self, word: &Q) -> bool
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.word_to_id_a(word).is_some_and(|value| self.can_translate_a_to_b(value))
    }

    fn can_translate_word_b<Q: ?Sized>(&self, word: &Q) -> bool
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.word_to_id_b(word).is_some_and(|value| self.can_translate_b_to_a(value))
    }

    fn translate_word_a_to_words_b<'a, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<&'a HashRef<T>>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: 'a + SearchableVocabulary<T>
    {
        Some(self.convert_ids_b_to_values(self.translate_word_a_to_ids_b(word)?.iter().copied()))
    }

    fn translate_word_b_to_words_a<'a, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<&'a HashRef<T>>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: 'a + SearchableVocabulary<T>
    {
        Some(self.convert_ids_a_to_values(self.translate_word_b_to_ids_a(word)?.iter().copied()))
    }
}

pub trait MergingDictionary<T, V>: DictionaryWithVocabulary<T, V> where V: BasicVocabulary<T> {
    /// Allows to merge two similar dictionary
    fn merge(self, other: impl Into<Self>) -> Self where Self: Sized;
}

pub trait DictionaryMut<T, V>: DictionaryWithVocabulary<T, V>
where
    T: Eq + Hash,
    V: VocabularyMut<T>
{
    fn set_language_a(&mut self, value: Option<LanguageHint>) -> Option<LanguageHint> {
        self.voc_a_mut().set_language(value)
    }
    fn set_language_b(&mut self, value: Option<LanguageHint>) -> Option<LanguageHint>{
        self.voc_b_mut().set_language(value)
    }

    unsafe fn reserve_for_single_value_a(&mut self, word_id: usize);
    unsafe fn reserve_for_single_value_b(&mut self, word_id: usize);

    fn insert_single_hash_ref_a(&mut self, word: HashRef<T>) -> usize {
        let word_id =  self.voc_a_mut().add_hash_ref(word);
        unsafe{self.reserve_for_single_value_a(word_id);}
        word_id
    }
    fn insert_single_hash_ref_b(&mut self, word: HashRef<T>) -> usize {
        let word_id =  self.voc_b_mut().add_hash_ref(word);
        unsafe{self.reserve_for_single_value_b(word_id);}
        word_id
    }

    fn insert_single_word_a(&mut self, word: T) -> usize {
        self.insert_single_hash_ref_a(HashRef::new(word))
    }
    fn insert_single_word_b(&mut self, word: T) -> usize {
        self.insert_single_hash_ref_b(HashRef::new(word))
    }

    fn insert_single_a(&mut self, word: impl Into<T>) -> usize {
        self.insert_single_word_a(word.into())
    }
    fn insert_single_b(&mut self, word: impl Into<T>) -> usize {
        self.insert_single_word_b(word.into())
    }

    unsafe fn insert_raw_values_a_to_b(&mut self, word_id_a: usize, word_id_b: usize);
    unsafe fn insert_raw_values_b_to_a(&mut self, word_id_a: usize, word_id_b: usize);
    unsafe fn insert_raw_values_invariant(&mut self, word_id_a: usize, word_id_b: usize) {
        self.insert_raw_values_a_to_b(word_id_a, word_id_b);
        self.insert_raw_values_b_to_a(word_id_a, word_id_b);
    }

    fn insert_hash_ref_a_to_b(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        let id_a = self.voc_a_mut().add_hash_ref(word_a);
        let id_b = self.voc_b_mut().add_hash_ref(word_b);
        unsafe { self.insert_raw_values_a_to_b(id_a, id_b); }
        DirectionTuple::new(id_a, id_b, DirectionKind::AToB)
    }
    fn insert_hash_ref_b_to_a(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        let id_a = self.voc_a_mut().add_hash_ref(word_a);
        let id_b = self.voc_b_mut().add_hash_ref(word_b);
        unsafe { self.insert_raw_values_b_to_a(id_a, id_b); }
        DirectionTuple::new(id_a, id_b, DirectionKind::BToA)
    }
    fn insert_hash_ref_invariant(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        let id_a = self.voc_a_mut().add_hash_ref(word_a);
        let id_b = self.voc_b_mut().add_hash_ref(word_b);
        unsafe { self.insert_raw_values_invariant(id_a, id_b); }
        DirectionTuple::new(id_a, id_b, DirectionKind::Invariant)
    }
    fn insert_hash_ref_dir(&mut self, dir: DirectionKind, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        match dir {
            DirectionKind::AToB => {
                self.insert_hash_ref_a_to_b(word_a, word_b)
            }
            DirectionKind::BToA => {
                self.insert_hash_ref_b_to_a(word_a, word_b)
            }
            DirectionKind::Invariant => {
                self.insert_hash_ref_invariant(word_a, word_b)
            }
        }
    }


    fn insert_value_a_to_b(&mut self, word_a: T, word_b: T) -> DirectionTuple<usize, usize> {
        self.insert_hash_ref_a_to_b(HashRef::new(word_a), HashRef::new(word_b))
    }
    fn insert_value_b_to_a(&mut self, word_a: T, word_b: T) -> DirectionTuple<usize, usize> {
        self.insert_hash_ref_b_to_a(HashRef::new(word_a), HashRef::new(word_b))
    }
    fn insert_value_invariant(&mut self, word_a: T, word_b: T) -> DirectionTuple<usize, usize> {
        self.insert_hash_ref_invariant(HashRef::new(word_a), HashRef::new(word_b))
    }
    fn insert_value_dir(&mut self, dir: DirectionKind, word_a: T, word_b: T) -> DirectionTuple<usize, usize> {
        self.insert_hash_ref_dir(dir, HashRef::new(word_a), HashRef::new(word_b))
    }

    fn insert_a_to_b(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) -> DirectionTuple<usize, usize> {
        self.insert_value_a_to_b(word_a.into(), word_b.into())
    }
    fn insert_b_to_a(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) -> DirectionTuple<usize, usize> {
        self.insert_value_b_to_a(word_a.into(), word_b.into())
    }
    fn insert_invariant(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) -> DirectionTuple<usize, usize> {
        self.insert_value_invariant(word_a.into(), word_b.into())
    }
    fn insert_dir(&mut self, dir: DirectionKind, word_a: impl Into<T>, word_b: impl Into<T>) -> DirectionTuple<usize, usize> {
        self.insert_value_dir(dir, word_a.into(), word_b.into())
    }

    fn delete_translations_of_word_a<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>;

    fn delete_translations_of_word_b<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>;


}

pub trait DictionaryFilterable<T, V>: DictionaryMut<T, V> where T: Eq + Hash, V: VocabularyMut<T> {

    /// Filters and processes the contents of the dictionary to create a new one.
    fn filter_and_process<'a, Fa, Fb, E>(&'a self, f_a: Fa, f_b: Fb) -> Result<Self, E>
    where
        Self: Sized,
        T: 'a,
        Fa: Fn(&'a HashRef<T>) -> Result<Option<HashRef<T>>, E>,
        Fb: Fn(&'a HashRef<T>) -> Result<Option<HashRef<T>>, E>;

    fn filter_by_ids<Fa, Fb>(&self, filter_a: Fa, filter_b: Fb) -> Self
    where
        Self: Sized,
        Fa: Fn(usize) -> bool,
        Fb: Fn(usize) -> bool;

    fn filter_by_values<'a, Fa, Fb>(&'a self, filter_a: Fa, filter_b: Fb) -> Self
    where
        Self: Sized,
        T: 'a,
        Fa: Fn(&'a HashRef<T>) -> bool,
        Fb: Fn(&'a HashRef<T>) -> bool;
}


pub trait FromVoc<T, V> where T: Eq + Hash, V: BasicVocabulary<T> {
    fn from_voc(voc_a: V, voc_b: V) -> Self;
    fn from_voc_lang_a(voc: V, other_lang: Option<LanguageHint>) -> Self;
    fn from_voc_lang_b(other_lang: Option<LanguageHint>, voc: V) -> Self;
}


#[allow(clippy::needless_lifetimes)]
pub trait BasicDictionaryWithMeta<M, V>: BasicDictionary
where
    M: MetadataManager,
    V: AnonymousVocabulary
{

    fn metadata(&self) -> &M;

    fn metadata_mut(&mut self) -> &mut M;

    fn iter_with_meta(&self) -> DictionaryWithMetaIter<Self, M, V> {
        DictionaryWithMetaIter::new(self)
    }

    fn get_meta_for_a<'a>(&'a self, word_id: usize) -> Option<<M as MetadataManager>::Reference<'a>>;

    fn get_meta_for_b<'a>(&'a self, word_id: usize) -> Option<<M as MetadataManager>::Reference<'a>>;
}

#[allow(clippy::needless_lifetimes)]
pub trait BasicDictionaryWithMutMeta<M, V>: BasicDictionaryWithMeta<M, V>
where
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut
{
    fn get_mut_meta_a<'a>(&'a mut self, word_id: usize) -> Option<<M as MetadataManager>::MutReference<'a>>;
    fn get_mut_meta_b<'a>(&'a mut self, word_id: usize) -> Option<<M as MetadataManager>::MutReference<'a>>;

    fn get_or_create_meta_a<'a>(&'a mut self, word_id: usize) -> <M as MetadataManager>::MutReference<'a>;
    fn get_or_create_meta_b<'a>(&'a mut self, word_id: usize) -> <M as MetadataManager>::MutReference<'a>;
}




pub trait MutableDictionaryWithMeta<M, T, V> : DictionaryMut<T, V> + BasicDictionaryWithMutMeta<M, V>
where
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut + VocabularyMut<T>,
    T: Eq + Hash,
{
    fn insert_single_ref_with_meta_to_a(
        &mut self,
        word: HashRef<T>,
        meta: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<usize, (usize, <M as MetadataManager>::UpdateError)> {
        let word_id = self.insert_single_hash_ref_a(word);
        if let Some(meta_to_add) = meta {
            self.get_or_create_meta_a(word_id)
                .update_with_resolved(meta_to_add)
                .map_err(|value| (word_id, value))?;
        }
        Ok(word_id)
    }

    fn insert_single_ref_with_meta_to_b(
        &mut self,
        word: HashRef<T>,
        meta: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<usize, (usize, <M as MetadataManager>::UpdateError)> {
        let word_id = self.insert_single_hash_ref_b(word);
        if let Some(meta_to_add) = meta {
            self.get_or_create_meta_b(word_id)
                .update_with_resolved(meta_to_add)
                .map_err(|value| (word_id, value))?;
        }
        Ok(word_id)
    }

    fn insert_translation_ref_with_meta_a_to_b(
        &mut self,
        word_a: HashRef<T>,
        meta_a: Option<&<M as MetadataManager>::ResolvedMetadata>,
        word_b: HashRef<T>,
        meta_b: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<(usize, usize), (usize, <M as MetadataManager>::UpdateError)> {
        let id_a = self.insert_single_ref_with_meta_to_a(word_a, meta_a)?;
        let id_b = self.insert_single_ref_with_meta_to_b(word_b, meta_b)?;
        unsafe { self.insert_raw_values_a_to_b(id_a, id_b); }
        Ok((id_a, id_b))
    }

    fn insert_translation_ref_with_meta_b_to_a(
        &mut self,
        word_a: HashRef<T>,
        meta_a: Option<&<M as MetadataManager>::ResolvedMetadata>,
        word_b: HashRef<T>,
        meta_b: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<(usize, usize), (usize, <M as MetadataManager>::UpdateError)> {
        let id_a = self.insert_single_ref_with_meta_to_a(word_a, meta_a)?;
        let id_b = self.insert_single_ref_with_meta_to_b(word_b, meta_b)?;
        unsafe { self.insert_raw_values_b_to_a(id_a, id_b); }
        Ok((id_a, id_b))
    }

    fn insert_translation_ref_with_meta_invariant(
        &mut self,
        word_a: HashRef<T>,
        meta_a: Option<&<M as MetadataManager>::ResolvedMetadata>,
        word_b: HashRef<T>,
        meta_b: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<(usize, usize), (usize, <M as MetadataManager>::UpdateError)> {
        let id_a = self.insert_single_ref_with_meta_to_a(word_a, meta_a)?;
        let id_b = self.insert_single_ref_with_meta_to_b(word_b, meta_b)?;
        unsafe { self.insert_raw_values_invariant(id_a, id_b); }
        Ok((id_a, id_b))
    }

    fn insert_translation_ref_with_meta_dir(
        &mut self,
        direction: DirectionKind,
        word_a: HashRef<T>,
        meta_a: Option<&<M as MetadataManager>::ResolvedMetadata>,
        word_b: HashRef<T>,
        meta_b: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<(usize, usize), (usize, <M as MetadataManager>::UpdateError)> {
        match direction {
            DirectionKind::AToB => {
                self.insert_translation_ref_with_meta_a_to_b(word_a, meta_a, word_b, meta_b)
            }
            DirectionKind::BToA => {
                self.insert_translation_ref_with_meta_b_to_a(word_a, meta_a, word_b, meta_b)
            }
            DirectionKind::Invariant => {
                self.insert_translation_ref_with_meta_invariant(word_a, meta_a, word_b, meta_b)
            }
        }
    }
}

impl<M, V, T, D>  MutableDictionaryWithMeta<M, T, V> for D
where
    D: DictionaryMut<T, V> + BasicDictionaryWithMutMeta<M, V>,
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut + VocabularyMut<T>,
    T: Eq + Hash
{}


// #[cfg(test)]
// pub mod tests {
//     use std::hash::Hash;
//     use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithVocabulary, DictionaryWithVocabulary};
//     use crate::topicmodel::dictionary::direction::{DirectionKind, DirectionTuple};
//     use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};
//     use crate::voc;
//
//     pub trait TestableBasicDictionary<T, V>
//     where
//         T: Eq + Hash,
//         V: VocabularyMut<T>
//     {
//         fn initialize_for_tests(
//             voc_a: V,
//             map_a_to_b: Vec<Vec<usize>>,
//             voc_b: V,
//             map_b_to_a: Vec<Vec<usize>>
//         ) -> Self;
//     }
//
//     fn test_data() -> ((Vocabulary<String>, Vec<Vec<usize>>), (Vocabulary<String>, Vec<Vec<usize>>)) {
//         let voc_a = voc! {
//             for "en":
//             "cat",
//             "dog",
//             "mouse",
//             "car",
//             "automobile",
//             "plane",
//             "elephant",
//             "tiger"
//         };
//         let voc_b = voc! {
//             for "de":
//             "katze",
//             "hund",
//             "maus",
//             "elefant",
//             "tiger",
//             "auto",
//             "flugzeug"
//         };
//
//         let a_to_b = vec![
//             vec![voc_b.get_id("katze").unwrap()],
//             vec![voc_b.get_id("hund").unwrap()],
//             vec![voc_b.get_id("maus").unwrap()],
//             vec![voc_b.get_id("auto").unwrap()],
//             vec![voc_b.get_id("auto").unwrap()],
//             vec![voc_b.get_id("flugzeug").unwrap()],
//             vec![voc_b.get_id("elefant").unwrap()],
//             vec![voc_b.get_id("tiger").unwrap()],
//         ];
//
//         assert_eq!(a_to_b.len(), voc_a.len());
//
//         let b_to_a = vec![
//             vec![voc_a.get_id("cat").unwrap()],
//             vec![voc_a.get_id("dog").unwrap()],
//             vec![voc_a.get_id("mouse").unwrap()],
//             vec![voc_a.get_id("elephant").unwrap()],
//             vec![voc_a.get_id("tiger").unwrap()],
//             vec![voc_a.get_id("car").unwrap(), voc_a.get_id("automobile").unwrap()],
//             vec![voc_a.get_id("plane").unwrap()],
//         ];
//         assert_eq!(b_to_a.len(), voc_b.len());
//
//         ((voc_a, a_to_b), (voc_b, b_to_a))
//     }
//
//     fn initialize<D>() -> D
//     where
//         D: TestableBasicDictionary<String, Vocabulary<String>>
//     {
//         let((voc_a, a_to_b), (voc_b, b_to_a)) = test_data();
//
//         D::initialize_for_tests(
//             voc_a,
//             a_to_b,
//             voc_b,
//             b_to_a
//         )
//     }
//
//     pub fn test_basic_dictionary<D, T, V>()
//     where
//         D: TestableBasicDictionary<T, V> + BasicDictionary,
//         T: Eq + Hash,
//         V: VocabularyMut<T>
//     {
//         let((voc_a, a_to_b), (voc_b, b_to_a)) = test_data();
//
//         let initialized = initialize::<D>();
//         assert_eq!(a_to_b.len(), initialized.map_a_to_b().len());
//         assert_eq!(b_to_a.len(), initialized.map_b_to_a().len());
//         assert_eq!(initialized.translate_id_a_to_id_b(0), a_to_b.get(0));
//         assert_eq!(initialized.translate_id_b_to_id_a(5), b_to_a.get(5));
//         for DirectionTuple { a, b, direction } in initialized.iter() {
//             match direction {
//                 DirectionKind::AToB => {
//                     assert!(initialized.translate_id_a_to_id_b(a).is_some_and(|value| value.contains(&b)));
//                 }
//                 DirectionKind::BToA => {
//                     assert!(initialized.translate_id_b_to_id_a(b).is_some_and(|value| value.contains(&a)));
//                 }
//                 DirectionKind::Invariant => {
//                     assert!(initialized.translate_id_a_to_id_b(a).is_some_and(|value| value.contains(&b)));
//                     assert!(initialized.translate_id_b_to_id_a(b).is_some_and(|value| value.contains(&a)));
//                 }
//             }
//         }
//
//         let initialized = initialized.switch_languages();
//         assert_eq!(b_to_a.len(), initialized.map_a_to_b().len());
//         assert_eq!(a_to_b.len(), initialized.map_b_to_a().len());
//         assert_eq!(initialized.translate_id_a_to_id_b(0), b_to_a.get(0));
//         assert_eq!(initialized.translate_id_b_to_id_a(5), a_to_b.get(5));
//         for DirectionTuple { a, b, direction } in initialized.iter() {
//             match direction {
//                 DirectionKind::AToB => {
//                     assert!(initialized.translate_id_a_to_id_b(a).is_some_and(|value| value.contains(&b)));
//                 }
//                 DirectionKind::BToA => {
//                     assert!(initialized.translate_id_b_to_id_a(b).is_some_and(|value| value.contains(&a)));
//                 }
//                 DirectionKind::Invariant => {
//                     assert!(initialized.translate_id_a_to_id_b(a).is_some_and(|value| value.contains(&b)));
//                     assert!(initialized.translate_id_b_to_id_a(b).is_some_and(|value| value.contains(&a)));
//                 }
//             }
//         }
//     }
//
//     pub fn test_basic_dictionary_with_vocabulary<D, T, V>()
//     where
//         D: TestableBasicDictionary<T, V> + BasicDictionaryWithVocabulary<V>,
//         T: Eq + Hash,
//         V: VocabularyMut<T> + PartialEq<V> + Eq
//     {
//         let((voc_a, a_to_b), (voc_b, b_to_a)) = test_data();
//
//         let mut initialized = initialize::<D>();
//         assert_eq!(&voc_a, initialized.voc_a());
//         assert_eq!(&voc_b, initialized.voc_b());
//         assert_eq!(&voc_a, initialized.voc_a_mut());
//         assert_eq!(&voc_b, initialized.voc_b_mut());
//
//         let mut initialized = initialized.switch_languages();
//         assert_eq!(&voc_b, initialized.voc_a(), "Switching failed!");
//         assert_eq!(&voc_a, initialized.voc_b(), "Switching failed!");
//         assert_eq!(&voc_b, initialized.voc_a_mut(), "Switching failed!");
//         assert_eq!(&voc_a, initialized.voc_b_mut(), "Switching failed!");
//     }
//
//     pub fn test_dictionary_with_vocabulary<D, T, V>()
//     where
//         D: TestableBasicDictionary<T, V> + DictionaryWithVocabulary<T, V>,
//         T: Eq + Hash,
//         V: VocabularyMut<T>
//     {
//         let((voc_a, a_to_b), (voc_b, b_to_a)) = test_data();
//
//         let mut initialized = initialize::<D>();
//
//         {
//             let len = initialized.len();
//             assert_eq!(len.voc_a, voc_a.len());
//             assert_eq!(len.voc_b, voc_b.len());
//             assert_eq!(len.map_a_to_b, a_to_b.len());
//             assert_eq!(len.map_b_to_a, b_to_a.len());
//         }
//
//         assert_eq!((voc_a.language(), voc_b.language()), initialized.language_direction_a_to_b());
//         assert_eq!((voc_b.language(), voc_a.language()), initialized.language_direction_b_to_a());
//         assert_eq!(voc_a.language(), initialized.language_a());
//         assert_eq!(voc_b.language(), initialized.language_b());
//
//         assert!(initialized.can_translate_a_to_b(0));
//         assert!(initialized.can_translate_a_to_b(5));
//         assert!(!initialized.can_translate_a_to_b(7));
//         assert!(!initialized.can_translate_a_to_b(300));
//
//         assert!(initialized.can_translate_b_to_a(0));
//         assert!(initialized.can_translate_b_to_a(5));
//         assert!(!initialized.can_translate_b_to_a(7));
//         assert!(!initialized.can_translate_b_to_a(300));
//
//         for id in 0..voc_a.len() {
//             let word = initialized.convert_id_a_to_word(id).expect("This should never be None!");
//             assert_eq!(word, voc_a.get_value(id).unwrap())
//         }
//     }
// }