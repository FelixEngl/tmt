use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, Dictionary, DictionaryFilterable, DictionaryMut, DictionaryWithVocabulary, FromVoc, MergingDictionary};
use crate::topicmodel::dictionary::direction::{AToB, BToA, Direction, DirectionKind, DirectionTuple, Invariant, Language, LanguageKind, Translation, A, B};
use crate::topicmodel::dictionary::iterators::DictionaryWithMetaIterator;
use crate::topicmodel::dictionary::metadata::{MetadataManager, MetadataContainerWithDict, MetadataContainerWithDictMut, MetadataMutReference, MetadataReference};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut, BasicVocabulary, MappableVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};
use crate::topicmodel::dictionary::metadata::classic::{
    ClassicMetadataManager,
};
use crate::topicmodel::dictionary::metadata::domain_matrix::DomainModel;
use crate::topicmodel::dictionary::metadata::ex::{MetadataEx, MetadataManagerEx, MetadataWithOrigin};
use crate::topicmodel::dictionary::metadata::update::WordIdUpdate;
use crate::topicmodel::reference::HashRef;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DictionaryWithMeta<T, V, C> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    pub(crate) inner: Dictionary<T, V>,
    #[serde(bound(serialize = "C: Serialize", deserialize = "C: Deserialize<'de>"))]
    pub(crate) metadata: C
}


impl<T, V, C> DictionaryWithMeta<T, V, C> {
    fn new(inner: Dictionary<T, V>, metadata: C) -> Self {
        Self { inner, metadata }
    }
}

impl<T, V> DictionaryWithMeta<T, V, ClassicMetadataManager> {

    pub fn subjects(&self) -> Vec<&str> {
        self.metadata.subject_interner.iter().map(|value| value.1).collect_vec()
    }

    pub fn unstemmed(&self) -> &Vocabulary<String> {
        &self.metadata.unstemmed_voc
    }
}



pub enum CreateTopicMatrixMode<T=()> {
    All,
    OnlyDefault,
    OnlyTargets(Vec<T>),
    DefaultAndTargets(Vec<T>)
}

impl<T> CreateTopicMatrixMode<T> {
    pub fn collects_default(&self) -> bool {
        matches!(self, Self::All | Self::OnlyDefault | Self::DefaultAndTargets(_))
    }
}

impl<T> CreateTopicMatrixMode<T> where T: PartialEq {
    pub fn contains(&self, value: &T) -> bool {
        match self {
            CreateTopicMatrixMode::All => true,
            CreateTopicMatrixMode::OnlyDefault => false,
            CreateTopicMatrixMode::OnlyTargets(v) => v.contains(value),
            CreateTopicMatrixMode::DefaultAndTargets(v) => v.contains(value),
        }
    }
}

impl<T, V> DictionaryWithMeta<T, V, MetadataManagerEx> where V: BasicVocabulary<T> {
    pub fn create_topic_matrix<'a, L: Language, S: AsRef<str>>(&'a self, mode: &CreateTopicMatrixMode<S>) -> DomainModel {

        let mode = match mode {
            CreateTopicMatrixMode::All => CreateTopicMatrixMode::All,
            CreateTopicMatrixMode::OnlyDefault => CreateTopicMatrixMode::OnlyDefault,
            CreateTopicMatrixMode::OnlyTargets(value) => {
                CreateTopicMatrixMode::OnlyTargets(
                    value.iter().filter_map(|value| {
                        self.metadata.dictionary_interner.get(value.as_ref())
                    }).collect_vec()
                )
            }
            CreateTopicMatrixMode::DefaultAndTargets(value) => {
                CreateTopicMatrixMode::OnlyTargets(
                    value.iter().filter_map(|value| {
                        self.metadata.dictionary_interner.get(value.as_ref())
                    }).collect_vec()
                )
            }
        };

        let mut matrix: DomainModel;
        let iter: std::slice::Iter<'a, MetadataEx>;

        if L::LANG.is_a() {
            matrix = DomainModel::with_capacity(self.voc_a().len());
            iter = self.metadata.meta_a().iter();
        } else {
            matrix = DomainModel::with_capacity(self.voc_b().len());
            iter = self.metadata.meta_b().iter();
        }

        for x in iter {
            let targ = matrix.create_next();
            for value in x.iter() {
                match value {
                    MetadataWithOrigin::General(value) if mode.collects_default() => {
                        targ.fill_by(value);
                    }
                    MetadataWithOrigin::Associated(origin, value) if mode.contains(&origin) => {
                        targ.fill_by(value);
                    }
                    _ => {}
                }
            }
        }

        matrix
    }
}


impl<T, V, M> DictionaryWithMeta<T, V, M> where V: From<Option<LanguageHint>>, M: Default  {
    pub fn new_with(language_a: Option<impl Into<LanguageHint>>, language_b: Option<impl Into<LanguageHint>>) -> Self {
        Self::new(
            Dictionary::new_with(language_a, language_b),
            M::default()
        )
    }
}

impl<T, V, M> DictionaryWithMeta<T, V, M> where V: BasicVocabulary<T> + AnonymousVocabulary + AnonymousVocabularyMut, M: MetadataManager
{
    pub fn metadata_with_dict(&self) -> MetadataContainerWithDict<Self, T, V, M> where Self: Sized {
        MetadataContainerWithDict::wrap(self)
    }

    pub fn metadata_with_dict_mut(&mut self) -> MetadataContainerWithDictMut<Self, T, V, M> where Self: Sized {
        MetadataContainerWithDictMut::wrap(self)
    }

    pub fn known_dictionaries(&self) -> Vec<&str> {
        self.metadata.dictionaries()
    }
}
unsafe impl<T, V, M> Send for DictionaryWithMeta<T, V, M>{}
unsafe impl<T, V, M> Sync for DictionaryWithMeta<T, V, M>{}



impl<T, V, M> DictionaryWithMeta<T, V, M>
where
    V: VocabularyMut<T> + From<Option<LanguageHint>> + AnonymousVocabulary + AnonymousVocabularyMut,
    T: Hash + Eq,
    M: MetadataManager
{
    fn insert_meta_for_create_subset<'a, L: Language>(
        &mut self,
        word_id: usize,
        metadata_ref: M::Reference<'a>
    ) {

        let mut meta = self.metadata.get_or_create_meta::<L>(
            match L::LANG {
                LanguageKind::A => {
                    &mut self.inner.voc_a
                }
                LanguageKind::B => {
                    &mut self.inner.voc_b
                }
            },
            word_id
        );
        meta.update_with_reference::<L>(metadata_ref)
    }

    pub fn create_subset_with_filters<F1, F2>(&self, filter_a: F1, filter_b: F2) -> DictionaryWithMeta<T, V, M>
    where
        F1: for<'a> Fn(&DictionaryWithMeta<T, V, M>, usize, Option<&M::Reference<'a>>) -> bool,
        F2: for<'a> Fn(&DictionaryWithMeta<T, V, M>, usize, Option<&M::Reference<'a>>) -> bool
    {

        let mut new = Self {
            inner: Dictionary::new_with(
                self.inner.voc_a.language().cloned(),
                self.inner.voc_b.language().cloned()
            ),
            metadata: self.metadata.copy_keep_vocabulary()
        };

        let mut update = WordIdUpdate::new(
            self.voc_a().len(),
            self.voc_b().len()
        );

        for DirectionTuple{
            a: (word_id_a, meta_a),
            b: (word_id_b, meta_b),
            direction
        } in self.iter_with_meta() {
            if filter_a(self, word_id_a, meta_a.as_ref()) {
                if filter_b(self, word_id_b, meta_b.as_ref()) {
                    let word_a = self.inner.voc_a.get_value(word_id_a).unwrap();
                    let word_b = self.inner.voc_b.get_value(word_id_b).unwrap();
                    let DirectionTuple{a: word_a, b: word_b, direction: _} = match direction {
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
                        if let Some(a) = meta_a.collect_all_associated_word_ids() {
                            for value in a.iter() {
                                if let Some(value) = self.voc_a().get_value(value) {
                                    update.add_id::<A>(
                                        word_id_a,
                                        new.inner.voc_a.add_hash_ref(value.clone())
                                    )
                                }
                            }
                        }
                        new.insert_meta_for_create_subset::<A>(word_a, meta_a);
                    }
                    if let Some(meta_b) = meta_b {
                        if let Some(b) = meta_b.collect_all_associated_word_ids() {
                            for value in b.iter() {
                                if let Some(value) = self.voc_b().get_value(value) {
                                    update.add_id::<B>(
                                        word_id_b,
                                        new.inner.voc_b.add_hash_ref(value.clone())
                                    )
                                }
                            }
                        }

                        new.insert_meta_for_create_subset::<B>(word_b, meta_b);
                    }
                    update.add_id::<A>(word_id_a, word_a);
                    update.add_id::<B>(word_id_b, word_b);
                }
            }
        }
        new.metadata.update_ids(&update);
        new.metadata.optimize();
        new
    }
}

impl<T, V, M> FromVoc<T, V> for DictionaryWithMeta<T, V, M>
where
    V: BasicVocabulary<T> + Default,
    T: Hash + Eq,
    M: Default + MetadataManager
{
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


impl<T, V, M> Clone for DictionaryWithMeta<T, V, M> where V: Clone, M: Clone {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone(), self.metadata.clone())
    }
}
impl<T, V, M> BasicDictionary for DictionaryWithMeta<T, V, M>
where
    M: MetadataManager
{
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

impl<T, V, M> BasicDictionaryWithMeta<M, V> for DictionaryWithMeta<T, V, M>
where
    M: MetadataManager,
    V: AnonymousVocabulary + BasicVocabulary<T>,
{
    fn metadata(&self) -> &M {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut M {
        &mut self.metadata
    }
}


impl<T, V, M> BasicDictionaryWithVocabulary<V> for DictionaryWithMeta<T, V, M>
where
    M: MetadataManager
{
    delegate::delegate! {
        to self.inner {
            fn voc_a(&self) -> &V;
            fn voc_b(&self) -> &V;
        }
    }
}



impl<T, V, M> DictionaryWithMeta<T, V, M> where T: Eq + Hash, V: MappableVocabulary<T>, M: Clone {
    pub fn map<Q: Eq + Hash, Voc, F>(self, f: F) -> DictionaryWithMeta<Q, Voc, M> where F: for<'a> Fn(&'a T)-> Q, Voc: BasicVocabulary<Q> {
        DictionaryWithMeta::<Q, Voc, M>::new(
            self.inner.map(&f),
            self.metadata.clone()
        )
    }
}
impl<T, V, M> DictionaryWithVocabulary<T, V> for  DictionaryWithMeta<T, V, M>
where
    V: BasicVocabulary<T>,
    M: MetadataManager
{

    #[inline(always)]
    fn can_translate_id<D: Translation>(&self, id: usize) -> bool {
        self.inner.can_translate_id::<D>(id)
    }

    #[inline(always)]
    fn id_to_word<'a, D: Translation>(&'a self, id: usize) -> Option<&'a HashRef<T>> where V: 'a {
        self.inner.id_to_word::<D>(id)
    }

    #[inline(always)]
    fn ids_to_id_entry<'a, D: Translation>(&'a self, ids: &Vec<usize>) -> Vec<(usize, &'a HashRef<T>)> where V: 'a {
        self.inner.ids_to_id_entry::<D>(ids)
    }

    #[inline(always)]
    fn ids_to_values<'a, D: Translation, I: IntoIterator<Item=usize>>(&'a self, ids: I) -> Vec<&'a HashRef<T>> where V: 'a {
        self.inner.ids_to_values::<D, _>(ids)
    }

    #[inline(always)]
    fn translate_value_to_ids<D: Translation, Q: ?Sized>(&self, word: &Q) -> Option<&Vec<usize>>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.inner.translate_value_to_ids::<D, _>(word)
    }

    #[inline(always)]
    fn word_to_id<D: Translation, Q: ?Sized>(&self, id: &Q) -> Option<usize>
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq,
        V: SearchableVocabulary<T>
    {
        self.inner.word_to_id::<D, _>(id)
    }
}



impl<T, V, M> DictionaryMut<T, V> for  DictionaryWithMeta<T, V, M>
where
    T: Eq + Hash,
    V: VocabularyMut<T>,
    M: MetadataManager
{
    fn set_language<L: Language>(&mut self, value: Option<LanguageHint>) -> Option<LanguageHint> {
        self.inner.set_language::<L>(value)
    }

    fn insert_single_ref<L: Language>(&mut self, word: HashRef<T>) -> usize {
        self.inner.insert_single_ref::<L>(word)
    }

    unsafe fn reserve_for_single_value<L: Language>(&mut self, word_id: usize) {
        self.inner.reserve_for_single_value::<L>(word_id)
    }

    unsafe fn insert_raw_values<D: Direction>(&mut self, word_id_a: usize, word_id_b: usize) {
        self.inner.insert_raw_values::<D>(word_id_a, word_id_b)
    }

    fn insert_hash_ref<D: Direction>(&mut self, word_a: HashRef<T>, word_b: HashRef<T>) -> DirectionTuple<usize, usize> {
        self.inner.insert_hash_ref::<D>(word_a, word_b)
    }
}
impl<T, V, M> DictionaryFilterable<T, V>  for DictionaryWithMeta<T, V, M>
where
    V: VocabularyMut<T> + From<Option<LanguageHint>> + AnonymousVocabulary + AnonymousVocabularyMut,
    T: Hash + Eq,
    M: MetadataManager
{
    fn filter_and_process<'a, Fa, Fb>(&'a self, f_a: Fa, f_b: Fb) -> Self
    where
        Self: Sized,
        T: 'a,
        Fa: Fn(&'a HashRef<T>) -> Option<HashRef<T>>,
        Fb: Fn(&'a HashRef<T>) -> Option<HashRef<T>>
    {
        let mut new = Self {
            inner: Dictionary::new_with(
                self.inner.voc_a.language().cloned(),
                self.inner.voc_b.language().cloned()
            ),
            metadata: self.metadata.copy_keep_vocabulary()
        };

        let mut update = WordIdUpdate::new(
            self.voc_a().len(),
            self.voc_b().len()
        );

        for DirectionTuple{
            a: (word_id_a, meta_a),
            b: (word_id_b, meta_b),
            direction
        } in self.iter_with_meta() {
            if let Some(a) = f_a(self.voc_a().get_value(word_id_a).unwrap()) {
                if let Some(b) = f_b(self.voc_b().get_value(word_id_b).unwrap()) {
                    let DirectionTuple{a: word_a, b: word_b, direction: _} = match direction {
                        DirectionKind::AToB => {
                            new.insert_hash_ref::<AToB>(a, b)
                        }
                        DirectionKind::BToA => {
                            new.insert_hash_ref::<BToA>(a, b)
                        },
                        DirectionKind::Invariant => {
                            new.insert_hash_ref::<Invariant>(a, b)
                        }
                    };
                    if let Some(meta_a) = meta_a {
                        if let Some(a) = meta_a.collect_all_associated_word_ids() {
                            for value in a.iter() {
                                if let Some(value) = self.voc_a().get_value(value) {
                                    if let Some(value) = f_a(value) {
                                        update.add_id::<A>(
                                            word_id_a,
                                            new.inner.voc_a.add_hash_ref(value.clone())
                                        )
                                    }
                                }
                            }
                        }
                        new.insert_meta_for_create_subset::<A>(word_a, meta_a);
                    }
                    if let Some(meta_b) = meta_b {
                        if let Some(b) = meta_b.collect_all_associated_word_ids() {
                            for value in b.iter() {
                                if let Some(value) = self.voc_b().get_value(value) {
                                    if let Some(value) = f_b(value) {
                                        update.add_id::<B>(
                                            word_id_b,
                                            new.inner.voc_b.add_hash_ref(value)
                                        )
                                    }
                                }
                            }
                        }
                        new.insert_meta_for_create_subset::<B>(word_b, meta_b);
                    }
                    update.add_id::<A>(word_id_a, word_a);
                    update.add_id::<B>(word_id_b, word_b);
                }
            }
        }

        new.metadata.update_ids(&update);
        new.metadata.optimize();
        new
    }

    fn filter_by_ids<Fa: Fn(usize) -> bool, Fb: Fn(usize) -> bool>(&self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized {

        self.create_subset_with_filters(
            |_, a, _| filter_a(a),
            |_, b, _| filter_b(b)
        )

        // let mut new_dict = DictionaryWithMeta::new(
        //     Default::default(),
        //     self.metadata.copy_keep_vocabulary()
        // );
        //
        // for DirectionTuple{
        //     a: (a, meta_a),
        //     b: (b, meta_b),
        //     direction
        // } in self.iter_with_meta() {
        //     match direction {
        //         DirectionKind::AToB => {
        //             if filter_a(a) {
        //                 new_dict.insert_hash_ref::<AToB>(
        //                     self.id_to_word::<A>(a).unwrap().clone(),
        //                     self.id_to_word::<B>(b).unwrap().clone()
        //                 );
        //             }
        //         }
        //         DirectionKind::BToA => {
        //             if filter_b(b) {
        //                 new_dict.insert_hash_ref::<BToA>(
        //                     self.id_to_word::<A>(a).unwrap().clone(),
        //                     self.id_to_word::<B>(b).unwrap().clone()
        //                 );
        //             }
        //         }
        //         DirectionKind::Invariant => {
        //             let filter_a = filter_a(a);
        //             let filter_b = filter_b(b);
        //             if filter_a && filter_b {
        //                 new_dict.insert_hash_ref::<Invariant>(
        //                     self.id_to_word::<A>(a).unwrap().clone(),
        //                     self.id_to_word::<B>(b).unwrap().clone()
        //                 );
        //             } else if filter_a {
        //                 new_dict.insert_hash_ref::<AToB>(
        //                     self.id_to_word::<A>(a).unwrap().clone(),
        //                     self.id_to_word::<B>(b).unwrap().clone()
        //                 );
        //             } else if filter_b {
        //                 new_dict.insert_hash_ref::<BToA>(
        //                     self.id_to_word::<A>(a).unwrap().clone(),
        //                     self.id_to_word::<B>(b).unwrap().clone()
        //                 );
        //             }
        //         }
        //     }
        // }
        //
        // new_dict
    }

    fn filter_by_values<'a, Fa: Fn(&'a HashRef<T>) -> bool, Fb: Fn(&'a HashRef<T>) -> bool>(&'a self, filter_a: Fa, filter_b: Fb) -> Self where Self: Sized, T: 'a {
        let voc_a = self.voc_a();
        let voc_b = self.voc_b();
        self.create_subset_with_filters(
            |_, a, _| filter_a(voc_a.get_value(a).unwrap()),
            |_, b, _| filter_b(voc_b.get_value(b).unwrap())
        )

        // let mut new_dict = DictionaryWithMeta::new(
        //     Default::default(),
        //     self.metadata.copy_keep_vocabulary()
        // );
        // for DirectionTuple{a, b, direction} in self.iter() {
        //     let a = self.id_to_word::<A>(a).unwrap();
        //     let b = self.id_to_word::<B>(b).unwrap();
        //     match direction {
        //         DirectionKind::AToB => {
        //             if filter_a(a) {
        //                 new_dict.insert_hash_ref::<AToB>(
        //                     a.clone(),
        //                     b.clone()
        //                 );
        //             }
        //         }
        //         DirectionKind::BToA => {
        //             if filter_b(b) {
        //                 new_dict.insert_hash_ref::<BToA>(
        //                     a.clone(),
        //                     b.clone()
        //                 );
        //             }
        //         }
        //         DirectionKind::Invariant => {
        //             let filter_a = filter_a(a);
        //             let filter_b = filter_b(a);
        //             if filter_a && filter_b {
        //                 new_dict.insert_hash_ref::<Invariant>(
        //                     a.clone(),
        //                     b.clone()
        //                 );
        //             } else if filter_a {
        //                 new_dict.insert_hash_ref::<AToB>(
        //                     a.clone(),
        //                     b.clone()
        //                 );
        //             } else if filter_b {
        //                 new_dict.insert_hash_ref::<BToA>(
        //                     a.clone(),
        //                     b.clone()
        //                 );
        //             }
        //         }
        //     }
        // }
        //
        // new_dict
    }
}

impl<T, V, M> Display for DictionaryWithMeta<T, V, M>
where
    T: Display,
    M: MetadataManager,
    for<'a> <M as MetadataManager>::Reference<'a>: Display,
    V: BasicVocabulary<T> + AnonymousVocabulary + AnonymousVocabularyMut
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)?;
        write!(f, "\n------\n")?;
        write!(f, "{}", self.metadata_with_dict())?;
        Ok(())
    }
}

impl<T, V, M> From<Dictionary<T, V>> for DictionaryWithMeta<T, V, M>
where M: Default
{
    fn from(value: Dictionary<T, V>) -> Self {
        Self::new(
            value,
            M::default()
        )
    }
}

impl<T, V, M> IntoIterator for DictionaryWithMeta<T, V, M>
where
    T: Hash + Eq,
    V: BasicVocabulary<T> + AnonymousVocabulary,
    M: MetadataManager
{
    type Item = DirectionTuple<
        (usize, HashRef<T>, Option<M::ResolvedMetadata>),
        (usize, HashRef<T>, Option<M::ResolvedMetadata>)
    >;
    type IntoIter = DictionaryWithMetaIterator<DictionaryWithMeta<T, V, M>, T, V, M>;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryWithMetaIterator::new(self)
    }
}


impl<T, V, M> MergingDictionary<T, V> for DictionaryWithMeta<T, V, M>
where
    T: Eq + Hash,
    V: BasicVocabulary<T> + From<Option<LanguageHint>> + AnonymousVocabulary + AnonymousVocabularyMut + VocabularyMut<T>,
    M: MetadataManager
{
    fn merge(mut self, other: impl Into<Self>) -> Self
    where
        Self: Sized
    {
        let other = other.into();
        let mut update = WordIdUpdate::new(
            other.voc_a().len(),
            other.voc_b().len()
        );

        for DirectionTuple {
            a: (word_id_a, meta_a),
            b: (word_id_b, meta_b),
            direction
        } in other.iter_with_meta() {
            let word_a = other.voc_a().get_value(word_id_a).unwrap();
            let word_b = other.voc_b().get_value(word_id_b).unwrap();
            let DirectionTuple{a: word_a, b: word_b, direction: _} = match direction {
                DirectionKind::AToB => {
                    self.insert_hash_ref::<AToB>(word_a.clone(), word_b.clone())
                }
                DirectionKind::BToA => {
                    self.insert_hash_ref::<BToA>(word_a.clone(), word_b.clone())
                },
                DirectionKind::Invariant => {
                    self.insert_hash_ref::<Invariant>(word_a.clone(), word_b.clone())
                }
            };
            if let Some(meta_a) = meta_a {
                if let Some(a) = meta_a.collect_all_associated_word_ids() {
                    for value in a.iter() {
                        if let Some(value) = other.voc_a().get_value(value) {
                            update.add_id::<A>(
                                word_id_a,
                                self.inner.voc_a.add_hash_ref(value.clone())
                            )
                        }
                    }
                }
                self.insert_meta_for_create_subset::<A>(word_a, meta_a);
            }
            if let Some(meta_b) = meta_b {
                if let Some(b) = meta_b.collect_all_associated_word_ids() {
                    for value in b.iter() {
                        if let Some(value) = other.voc_b().get_value(value) {
                            update.add_id::<B>(
                                word_id_b,
                                self.inner.voc_b.add_hash_ref(value.clone())
                            )
                        }
                    }
                }
                self.insert_meta_for_create_subset::<B>(word_b, meta_b);
            }
        }
        self.metadata.update_ids(&update);
        // No optimize needed because we only grow.
        // self.metadata.optimize();
        self
    }
}
