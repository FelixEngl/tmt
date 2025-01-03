use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use num::traits::AsPrimitive;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ldatranslate_topicmodel::dictionary::DictionaryWithVocabulary;
use ldatranslate_topicmodel::dictionary::google_ngram::{NGramCount, TotalCount};
use ldatranslate_topicmodel::language_hint::LanguageHint;
use ldatranslate_topicmodel::vocabulary::BasicVocabulary;
use crate::tools::tf_idf::{CorpusDocumentStatistics, IdfAlgorithm};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NGramStatistics<T> where T: Eq + Hash {
    inner: HashMap<LanguageHint, NGramStatisticsLangSpecific<T>>
}

impl<T> NGramStatistics<T> where T: Eq + Hash {
    pub fn new(inner: HashMap<LanguageHint, NGramStatisticsLangSpecific<T>>) -> Self {
        Self { inner }
    }

    pub fn builder() -> NGramStatisticsBuilder<T> {
        NGramStatisticsBuilder::new()
    }

    pub fn into_inner(self) -> HashMap<LanguageHint, NGramStatisticsLangSpecific<T>> {
        self.inner
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&NGramStatisticsLangSpecific<T>>
    where
        LanguageHint: Borrow<Q>,
        Q: Hash + Eq
    {
        self.inner.get(key)
    }

    pub fn get_idf_voc<Idf, D, V>(&self, idf: &Idf, dict: &D) -> Result<HashMap<LanguageHint, (usize, Vec<f64>)>, IdfProviderError<Idf>>
    where
        D: DictionaryWithVocabulary<T, V>,
        V: BasicVocabulary<T>,
        Idf: IdfAlgorithm + Send + Sync,
        Idf::Error: Send,
        T: Send + Sync + Clone
    {
        self.inner.iter().filter_map(|(k, v)| {
            v.create_idf_mapping(idf, dict).transpose().map(|v| v.map(|v| (k.clone(), v)))
        }).collect::<Result<HashMap<_, _>, _>>()
    }
}

pub struct NGramStatisticsBuilder<T> where T: Eq + Hash {
    inner: HashMap<LanguageHint, NGramStatisticsLangSpecificBuilder<T>>,
}

impl<T> NGramStatisticsBuilder<T> where T: Eq + Hash {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn add<F>(mut self, language_hint: impl Into<LanguageHint>, action: F) -> Self
    where
        F: FnOnce(&mut NGramStatisticsLangSpecificBuilder<T>) -> ()
    {
        let language_hint = language_hint.into();
        let value = self.inner.entry(language_hint.clone()).or_insert_with(|| NGramStatisticsLangSpecificBuilder::new(language_hint));
        action(value);
        self
    }

    pub fn build(self) -> NGramStatistics<T> {
        NGramStatistics::new(
            self.inner.into_iter().map(|(k, v)| (k, v.build())).collect()
        )
    }
}

pub struct NGramStatisticsLangSpecificBuilder<T> where T: Eq + Hash {
    language_hint: LanguageHint,
    counts: HashMap<T, NGramCount>,
    meta: HashMap<u8, NGramStatisticMeta>,
}

impl<T> NGramStatisticsLangSpecificBuilder<T> where T: Eq + Hash {
    pub fn new(language_hint: LanguageHint) -> Self {
        Self {
            language_hint,
            meta: HashMap::new(),
            counts: HashMap::new(),
        }
    }

    pub fn add(&mut self, word: impl Into<T>, frequency: impl AsPrimitive<u128>, volumes: impl AsPrimitive<u128>) -> &mut Self {
        self.put(word.into(), frequency.as_(), volumes.as_())
    }

    pub fn put(&mut self, word: T, frequency: u128, volumes: u128) -> &mut Self {
        self.counts.insert(word, NGramCount::new(frequency, volumes));
        self
    }

    pub fn register_meta(&mut self, n_gram_size: u8, unique_word_count: u128, total_counts: TotalCount) -> &mut Self {
        self.meta.insert(n_gram_size, NGramStatisticMeta::new(unique_word_count, total_counts));
        self
    }

    pub fn build(self) -> NGramStatisticsLangSpecific<T> {
        NGramStatisticsLangSpecific::new(
            self.language_hint,
            self.counts,
            self.meta
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NGramStatisticMeta {
    unique_count: u128,
    total_count: TotalCount,
}

impl NGramStatisticMeta {
    pub fn new(unique_count: u128, total_count: TotalCount) -> Self {
        Self { unique_count, total_count }
    }
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NGramStatisticsLangSpecific<T> where T: Eq + Hash {
    language: LanguageHint,
    counts: HashMap<T, NGramCount>,
    meta: HashMap<u8, NGramStatisticMeta>,
    min_counts: NGramCount,
    max_total_counts: u128
}

impl<T> NGramStatisticsLangSpecific<T> where T: Eq + Hash {
    pub fn new(
        language: LanguageHint,
        counts: HashMap<T, NGramCount>,
        meta: HashMap<u8, NGramStatisticMeta>,
    ) -> Self {
        let min = counts.values().min().cloned().unwrap_or(NGramCount::ZERO);
        Self::with_min(language, counts, meta, min)
    }

    pub fn with_min(
        language: LanguageHint,
        counts: HashMap<T, NGramCount>,
        meta: HashMap<u8, NGramStatisticMeta>,
        min_counts: NGramCount,
    ) -> Self {
        let max_total_counts = meta.values().map(|v| v.unique_count).max().unwrap_or(0);
        Self { language, counts, meta, min_counts, max_total_counts }
    }
}

#[derive(Debug, Error)]
pub enum IdfProviderError<Idf: IdfAlgorithm> {
    #[error("Expected voc for {0} but none was supplied from dict.")]
    NoLanguageInDictFound(LanguageHint),
    #[error(transparent)]
    Idf(Idf::Error)
}

impl<T> NGramStatisticsLangSpecific<T> where T: Clone + Eq + Hash
{
    #[inline(always)]
    pub fn idf<Q, Idf>(&self, idf: &Idf, word: &Q, adjusted: bool) -> Result<f64, <Idf as IdfAlgorithm>::Error>
    where
        Idf: IdfAlgorithm,
        Q: Hash + Eq + ?Sized,
        T: Borrow<Q>,
    {
        idf.calculate_idf(
            self,
            word,
            adjusted
        )
    }

    /// Returns
    pub fn create_idf_mapping<Idf, D, V>(&self, idf_alg: &Idf, d: &D) -> Result<Option<(usize, Vec<f64>)>, IdfProviderError<Idf>>
    where
        D: DictionaryWithVocabulary<T, V>,
        V: BasicVocabulary<T>,
        Idf: IdfAlgorithm + Send + Sync,
        Idf::Error: Send,
        T: Send + Sync
    {
        let voc = d.voc_by_hint(&self.language).ok_or_else(|| IdfProviderError::NoLanguageInDictFound(self.language.clone()))?;
        let overlap = voc.iter().filter(|&v| self.counts.contains_key(v)).count();
        if overlap == 0 {
            return Ok(None);
        }
        let empty_replacement = idf_alg.max(self, true).map_err(IdfProviderError::Idf)?;
        let idf = self.all_idf(idf_alg, true).map_err(IdfProviderError::Idf)?;

        let idf_of_voc = voc.iter().map(|value| idf.get(value).copied().unwrap_or(empty_replacement)).collect::<Vec<_>>();

        Ok(Some((overlap, idf_of_voc)))
    }

    pub fn all_idf<Idf>(&self, idf: &Idf, adjusted: bool) -> Result<HashMap<T, f64>, <Idf as IdfAlgorithm>::Error>
    where
        Idf: IdfAlgorithm + Send + Sync,
        Idf::Error: Send,
        T: Send + Sync
    {
        self.counts.par_iter().map(|(k, v)| {
            idf.calculate_idf_with_word_frequency(
                self,
                v.volumes,
                adjusted
            ).map(|v| {
                (k.clone(), v)
            })
        }).collect::<Result<HashMap<_, _>, _>>()
    }

    #[cfg(test)]
    pub fn all_idf_with_freq<Idf>(&self, idf: &Idf, adjusted: bool) -> Result<HashMap<T, (f64, NGramCount)>, <Idf as IdfAlgorithm>::Error>
    where
        Idf: IdfAlgorithm,
    {
        self.counts.iter().map(|(k, v)| {
            idf.calculate_idf_with_word_frequency(
                self,
                v.volumes,
                adjusted
            ).map(|p| {
                (k.clone(), (p, v.clone()))
            })
        }).collect::<Result<HashMap<_, _>, _>>()
    }
}

impl<T> CorpusDocumentStatistics for NGramStatisticsLangSpecific<T> where T: Eq + Hash {
    type Word = T;

    fn document_count(&self) -> u128 {
        self.max_total_counts
    }

    fn word_frequency<Q>(&self, word: &Q) -> Option<u128>
    where
        Q: Hash + Eq + ?Sized,
        Self::Word: Borrow<Q>
    {
        self.counts.get(word).map(|counts| counts.frequency)
    }

    fn word_document_count<Q>(&self, word: &Q) -> Option<u128>
    where
        Q: Hash + Eq + ?Sized,
        Self::Word: Borrow<Q>
    {
        self.counts.get(word).map(|counts| counts.volumes)
    }

    fn iter_document_count(&self) -> impl Iterator<Item=(&Self::Word, u128)> {
        self.counts.iter().map(|(k, v)| (k, v.volumes))
    }

    fn iter_frequency(&self) -> impl Iterator<Item=(&Self::Word, u128)> {
        self.counts.iter().map(|(k, v)| (k, v.frequency))
    }
}
