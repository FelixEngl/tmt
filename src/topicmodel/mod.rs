//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::hash::Hash;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use crate::topicmodel::dictionary::{DictionaryMut, FromVoc};
use crate::topicmodel::dictionary::direction::*;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{MappableVocabulary, VocabularyMut};

pub mod model;
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
    use crate::topicmodel::create_topic_model_specific_dictionary;
    use crate::topicmodel::dictionary::{BasicDictionaryWithVocabulary, Dictionary, DictionaryFilterable};
    use crate::topicmodel::reference::HashRef;
    use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary, Vocabulary};

    #[test]
    fn can_transfer(){

        let (voc_a, _, dict) = crate::translate::test::create_test_data();

        for (a, b) in voc_a.iter().zip(dict.voc_a().iter()) {
            assert_eq!(a.clone(), b.clone())
        }

        let voc = voc_a.filter_by_value(
            |a| {
                a.eq(&HashRef::new("plane".to_string())) || a.eq(&HashRef::new("aircraft".to_string()))
            }
        );
        println!("{dict}\n------\n");
        let dict = dict.filter_by_values(
            |_| true,
            |b| !b.eq(&HashRef::new("Ebene".to_string()))
        );

        let d: Dictionary<_, Vocabulary<_>> = create_topic_model_specific_dictionary(
            &dict,
            &voc
        );

        println!("{voc}\n------\n");
        println!("{dict}\n------\n");
        println!("{d}");

        for (a, b) in voc.iter().zip(d.voc_a().iter()) {
            assert_eq!(a.clone(), b.clone())
        }

        // println!("{}", dict);
    }
}
