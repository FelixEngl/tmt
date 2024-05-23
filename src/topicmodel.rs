use std::hash::Hash;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use crate::topicmodel::dictionary::Dictionary;
use crate::topicmodel::dictionary::direction::*;
use crate::topicmodel::topic_model::TopicModel;
use crate::topicmodel::vocabulary::Vocabulary;

pub mod topic_model;
pub mod vocabulary;
pub mod dictionary;
pub mod traits;
pub mod enums;
mod io;


pub fn create_topic_model_specific_dictionary<T: Eq + Hash + Clone>(vocabulary: &Vocabulary<T>, dictionary: &Dictionary<T>) -> Dictionary<T> {
    let mut new_vocabulary = Dictionary::from_voc_a(vocabulary.clone());
    let translations = {
        new_vocabulary.voc_a().as_ref().par_iter().map(|value| {
            (value.clone(), dictionary.translate_word_to_hash_refs::<AToB, _>(value))
        }).collect::<Vec<_>>()
    };

    for (t, other) in translations.iter() {
        if let Some(other) = other {
            for o in other {
                new_vocabulary.insert_hash_ref::<AToB>((*t).clone(), (*o).clone());
            }
        }
    }

    let retranslations = new_vocabulary.voc_b().as_ref().par_iter().map(|value| {
        (value.clone(), dictionary.translate_word_to_hash_refs::<BToA, _>(value))
    }).collect::<Vec<_>>();

    for (t, other) in retranslations {
        if let Some(other) = other {
            for o in other {
                new_vocabulary.insert_hash_ref::<BToA>(t.clone(), (*o).clone());
            }
        }
    }

    return new_vocabulary;
}

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::Dictionary;
    use crate::topicmodel::dictionary::direction::Invariant;
    use crate::topicmodel::vocabulary::Vocabulary;

    #[test]
    fn can_transfer(){
        let mut voc_a = Vocabulary::new();
        voc_a.add("hallo");
        voc_a.add("welt");
        voc_a.add("katze");

        let mut dictionary = Dictionary::new();

        dictionary.insert::<Invariant>("hallo", "hello");
        dictionary.insert::<Invariant>("hallo", "hi");
        dictionary.insert::<Invariant>("welt", "world");
        dictionary.insert::<Invariant>("erde", "world");
        dictionary.insert::<Invariant>("katze", "cat");
        dictionary.insert::<Invariant>("kadse", "cat");


    }
}