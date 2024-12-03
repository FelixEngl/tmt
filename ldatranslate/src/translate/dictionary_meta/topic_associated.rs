use std::sync::Arc;
use itertools::Itertools;
use ndarray::{ArcArray, Array1, Ix1};
use rayon::prelude::*;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DictionaryMetaIndex, META_DICT_ARRAY_LENTH};
use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataEx;
use ldatranslate_translate::{TopicLike, TopicModelLikeMatrix};
use crate::translate::dictionary_meta::dict_meta::MetaTagTemplate;
use crate::translate::dictionary_meta::{SparseVectorFactory};
use crate::translate::entropies::{EntropyWithAlphaError, FDivergenceCalculator};

pub struct VerticalCountDictionaryMetaVector {
    topic_model: [Option<ArcArray<f64, Ix1>>; META_DICT_ARRAY_LENTH],
    template: MetaTagTemplate,
    calculator: Arc<FDivergenceCalculator>
}

impl VerticalCountDictionaryMetaVector {
    pub fn new<T: TopicModelLikeMatrix>(meta: Vec<&MetadataEx>, factory: &SparseVectorFactory, calculator: Arc<FDivergenceCalculator>, matrix: &T) -> Result<Self, EntropyWithAlphaError<f64, f64>>
    {
        let template = if let Some(targ_fields) = calculator.target_fields.as_ref() {
            if calculator.invert_target_fields {
                let value = DictMetaTagIndex::all().into_iter().copied().filter(|v| !targ_fields.contains(v)).collect_vec();
                factory.convert_to_template(&value)
            } else {
                factory.convert_to_template(targ_fields)
            }
        } else {
            factory.convert_to_template(DictMetaTagIndex::all())
        };

        let value: Vec<_> = template.iter().par_bridge().map(|target| {
            (*target, meta.iter().map(|value| value.get_domain_count_for(*target) as f64).collect::<Array1<f64>>())
        }).collect();

        let values = value.into_iter().map(|(k, v)| {
            let sum = v.sum();
            (k, v/sum)
        }).collect::<Vec<_>>();

        let mut topic_calcs = [const { None }; META_DICT_ARRAY_LENTH];

        for (k, v) in values {
            topic_calcs[k.as_index()] = Some(ArcArray::from(v));
        }

        let result = matrix.par_iter().map(|topic| {
            Self::calculate_for_topic_model(
                topic,
                &calculator,
                &topic_calcs,
            ).map(|topic_assoc| {

            })

        }).collect::<Result<Vec<_>, _>>()?;


        Self {
            template,
            topic_model: topic_calcs,
            calculator
        }
    }

    pub fn calculate_for_topic_model<T: TopicLike>(topic: &T, calculator: &FDivergenceCalculator, topic_model_meta: &[Option<ArcArray<f64, Ix1>>; META_DICT_ARRAY_LENTH]) -> Result<[f64; META_DICT_ARRAY_LENTH], EntropyWithAlphaError<f64, f64>> {
        let p = Array1::from(topic.iter().copied().collect::<Vec<_>>());
        let mut result = [0.0; META_DICT_ARRAY_LENTH];
        for (idx, value) in topic_model_meta.iter().enumerate() {
            if let Some(q) = value {
                result[idx] = calculator.calculate(&p, q)?;
            }
        }
        Ok(result)
    }
}