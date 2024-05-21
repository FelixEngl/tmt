use std::num::NonZeroUsize;
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

}