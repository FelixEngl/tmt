use std::hash::Hash;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use crate::topicmodel::dictionary::Dictionary;
use crate::topicmodel::dictionary::direction::*;
use crate::topicmodel::vocabulary::Vocabulary;

pub mod topic_model;
pub mod vocabulary;
pub mod dictionary;
pub mod traits;
pub mod enums;
mod io;
pub mod reference;


pub fn create_topic_model_specific_dictionary<T: Eq + Hash + Clone>(vocabulary: &Vocabulary<T>, dictionary: &Dictionary<T>) -> Dictionary<T> {
    let mut new_vocabulary = Dictionary::from_voc_a(vocabulary.clone());
    let translations = {
        new_vocabulary.voc_a().as_ref().par_iter().map(|value| {
            (value.clone(), dictionary.translate_value_to_values::<AToB, _>(value))
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
        (value.clone(), dictionary.translate_value_to_values::<BToA, _>(value))
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
        voc_a.add_value("hallo");
        voc_a.add_value("welt");
        voc_a.add_value("katze");

        let mut dictionary = Dictionary::new();

        dictionary.insert_value::<Invariant>("hallo", "hello");
        dictionary.insert_value::<Invariant>("hallo", "hi");
        dictionary.insert_value::<Invariant>("welt", "world");
        dictionary.insert_value::<Invariant>("erde", "world");
        dictionary.insert_value::<Invariant>("katze", "cat");
        dictionary.insert_value::<Invariant>("kadse", "cat");


    }
}