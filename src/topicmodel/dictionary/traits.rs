use std::borrow::Borrow;
use std::hash::Hash;
use either::Either;
use crate::topicmodel::dictionary::direction::{Direction, DirectionTuple, Language, Translation, A, B};
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

    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        if D::DIRECTION.is_a_to_b() {
            self.map_a_to_b().get(word_id)
        } else {
            self.map_b_to_a().get(word_id)
        }
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
    fn voc<L: Language>(&self) -> &V {
        if L::LANG.is_a() {
            self.voc_a()
        } else {
            self.voc_b()
        }
    }

    fn voc_a(&self) -> &V;

    fn voc_b(&self) -> &V;

    fn voc_mut<L: Language>(&mut self) -> &mut V {
        if L::LANG.is_a() {
            self.voc_a_mut()
        } else {
            self.voc_b_mut()
        }
    }

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

    /// Returns the direction of the dictionary
    fn language_direction<'a>(&'a self) -> (Option<&'a LanguageHint>, Option<&'a LanguageHint>) where V: 'a {
        (self.language::<A>(), self.language::<B>())
    }

    /// The typed access based on the language
    fn language<'a, L: Language>(&'a self) -> Option<&'a LanguageHint> where V: 'a {
        if L::LANG.is_a() {
            self.voc_a().language()
        } else {
            self.voc_b().language()
        }
    }

    /// Check if the translation is possible
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a().contains_id(id) && self.map_a_to_b().get(id).is_some_and(|value| !value.is_empty())
        } else {
            self.voc_b().contains_id(id) && self.map_b_to_a().get(id).is_some_and(|value| !value.is_empty())
        }
    }

    /// Convert an ID to a word
    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a().get_value(id)
        } else {
            self.voc_b().get_value(id)
        }
    }

    /// Convert ids to ids with entries
    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            ids.iter().map(|value| unsafe {
                self.voc_b().get_id_entry(*value).unwrap_unchecked()
            }).collect()
        } else {
            ids.iter().map(|value| unsafe {
                self.voc_a().get_id_entry(*value).unwrap_unchecked()
            }).collect()
        }
    }

    /// Convert ids to values
    fn ids_to_values<'a, D: Translation, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<&'a HashRef<T>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            ids.into_iter().map(|value| unsafe {
                self.voc_b().get_value(value).unwrap_unchecked()
            }).collect()
        } else {
            ids.into_iter().map(|value| unsafe {
                self.voc_a().get_value(value).unwrap_unchecked()
            }).collect()
        }
    }

    /// Translates a single [word_id]
    fn translate_id<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<(usize, &'a HashRef<T>)>> where V: 'a {
        Some(self.ids_to_id_entry::<D>(self.translate_id_to_ids::<D>(word_id)?))
    }

    /// Translates a single [word_id]
    fn translate_id_to_values<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<&'a HashRef<T>>> where V: 'a {
        Some(self.ids_to_values::<D, _>(self.translate_id_to_ids::<D>(word_id)?.iter().copied()))
    }

    /// Iterate language [L]
    fn iter_language<'a, L: Language>(&'a self) -> DictLangIter<'a, T, L, Self, V> where V: 'a {
        DictLangIter::<T, L, Self, V>::new(self)
    }


    /// Translate a value
    fn translate_value<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<(usize, &'a HashRef<T>)>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: 'a + SearchableVocabulary<T>
    {
        Some(self.ids_to_id_entry::<D>(self.translate_value_to_ids::<D, Q>(word)?))
    }

    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        let id = if D::DIRECTION.is_a_to_b() {
            self.voc_a().get_id(word)
        } else {
            self.voc_b().get_id(word)
        }?;
        self.translate_id_to_ids::<D>(id)
    }

    fn word_to_id<D: Translation, Q: ?Sized>(&self, id: &Q) -> Option<usize>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        if D::DIRECTION.is_a_to_b() {
            self.voc_a().get_id(id)
        } else {
            self.voc_b().get_id(id)
        }
    }

    fn can_translate_word<D: Translation, Q: ?Sized>(&self, word: &Q) -> bool
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.word_to_id::<D, _>(word).is_some_and(|value| self.can_translate_id::<D>(value))
    }

    fn translate_value_to_values<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<&'a HashRef<T>>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: 'a + SearchableVocabulary<T>
    {
        Some(self.ids_to_values::<D, _>(self.translate_value_to_ids::<D, Q>(word)?.iter().copied()))
    }
}

pub trait MergingDictionary<T, V>: DictionaryWithVocabulary<T, V> where V: BasicVocabulary<T> {
    /// Allows to merge two similar dictionary
    fn merge(self, other: impl Into<Self>) -> Self where Self: Sized;
}

pub trait DictionaryMut<T, V>: DictionaryWithVocabulary<T, V> where T: Eq + Hash, V: VocabularyMut<T> {
    fn set_language<L: Language>(&mut self, value: Option<LanguageHint>) -> Option<LanguageHint>;

    fn insert_single_word<L: Language>(&mut self, word: impl Into<T>) -> usize{
        self.insert_single_value::<L>(word.into())
    }

    fn insert_single_value<L: Language>(&mut self, word: T) -> usize {
        self.insert_single_ref::<L>(HashRef::new(word))
    }

    fn insert_single_ref<L: Language>(&mut self, word: HashRef<T>) -> usize;
    unsafe fn reserve_for_single_value<L: Language>(&mut self, word_id: usize);

    unsafe fn insert_raw_values<D: Direction>(&mut self, word_id_a: usize, word_id_b: usize);
    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize>;

    fn insert_value<D: Direction>(&mut self, word_a: T, word_b: T) -> DirectionTuple<usize, usize> {
        self.insert_hash_ref::<D>(HashRef::new(word_a), HashRef::new(word_b))
    }

    fn insert<D: Direction>(&mut self, word_a: impl Into<T>, word_b: impl Into<T>) -> DirectionTuple<usize, usize> {
        self.insert_value::<D>(word_a.into(), word_b.into())
    }

    fn delete_translation<L: Language, Q: ?Sized>(&mut self, value: &Q) -> bool
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


pub trait FromVoc<T, V>: DictionaryWithVocabulary<T, V> where T: Eq + Hash, V: BasicVocabulary<T> {
    fn from_voc(voc_a: V, voc_b: V) -> Self;
    fn from_voc_lang<L: Language>(voc: V, other_lang: Option<LanguageHint>) -> Self;
}


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

    fn get_meta_for<'a, L: Language>(&'a mut self, word_id: usize) -> Option<<M as MetadataManager>::Reference<'a>>;
}

pub trait BasicDictionaryWithMutMeta<M, V>: BasicDictionaryWithMeta<M, V>
where
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut
{
    fn get_mut_meta_for<'a, L: Language>(&'a mut self, word_id: usize) -> Option<<M as MetadataManager>::MutReference<'a>>;

    fn get_or_create_meta_for<'a, L: Language>(&'a mut self, word_id: usize) -> <M as MetadataManager>::MutReference<'a>;
}




pub trait MutableDictionaryWithMeta<M, V, T> : DictionaryMut<T, V> + BasicDictionaryWithMutMeta<M, V>
where
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut + VocabularyMut<T>,
    T: Eq + Hash,
{
    fn insert_single_ref_with_meta<L: Language>(
        &mut self,
        word: HashRef<T>,
        meta: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<usize, (usize, <M as MetadataManager>::UpdateError)> {
        let word_id = self.insert_single_ref::<L>(word);
        if let Some(meta_to_add) = meta {
            self.get_or_create_meta_for::<L>(word_id)
                .update_with_resolved(meta_to_add)
                .map_err(|value| (word_id, value))?;
        }
        Ok(word_id)
    }

    fn insert_translation_ref_with_meta<D: Direction>(
        &mut self,
        word_a: HashRef<T>,
        meta_a: Option<&<M as MetadataManager>::ResolvedMetadata>,
        word_b: HashRef<T>,
        meta_b: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<(usize, usize), (usize, <M as MetadataManager>::UpdateError)> {
        let id_a = {
            match self.insert_single_ref_with_meta::<A>(word_a, meta_a) {
                Ok(value) => value,
                Err(err) => {
                    return Err(err);
                }
            }
        };
        let id_b = {
            match self.insert_single_ref_with_meta::<B>(word_b, meta_b) {
                Ok(value) => value,
                Err(err) => {
                    return Err(err);
                }
            }
        };
        unsafe { self.insert_raw_values::<D>(id_a, id_b); }
        Ok((id_a, id_b))
    }
}

impl<M, V, T, D>  MutableDictionaryWithMeta<M, V, T> for D
where
    D: DictionaryMut<T, V> + BasicDictionaryWithMutMeta<M, V>,
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut + VocabularyMut<T>,
    T: Eq + Hash
{}