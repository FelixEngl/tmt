use std::hash::Hash;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use crate::topicmodel::dictionary::{DictionaryMut, FromVoc};
use crate::topicmodel::dictionary::direction::*;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{MappableVocabulary, VocabularyMut};

pub mod topic_model;
pub mod vocabulary;
pub mod dictionary;
pub mod traits;
pub mod enums;
mod io;
pub mod reference;
mod math;
pub mod language_hint;


pub fn create_topic_model_specific_dictionary<D2, D1, T, V1, V2>(
    dictionary: &D1,
    vocabulary: &V1
) -> D2
    where
        V1: VocabularyMut<T> + MappableVocabulary<T> + Clone,
        V2: VocabularyMut<T>,
        T: Eq + Hash + Clone,
        D1: DictionaryMut<T, V1>,
        D2: DictionaryMut<T, V2> + FromVoc<T, V2>
{
    let mut new_dict: D2 = D2::from_voc_lang::<A>(
        vocabulary.clone().map(|value| value.clone()),
        dictionary.language::<B>().cloned()
    );

    let translations: Vec<(HashRef<T>, Option<Vec<&HashRef<T>>>)> = {
        new_dict.voc_a().as_ref().par_iter().map(|value| {
            (value.clone(), dictionary.translate_value_to_values::<AToB, _>(value))
        }).collect::<Vec<_>>()
    };

    fn insert_into<L: Language, T: Eq + Hash, V: VocabularyMut<T>>(dict: &mut impl DictionaryMut<T, V>, translations: &Vec<(HashRef<T>, Option<Vec<&HashRef<T>>>)>) {
        for (t, other) in translations.iter() {
            if let Some(other) = other {
                for o in other {
                    if L::LANG.is_a() {
                        dict.insert_hash_ref::<L>(t.clone(), (*o).clone());
                    } else {
                        dict.insert_hash_ref::<L>((*o).clone(), t.clone());
                    }
                }
            }
        }
    }



    insert_into::<A, _, _>(&mut new_dict, &translations);

    let retranslations = new_dict.voc_b().as_ref().par_iter().map(|value| {
        (value.clone(), dictionary.translate_value_to_values::<BToA, _>(value))
    }).collect::<Vec<_>>();

    insert_into::<B, _, _>(&mut new_dict, &retranslations);

    return new_dict;
}

#[cfg(test)]
mod test {
    use crate::topicmodel::{create_topic_model_specific_dictionary};
    use crate::{dict, voc};
    use crate::topicmodel::dictionary::Dictionary;
    use crate::topicmodel::vocabulary::Vocabulary;

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


        let dict = create_topic_model_specific_dictionary::<Dictionary<_, Vocabulary<_>>, _,_,_,_>(
            &dictionary,
            &voc_a,
        );

        println!("{:?}", dictionary);
        println!("{:?}", dict);

        // println!("{}", dict);
    }
}
