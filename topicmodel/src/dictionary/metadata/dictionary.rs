use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use arcstr::ArcStr;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::dictionary::{BasicDictionary, BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, BasicDictionaryWithVocabulary, Dictionary, DictionaryFilterable, DictionaryMut, DictionaryWithSearch, DictionaryWithVocabulary, FromVoc, MergingDictionary};
use crate::dictionary::direction::{DirectedElement, Language, LanguageMarker, A, B};
use crate::dictionary::iterators::DictionaryWithMetaIterator;
use crate::dictionary::metadata::{MetadataManager, MetadataContainerWithDict, MetadataContainerWithDictMut, MetadataMutReference, MetadataReference, MetadataManagerGen};
use crate::language_hint::LanguageHint;
use crate::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut, BasicVocabulary, MappableVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};
use crate::dictionary::metadata::classic::{
    ClassicMetadataManager,
};
use crate::dictionary::metadata::dict_meta_topic_matrix::DictMetaModel;
use crate::dictionary::metadata::ex::{MetadataEx, MetadataManagerEx, MetadataWithOrigin};
use crate::dictionary::metadata::update::WordIdUpdate;
use crate::dictionary::search::{DictionarySearcher, SearchIndex};

pub type DictWithMeta<T> = DictionaryWithMeta<T, Vocabulary<T>, MetadataManagerEx>;
pub type EfficientDictWithMetaDefault = DictWithMeta<ArcStr>;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DictionaryWithMeta<T = ArcStr, V = Vocabulary<T>, C = MetadataManagerEx> {
    #[serde(bound(serialize = "V: Serialize, T: Serialize", deserialize = "V: Deserialize<'de>, T: Deserialize<'de> + Hash + Eq"))]
    inner: Dictionary<T, V>,
    #[serde(bound(serialize = "C: Serialize", deserialize = "C: Deserialize<'de>"))]
    metadata: C
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
    pub fn create_topic_matrix<'a, L: Language, S: AsRef<str>>(&'a self, mode: &CreateTopicMatrixMode<S>) -> DictMetaModel {

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

        let mut matrix: DictMetaModel;
        let iter: std::slice::Iter<'a, MetadataEx>;

        if L::LANG.is_a() {
            matrix = DictMetaModel::with_capacity(self.voc_a().len());
            iter = self.metadata.meta_a().iter();
        } else {
            matrix = DictMetaModel::with_capacity(self.voc_b().len());
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
    T: Hash + Eq + Clone,
    M: MetadataManager
{
    fn insert_meta_for_create_subset<'a, 'b, L: Language>(
        &'b mut self,
        word_id: usize,
        metadata_ref: M::Reference<'a>,
        add_only_associated_count: bool,
    ) -> M::MutReference<'b> {
        let mut meta = self.metadata.get_or_create_meta::<L>(
            match L::LANG {
                LanguageMarker::A => {
                    &mut self.inner.voc_a
                }
                LanguageMarker::B => {
                    &mut self.inner.voc_b
                }
            },
            word_id
        );
        meta.update_with_reference(metadata_ref, add_only_associated_count);
        meta
    }

    /// Keeps every entry where the filter functions return true
    pub fn create_subset_with_filters<'a, F1, F2>(&'a self, filter_a: F1, filter_b: F2) -> DictionaryWithMeta<T, V, M>
    where
        F1: Fn(&'a DictionaryWithMeta<T, V, M>, usize, Option<&M::Reference<'a>>) -> bool,
        F2: Fn(&'a DictionaryWithMeta<T, V, M>, usize, Option<&M::Reference<'a>>) -> bool
    {

        let mut new_dictionary = Self {
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

        let mut post_update_a = HashSet::new();
        let mut post_update_b = HashSet::new();

        let mut meta_a_added = vec![false; self.voc_a().len()];
        let mut meta_b_added = vec![false; self.voc_b().len()];

        for DirectedElement {
            a: (original_word_id_a, original_meta_a),
            b: (original_word_id_b, original_meta_b),
            direction
        } in self.iter_with_meta() {
            if filter_a(self, original_word_id_a, original_meta_a.as_ref()) {
                if filter_b(self, original_word_id_b, original_meta_b.as_ref()) {
                    let word_a = self.voc_a().get_value_by_id(original_word_id_a).unwrap();
                    let word_b = self.voc_b().get_value_by_id(original_word_id_b).unwrap();
                    let word_a_processed_is_known = new_dictionary.voc_a().contains_value(word_a);
                    let word_b_processed_is_known = new_dictionary.voc_b().contains_value(word_b);
                    let DirectedElement {
                        a: id_a_processed,
                        b: id_b_processed,
                        direction: _
                    } = new_dictionary.insert_value_dir(direction, word_a.clone(), word_b.clone());
                    update.add_id::<A>(original_word_id_a, id_a_processed);
                    update.add_id::<B>(original_word_id_b, id_b_processed);

                    if let Some(original_meta_a) = original_meta_a {
                        if let Some(associated_word_ids) = original_meta_a.collect_all_associated_word_ids() {
                            for value in associated_word_ids.iter() {
                                if let Some(word) = self.voc_a().get_value_by_id(value) {
                                    if let Some(new_id_assoc) = new_dictionary.voc_a().get_id(word) {
                                        update.add_id::<A>(original_word_id_a, new_id_assoc)
                                    } else {
                                        // This is necessary to properly insert the words if they are encountered during the main loop.
                                        post_update_a.insert((original_word_id_a, word.clone()));
                                    }
                                }
                            }
                        }
                        if unsafe{!meta_a_added.get_unchecked(original_word_id_a)} {
                            meta_a_added[original_word_id_a] = true;
                            new_dictionary.insert_meta_for_create_subset::<A>(id_a_processed, original_meta_a, word_a_processed_is_known);
                        }
                    }
                    if let Some(original_meta_b) = original_meta_b {
                        if let Some(associated_word_ids) = original_meta_b.collect_all_associated_word_ids() {
                            for value in associated_word_ids.iter() {
                                if let Some(word) = self.voc_b().get_value_by_id(value) {
                                    if let Some(new_id_assoc) = new_dictionary.voc_b().get_id(word) {
                                        update.add_id::<B>(original_word_id_b, new_id_assoc)
                                    } else {
                                        // This is necessary to properly insert the words if they are encountered during the main loop.
                                        post_update_b.insert((original_word_id_b, word.clone()));
                                    }
                                }
                            }
                        }

                        if unsafe{!meta_b_added.get_unchecked(original_word_id_b)} {
                            meta_b_added[original_word_id_b] = true;
                            new_dictionary.insert_meta_for_create_subset::<B>(id_b_processed, original_meta_b, word_b_processed_is_known);
                        }
                    }

                }
            }
        }

        for (a, v) in post_update_a {
            update.add_id::<A>(a, new_dictionary.voc_a_mut().add_value(v));
        }
        for (b, v) in post_update_b {
            update.add_id::<B>(b, new_dictionary.voc_b_mut().add_value(v));
        }

        new_dictionary.metadata.update_ids(&update);
        new_dictionary.metadata.optimize();
        new_dictionary
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


impl<T, V, C> DictionaryWithSearch<T, V> for DictionaryWithMeta<T, V, C>
where
    V: BasicVocabulary<T>,
    T: AsRef<str> + Send + Sync,
    C: MetadataManager
{
    fn get_searcher(&self) -> DictionarySearcher<Self, V, T> {
        let index = self.inner.search_index.get_or_init(SearchIndex::new);
        DictionarySearcher::new(self, index)
    }
}



impl<T, V, M> DictionaryWithMeta<T, V, M> where T: Eq + Hash, V: MappableVocabulary<T>, M: Clone {
    pub fn map<R, Voc, F>(self, f: F) -> DictionaryWithMeta<R, Voc, M>
    where
        F: Fn(T)-> R,
        Voc: BasicVocabulary<R>,
        R: Eq + Hash + Clone
    {
        DictionaryWithMeta::<R, Voc, M>::new(
            self.inner.map(f),
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
    T: Eq + Hash + Clone,
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

pub struct DictionaryWithMetaProcessResult<T> {
    word: T,
    unprocessed: Option<T>
}

impl<T> DictionaryWithMetaProcessResult<T> {
    pub fn new(word: T) -> DictionaryWithMetaProcessResult<T> {
        Self {
            word,
            unprocessed: None
        }
    }

    pub fn with_unprocessed(word: T, unprocessed: T) -> DictionaryWithMetaProcessResult<T> {
        Self {
            word,
            unprocessed: Some(unprocessed)
        }
    }
}

impl<T> From<T> for DictionaryWithMetaProcessResult<T> {
    fn from(word: T) -> Self {
        DictionaryWithMetaProcessResult::new(word)
    }
}

impl<T> From<(T, T)> for DictionaryWithMetaProcessResult<T> {
    fn from((word, unprocessed): (T, T)) -> Self {
        DictionaryWithMetaProcessResult::with_unprocessed(word, unprocessed)
    }
}

impl<T> From<(T, Option<T>)> for DictionaryWithMetaProcessResult<T> {
    fn from((word, unprocessed): (T, Option<T>)) -> Self {
        if let Some(unprocessed) = unprocessed {
            DictionaryWithMetaProcessResult::with_unprocessed(word, unprocessed)
        } else {
            DictionaryWithMetaProcessResult::new(word)
        }
    }
}


impl<T, V, M> DictionaryFilterable<T, V>  for DictionaryWithMeta<T, V, M>
where
    V: VocabularyMut<T> + From<Option<LanguageHint>> + AnonymousVocabulary + AnonymousVocabularyMut,
    T: Hash + Eq + Clone + AsRef<str>,
    for<'a> &'a str: Into<<M as MetadataManager>::FieldValue>,
    M: MetadataManager
{
    type ProcessResult<U> = DictionaryWithMetaProcessResult<T>;

    /// The result
    fn filter_and_process<'a, Fa, Fb, E>(&'a self, f_a: Fa, f_b: Fb) -> Result<Self, E>
    where
        Self: Sized,
        T: 'a,
        Fa: Fn(&'a T) -> Result<Option<Self::ProcessResult<T>>, E>,
        Fb: Fn(&'a T) -> Result<Option<Self::ProcessResult<T>>, E>
    {
        let mut new_dictionary = Self {
            inner: Dictionary::new_with(
                self.voc_a().language().cloned(),
                self.voc_b().language().cloned()
            ),
            metadata: self.metadata.copy_keep_vocabulary()
        };

        let mut update = WordIdUpdate::new(
            self.voc_a().len(),
            self.voc_b().len()
        );

        let mut post_update_a = HashSet::new();
        let mut post_update_b = HashSet::new();

        let mut meta_a_added = vec![false; self.voc_a().len()];
        let mut meta_b_added = vec![false; self.voc_b().len()];

        for DirectedElement {
            a: (original_word_id_a, original_meta_a),
            b: (original_word_id_b, original_meta_b),
            direction
        } in self.iter_with_meta() {
            let DictionaryWithMetaProcessResult{
                word: word_a_processed,
                unprocessed: word_a_unprocessed
            } = match f_a(self.voc_a().get_value_by_id(original_word_id_a).unwrap())? {
                None => {
                    continue;
                }
                Some(value) => {
                    value
                }
            };

            let DictionaryWithMetaProcessResult {
                word: word_b_processed,
                unprocessed: word_b_unprocessed
            } = match f_b(self.voc_b().get_value_by_id(original_word_id_b).unwrap())? {
                None => {
                    continue;
                }
                Some(value) => {
                    value
                }
            };

            let word_a_processed_is_known = new_dictionary.voc_a().contains_value(&word_a_processed);
            let word_b_processed_is_known = new_dictionary.voc_b().contains_value(&word_b_processed);

            let DirectedElement {
                a: id_a_processed,
                b: id_b_processed,
                direction: _
            } = new_dictionary.insert_value_dir(direction, word_a_processed, word_b_processed);

            update.add_id::<A>(original_word_id_a, id_a_processed);
            update.add_id::<B>(original_word_id_b, id_b_processed);

            // Update meta a
            if let Some(original_meta_a) = original_meta_a {
                if let Some(associated_word_ids) = original_meta_a.collect_all_associated_word_ids() {
                    for value in associated_word_ids.iter() {
                        if let Some(value) = self.voc_a().get_value_by_id(value) {
                            if let Some(DictionaryWithMetaProcessResult{word, ..}) = f_a(value)? {
                                if let Some(new_id_assoc) = new_dictionary.voc_a().get_id(&word) {
                                    update.add_id::<A>(original_word_id_a, new_id_assoc)
                                } else {
                                    // This is necessary to properly insert the words if they are encountered during the main loop.
                                    post_update_a.insert((original_word_id_a, word));
                                }
                            }
                        }
                    }
                }
                if unsafe{!meta_a_added.get_unchecked(original_word_id_a)} {
                    meta_a_added[original_word_id_a] = true;
                    new_dictionary.insert_meta_for_create_subset::<A>(id_a_processed, original_meta_a, word_a_processed_is_known);
                }
            }

            // Update meta b
            if let Some(original_meta_b) = original_meta_b {
                if let Some(associated_word_ids) = original_meta_b.collect_all_associated_word_ids() {
                    for value in associated_word_ids.iter() {
                        if let Some(value) = self.voc_b().get_value_by_id(value) {
                            if let Some(DictionaryWithMetaProcessResult{word, ..}) = f_b(value)? {
                                if let Some(new_id_assoc) = new_dictionary.voc_b().get_id(&word) {
                                    update.add_id::<B>(original_word_id_b, new_id_assoc)
                                } else {
                                    // This is necessary to properly insert the words if they are encountered during the main loop.
                                    post_update_b.insert((original_word_id_b, word));
                                }
                            }
                        }
                    }
                }
                if unsafe{!meta_b_added.get_unchecked(original_word_id_b)} {
                    meta_b_added[original_word_id_b] = true;
                    new_dictionary.insert_meta_for_create_subset::<B>(id_b_processed, original_meta_b, word_b_processed_is_known);
                }
            }

            if let Some(targ) = M::unprocessed_field() {
                if let Some(a_unprocessed) = word_a_unprocessed {
                    match new_dictionary.get_or_create_meta_a(id_a_processed).insert_value(targ.clone(), None, a_unprocessed.as_ref()) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Failed to insert unprocessed word a {} into metadata {:?}", id_a_processed, err);
                        }
                    }
                }

                if let Some(b_unprocessed) = word_b_unprocessed {
                    match new_dictionary.get_or_create_meta_b(id_b_processed).insert_value(targ, None, b_unprocessed.as_ref()) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Failed to insert unprocessed word b {} into metadata {:?}", id_b_processed, err);
                        }
                    }
                }
            }


        }

        for (a, v) in post_update_a {
            update.add_id::<A>(a, new_dictionary.voc_a_mut().add_value(v));
        }
        for (b, v) in post_update_b {
            update.add_id::<B>(b, new_dictionary.voc_b_mut().add_value(v));
        }

        new_dictionary.metadata.update_ids(&update);
        new_dictionary.metadata.optimize();
        Ok(new_dictionary)
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
        Fa: Fn(&'a T) -> bool,
        Fb: Fn(&'a T) -> bool
    {
        self.create_subset_with_filters(
            |slf, a, _| {
                let voc = slf.voc_a();
                filter_a(voc.get_value_by_id(a).unwrap())
            },
            |slf, b, _| {
                let voc = slf.voc_b();
                filter_b(voc.get_value_by_id(b).unwrap())
            }
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
    T: Hash + Eq + Clone,
    V: BasicVocabulary<T> + AnonymousVocabulary,
    M: MetadataManager
{
    type Item = DirectedElement<
        (usize, T, Option<M::ResolvedMetadata>),
        (usize, T, Option<M::ResolvedMetadata>)
    >;
    type IntoIter = DictionaryWithMetaIterator<DictionaryWithMeta<T, V, M>, T, V, M>;

    fn into_iter(self) -> Self::IntoIter {
        DictionaryWithMetaIterator::new(self)
    }
}


impl<T, V, M> MergingDictionary<T, V> for DictionaryWithMeta<T, V, M>
where
    T: Eq + Hash + Clone,
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

        for DirectedElement {
            a: (word_id_a, meta_a),
            b: (word_id_b, meta_b),
            direction
        } in other.iter_with_meta() {
            let word_a = other.voc_a().get_value_by_id(word_id_a).unwrap();
            let word_b = other.voc_b().get_value_by_id(word_id_b).unwrap();
            let DirectedElement {
                a: word_a,
                b: word_b,
                direction: _
            } = self.insert_value_dir(direction, word_a.clone(), word_b.clone());
            if let Some(meta_a) = meta_a {
                if let Some(a) = meta_a.collect_all_associated_word_ids() {
                    for value in a.iter() {
                        if let Some(value) = other.voc_a().get_value_by_id(value) {
                            update.add_id::<A>(
                                word_id_a,
                                self.inner.voc_a.add_value(value.clone())
                            )
                        }
                    }
                }
                self.insert_meta_for_create_subset::<A>(word_a, meta_a, true);
            }
            if let Some(meta_b) = meta_b {
                if let Some(b) = meta_b.collect_all_associated_word_ids() {
                    for value in b.iter() {
                        if let Some(value) = other.voc_b().get_value_by_id(value) {
                            update.add_id::<B>(
                                word_id_b,
                                self.inner.voc_b.add_value(value.clone())
                            )
                        }
                    }
                }
                self.insert_meta_for_create_subset::<B>(word_b, meta_b, true);
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