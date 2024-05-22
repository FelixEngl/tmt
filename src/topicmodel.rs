use std::hash::Hash;
use crate::topicmodel::dictionary::Dictionary;
use crate::topicmodel::topic_model::TopicModel;

pub mod topic_model;
pub mod vocabulary;
pub mod dictionary;
pub mod traits;
pub mod enums;
mod io;


pub fn create_topic_model_specific_dictionary<T: Eq + Hash + Clone>(topic_model: &TopicModel<T>, dictionary: &Dictionary<T>) -> Dictionary<T> {
    topic_model.vocabulary().iter()
    todo!()
}