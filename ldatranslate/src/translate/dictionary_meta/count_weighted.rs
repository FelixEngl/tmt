use std::fmt::{Display, Formatter};
use std::ops::{AddAssign, Deref, DerefMut, DivAssign};
use std::sync::Arc;
use itertools::Itertools;
use ndarray::Zip;
use strum::EnumIs;
use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataEx;
use ldatranslate_topicmodel::model::WordId;
use crate::translate::dictionary_meta::{ DictMetaFieldPattern, DictionaryMetaProbabilityProvider, IllegalValueCount, SparseMetaVector, SparseVectorFactory};

#[derive(Clone, Debug)]
pub struct ByCountWeigthed {
    overall_topic_model: Arc<SparseMetaVector>,
    topic_model: Arc<Vec<SparseMetaVector>>,
    word_per_topic: Arc<Vec<Vec<SparseMetaVector>>>,
}

#[derive(Debug, Copy, Clone, EnumIs)]
pub enum FitTo {
    Model,
    Topic
}

impl ByCountWeigthed {
    pub fn new<P>(topic_model: &[Vec<f64>], meta: &[MetadataEx], factory: &SparseVectorFactory, pattern: &P, fit_to_topic: Option<FitTo>) -> Result<Self, IllegalValueCount>
    where
        P: DictMetaFieldPattern + ?Sized
    {
        let mut overall_topic_model = factory.create_empty(pattern);
        let mut topic_model_ct = Vec::with_capacity(topic_model.len());
        let mut word_per_topic = Vec::with_capacity(topic_model.len());

        for (prob, value) in topic_model.into_iter().zip_eq(meta) {
            let mut topic_model_values = factory.create_empty(pattern);
            let mut words_col = Vec::with_capacity(prob.len());
            let value = value.domain_count();
            for prob in prob {
                let value = factory.create(
                    pattern,
                    value.map(|v| v as f64 * prob)
                )?;
                topic_model_values.add_assign(&value);
                words_col.push(value);
            }
            if matches!(fit_to_topic, Some(FitTo::Topic)) {

            }
            overall_topic_model.add_assign(&topic_model_values);
            topic_model_ct.push(topic_model_values);
            word_per_topic.push(words_col);
        }

        Ok(
            Self {
                overall_topic_model: Arc::new(overall_topic_model),
                topic_model: Arc::new(topic_model_ct),
                word_per_topic: Arc::new(word_per_topic),
            }
        )
    }
}

// impl DictionaryMetaProbabilityProvider for ByCountWeigthed {
//     fn whole_topic_model(&self) -> &SparseMetaVector {
//         &self.overall_topic_model
//     }
//
//     fn for_topic(&self, _: usize) -> Option<&SparseMetaVector> {
//         Some(&self.overall_topic_model)
//     }
//
//     fn for_word_in_topic(&self, _: usize, word_id: WordId) -> Option<&SparseMetaVector> {
//         self.word_count.get(word_id)
//     }
// }