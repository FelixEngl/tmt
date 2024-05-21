use std::num::NonZeroUsize;
use evalexpr::{ContextWithMutableVariables, Value};
use itertools::Itertools;
use rayon::prelude::*;
use crate::topicmodel::topic_model::TopicModel;
use crate::topicmodel::dictionary::Dictionary;

struct TranslateConfig {
    epsilon: Option<f64>,
    voting: String,
    voting_limit: Option<NonZeroUsize>,
    threshold: Option<f64>
}

pub fn translate<T>(
    topic_model: TopicModel<T>,
    dictionary: Dictionary<T>,
    translate_config: TranslateConfig
) {
    let mut context = evalexpr::HashMapContext::new();

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

    let base_context = evalexpr::context_map! {
        "epsilon" => epsilon
    }.unwrap();


    topic_model.topics().iter().enumerate().collect_vec().par_iter().map(|(topic_id, topic)| {

    });
}