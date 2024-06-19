use std::hash::Hash;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use crate::topicmodel::dictionary::{BasicDictionaryWithVocabulary, Dictionary, DictionaryMut};
use crate::topicmodel::dictionary::direction::*;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{MappableVocabulary, Vocabulary, VocabularyImpl, VocabularyMut};

pub mod topic_model;
pub mod vocabulary;
pub mod dictionary;
pub mod traits;
pub mod enums;
mod io;
pub mod reference;
mod math;
pub mod language_hint;

pub fn create_topic_model_specific_dictionary<T: Eq + Hash + Clone, V>(
    dictionary: &impl DictionaryMut<T, V>,
    vocabulary: &(impl Vocabulary<T> + MappableVocabulary<T> + Clone)
) -> Dictionary<T, VocabularyImpl<T>> where V: VocabularyMut<T> {
    let mut new_dict: Dictionary<_, VocabularyImpl<_>> = Dictionary::from_voc_a(vocabulary.clone().map(|value| value.clone()));

    let translations: Vec<(HashRef<T>, Option<Vec<&HashRef<T>>>)> = {
        new_dict.voc_a().as_ref().par_iter().map(|value| {
            (value.clone(), dictionary.translate_value_to_values::<AToB, _>(value))
        }).collect::<Vec<_>>()
    };

    fn insert_into<D: Translation, T: Eq + Hash>(dict: &mut Dictionary<T, VocabularyImpl<T>>, translations: &Vec<(HashRef<T>, Option<Vec<&HashRef<T>>>)>) {
        for (t, other) in translations.iter() {
            if let Some(other) = other {
                for o in other {
                    if D::DIRECTION.is_a_to_b() {
                        dict.insert_hash_ref::<D>(t.clone(), (*o).clone());
                    } else {
                        dict.insert_hash_ref::<D>((*o).clone(), t.clone());
                    }
                }
            }
        }
    }



    insert_into::<AToB, _>(&mut new_dict, &translations);

    let retranslations = new_dict.voc_b().as_ref().par_iter().map(|value| {
        (value.clone(), dictionary.translate_value_to_values::<BToA, _>(value))
    }).collect::<Vec<_>>();

    insert_into::<BToA, _>(&mut new_dict, &retranslations);

    return new_dict;
}

#[cfg(test)]
mod test {
    use crate::topicmodel::{create_topic_model_specific_dictionary};
    use crate::{dict, voc};

    #[test]
    fn can_transfer(){

        let voc_a = voc![
            for "de":
            "hallo", "welt", "bier"
        ];

        let dictionary = dict! {
            for "de", "en":
            "hallo" : "hello"
            "hallo" : "hi"
            "welt" : "world"
            "erde" : "world"
            "katze" : "cat"
            "kadse" : "cat"
        };


        let dict = create_topic_model_specific_dictionary(
            &dictionary,
            &voc_a,
        );

        println!("{:?}", dictionary);
        println!("{:?}", dict);

        // println!("{}", dict);
    }
}
