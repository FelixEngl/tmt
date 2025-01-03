//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::borrow::Borrow;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::Debug;
use std::hash::Hash;
use strum::Display;
use thiserror::Error;
use ldatranslate_toolkit::register_python;

/// The statistics over the documents in a corpus
pub trait CorpusDocumentStatistics {
    /// A word in a corpus
    type Word;
    /// The number of documents in the corpus
    fn document_count(&self) -> u128;

    /// The frequency of a [word] in a corpus
    fn word_frequency<Q>(&self, word: &Q) -> Option<u128> where Q: Hash + Eq + ?Sized, Self::Word: Borrow<Q>;

    /// The number of documents containing [word] in a corpus
    fn word_document_count<Q>(&self, word: &Q) -> Option<u128> where Q: Hash + Eq + ?Sized, Self::Word: Borrow<Q>;

    /// Returns an iterator over the words and associated values
    fn iter_document_count(&self) -> impl Iterator<Item = (&Self::Word, u128)>;

    /// Returns an iterator over the words and associated values
    fn iter_frequency(&self) -> impl Iterator<Item = (&Self::Word, u128)>;
}


#[allow(dead_code)]
pub mod defaults {
    use super::{Idf, Tf, TfIdf};
    pub const RAW_INVERSE: TfIdf<Tf, Idf> = TfIdf::new(Tf::RawCount, Idf::InverseDocumentFrequency);
    pub const TERM_FREQUENCY_INVERSE: TfIdf<Tf, Idf> =
        TfIdf::new(Tf::TermFrequency, Idf::InverseDocumentFrequency);
    pub const RAW_INVERSE_SMOOTH: TfIdf<Tf, Idf> =
        TfIdf::new(Tf::RawCount, Idf::InverseDocumentFrequencySmooth);
    pub const TERM_FREQUENCY_INVERSE_SMOOTH: TfIdf<Tf, Idf> =
        TfIdf::new(Tf::TermFrequency, Idf::InverseDocumentFrequencySmooth);
}


/// A combination of Tf and Idf
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "Tf: Serialize, Idf: Serialize",
    deserialize = "Tf: DeserializeOwned, Idf: DeserializeOwned"
))]
pub struct TfIdf<Tf, Idf> {
    pub tf: Tf,
    pub idf: Idf,
}

#[allow(dead_code)]
impl<Tf, Idf> TfIdf<Tf, Idf>
where
    Tf: TfAlgorithm,
{
    delegate::delegate! {
        to self.tf {
            fn calculate_tf<W, D: IntoIterator<Item=W>>(&self, doc: D) -> HashMap<W, f64> where W: Hash + Eq;
        }
    }
}

#[allow(dead_code)]
impl<Tf, Idf> TfIdf<Tf, Idf>
where
    Idf: IdfAlgorithm,
{
    delegate::delegate! {
        to self.idf {
            fn calculate_idf<Q, S>(
                &self,
                statistics: &S,
                word: &Q,
                adjusted: bool,
            ) -> Result<f64, Idf::Error>
            where
                S: CorpusDocumentStatistics,
                Q: Hash + Eq + ?Sized,
                S::Word: Borrow<Q>;

            fn calculate_idf_with_word_frequency<S>(
                &self,
                statistics: &S,
                word_frequency: u128,
                adjusted: bool,
            ) -> Result<f64, Idf::Error>
            where
                S: CorpusDocumentStatistics,
            ;
        }
    }
}

impl<Idf> TfIdf<(), Idf> {
    #[allow(dead_code)]
    pub const fn new_idf_only(idf: Idf) -> Self {
        Self::new((), idf)
    }
}

impl<Tf, Idf> TfIdf<Tf, Idf> {
    pub const fn new(tf: Tf, idf: Idf) -> Self {
        Self { tf, idf }
    }

    #[allow(dead_code)]
    pub fn to_idf_only(self) -> TfIdf<(), Idf> {
        TfIdf::<(), Idf>::new_idf_only(self.idf)
    }
}

impl<T> From<T> for TfIdf<(), T>
where
    T: IdfAlgorithm,
{
    fn from(value: T) -> Self {
        Self::new((), value)
    }
}

impl<Tf, Idf> Copy for TfIdf<Tf, Idf>
where
    Tf: Copy,
    Idf: Copy,
{
}

/// Trait for IDF Algorithms
pub trait IdfAlgorithm {
    type Error: StdError;

    /// Calculates the IDF value for a single word based on the provided statistics.
    ///
    /// [number_of_documents] denotes the number of documents in the corpus
    /// [number_of_words] denote the number of distinct words in the whole corpus
    /// [word_frequency] denotes the frequency of a specific word in a corpus
    #[inline]
    fn calculate_idf<Q, S>(
        &self,
        statistics: &S,
        word: &Q,
        adjusted: bool
    ) -> Result<f64, Self::Error>
    where
        S: CorpusDocumentStatistics,
        Q: Hash + Eq + ?Sized,
        S::Word: Borrow<Q>,
    {
        let word_doc = if adjusted {
            statistics.word_document_count(word).unwrap_or(0)
        } else {
            match statistics.word_document_count(word) {
                None => {
                    return Ok(f64::NAN);
                }
                Some(value) => {
                    value
                }
            }
        };

        self.calculate_idf_with_word_frequency(statistics, word_doc, adjusted)
    }

    /// Returns the max value of the idf for a word that is in 0 docs.
    #[inline]
    fn max<S>(&self, statistics: &S, adjusted: bool) -> Result<f64, Self::Error>
    where
        S: CorpusDocumentStatistics,

    {
        if !adjusted {
            return Ok(f64::NAN);
        }
        self.calculate_idf_with_word_frequency(
            statistics,
            0,
            adjusted
        )
    }

    /// Calculates the IDF value for a single word based on the provided statistics.
    /// [word_frequency] denotes the frequency of a specific word in a corpus.
    ///
    /// Returns nan if the calculation is not possible.
    fn calculate_idf_with_word_frequency<S>(
        &self,
        statistics: &S,
        word_frequency: u128,
        adjusted: bool
    ) -> Result<f64, Self::Error>
    where
        S: CorpusDocumentStatistics;
}

/// Default IDF Algorithms
/// From https://en.wikipedia.org/wiki/Tf%E2%80%93idf

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyo3::pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, Display)]
pub enum Idf {
    Unary,
    #[default]
    InverseDocumentFrequency,
    InverseDocumentFrequencySmooth,
    InverseDocumentFrequencyMax,
    ProbabilisticInverseDocumentFrequency,
}


#[cfg(not(feature = "gen_python_api"))]
#[pyo3::pymethods]
impl Idf {
    fn __str__(&self) -> String {
        self.to_string()
    }
}

register_python!(enum Idf;);


#[derive(Debug, Error, Copy, Clone)]
pub enum IdfError {
    #[error("The CorpusDocumentStatistics is seen as empty but this should not be possible.")]
    StatisticsEmptyError,
}

impl IdfAlgorithm for Idf {
    type Error = IdfError;

    /// Calculates the IDF value for a single word based on the provided statistics.
    ///
    /// [number_of_documents] denotes the number of documents in the corpus
    /// [number_of_words] denote the number of distinct words in the whole corpus
    /// [word_frequency] denotes the frequency of a specific word in a corpus
    #[inline]
    fn calculate_idf<Q, S>(
        &self,
        statistics: &S,
        word: &Q,
        adjusted: bool
    ) -> Result<f64, Self::Error>
    where
        S: CorpusDocumentStatistics,
        Q: Hash + Eq + ?Sized,
        S::Word: Borrow<Q>
    {
        match self {
            Idf::Unary => Ok(1.0),
            other => {
                let wc = if adjusted {
                    statistics.word_document_count(word).unwrap_or(0)
                } else {
                    match statistics.word_document_count(word) {
                        None => {
                            return Ok(f64::NAN);
                        }
                        Some(value) => {
                            value
                        }
                    }
                };
                other.calculate_idf_with_word_frequency(statistics, wc, adjusted)
            }
        }
    }

    /// Calculates the IDF value for a single word based on the provided statistics.
    /// [word_frequency] denotes the frequency of a specific word in a corpus.
    ///
    /// Returns nan if the calculation is not possible.
    fn calculate_idf_with_word_frequency<S>(
        &self,
        statistics: &S,
        mut word_frequency: u128,
        adjusted: bool
    ) -> Result<f64, Self::Error>
    where
        S: CorpusDocumentStatistics
    {
        if adjusted {
            word_frequency += 1;
        }


        match self {
            Idf::Unary => Ok(1.0),
            Idf::InverseDocumentFrequency => {
                let document_count = if adjusted {
                    statistics.document_count() + 1
                } else {
                    statistics.document_count()
                };

                Ok((document_count as f64 / word_frequency as f64).log10())
            }
            Idf::InverseDocumentFrequencySmooth => {
                let document_count = if adjusted {
                    statistics.document_count() + 1
                } else {
                    statistics.document_count()
                };

                Ok((document_count as f64 / (word_frequency as f64 + 1.0)).log10() + 1.0)
            },
            Idf::InverseDocumentFrequencyMax => {
                if let Some((_, max_value)) = statistics
                    .iter_document_count()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                {
                    let max_value = if adjusted {
                        max_value + 1
                    } else {
                        max_value
                    };
                    Ok(((max_value as f64) / (word_frequency as f64 + 1.0)).log10())
                } else {
                    Err(IdfError::StatisticsEmptyError)
                }
            }
            Idf::ProbabilisticInverseDocumentFrequency => {
                let document_count = if adjusted {
                    statistics.document_count() + 1
                } else {
                    statistics.document_count()
                };

                let word_frequency = word_frequency as f64;
                Ok((document_count as f64 - word_frequency) / (word_frequency))
            }
        }
    }
}

/// Trait for TF Algorithm
pub trait TfAlgorithm {
    /// Calculates the TF value for a [doc].
    /// If a specific value can not be calculated
    #[allow(dead_code)]
    fn calculate_tf<W, D: IntoIterator<Item = W>>(&self, doc: D) -> HashMap<W, f64>
    where
        W: Hash + Eq;
}

/// Default TF Algorithms
/// From https://en.wikipedia.org/wiki/Tf%E2%80%93idf
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tf {
    Binary,
    RawCount,
    TermFrequency,
    LogNormalization,
    DoubleNormalization,
}

impl Tf {
    /// The implementation for Tf::RawCount, used in multiple impls.
    fn raw_count<W, D: IntoIterator<Item = W>>(doc: D) -> HashMap<W, f64>
    where
        W: Hash + Eq,
    {
        let mut result = HashMap::new();
        for word in doc {
            match result.entry(word) {
                Entry::Occupied(mut value) => {
                    value.insert(*value.get() + 1.0);
                }
                Entry::Vacant(value) => {
                    value.insert(1.0);
                }
            }
        }
        result
    }
}

impl TfAlgorithm for Tf {
    /// Calculates the TF value for a [doc].
    /// If a specific value can not be calculated
    fn calculate_tf<W, D: IntoIterator<Item = W>>(&self, doc: D) -> HashMap<W, f64>
    where
        W: Hash + Eq,
    {
        match self {
            Tf::Binary => {
                let mut result = HashMap::new();
                for word in doc.into_iter() {
                    result.insert(word, 1.0);
                }
                result
            }
            Tf::RawCount => Self::raw_count(doc),
            Tf::TermFrequency => {
                let mut result = Self::raw_count(doc);
                let divider = result.values().sum::<f64>();
                for value in result.values_mut() {
                    *value /= divider;
                }
                result
            }
            Tf::LogNormalization => {
                let mut result = Self::raw_count(doc);
                for value in result.values_mut() {
                    *value = (*value + 1.0).log10();
                }
                result
            }
            Tf::DoubleNormalization => {
                let mut result = Self::raw_count(doc);
                let max_value = result
                    .values()
                    .max_by(|&a, &b| a.partial_cmp(b).unwrap())
                    .copied();
                if let Some(max_value) = max_value {
                    for value in result.values_mut() {
                        *value = 0.5 + 0.5 * (*value / max_value);
                    }
                }
                result
            }
        }
    }
}


