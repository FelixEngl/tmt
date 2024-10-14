use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, Dictionary, DictionaryFilterable, DictionaryMut, DictionaryWithVocabulary, FromVoc};
use crate::topicmodel::dictionary::direction::{AToB, BToA, Direction, DirectionKind, DirectionTuple, Invariant, Language, Translation, A, B};
use crate::topicmodel::dictionary::iterators::{DictIter, DictionaryWithMetaIterator};
use crate::topicmodel::dictionary::metadata::{MetadataContainer, MetadataContainerWithDict, MetadataContainerWithDictMut, MetadataRef, SolvedMetadata};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{BasicVocabulary, MappableVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};

pub struct DictionaryWithMetaIter<'a, D> where D: BasicDictionaryWithMeta + ?Sized {
    dictionary_with_meta: &'a D,
    iter: DictIter<'a>
}

impl<'a, D> DictionaryWithMetaIter<'a, D> where D: BasicDictionaryWithMeta + ?Sized {
    pub fn new(dictionary_with_meta: &'a D) -> Self {
        Self {
            iter: dictionary_with_meta.iter(),
            dictionary_with_meta
        }
    }
}

impl<'a, D> Iterator for DictionaryWithMetaIter<'a, D> where D: BasicDictionaryWithMeta {
    type Item = DirectionTuple<(usize, Option<MetadataRef<'a>>), (usize, Option<MetadataRef<'a>>)>;

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


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DictionaryWithMeta<T, V> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    pub(crate) inner: Dictionary<T, V>,
    pub(crate) metadata: MetadataContainer
}

impl<T, V> DictionaryWithMeta<T, V> {
    fn new(inner: Dictionary<T, V>, metadata: MetadataContainer) -> Self {
        Self { inner, metadata }
    }

    pub fn known_dictionaries(&self) -> Vec<&str> {
        self.metadata.dictionary_interner.iter().map(|value| value.1).collect_vec()
    }

    pub fn subjects(&self) -> Vec<&str> {
        self.metadata.subject_interner.iter().map(|value| value.1).collect_vec()
    }

    pub fn unstemmed(&self) -> &Vocabulary<String> {
        &self.metadata.unstemmed_voc
    }
}


impl<T, V> DictionaryWithMeta<T, V> where V: From<Option<LanguageHint>>  {
    pub fn new_with(language_a: Option<impl Into<LanguageHint>>, language_b: Option<impl Into<LanguageHint>>) -> Self {
        Self::new(
            Dictionary::new_with(language_a, language_b),
            MetadataContainer::new()
        )
    }
}

impl<T, V> DictionaryWithMeta<T, V> where V: BasicVocabulary<T> {
    pub fn metadata_with_dict(&self) -> MetadataContainerWithDict<Self, T, V> where Self: Sized {
        MetadataContainerWithDict::wrap(self)
    }

    pub fn metadata_with_dict_mut(&mut self) -> MetadataContainerWithDictMut<Self, T, V> where Self: Sized {
        MetadataContainerWithDictMut::wrap(self)
    }
}
unsafe impl<T, V> Send for DictionaryWithMeta<T, V>{}
unsafe impl<T, V> Sync for DictionaryWithMeta<T, V>{}
impl<T, V> DictionaryWithMeta<T, V> where V: VocabularyMut<T> + From<Option<LanguageHint>>, T: Hash + Eq  {

    fn insert_meta_for_create_subset<'a, L: Language>(&mut self, word_id: usize, metadata_ref: MetadataRef<'a>) {
        let tags = metadata_ref.raw.subjects.get();
        let dics = metadata_ref.raw.associated_dictionaries.get();
        let unstemmed = metadata_ref.raw.unstemmed.get();

        if tags.is_none() && dics.is_none() {
            return;
        }

        let meta = self.metadata.get_or_init_meta::<L>(word_id).meta;

        if let Some(dics) = dics {
            unsafe { meta.add_all_associated_dictionaries(&dics) }
        }
        if let Some(tags) = tags {
            unsafe { meta.add_all_subjects(&tags) }
        }
        if let Some(unstemmed) = unstemmed {
            meta.add_all_unstemmed(unstemmed)
        }
    }

    pub fn create_subset_with_filters<F1, F2>(&self, filter_a: F1, filter_b: F2) -> DictionaryWithMeta<T, V> where F1: Fn(&DictionaryWithMeta<T, V>, usize, Option<&MetadataRef>) -> bool, F2: Fn(&DictionaryWithMeta<T, V>, usize, Option<&MetadataRef>) -> bool {

        let mut new = Self {
            inner: Dictionary::new_with(
                self.inner.voc_a.language().cloned(),
                self.inner.voc_b.language().cloned()
            ),
            metadata: self.metadata.copy_keep_vocebulary()
        };
        for DirectionTuple{
            a: (word_id_a, meta_a),
            b: (word_id_b, meta_b),
            direction
        } in self.iter_with_meta() {
            if filter_a(self, word_id_a, meta_a.as_ref()) {
                if filter_b(self, word_id_b, meta_b.as_ref()) {
                    let word_a = self.inner.voc_a.get_value(word_id_a).unwrap();
                    let word_b = self.inner.voc_b.get_value(word_id_b).unwrap();
                    let DirectionTuple{ a: word_a, b: word_b, direction: _ } = match direction {
                        DirectionKind::AToB => {
                            new.insert_hash_ref::<AToB>(word_a.clone(), word_b.clone())
                        }
                        DirectionKind::BToA => {
                            new.insert_hash_ref::<BToA>(word_a.clone(), word_b.clone())
                        },
                        DirectionKind::Invariant => {
                            new.insert_hash_ref::<Invariant>(word_a.clone(), word_b.clone())
                        }
                    };
                    if let Some(meta_a) = meta_a {
                        new.insert_meta_for_create_subset::<A>(word_a, meta_a);
                    }
                    if let Some(meta_b) = meta_b {
                        new.insert_meta_for_create_subset::<B>(word_b, meta_b);
                    }
                }
            }
        }
        new
    }
}

impl<T, V> FromVoc<T, V> for DictionaryWithMeta<T, V> where V: BasicVocabulary<T> + Default, T: Hash + Eq  {
    fn from_voc(voc_a: V, voc_b: V) -> Self {
        Self::new(
            Dictionary::from_voc(voc_a, voc_b),
            Default::default()
        )
    }

    fn from_voc_lang<L: Language>(voc: V, other_lang: Option<LanguageHint>) -> Self {
        Self::new(
            Dictionary::from_voc_lang::<L>(voc, other_lang),
            Default::default()
        )
    }
}


impl<T, V> Clone for DictionaryWithMeta<T, V> where V: Clone {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone(), self.metadata.clone())
    }
}
impl<T, V> BasicDictionary for DictionaryWithMeta<T, V> {
    delegate::delegate! {
        to self.inner {
            fn map_a_to_b(&self) -> &Vec<Vec<usize>>;
            fn map_b_to_a(&self) -> &Vec<Vec<usize>>;
        }
    }

    fn translate_id_to_ids<D: Translation>(&self, word_id: usize) -> Option<&Vec<usize>> {
        self.inner.translate_id_to_ids::<D>(word_id)
    }

    fn switch_languages(self) -> Self where Self: Sized {
        Self {
            inner: self.inner.switch_languages(),
            metadata: self.metadata.switch_languages()
        }
    }
}
impl<T, V> BasicDictionaryWithMeta for DictionaryWithMeta<T, V> where V: BasicVocabulary<T> {
    fn metadata(&self) -> &MetadataContainer {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut MetadataContainer {
        &mut self.metadata
    }
}
impl<T, V> BasicDictionaryWithVocabulary<V> for DictionaryWithMeta<T, V> {
    delegate::delegate! {
        to self.inner {
            fn voc_a(&self) -> &V;
            fn voc_b(&self) -> &V;
        }
    }
}
impl<T, V> DictionaryWithMeta<T, V> where T: Eq + Hash, V: MappableVocabulary<T> {
    pub fn map<Q: Eq + Hash, Voc, F>(self, f: F) -> DictionaryWithMeta<Q, Voc> where F: for<'a> Fn(&'a T)-> Q, Voc: BasicVocabulary<Q> {
        DictionaryWithMeta::<Q, Voc>::new(
            self.inner.map(&f),
            self.metadata.clone()
        )
    }
}
impl<T, V> DictionaryWithVocabulary<T, V> for  DictionaryWithMeta<T, V> where V: BasicVocabulary<T> {

    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        self.inner.can_translate_id::<D>(id)
    }

    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        self.inner.id_to_word::<D>(id)
    }

    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        self.inner.ids_to_id_entry::<D>(ids)
    }

    fn ids_to_values<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<&'a HashRef<T>> where V: 'a {
        self.inner.ids_to_values::<D>(ids)
    }

    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.inner.translate_value_to_ids::<D, _>(word)
    }

    fn word_to_id<D: Translation, Q: ?Sized>(&self, id: &Q) -> Option<usize>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.inner.word_to_id::<D, _>(id)
    }
}
impl<T, V> DictionaryMut<T, V> for  DictionaryWithMeta<T, V> where T: Eq + Hash, V: VocabularyMut<T> {
    fn set_language<L: Language>(&mut self, value: Option<LanguageHint>) -> Option<LanguageHint> {
        self.inner.set_language::<L>(value)
    }

    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        self.inner.insert_hash_ref::<D>(word_a, word_b)
    }
}
impl<T, V> DictionaryFilterable<T, V>  for DictionaryWithMeta<T, V> where T: Eq + Hash, V: VocabularyMut<T> + Default{
    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized {
        let mut new_dict = DictionaryWithMeta::new(
            Default::default(),
            self.metadata.copy_keep_vocebulary()
        );

        for DirectionTuple{a, b, direction} in self.iter() {
            match direction {
                DirectionKind::AToB => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionKind::BToA => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
                DirectionKind::Invariant => {
                    let filter_a = filter_a(a);
                    let filter_b = filter_b(b);
                    if filter_a && filter_b {
                        new_dict.insert_hash_ref::<Invariant>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    } else if filter_a {
                        new_dict.insert_hash_ref::<AToB>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    } else if filter_b {
                        new_dict.insert_hash_ref::<BToA>(
                            self.id_to_word::<A>(a).unwrap().clone(),
                            self.id_to_word::<B>(b).unwrap().clone()
                        );
                    }
                }
            }
        }

        new_dict
    }

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<T>) -> bool, Fb: Fn(&'a HashRef<T>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, T: 'a {
        let mut new_dict = DictionaryWithMeta::new(
            Default::default(),
            self.metadata.copy_keep_vocebulary()
        );
        for DirectionTuple{a, b, direction} in self.iter() {
            let a = self.id_to_word::<A>(a).unwrap();
            let b = self.id_to_word::<B>(b).unwrap();
            match direction {
                DirectionKind::AToB => {
                    if filter_a(a) {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionKind::BToA => {
                    if filter_b(b) {
                        new_dict.insert_hash_ref::<BToA>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
                DirectionKind::Invariant => {
                    let filter_a = filter_a(a);
                    let filter_b = filter_b(a);
                    if filter_a && filter_b {
                        new_dict.insert_hash_ref::<Invariant>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_a {
                        new_dict.insert_hash_ref::<AToB>(
                            a.clone(),
                            b.clone()
                        );
                    } else if filter_b {
                        new_dict.insert_hash_ref::<BToA>(
                            a.clone(),
                            b.clone()
                        );
                    }
                }
            }
        }

        new_dict
    }
}

impl<T: Display, V: BasicVocabulary<T>> Display for DictionaryWithMeta<T, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)?;
        write!(f, "\n------\n")?;
        write!(f, "{}", self.metadata_with_dict())?;
        Ok(())
    }
}

impl<T, V> From<Dictionary<T, V>> for DictionaryWithMeta<T, V> {
    fn from(value: Dictionary<T, V>) -> Self {
        Self::new(
            value,
            MetadataContainer::new()
        )
    }
}

impl<T, V> IntoIterator for DictionaryWithMeta<T, V> where V: BasicVocabulary<T>, T: Hash + Eq {
    type Item = DirectionTuple<(usize, HashRef<T>, Option<SolvedMetadata>), (usize, HashRef<T>, Option<SolvedMetadata>)>;
    type IntoIter = DictionaryWithMetaIterator<DictionaryWithMeta<T, V>, T, V>;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryWithMetaIterator::new(self)
    }
}
