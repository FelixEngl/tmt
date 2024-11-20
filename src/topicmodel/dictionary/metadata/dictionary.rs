use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::{BasicDictionary, BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, BasicDictionaryWithVocabulary, Dictionary, DictionaryFilterable, DictionaryMut, DictionaryWithVocabulary, FromVoc, MergingDictionary};
use crate::topicmodel::dictionary::direction::{DirectionTuple, Language, LanguageKind, A, B};
use crate::topicmodel::dictionary::iterators::DictionaryWithMetaIterator;
use crate::topicmodel::dictionary::metadata::{MetadataManager, MetadataContainerWithDict, MetadataContainerWithDictMut, MetadataMutReference, MetadataReference, MetadataManagerGen};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut, BasicVocabulary, MappableVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};
use crate::topicmodel::dictionary::metadata::classic::{
    ClassicMetadataManager,
};
use crate::topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTopicModel;
use crate::topicmodel::dictionary::metadata::ex::{MetadataEx, MetadataManagerEx, MetadataWithOrigin};
use crate::topicmodel::dictionary::metadata::update::WordIdUpdate;
use crate::topicmodel::reference::HashRef;

pub type StringDictWithMeta<V> = DictionaryWithMeta<String, V, MetadataManagerEx>;
pub type StringDictWithMetaDefault = DictionaryWithMeta<String, Vocabulary<String>, MetadataManagerEx>;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DictionaryWithMeta<T, V, C> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    pub(crate) inner: Dictionary<T, V>,
    #[serde(bound(serialize = "C: Serialize", deserialize = "C: Deserialize<'de>"))]
    pub(crate) metadata: C
}

unsafe impl<T, V, M> Send for DictionaryWithMeta<T, V, M>{}
unsafe impl<T, V, M> Sync for DictionaryWithMeta<T, V, M>{}


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
    pub fn create_topic_matrix<'a, L: Language, S: AsRef<str>>(&'a self, mode: &CreateTopicMatrixMode<S>) -> DictMetaTopicModel {

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

        let mut matrix: DictMetaTopicModel;
        let iter: std::slice::Iter<'a, MetadataEx>;

        if L::LANG.is_a() {
            matrix = DictMetaTopicModel::with_capacity(self.voc_a().len());
            iter = self.metadata.meta_a().iter();
        } else {
            matrix = DictMetaTopicModel::with_capacity(self.voc_b().len());
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
        meta.update_with_reference(metadata_ref)
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
                    let DirectionTuple{
                        a: word_a,
                        b: word_b,
                        direction: _
                    } = new.insert_hash_ref_dir(direction, word_a.clone(), word_b.clone());
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



    fn from_voc_lang_a(voc: V, other_lang: Option<LanguageHint>) -> Self {
        Self::new(
            Dictionary::from_voc_lang_a(voc, other_lang),
            Default::default()
        )
    }

    fn from_voc_lang_b(other_lang: Option<LanguageHint>, voc: V) -> Self {
        Self::new(
            Dictionary::from_voc_lang_b(other_lang, voc),
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

    fn switch_languages(self) -> Self where Self: Sized {
        Self {
            inner: self.inner.switch_languages(),
            metadata: self.metadata.switch_languages()
        }
    }
}

#[allow(clippy::needless_lifetimes)]
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

    fn get_meta_for_a<'a>(&'a self, word_id: usize) -> Option<<M as MetadataManager>::Reference<'a>> {
        self.metadata.get_meta_ref::<A>(
            &self.inner.voc_a,
            word_id
        )
    }

    fn get_meta_for_b<'a>(&'a self, word_id: usize) -> Option<<M as MetadataManager>::Reference<'a>> {
        self.metadata.get_meta_ref::<B>(
            &self.inner.voc_b,
            word_id
        )
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
            fn voc_a_mut(&mut self) -> &mut V;
            fn voc_b_mut(&mut self) -> &mut V;
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
{}



impl<T, V, M> DictionaryMut<T, V> for  DictionaryWithMeta<T, V, M>
where
    T: Eq + Hash,
    V: VocabularyMut<T>,
    M: MetadataManager
{
    delegate::delegate! {
        to self.inner {
            unsafe fn reserve_for_single_value_a(&mut self, word_id: usize);
            unsafe fn reserve_for_single_value_b(&mut self, word_id: usize);
            unsafe fn insert_raw_values_a_to_b(&mut self, word_id_a: usize, word_id_b: usize);
            unsafe fn insert_raw_values_b_to_a(&mut self, word_id_a: usize, word_id_b: usize);

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
    }

}
impl<T, V, M> DictionaryFilterable<T, V>  for DictionaryWithMeta<T, V, M>
where
    V: VocabularyMut<T> + From<Option<LanguageHint>> + AnonymousVocabulary + AnonymousVocabularyMut,
    T: Hash + Eq,
    M: MetadataManager
{
    fn filter_and_process<'a, Fa, Fb, E>(&'a self, f_a: Fa, f_b: Fb) -> Result<Self, E>
    where
        Self: Sized,
        T: 'a,
        Fa: Fn(&'a HashRef<T>) -> Result<Option<HashRef<T>>, E>,
        Fb: Fn(&'a HashRef<T>) -> Result<Option<HashRef<T>>, E>
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
            if let Some(a) = f_a(self.voc_a().get_value(word_id_a).unwrap())? {
                if let Some(b) = f_b(self.voc_b().get_value(word_id_b).unwrap())? {
                    let DirectionTuple{
                        a: word_a,
                        b: word_b,
                        direction: _
                    } = new.insert_hash_ref_dir(direction, a, b);
                    if let Some(meta_a) = meta_a {
                        if let Some(a) = meta_a.collect_all_associated_word_ids() {
                            for value in a.iter() {
                                if let Some(value) = self.voc_a().get_value(value) {
                                    if let Some(value) = f_a(value)? {
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
                                    if let Some(value) = f_b(value)? {
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
        Ok(new)
    }

    fn filter_by_ids<Fa, Fb>(&self, filter_a: Fa, filter_b: Fb) -> Self
    where
        Self: Sized,
        Fa: Fn(usize) -> bool,
        Fb: Fn(usize) -> bool
    {
        self.create_subset_with_filters(
            |_, a, _| filter_a(a),
            |_, b, _| filter_b(b)
        )
    }

    fn filter_by_values<'a, Fa, Fb>(&'a self, filter_a: Fa, filter_b: Fb) -> Self
    where
        Self: Sized, T: 'a,
        Fa: Fn(&'a HashRef<T>) -> bool,
        Fb: Fn(&'a HashRef<T>) -> bool
    {
        let voc_a = self.voc_a();
        let voc_b = self.voc_b();
        self.create_subset_with_filters(
            |_, a, _| filter_a(voc_a.get_value(a).unwrap()),
            |_, b, _| filter_b(voc_b.get_value(b).unwrap())
        )
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
            let DirectionTuple{
                a: word_a,
                b: word_b,
                direction: _
            } = self.insert_hash_ref_dir(direction, word_a.clone(), word_b.clone());
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


impl<T, V, M> BasicDictionaryWithMutMeta<M, V> for DictionaryWithMeta<T, V, M>
where
    M: MetadataManager,
    V: AnonymousVocabulary + AnonymousVocabularyMut + BasicVocabulary<T>,
{

    fn get_mut_meta_a<'a>(&'a mut self, word_id: usize) -> Option<<M as MetadataManager>::MutReference<'a>> {
        self.metadata.get_meta_mut::<A>(
            &mut self.inner.voc_a,
            word_id
        )
    }

    fn get_mut_meta_b<'a>(&'a mut self, word_id: usize) -> Option<<M as MetadataManager>::MutReference<'a>> {
        self.metadata.get_meta_mut::<B>(
            &mut self.inner.voc_b,
            word_id
        )
    }

    fn get_or_create_meta_a<'a>(&'a mut self, word_id: usize) -> <M as MetadataManager>::MutReference<'a> {
        self.metadata.get_or_create_meta::<A>(
            &mut self.inner.voc_a,
            word_id
        )
    }

    fn get_or_create_meta_b<'a>(&'a mut self, word_id: usize) -> <M as MetadataManager>::MutReference<'a> {
        self.metadata.get_or_create_meta::<B>(
            &mut self.inner.voc_b,
            word_id
        )
    }
}