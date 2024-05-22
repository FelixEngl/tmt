use std::hash::Hash;
use rayon::prelude::IntoParallelRefIterator;
use crate::topicmodel::dictionary::Dictionary;
use crate::topicmodel::dictionary::direction::*;
use crate::topicmodel::topic_model::TopicModel;
use crate::topicmodel::vocabulary::HashRef;

pub mod topic_model;
pub mod vocabulary;
pub mod dictionary;
pub mod traits;
pub mod enums;
mod io;


pub fn create_topic_model_specific_dictionary<T: Eq + Hash + Clone>(topic_model: &TopicModel<T>, dictionary: &Dictionary<T>) -> Dictionary<T> {
    let translations: Vec<(_, Option<_>)> = topic_model.vocabulary().par_iter().map(|value| {
        dictionary
            .translate_word_to_hash_refs::<AToB, _>(value)
            .map(|translations| (value, translations))
    }).collect();

    todo!()
}