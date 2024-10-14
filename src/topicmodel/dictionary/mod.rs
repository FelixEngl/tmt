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

#![allow(dead_code)]

pub mod metadata;
pub mod direction;
pub mod iterators;
mod traits;
mod dictionary;

pub use traits::*;

pub use dictionary::*;

pub use metadata::dictionary::*;

#[macro_export]
macro_rules! dict_insert {
    ($d: ident, $a: tt : $b: tt) => {
        $crate::topicmodel::dictionary::DictionaryMut::insert_value::<$crate::topicmodel::dictionary::direction::Invariant>(&mut $d, $a, $b)
    };
    ($d: ident, $a: tt :: $b: tt) => {
        dict_insert!($d, $a : $b);
    };
    ($d: ident, $a: tt :=: $b: tt) => {
        dict_insert!($d, $a : $b);
    };
    ($d: ident, $a: tt :>: $b: tt) => {
        $crate::topicmodel::dictionary::DictionaryMut::insert_value::<$crate::topicmodel::dictionary::direction::AToB>(&mut $d, $a, $b)
    };
    ($d: ident, $a: tt :<: $b: tt) => {
        $crate::topicmodel::dictionary::DictionaryMut::insert_value::<$crate::topicmodel::dictionary::direction::BToA>(&mut $d, $a, $b)
    };
}

#[macro_export]
macro_rules! dict {
    () => {$crate::topicmodel::Dictionary::<_, $crate::topicmodel::vocabulary::Vocabulary<_>>::new()};
    (for $lang_a: tt, $lang_b: tt;) => {$crate::topicmodel::dictionary::Dictionary::<_, $crate::topicmodel::vocabulary::Vocabulary<_>>::new_with(Some($lang_a), Some($lang_b))};

    (for $lang_a: tt, $lang_b: tt: $($a:tt $op:tt $b:tt)+) => {
        {
            let mut __dict = $crate::topicmodel::dictionary::Dictionary::<_, $crate::topicmodel::vocabulary::Vocabulary<_>>::new_with(Some($lang_a), Some($lang_b));
            $(
                $crate::dict_insert!(__dict, $a $op $b);
            )+
            __dict
        }
    };

    ($($a:tt $op:tt $b:tt)+) => {
        {
            let mut __dict = $crate::topicmodel::dictionary::Dictionary::<_, $crate::topicmodel::vocabulary::Vocabulary<_>>::new();
            $(
                $crate::dict_insert!(__dict, $a $op $b);
            )+
            __dict
        }
    }
}



#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, DictionaryMut, DictionaryWithMeta, DictionaryWithVocabulary, FromVoc};
    use crate::topicmodel::dictionary::direction::{A, B, DirectionTuple, Invariant};
    use crate::topicmodel::dictionary::metadata::SolvedMetadata;
    use crate::topicmodel::vocabulary::{SearchableVocabulary, Vocabulary};

    #[test]
    fn can_create_with_meta(){
        let mut voc_a = Vocabulary::<String>::default();
        voc_a.extend(vec![
            "plane".to_string(),
            "aircraft".to_string(),
            "airplane".to_string(),
            "flyer".to_string(),
            "airman".to_string(),
            "airfoil".to_string(),
            "wing".to_string(),
            "deck".to_string(),
            "hydrofoil".to_string(),
            "foil".to_string(),
            "bearing surface".to_string()
        ]);
        let mut voc_b = Vocabulary::<String>::default();
        voc_b.extend(vec![
            "Flugzeug".to_string(),
            "Flieger".to_string(),
            "Tragfläche".to_string(),
            "Ebene".to_string(),
            "Planum".to_string(),
            "Platane".to_string(),
            "Maschine".to_string(),
            "Bremsberg".to_string(),
            "Berg".to_string(),
            "Fläche".to_string(),
            "Luftfahrzeug".to_string(),
            "Fluggerät".to_string(),
            "Flugsystem".to_string(),
            "Motorflugzeug".to_string(),
        ]);

        let mut dict = DictionaryWithMeta::from_voc(voc_a.clone(), voc_b.clone());
        {
            dict.metadata_with_dict_mut().reserve_meta();
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Ebene").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Planum").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Platane").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Maschine").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Bremsberg").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Berg").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Fläche").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Luftfahrzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Fluggerät").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flugsystem").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Motorflugzeug").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("flyer").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airman").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airfoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictE");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictC");
            dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("wing").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("deck").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("hydrofoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictC");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictC");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("foil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictB");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");
            meta_b.push_associated_dictionary("DictB");
            let DirectionTuple{ a, b, direction:_ } = dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("bearing surface").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(a);
            meta_a.push_associated_dictionary("DictA");
            drop(meta_a);
            let mut meta_b = dict.metadata.get_or_init_meta::<B>(b);
            meta_b.push_associated_dictionary("DictA");

            drop(meta_b);
            let mut meta_a = dict.metadata.get_or_init_meta::<A>(0);
            meta_a.push_associated_dictionary("DictA");
            meta_a.push_associated_dictionary("DictB");
        }

        println!("{}", dict);

        let result = dict.create_subset_with_filters(
            |_, _, meaning| {
                if let Some(found) = meaning {
                    found.has_associated_dictionary("DictA") || found.has_associated_dictionary("DictB")
                } else {
                    false
                }
            },
            |_,_,_| { true }
        );
        println!(".=======.");
        println!("{}", result);
        println!("--==========--");

        for value in dict.iter_with_meta() {
            println!("{}", value.map(
                |(id, meta)| {
                    format!("'{}({id})': {}", dict.id_to_word::<A>(id).unwrap().to_string(), meta.map_or("NONE".to_string(), |value| SolvedMetadata::from(value).to_string()))
                },
                |(id, meta)| {
                    format!("'{}({id})': {}", dict.id_to_word::<B>(id).unwrap().to_string(), meta.map_or("NONE".to_string(), |value| SolvedMetadata::from(value).to_string()))
                }
            ))
        }
        println!("--==========--");
        for value in dict.into_iter() {
            println!("'{}({})': {}, '{}({})': {}",
                     value.a.1, value.a.0, value.a.clone().2.map_or("NONE".to_string(), |value| value.to_string()),
                     value.b.1, value.b.0, value.b.clone().2.map_or("NONE".to_string(), |value| value.to_string())
            )
        }
    }
}
