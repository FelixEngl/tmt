use std::hash::Hash;
use std::sync::Arc;
use thiserror::Error;
use ldatranslate_topicmodel::dictionary::DictionaryWithVocabulary;
use ldatranslate_topicmodel::vocabulary::BasicVocabulary;
use ldatranslate_translate::TopicLike;
use crate::py::word_counts::{IdfProviderError, NGramStatisticsLangSpecific};
use crate::tools::boosting::BoostMethod;
use crate::tools::tf_idf::{Idf};
use crate::translate::{NGramLanguageBoostConfig};

#[derive(Clone, Debug)]
pub struct NGramScoreBooster {
    values: Arc<Vec<f64>>,
    factor: f64,
    boost_method: BoostMethod,
}

#[derive(Debug, Error)]
pub enum NGramScoreBoosterError {
    #[error(transparent)]
    Idf(#[from] IdfProviderError<Idf>),
}

impl NGramScoreBooster {
    pub fn new<D, V, T>(config: &NGramLanguageBoostConfig, ngram_idf_provider: &NGramStatisticsLangSpecific<T>, dict: &D) -> Result<Self, NGramScoreBoosterError>
    where
        D: DictionaryWithVocabulary<T, V>,
        V: BasicVocabulary<T>,
        T: Send + Sync + Eq + Hash + Clone
    {
        let (_, mut mapping) = ngram_idf_provider.create_idf_mapping(
            &config.idf,
            dict
        )?.unwrap_or_else(|| (0, Vec::with_capacity(0)));
        config.norm.norm(&mut mapping);
        Ok(
            Self {
                values: Arc::new(mapping),
                factor: config.factor,
                boost_method: config.boosting,
            }
        )
    }

    pub fn boost(&self, word_id: usize, probability: f64) -> f64 {
        self.values.get(word_id).map(|&boost| self.boost_method.boost(
            probability,
            boost,
            self.factor
        )).unwrap_or(probability)
    }
}