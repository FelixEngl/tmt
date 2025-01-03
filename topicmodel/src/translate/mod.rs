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

mod standard;
mod traits;

pub use traits::*;


use std::hash::Hash;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use crate::dictionary::{DictionaryMut, DictionaryMutGen, DictionaryWithVocabulary, FromVoc};
use crate::dictionary::direction::*;
use crate::vocabulary::{MappableVocabulary, SearchableVocabulary, VocabularyMut};

pub fn create_topic_vocabulary_specific_dictionary<T, Voc, D1, V1, D2, V2>(
    dictionary: &D1,
    vocabulary: &Voc
) -> D2
where
    T: Eq + Hash + Clone + Send + Sync,
    Voc: MappableVocabulary<T> + Clone,
    V1: SearchableVocabulary<T>,
    D1: DictionaryWithVocabulary<T, V1>,
    V2: VocabularyMut<T>,
    D2: DictionaryMut<T, V2> + FromVoc<T, V2>
{
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

    let mut new_dict: D2 = D2::from_voc_lang_a(
        vocabulary.clone().map(|value| value.clone()),
        dictionary.language_b().cloned()
    );

    new_dict.set_language_a(dictionary.language_a().cloned());
    new_dict.set_language_b(dictionary.language_b().cloned());

    let translations = new_dict.voc_a().as_ref().par_iter()
        .map(|value| (value.clone(), dictionary.translate_word_a_to_words_b(value)))
        .collect::<Vec<_>>();

    insert_into::<A, _, _>(&mut new_dict, &translations);

    let retranslations = new_dict.voc_b().as_ref().par_iter()
        .map(|value| (value.clone(), dictionary.translate_word_b_to_words_a(value)))
        .collect::<Vec<_>>();

    insert_into::<B, _, _>(&mut new_dict, &retranslations);

    new_dict
}


#[cfg(test)]
mod test {
    use log::LevelFilter::Trace;
    use crate::dictionary::{Dictionary, DictionaryMutGen};
    use crate::dictionary::direction::Invariant;
    use crate::translate::create_topic_vocabulary_specific_dictionary;
    use crate::voc;
    use crate::vocabulary::{SearchableVocabulary, Vocabulary};

    fn create_test_data() -> (Vocabulary<String>, Vocabulary<String>, Dictionary<String, Vocabulary<String>>){
        let voc_a: Vocabulary<String> = voc![
            "plane",
            "aircraft",
            "airplane",
            "flyer",
            "airman",
            "airfoil",
            "wing",
            "deck",
            "hydrofoil",
            "foil",
            "bearing surface"
        ];
        let voc_b: Vocabulary<String> = voc![
            "Flugzeug",
            "Flieger",
            "Tragfläche",
            "Ebene",
            "Planum",
            "Platane",
            "Maschine",
            "Bremsberg",
            "Berg",
            "Fläche",
            "Luftfahrzeug",
            "Fluggerät",
            "Flugsystem",
            "Motorflugzeug",
        ];

        let mut dict = Dictionary::default();
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Flugzeug").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Flieger").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Tragfläche").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Ebene").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Planum").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Platane").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Maschine").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Bremsberg").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Berg").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Fläche").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("plane").unwrap(), voc_b.get_value("Flieger").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("aircraft").unwrap(), voc_b.get_value("Flugzeug").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("aircraft").unwrap(), voc_b.get_value("Flieger").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("aircraft").unwrap(), voc_b.get_value("Luftfahrzeug").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("aircraft").unwrap(), voc_b.get_value("Fluggerät").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("aircraft").unwrap(), voc_b.get_value("Flugsystem").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("airplane").unwrap(), voc_b.get_value("Flugzeug").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("airplane").unwrap(), voc_b.get_value("Flieger").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("airplane").unwrap(), voc_b.get_value("Motorflugzeug").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("flyer").unwrap(), voc_b.get_value("Flieger").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("airman").unwrap(), voc_b.get_value("Flieger").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("airfoil").unwrap(), voc_b.get_value("Tragfläche").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("wing").unwrap(), voc_b.get_value("Tragfläche").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("deck").unwrap(), voc_b.get_value("Tragfläche").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("hydrofoil").unwrap(), voc_b.get_value("Tragfläche").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("foil").unwrap(), voc_b.get_value("Tragfläche").unwrap());
        dict.insert::<Invariant>(voc_a.get_value("bearing surface").unwrap(), voc_b.get_value("Tragfläche").unwrap());

        (voc_a, voc_b, dict)
    }

    #[test]
    fn does_propery_generate_the_data(){
        env_logger::builder().is_test(true).filter_level(Trace).init();

        let voc: Vocabulary<String> = voc![
            "plane",
            "flyer",
            "wing"
        ];
        let (_, _, dict) = create_test_data();
        log::info!("{}", dict);

        let _: Dictionary<_> = create_topic_vocabulary_specific_dictionary(
            &dict,
            &voc
        );
    }
}
