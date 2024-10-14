use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::toolkit::once_lock_serializer::OnceLockDef;
use crate::topicmodel::dictionary::metadata::typesafe_interner::{DefaultDictionaryOrigin, DefaultTag};


/// The container for the metadata
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq)]
pub struct Metadata {
    #[serde(with = "OnceLockDef")]
    pub associated_dictionaries: OnceLock<HashSet<DefaultDictionaryOrigin>>,
    #[serde(with = "OnceLockDef")]
    #[serde(alias = "meta_tags")]
    pub subjects: OnceLock<HashSet<DefaultTag>>,
    #[serde(with = "OnceLockDef")]
    pub unstemmed: OnceLock<HashMap<usize, HashSet<DefaultDictionaryOrigin>>>,
}

macro_rules! create_methods {
    ($($self: ident.$target:ident($_type: ty) || $single_target: ident),+) => {
        $(
           paste::paste! {
            pub fn [<has_ $single_target>](&$self, q: $_type) -> bool {
                $self.$target.get().is_some_and(|value| value.contains(&q))
            }

            pub unsafe fn [<add_ $single_target>](&mut self, q: $_type) {
                if let Some(to_edit) = self.$target.get_mut() {
                    if to_edit.is_empty() || !to_edit.contains(&q) {
                        to_edit.insert(q);
                    }
                } else {
                    let mut new = HashSet::with_capacity(1);
                    new.insert(q);
                    self.$target.set(new).expect("This should be unset!");
                }
            }

            pub unsafe fn [<add_all_ $target>](&mut self, q: &HashSet<$_type>) {
                if let Some(to_edit) = self.$target.get_mut() {
                    to_edit.extend(q);
                } else {
                    self.$target.set(q.clone()).expect("This should be unset!");
                }
            }
        }
        )+

    };
}

impl Metadata {
    create_methods! {
        self.associated_dictionaries(DefaultDictionaryOrigin) || associated_dictionary,
        self.subjects(DefaultTag) || subject
    }


    pub fn add_all_unstemmed(&mut self, unstemmed_words: &HashMap<usize, HashSet<DefaultDictionaryOrigin>>) {
        if let Some(found) = self.unstemmed.get_mut() {
            for (word, v) in unstemmed_words.iter() {
                match found.entry(*word) {
                    Entry::Vacant(value) => {
                        value.insert(v.clone());
                    }
                    Entry::Occupied(mut value) => {
                        value.get_mut().extend(v);
                    }
                }
            }
        } else {
            self.unstemmed.set(unstemmed_words.clone()).unwrap()
        }
    }

    pub fn has_unstemmed(&self, unstemmed_word: usize) -> bool {
        self.unstemmed
            .get()
            .is_some_and(|value| value.contains_key(&unstemmed_word))
    }

    pub fn add_unstemmed(&mut self, unstemmed_word: usize) {
        if let Some(found) = self.unstemmed.get_mut() {
            match found.entry(unstemmed_word) {
                Entry::Vacant(value) => {
                    value.insert(HashSet::with_capacity(0));
                }
                _ => {}
            }
        } else {
            let mut new = HashMap::with_capacity(1);
            new.insert(unstemmed_word, HashSet::<_>::with_capacity(0));
            self.unstemmed.set(new).unwrap();
        }
    }


    pub fn add_all_unstemmed_words(&mut self, unstemmed_words: &[usize]) {
        if let Some(found) = self.unstemmed.get_mut() {
            for word in unstemmed_words {
                match found.entry(*word) {
                    Entry::Vacant(value) => {
                        value.insert(HashSet::with_capacity(0));
                    }
                    _ => {}
                }
            }

        } else {
            let mut new = HashMap::with_capacity(unstemmed_words.len());
            for word in unstemmed_words {
                new.insert(*word, HashSet::with_capacity(0));
            }
            self.unstemmed.set(new).unwrap();
        }
    }

    pub fn has_unstemmed_origin(&self, unstemmed_word: usize, origin: DefaultDictionaryOrigin) -> bool {
        self.unstemmed
            .get()
            .is_some_and(|value|
                value.get(&unstemmed_word).is_some_and(|value| value.contains(&origin))
            )
    }

    pub unsafe fn add_unstemmed_origin(&mut self, unstemmed_word: usize, origin: DefaultDictionaryOrigin) {
        if let Some(found) = self.unstemmed.get_mut() {
            match found.entry(unstemmed_word) {
                Entry::Vacant(value) => {
                    let mut new = HashSet::with_capacity(1);
                    new.insert(origin);
                    value.insert(new);
                }
                Entry::Occupied(mut value) => {
                    value.get_mut().insert(origin);
                }
            }
        } else {
            let mut new = HashMap::with_capacity(1);
            let mut set = HashSet::with_capacity(1);
            set.insert(origin);
            new.insert(unstemmed_word, set);
            self.unstemmed.set(new).unwrap();
        }
    }

    pub unsafe fn add_all_unstemmed_origins(&mut self, unstemmed_word: usize, origins: &[DefaultDictionaryOrigin]) {
        if let Some(found) = self.unstemmed.get_mut() {
            match found.entry(unstemmed_word) {
                Entry::Vacant(value) => {
                    value.insert(origins.into_iter().copied().collect());
                }
                Entry::Occupied(mut value) => {
                    value.get_mut().extend(origins);
                }
            }
        } else {
            let mut new = HashMap::with_capacity(1);
            new.insert(unstemmed_word, origins.into_iter().copied().collect());
            self.unstemmed.set(new).unwrap();
        }
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        if let Some(associated_dictionaries) = self.associated_dictionaries.get() {
            if let Some(other_associated_dictionaries) = other.associated_dictionaries.get() {
                if associated_dictionaries != other_associated_dictionaries {
                    return false;
                }
            } else {
                return false;
            }
        } else if other.associated_dictionaries.get().is_some() {
            return false;
        }

        if let Some(subjectsgs) = self.subjects.get() {
            if let Some(other_subjects) = other.subjects.get() {
                if subjectsgs != other_subjects {
                    return false;
                }
            } else {
                return false;
            }
        } else if other.subjects.get().is_some() {
            return false;
        }

        if let Some(unstemmed) = self.unstemmed.get() {
            if let Some(other_unstemmed) = other.unstemmed.get() {
                if unstemmed != other_unstemmed {
                    return false;
                }
            } else {
                return false;
            }
        } else if other.unstemmed.get().is_some() {
            return false;
        }

        return true;
    }
}