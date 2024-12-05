use std::borrow::Borrow;
use std::hash::Hash;
use itertools::Itertools;
use ndarray::Array1;
use ldatranslate_topicmodel::dictionary::metadata::{MetadataReference};
use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataManagerEx;
use ldatranslate_topicmodel::dictionary::SearchableDictionaryWithMetadata;
use ldatranslate_topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary, SearchableVocabulary};
use crate::translate::dictionary_meta::{DictMetaFieldPattern, Similarity, SparseVectorFactory};
use crate::translate::dictionary_meta::topic_associated::MetaFieldCountProvider;


pub trait Normalizer {
    fn normalize_probability_a(&self, score: f64) -> f64;
}

#[derive(Debug, Copy, Clone)]
pub enum StandardNormalizer {
    Mean,
    Relative,
    Custom(f64)
}


impl Normalizer for StandardNormalizer {
    fn normalize_probability_a(&self, score: f64) -> f64 {
        match self {
            StandardNormalizer::Mean => {
                score + 0.5
            }
            StandardNormalizer::Relative => {
                score + 1.0
            }
            StandardNormalizer::Custom(value) => {
                *value + score
            }
        }
    }
}


pub fn calculate_cross_language_topic_association<Q: ?Sized, T, V, D, P: ?Sized, A, N>(
    dictionary: &D,
    word_a: &Q,
    words_b: &[&Q],
    factory: &SparseVectorFactory,
    pattern: &P,
    algorithm: &A,
    normalizer: &N,
    probability_a: f64
) -> Result<Vec<A>, A::Error<f64>>
where
    D: SearchableDictionaryWithMetadata<T, V, MetadataManagerEx>,
    V: AnonymousVocabulary + BasicVocabulary<T> + SearchableVocabulary<T>,
    T: Borrow<Q> + Eq + Hash,
    Q: Hash + Eq,
    P: DictMetaFieldPattern,
    A: Similarity,
    N: Normalizer
{
    let pattern = factory.convert_to_template(pattern);
    let meta_a = if let Some(found) = dictionary.meta_for_word_a(word_a) {
        found
    } else {
        return Ok(vec![normalizer.normalize_probability_a(probability_a); words_b.len()])
    };
    let metas_b = words_b.into_iter().map(|&v| dictionary.meta_for_word_b(v)).collect::<Vec<_>>();

    let pattern = pattern.pattern();
    let counts_a = pattern.iter().map(|&value| {
        meta_a.raw().get_count_for(value) as f64
    }).collect::<Array1<f64>>();

    let values = metas_b.into_iter().map(|meta_b| {
        if let Some(meta_b) = meta_b {
            let counts_b = pattern.iter().map(|&value| {
                meta_b.raw().get_count_for(value) as f64
            }).collect::<Array1<f64>>();
            algorithm.calculate(&counts_a, &counts_b)
        } else {
            Ok(0.0)
        }
    }).collect::<Result<Vec<f64>, A::Error<f64>>>()?;


}