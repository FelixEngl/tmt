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
use crate::dictionary::{DictionaryMut, DictionaryMutGen, FromVoc};
use crate::dictionary::direction::*;
use crate::vocabulary::{MappableVocabulary, VocabularyMut};

pub mod model;
pub mod vocabulary;
pub mod dictionary;
pub mod traits;
pub mod enums;
mod io;
mod math;
pub mod language_hint;
pub mod interners;

pub fn create_topic_model_specific_dictionary<D2, D1, T, V1, V2>(
    dictionary: &D1,
    vocabulary: &V1
) -> D2
where
    V1: VocabularyMut<T> + MappableVocabulary<T> + Clone,
    V2: VocabularyMut<T>,
    T: Eq + Hash + Clone + Send + Sync,
    D1: DictionaryMut<T, V1>,
    D2: DictionaryMut<T, V2> + FromVoc<T, V2>
{
    let mut new_dict: D2 = D2::from_voc_lang_a(
        vocabulary.clone().map(|value| value.clone()),
        dictionary.language_b().cloned()
    );

    let translations: Vec<(T, Option<Vec<&T>>)> = {
        new_dict.voc_a().as_ref().par_iter().map(|value| {
            (value.clone(), dictionary.translate_word_a_to_words_b(value))
        }).collect::<Vec<_>>()
    };

    fn insert_into<L: Language, T: Eq + Hash + Clone, V: VocabularyMut<T>>(dict: &mut impl DictionaryMut<T, V>, translations: &Vec<(T, Option<Vec<&T>>)>) {
        for (t, other) in translations.iter() {
            if let Some(other) = other {
                for o in other {
                    if L::LANG.is_a() {
                        dict.insert_value::<L>(t.clone(), (*o).clone());
                    } else {
                        dict.insert_value::<L>((*o).clone(), t.clone());
                    }
                }
            }
        }
    }



    insert_into::<A, _, _>(&mut new_dict, &translations);

    let retranslations = new_dict.voc_b().as_ref().par_iter().map(|value| {
        (value.clone(), dictionary.translate_word_b_to_words_a(value))
    }).collect::<Vec<_>>();

    insert_into::<B, _, _>(&mut new_dict, &retranslations);

    new_dict
}

