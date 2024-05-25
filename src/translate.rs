use std::num::NonZeroUsize;
use std::sync::Arc;
use evalexpr::{context_map, EvalexprError};
use itertools::{Itertools};
use rayon::prelude::*;
use thiserror::Error;
use crate::topicmodel::topic_model::{TopicModel};
use crate::topicmodel::dictionary::Dictionary;

#[derive(Debug)]
struct TranslateConfig {
    epsilon: Option<f64>,
    voting: String,
    voting_limit: Option<NonZeroUsize>,
    threshold: Option<f64>
}

#[derive(Debug, Error)]
pub enum TranslateError<T> {
    #[error("The dictionary is not compatible with the topic model.")]
    InvalidDictionary(TopicModel<T>, Dictionary<T>),
    #[error(transparent)]
    EvalExpressionError(#[from] EvalexprError),
}



fn translate_impl<T>(
    topic_model: TopicModel<T>,
    dictionary: Dictionary<T>,
    translate_config: TranslateConfig
) -> Result<TopicModel<T>, TranslateError<T>> {
    if topic_model.vocabulary().len() != dictionary.voc_a().len() {
        return Err(TranslateError::InvalidDictionary(topic_model, dictionary));
    }
    let epsilon = if let Some(value) = translate_config.epsilon {
        value
    } else {
        topic_model.topics().iter().flatten().fold(
            f64::MAX,
            |old, other| {
                old.min(*other)
            }
        ) - f64::EPSILON
    };



    let base_context = Arc::new(
        context_map! {
            "epsilon" => epsilon,
            "n_voc" => dictionary.voc_a().len() as i64,
            "n_voc_target" => dictionary.voc_b().len() as i64,
        }?
    );

    let precompiled = Arc::new(evalexpr::build_operator_tree(&translate_config.voting)?);

    let dictionary = Arc::new(dictionary);

    let result = topic_model.topics().iter()
        .zip_eq(topic_model.stats())
        .enumerate()
        .collect_vec()
        .par_iter()
        .copied()
        .map(|(topic_id, (topic, stats))| {
            let mut topic_context = context_map! {
                "topic_max" => stats.max_value,
                "topic_min" => stats.min_value,
                "topic_avg" => stats.average_value,
                "topic_sum" => stats.sum_value,
            }.unwrap();


    }).collect::<Vec<_>>();

    todo!()
}


fn translate_topic<T>(
    topic_model: Arc<TopicModel<T>>,
    dictionary: Arc<Dictionary<T>>,
    topic_id: usize,
    topic: &Vec<f64>
) -> Result<(), TranslateError<T>>{
    // topic
    //     .iter()
    //     .enumerate()
    //     .collect_vec()
    //     .par_iter()
    //     .cloned()
    //     .map(|(word_id_a, probability)| {
    //         if let Some(voc_b) = dictionary.translate_id_to_ids::<AToB>(word_id_a) {
    //             for word_id_b in voc_b {
    //                 dictionary.translate_id_to_ids::<BToA>(*word_id_b)
    //                     .iter()
    //                     .map(|word_id_a_retrans| {
    //                         topic_model.rank_and_probability(topic_id, word_id_a_retrans)
    //                     })
    //             }
    //             voc_b.iter().map(|word_id_b| )
    //         } else {
    //             None
    //         }
    //     })
    //
    // for (word_id, probability) in topic.iter().enumerate() {
    //     let mut word_bound_context = context.clone();
    //     word_bound_context.set_value_direct("hasTranslation", dictionary.can_translate_id(word_id))?;
    //
    // }




    todo!()
}



#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let a = evalexpr::context_map! {
            "epsilon" => 0.7
        }.unwrap();

        let mut b = evalexpr::context_map! {
            "katze" => 1
        }.unwrap();

        println!("{b:?}")
    }
}

