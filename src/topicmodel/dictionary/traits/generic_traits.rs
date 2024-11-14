use std::borrow::Borrow;
use std::hash::Hash;
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, BasicDictionaryWithVocabulary, DictionaryMut, DictionaryWithVocabulary, FromVoc, MutableDictionaryWithMeta};
use crate::topicmodel::dictionary::direction::{Direction, DirectionKind, DirectionTuple, Language, Translation};
use crate::topicmodel::dictionary::iterators::DictLangIter;
use crate::topicmodel::dictionary::metadata::MetadataManager;
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut, BasicVocabulary, SearchableVocabulary, VocabularyMut};

/// A generified trait
pub trait BasicDictionaryGen: BasicDictionary {
    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        if D::DIRECTION.is_a_to_b() {
            self.map_a_to_b().get(word_id)
        } else {
            self.map_b_to_a().get(word_id)
        }
    }
}

impl<D> BasicDictionaryGen for D where D: BasicDictionary {}


pub trait BasicDictionaryWithVocabularyGen<V>: BasicDictionaryWithVocabulary<V> {
    fn voc<L: Language>(&self) -> &V {
        if L::LANG.is_a() {
            self.voc_a()
        } else {
            self.voc_b()
        }
    }

    fn voc_mut<L: Language>(&mut self) -> &mut V {
        if L::LANG.is_a() {
            self.voc_a_mut()
        } else {
            self.voc_b_mut()
        }
    }
}

impl<D, V> BasicDictionaryWithVocabularyGen<V> for D where D: BasicDictionaryWithVocabulary<V> {}


pub trait DictionaryWithVocabularyGen<T, V>: DictionaryWithVocabulary<T, V> where V: BasicVocabulary<T> {
    /// The typed access based on the language
    fn language<'a, L: Language>(&'a self) -> Option<&'a LanguageHint> where V: 'a {
        if L::LANG.is_a() {
            self.language_a()
        } else {
            self.language_b()
        }
    }

    fn language_direction<'a, D: Direction>(&'a self) -> (Option<&'a LanguageHint>, Option<&'a LanguageHint>) where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            self.language_direction_a_to_b()
        } else {
            self.language_direction_b_to_a()
        }
    }

    /// Check if the translation is possible
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        if D::DIRECTION.is_a_to_b() {
            self.can_translate_a_to_b(id)
        } else {
            self.can_translate_b_to_a(id)
        }
    }

    /// Convert an ID to a word
    fn convert_id_to_word<'a, D: Language>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        if D::LANG.is_a() {
            self.convert_id_a_to_word(id)
        } else {
            self.convert_id_b_to_word(id)
        }
    }


    /// Convert ids to ids with entries
    fn convert_ids_to_id_entries<'a, D: Language, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        if D::LANG.is_a() {
            self.convert_ids_a_to_id_entries(ids)
        } else {
            self.convert_ids_b_to_id_entries(ids)
        }
    }

    /// Convert ids to values
    fn convert_ids_to_values<'a, D: Language, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<&'a HashRef<T>> where V: 'a {
        if D::LANG.is_a() {
            self.convert_ids_a_to_values(ids)
        } else {
            self.convert_ids_b_to_values(ids)
        }
    }

    /// Iterate language [L]
    fn iter_language<'a, L: Language>(&'a self) -> DictLangIter<'a, T, Self, V> where V: 'a {
        DictLangIter::<T, Self, V>::new::<L>(self)
    }

    /// Translates a single [word_id]
    fn translate_id<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<(usize, &'a HashRef<T>)>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            self.translate_id_a_to_entries_b(word_id)
        } else {
            self.translate_id_b_to_entries_a(word_id)
        }
    }

    /// Translates a single [word_id]
    fn translate_id_to_values<'a, D: Translation>(&'a self, word_id: usize) -> Option<Vec<&'a HashRef<T>>> where V: 'a {
        if D::DIRECTION.is_a_to_b() {
            self.translate_id_a_to_words_b(word_id)
        } else {
            self.translate_id_b_to_words_a(word_id)
        }
    }

    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        if D::DIRECTION.is_a_to_b() {
            self.translate_word_a_to_ids_b(word)
        } else {
            self.translate_word_b_to_ids_a(word)
        }
    }

    /// Translate a value
    fn translate_value<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<(usize, &'a HashRef<T>)>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: 'a + SearchableVocabulary<T>
    {
        if D::DIRECTION.is_a_to_b() {
            self.translate_word_a_to_entries_b(word)
        } else {
            self.translate_word_b_to_entries_a(word)
        }
    }

    fn word_to_id<D: Language, Q: ?Sized>(&self, word: &Q) -> Option<usize>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        if D::DIRECTION.is_a_to_b() {
            self.word_to_id_a(word)
        } else {
            self.word_to_id_b(word)
        }
    }

    fn can_translate_word<D: Translation, Q: ?Sized>(&self, word: &Q) -> bool
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        if D::DIRECTION.is_a_to_b() {
            self.can_translate_word_a(word)
        } else {
            self.can_translate_word_b(word)
        }
    }


    fn translate_value_to_values<'a, D: Translation, Q: ?Sized>(&'a self, word: &Q) -> Option<Vec<&'a HashRef<T>>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: 'a + SearchableVocabulary<T>
    {
        if D::DIRECTION.is_a_to_b() {
            self.translate_word_a_to_words_b(word)
        } else {
            self.translate_word_b_to_words_a(word)
        }
    }

}

impl<D, T, V> DictionaryWithVocabularyGen<T, V> for D
where
    D:DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T>
{}


pub trait DictionaryMutGen<T, V>: DictionaryMut<T, V> where T: Eq + Hash, V: VocabularyMut<T>  {
    fn set_language<L: Language>(&mut self, value: Option<LanguageHint>) -> Option<LanguageHint> {
        if L::LANG.is_a() {
            self.set_language_a(value)
        } else {
            self.set_language_b(value)
        }
    }

    unsafe fn reserve_for_single_value<L: Language>(&mut self, word_id: usize) {
        if L::LANG.is_a() {
            self.reserve_for_single_value_a(word_id)
        } else {
            self.reserve_for_single_value_b(word_id)
        }
    }

    unsafe fn insert_raw_values<D: Direction>(&mut self, word_id_a: usize, word_id_b: usize) {
        match D::DIRECTION {
            DirectionKind::AToB => {
                self.insert_raw_values_a_to_b(word_id_a, word_id_b)
            }
            DirectionKind::BToA => {
                self.insert_raw_values_b_to_a(word_id_a, word_id_b)
            }
            DirectionKind::Invariant => {
                self.insert_raw_values_invariant(word_id_a, word_id_b)
            }
        }
    }

    fn insert_single_ref<L: Language>(&mut self, word: HashRef<T>) -> usize {
        if L::LANG.is_a() {
            self.insert_single_hash_ref_a(word)
        } else {
            self.insert_single_hash_ref_b(word)
        }
    }

    fn insert_single_value<L: Language>(&mut self, word: T) -> usize {
        self.insert_single_ref::<L>(HashRef::new(word))
    }

    fn insert_single_word<L: Language>(&mut self, word: impl Into<T>) -> usize{
        self.insert_single_value::<L>(word.into())
    }

    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        self.insert_hash_ref_dir(
            D::DIRECTION,
            word_a,
            word_b
        )
    }

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
        V: SearchableVocabulary<T>
    {
        if L::LANG.is_a() {
            self.delete_translations_of_word_a(value)
        } else {
            self.delete_translations_of_word_b(value)
        }
    }
}

impl<D, T, V> DictionaryMutGen<T, V> for D where
    D: DictionaryMut<T, V>,
    T: Eq + Hash,
    V: VocabularyMut<T>
{}

pub trait FromVocGen<T, V>: FromVoc<T, V>
where
    T: Eq + Hash,
    V: BasicVocabulary<T>
{
    fn from_voc_lang<L: Language>(voc: V, other_lang: Option<LanguageHint>) -> Self where Self: Sized {
        if L::LANG.is_a() {
            Self::from_voc_lang_a(voc, other_lang)
        } else {
            Self::from_voc_lang_b(other_lang, voc)
        }
    }
}

impl<D, T, V> FromVocGen<T, V> for D
where
    D: FromVoc<T, V>,
    T: Eq + Hash,
    V: BasicVocabulary<T>
{}

pub trait BasicDictionaryWithMetaGen<M, V>: BasicDictionaryWithMeta<M, V>
where
    M: MetadataManager,
    V: AnonymousVocabulary
{
    fn get_meta_for<'a, L: Language>(&'a mut self, word_id: usize) -> Option<<M as MetadataManager>::Reference<'a>> {
        if L::LANG.is_a() {
            self.get_meta_for_a(word_id)
        } else {
            self.get_meta_for_b(word_id)
        }
    }
}

impl<D, M, V> BasicDictionaryWithMetaGen<M, V> for D
where
    D: BasicDictionaryWithMeta<M, V>,
    M: MetadataManager,
    V: AnonymousVocabulary
{}


pub trait BasicDictionaryWithMutMetaGen<M, V>: BasicDictionaryWithMutMeta<M, V>
where
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut
{
    fn get_mut_meta_for<'a, L: Language>(&'a mut self, word_id: usize) -> Option<<M as MetadataManager>::MutReference<'a>> {
        if L::LANG.is_a() {
            self.get_mut_meta_a(word_id)
        } else {
            self.get_mut_meta_b(word_id)
        }
    }

    fn get_or_create_meta_for<'a, L: Language>(&'a mut self, word_id: usize) -> <M as MetadataManager>::MutReference<'a> {
        if L::LANG.is_a() {
            self.get_or_create_meta_a(word_id)
        } else {
            self.get_or_create_meta_b(word_id)
        }
    }
}

impl<D, M, V> BasicDictionaryWithMutMetaGen<M, V> for D
where
    D: BasicDictionaryWithMutMeta<M, V>,
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut
{}


pub trait MutableDictionaryWithMetaGen<M, T, V>: MutableDictionaryWithMeta<M, T, V>
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
        if L::LANG.is_a() {
            self.insert_single_ref_with_meta_to_a(word, meta)
        } else {
            self.insert_single_ref_with_meta_to_b(word, meta)
        }
    }

    fn insert_translation_ref_with_meta<D: Direction>(
        &mut self,
        word_a: HashRef<T>,
        meta_a: Option<&<M as MetadataManager>::ResolvedMetadata>,
        word_b: HashRef<T>,
        meta_b: Option<&<M as MetadataManager>::ResolvedMetadata>
    ) -> Result<(usize, usize), (usize, <M as MetadataManager>::UpdateError)> {
        self.insert_translation_ref_with_meta_dir(
            D::DIRECTION,
            word_a,
            meta_a,
            word_b,
            meta_b
        )
    }
}

impl<D, M, T, V> MutableDictionaryWithMetaGen<M, T, V> for D
where
    D: MutableDictionaryWithMeta<M, T, V>,
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut + VocabularyMut<T>,
    T: Eq + Hash,
{}