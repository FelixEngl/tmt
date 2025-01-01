// //Copyright 2024 Felix Engl
// //
// //Licensed under the Apache License, Version 2.0 (the "License");
// //you may not use this file except in compliance with the License.
// //You may obtain a copy of the License at
// //
// //    http://www.apache.org/licenses/LICENSE-2.0
// //
// //Unless required by applicable law or agreed to in writing, software
// //distributed under the License is distributed on an "AS IS" BASIS,
// //WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// //See the License for the specific language governing permissions and
// //limitations under the License.
//
// use std::any::type_name;
// use std::borrow::Borrow;
// use serde::de::DeserializeOwned;
// use serde::{Deserialize, Serialize};
// use std::collections::hash_map::Entry;
// use std::collections::HashMap;
// use std::error::Error as StdError;
// use std::fmt::Debug;
// use std::hash::Hash;
// use bigdecimal::BigDecimal;
// use thiserror::Error;
//
// /// The statistics over the documents in a corpus
// pub trait CorpusDocumentStatistics {
//     /// A word in a corpus
//     type Word;
//     /// The number of documents in the corpus
//     fn document_count(&self) -> u128;
//     /// The number of distinct words in the corpus
//     #[allow(dead_code)]
//     fn word_count(&self) -> u128;
//     /// The number of unique words in the corpus
//     fn unique_word_count(&self) -> usize;
//
//     /// The frquency of a [word] in a corpus
//     fn word_frequency<Q>(&self, word: &Q) -> Option<u128> where Q: Hash + Eq + ?Sized, Self::Word: Borrow<Q>;
//
//     /// Returns an iterator over the words and associated values
//     fn iter(&self) -> impl Iterator<Item = (&Self::Word, u128)>;
// }
//
// #[allow(dead_code)]
// pub mod defaults {
//     use super::{Idf, Tf, TfIdf};
//     pub const RAW_INVERSE: TfIdf<Tf, Idf> = TfIdf::new(Tf::RawCount, Idf::InverseDocumentFrequency);
//     pub const TERM_FREQUENCY_INVERSE: TfIdf<Tf, Idf> =
//         TfIdf::new(Tf::TermFrequency, Idf::InverseDocumentFrequency);
//     pub const RAW_INVERSE_SMOOTH: TfIdf<Tf, Idf> =
//         TfIdf::new(Tf::RawCount, Idf::InverseDocumentFrequencySmooth);
//     pub const TERM_FREQUENCY_INVERSE_SMOOTH: TfIdf<Tf, Idf> =
//         TfIdf::new(Tf::TermFrequency, Idf::InverseDocumentFrequencySmooth);
// }
//
// /// A combination of Tf and Idf
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(bound(
//     serialize = "Tf: Serialize, Idf: Serialize",
//     deserialize = "Tf: DeserializeOwned, Idf: DeserializeOwned"
// ))]
// pub struct TfIdf<Tf, Idf> {
//     pub tf: Tf,
//     pub idf: Idf,
// }
//
// #[allow(dead_code)]
// impl<Tf, Idf> TfIdf<Tf, Idf>
// where
//     Tf: TfAlgorithm,
// {
//     delegate::delegate! {
//         to self.tf {
//             fn calculate_tf<W, D: IntoIterator<Item=W>>(&self, doc: D) -> HashMap<W, f64> where W: Hash + Eq;
//         }
//     }
// }
//
// #[allow(dead_code)]
// impl<Tf, Idf> TfIdf<Tf, Idf>
// where
//     Idf: IdfAlgorithm,
// {
//     delegate::delegate! {
//         to self.idf {
//             fn calculate_idf<W, S: CorpusDocumentStatistics<Word=W>>(&self, statistics: & S, word: &W) -> Result<Option<f64>, Idf::Error>;
//             fn calculate_idf_with_word_frequency<W, S: CorpusDocumentStatistics<Word=W>>(&self, statistics: & S, word: &W, word_frequency: u64) -> Result<f64, Idf::Error>;
//         }
//     }
// }
//
// impl<Idf> TfIdf<(), Idf> {
//     pub const fn new_idf_only(idf: Idf) -> Self {
//         Self::new((), idf)
//     }
// }
//
// impl<Tf, Idf> TfIdf<Tf, Idf> {
//     pub const fn new(tf: Tf, idf: Idf) -> Self {
//         Self { tf, idf }
//     }
//
//     pub fn to_idf_only(self) -> TfIdf<(), Idf> {
//         TfIdf::<(), Idf>::new_idf_only(self.idf)
//     }
// }
//
// impl<T> From<T> for TfIdf<(), T>
// where
//     T: IdfAlgorithm,
// {
//     fn from(value: T) -> Self {
//         Self::new((), value)
//     }
// }
//
// impl<Tf, Idf> Copy for TfIdf<Tf, Idf>
// where
//     Tf: Copy,
//     Idf: Copy,
// {
// }
//
// /// Trait for IDF Algorithms
// pub trait IdfAlgorithm {
//     type Error: StdError;
//
//     /// Calculates the IDF value for a single word based on the provided statistics.
//     ///
//     /// [number_of_documents] denotes the number of documents in the corpus
//     /// [number_of_words] denote the number of distinct words in the whole corpus
//     /// [word_frequency] denotes the frequency of a specific word in a corpus
//     #[inline]
//     #[allow(dead_code)]
//     fn calculate_idf<Float, W, S>(
//         &self,
//         statistics: &S,
//         word: &W,
//     ) -> Result<Option<Float>, Self::Error>
//     where
//         S: CorpusDocumentStatistics<Word = W>,
//         Float: num::Float + num::One
//     {
//         statistics
//             .word_frequency(word)
//             .map(|value| self.calculate_idf_with_word_frequency(statistics, word, value))
//             .transpose()
//     }
//
//     /// Calculates the IDF value for a single word based on the provided statistics.
//     /// [word_frequency] denotes the frequency of a specific word in a corpus.
//     ///
//     /// Returns nan if the calculation is not possible.
//     fn calculate_idf_with_word_frequency<Float, Integer, W, S>(
//         &self,
//         statistics: &S,
//         word: &W,
//         word_frequency: Integer,
//     ) -> Result<Float, Self::Error>
//     where
//         S: CorpusDocumentStatistics<Word = W>,
//         Integer: num::Integer + num::NumCast,
//         Float: num::Float + num::One + num::NumCast,
//     ;
// }
//
// /// Default IDF Algorithms
// /// From https://en.wikipedia.org/wiki/Tf%E2%80%93idf
// #[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
// pub enum Idf {
//     Unary,
//     InverseDocumentFrequency,
//     InverseDocumentFrequencySmooth,
//     InverseDocumentFrequencyMax,
//     ProbabilisticInverseDocumentFrequency,
// }
//
// #[derive(Debug, Error, Copy, Clone)]
// pub enum IdfError {
//     #[error("The CorpusDocumentStatistics is seen as empty but this should not be possible.")]
//     StatisticsEmptyError,
//     #[error("Falied to cast {from_type} to {to_type}")]
//     CastError {
//         from_type: &'static str,
//         to_type: &'static str,
//     }
// }
//
// impl IdfAlgorithm for Idf {
//     type Error = IdfError;
//
//     /// Calculates the IDF value for a single word based on the provided statistics.
//     ///
//     /// [number_of_documents] denotes the number of documents in the corpus
//     /// [number_of_words] denote the number of distinct words in the whole corpus
//     /// [word_frequency] denotes the frequency of a specific word in a corpus
//     #[inline]
//     fn calculate_idf<Float, W, S>(
//         &self,
//         statistics: &S,
//         word: &W,
//     ) -> Result<Option<Float>, Self::Error>
//     where
//         S: CorpusDocumentStatistics<Word = W>,
//         Float: num::Float + num::One
//     {
//         match self {
//             Idf::Unary => Ok(Some(Float::one())),
//             other => statistics
//                 .word_frequency(word)
//                 .map(|value| other.calculate_idf_with_word_frequency(statistics, word, value))
//                 .transpose(),
//         }
//     }
//
//     /// Calculates the IDF value for a single word based on the provided statistics.
//     /// [word_frequency] denotes the frequency of a specific word in a corpus.
//     ///
//     /// Returns nan if the calculation is not possible.
//     fn calculate_idf_with_word_frequency<Float, Integer, W, S>(
//         &self,
//         statistics: &S,
//         _: &W,
//         word_frequency: Integer,
//     ) -> Result<Float, Self::Error>
//     where
//         S: CorpusDocumentStatistics<Word = W>,
//         Integer: num::Integer + num::NumCast,
//         Float: num::Float + num::One + num::NumCast,
//     {
//         match self {
//             Idf::Unary => Ok(Float::one()),
//             Idf::InverseDocumentFrequency => {
//                 let document_count = Float::from(statistics.document_count()).ok_or_else(|| IdfError::CastError {
//                     from_type: type_name::<u128>(),
//                     to_type: type_name::<Float>(),
//                 })?;
//
//                 let word_frequency = Float::from(word_frequency).ok_or_else(|| IdfError::CastError {
//                     from_type: type_name::<Integer>(),
//                     to_type: type_name::<Float>(),
//                 })?;
//                 // Ok((statistics.document_count() as f64 / word_frequency as f64).log10())
//                 Ok((document_count / word_frequency).log10())
//             }
//             Idf::InverseDocumentFrequencySmooth => {
//                 let document_count = Float::from(statistics.document_count()).ok_or_else(|| IdfError::CastError {
//                     from_type: type_name::<u128>(),
//                     to_type: type_name::<Float>(),
//                 })?;
//
//                 let word_frequency = Float::from(word_frequency).ok_or_else(|| IdfError::CastError {
//                     from_type: type_name::<Integer>(),
//                     to_type: type_name::<Float>(),
//                 })?;
//
//                 // Ok((statistics.document_count() as f64
//                 //     / (word_frequency as f64 + 1.0))
//                 //     .log10()
//                 //     + 1.0)
//
//                 Ok((document_count / (word_frequency + Float::one())).log10() + Float::one())
//             },
//             Idf::InverseDocumentFrequencyMax => {
//                 if let Some((_, max_value)) = statistics
//                     .iter()
//                     .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
//                 {
//                     let max_value = Float::from(*max_value).ok_or_else(|| IdfError::CastError {
//                         from_type: type_name::<u64>(),
//                         to_type: type_name::<Float>(),
//                     })?;
//                     let word_frequency = Float::from(word_frequency).ok_or_else(|| IdfError::CastError {
//                         from_type: type_name::<Integer>(),
//                         to_type: type_name::<Float>(),
//                     })?;
//                     // Ok(((*max_value as f64) / (word_frequency as f64 + 1.0)).log10())
//                     Ok((max_value / (word_frequency + Float::one())).log10())
//                 } else {
//                     Err(IdfError::StatisticsEmptyError)
//                 }
//             }
//             Idf::ProbabilisticInverseDocumentFrequency => {
//                 let document_count = Float::from(statistics.document_count()).ok_or_else(|| IdfError::CastError {
//                     from_type: type_name::<u128>(),
//                     to_type: type_name::<Float>(),
//                 })?;
//
//                 let word_frequency = Float::from(word_frequency).ok_or_else(|| IdfError::CastError {
//                     from_type: type_name::<Integer>(),
//                     to_type: type_name::<Float>(),
//                 })?;
//
//                 // let word_frequency = word_frequency as f64;
//                 // Ok((statistics.document_count() as f64 - word_frequency) / (word_frequency))
//
//                 Ok((document_count - word_frequency) / (word_frequency))
//             }
//         }
//     }
// }
//
// /// Trait for TF Algorithm
// pub trait TfAlgorithm {
//     /// Calculates the TF value for a [doc].
//     /// If a specific value can not be calculated
//     #[allow(dead_code)]
//     fn calculate_tf<Float, W, D>(&self, doc: D) -> HashMap<W, Float>
//     where
//         Float: num::Float + num::One + num::traits::NumAssignOps + num::traits::NumOps,
//         W: Hash + Eq,
//         D: IntoIterator<Item = W>
//     ;
// }
//
// /// Default TF Algorithms
// /// From https://en.wikipedia.org/wiki/Tf%E2%80%93idf
// #[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
// pub enum Tf {
//     Binary,
//     RawCount,
//     TermFrequency,
//     LogNormalization,
//     DoubleNormalization,
// }
//
// impl Tf {
//     /// The implementation for Tf::RawCount, used in multiple impls.
//     fn raw_count<CountValue, W, D>(doc: D) -> HashMap<W, CountValue>
//     where
//         CountValue: std::ops::AddAssign + num::One,
//         W: Hash + Eq,
//         D: IntoIterator<Item = W>,
//     {
//         let mut result = HashMap::new();
//         for word in doc {
//             result.entry(word).and_modify(|v| *v += CountValue::one()).or_insert(CountValue::one());
//         }
//         result
//     }
// }
//
// impl TfAlgorithm for Tf {
//     /// Calculates the TF value for a [doc].
//     /// If a specific value can not be calculated
//     fn calculate_tf<Float, W, D>(&self, doc: D) -> HashMap<W, Float>
//     where
//         Float: num::Float + num::One + num::traits::NumAssignOps + num::traits::NumOps + num::cast::FromPrimitive,
//         W: Hash + Eq,
//         D: IntoIterator<Item = W>
//     {
//         match self {
//             Tf::Binary => {
//                 let mut result = HashMap::new();
//                 for word in doc.into_iter() {
//                     result.insert(word, Float::one());
//                 }
//                 result
//             }
//             Tf::RawCount => Self::raw_count(doc),
//             Tf::TermFrequency => {
//                 let mut result = Self::raw_count(doc);
//                 let divider = result.values().sum::<Float>();
//                 for value in result.values_mut() {
//                     *value /= divider;
//                 }
//                 result
//             }
//             Tf::LogNormalization => {
//                 let mut result: HashMap<_, Float> = Self::raw_count(doc);
//                 for value in result.values_mut() {
//                     *value = (value.clone() + Float::one()).log10();
//                 }
//                 result
//             }
//             Tf::DoubleNormalization => {
//                 let mut result = Self::raw_count(doc);
//                 let max_value = result
//                     .values()
//                     .max_by(|a, b| a.partial_cmp(b).unwrap())
//                     .copied();
//                 let point_five: Float = num::cast(0.5f64).expect("The cast should not fail!");
//                 if let Some(max_value) = max_value {
//                     for value in result.values_mut() {
//                         *value = point_five + point_five * (*value / max_value);
//                     }
//                 }
//                 result
//             }
//         }
//     }
// }
