use std::borrow::Borrow;
use std::hash::Hash;
use crate::topicmodel::dictionary::direction::{Direction, DirectionTuple, Language, Translation, A, B};
use crate::topicmodel::dictionary::iterators::{DictIter, DictIterImpl, DictLangIter};
use crate::topicmodel::dictionary::metadata::containers::MetadataManager;
use crate::topicmodel::dictionary::metadata::DictionaryWithMetaIter;
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary, VocabularyMut};

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
    fn voc_a(&self) -> &V;

    fn voc_b(&self) -> &V;

}

/// A dictionary with known vocabulary types.
pub trait DictionaryWithVocabulary<T, V>: BasicDictionaryWithVocabulary<V> where V: BasicVocabulary<T> {

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
    fn ids_to_values<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<&'a HashRef<T>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            ids.iter().map(|value| unsafe {
                self.voc_b().get_value(*value).unwrap_unchecked()
            }).collect()
        } else {
            ids.iter().map(|value| unsafe {
                self.voc_a().get_value(*value).unwrap_unchecked()
            }).collect()
        }
    }

    /// Translates a single [word_id]
    fn translate_id<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<(usize, &'a HashRef<T>)>> where V: 'a {
        Some(self.ids_to_id_entry::<D>(self.translate_id_to_ids::<D>(word_id)?))
    }

    /// Translates a single [word_id]
    fn translate_id_to_values<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<&'a HashRef<T>>> where V: 'a {
        Some(self.ids_to_values::<D>(self.translate_id_to_ids::<D>(word_id)?))
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
        Some(self.ids_to_values::<D>(self.translate_value_to_ids::<D, Q>(word)?))
    }
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
}

pub trait DictionaryFilterable<T, V>: DictionaryMut<T, V> where T: Eq + Hash, V: VocabularyMut<T> + Default {
    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized;

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<T>) -> bool, Fb: Fn(&'a HashRef<T>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, T: 'a;
}


pub trait FromVoc<T, V>: DictionaryWithVocabulary<T, V> where T: Eq + Hash, V: BasicVocabulary<T> {
    fn from_voc(voc_a: V, voc_b: V) -> Self;
    fn from_voc_lang<L: Language>(voc: V, other_lang: Option<LanguageHint>) -> Self;
}


pub trait BasicDictionaryWithMeta<M: MetadataManager>: BasicDictionary {
    fn metadata(&self) -> &M;
    fn metadata_mut(&mut self) -> &mut M;

    fn iter_with_meta(&self) -> DictionaryWithMetaIter<Self, M> {
        DictionaryWithMetaIter::new(self)
    }
}