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

mod traits;
mod anonymous;

pub use traits::*;
pub use anonymous::*;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash};
use std::collections::hash_map::{Entry};
use std::fmt::{Debug, Display, Formatter};
use std::io::{BufWriter, Write};
use std::marker::PhantomData;
use std::ops::{Range};
use std::slice::Iter;
use std::str::FromStr;
use std::vec::IntoIter;
use arcstr::ArcStr;
use itertools::Itertools;
use rayon::prelude::{FromParallelIterator, IntoParallelIterator, ParallelIterator};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, SerializeStruct};
use thiserror::Error;
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::traits::AsParseableString;

pub type EfficientStringVocabulary = Vocabulary<ArcStr>;

#[macro_export]
macro_rules! voc {
    () => {
        $crate::topicmodel::vocabulary::Vocabulary::default()
    };
    (for $lang: tt;) => {
        $crate::topicmodel::vocabulary::Vocabulary:new_forr($lang)
    };
    (for $lang: tt: $($value: tt),+ $(,)?) => {
        {
            let mut __voc = $crate::topicmodel::vocabulary::Vocabulary:new_forr($lang);
            $(
                $crate::topicmodel::vocabulary::VocabularyMut::add_value(&mut __voc, $value.into());
            )+
            __voc
        }
    };
    ($($value: tt),+ $(,)?) => {
        {
            let mut __voc = $crate::topicmodel::vocabulary::Vocabulary::default();
            $(
                $crate::topicmodel::vocabulary::VocabularyMut::add_value(&mut __voc, $value.into());
            )+
            __voc
        }
    };
}


/// A vocabulary mapping between an usize id and a specific object (word)
#[derive(Clone, Debug)]
pub struct Vocabulary<T> {
    language: Option<LanguageHint>,
    id2entry: Vec<T>,
    entry2id: HashMap<T, usize>,
}

impl <T> Vocabulary<T> {

    /// Create a new vocabulary with the default sizes
    pub fn new(language: Option<LanguageHint>, id2entry: Vec<T>, entry2id: HashMap<T, usize>) -> Self {
        Self {
            language,
            id2entry,
            entry2id,
        }
    }

    /// Create a new vocabulary with the default sizes
    pub fn empty_from(language: impl Into<LanguageHint>) -> Self {
        Self::empty(Some(language.into()))
    }

    /// Create a new empty vocabulary.
    pub fn empty(language: Option<LanguageHint>) -> Self {
        Self::new(
            language,
            Default::default(),
            Default::default()
        )
    }

    /// Create a new empty vocabulary but sets the [capacity] of the mappings.
    pub fn with_capacity(language: Option<LanguageHint>, capacity: usize) -> Self {
        Self::new(
            language,
            Vec::with_capacity(capacity),
            HashMap::with_capacity(capacity),
        )
    }
}

unsafe impl<T> Send for Vocabulary<T> {}
unsafe impl<T> Sync for Vocabulary<T> {}

impl <T> BasicVocabulary<T> for Vocabulary<T> {
    fn language(&self) -> Option<&LanguageHint> {
        self.language.as_ref()
    }

    fn set_language(&mut self, new: Option<LanguageHint>) -> Option<LanguageHint> {
        std::mem::replace(&mut self.language, new.map(|value| value.into()))
    }

    /// The number of entries in the vocabulary
    fn len(&self) -> usize {
        self.id2entry.len()
    }

    /// Clear the whole thing
    fn clear(&mut self){
        self.id2entry.clear();
        self.entry2id.clear();
    }

    /// Get the ids
    fn ids(&self) -> Range<usize> {
        0..self.id2entry.len()
    }

    /// Iterate over the words
    fn iter(&self) -> Iter<T> {
        self.id2entry.iter()
    }

    fn iter_entries<'a>(&'a self) -> impl Iterator<Item=(usize, &'a T)> + 'a  where T: 'a {
        self.id2entry.iter().enumerate()
    }


    fn get_entry_by_id(&self, id: usize) -> Option<(usize, &T)> {
        self.get_value_by_id(id).map(|value| (id, value))
    }

    /// Get the HashRef for a specific `id` or none
    fn get_value_by_id(&self, id: usize) -> Option<&T> {
        self.id2entry.get(id)
    }

    unsafe fn get_value_unchecked(&self, id: usize) -> &T {
        self.id2entry.get_unchecked(id)
    }


    /// Check if the `id` is contained in this
    fn contains_id(&self, id: usize) -> bool {
        self.id2entry.len() > id
    }

    fn create(language: Option<LanguageHint>) -> Self where Self: Sized {
        Self::empty(language)
    }


    fn create_from(language: Option<LanguageHint>, voc: Vec<T>) -> Self where Self: Sized, T: Eq + Hash + Clone {
        let id2entry = voc;
        let entry2id = id2entry.iter().cloned().enumerate().map(|(a, b)| (b, a)).collect();
        Self::new(
            language,
            id2entry,
            entry2id,
        )
    }
}

impl<T> Default for Vocabulary<T> {
    fn default() -> Self {
        Self::empty(Default::default())
    }
}

impl<T> AsRef<[T]> for Vocabulary<T> {
    fn as_ref(&self) -> &[T] {
        &self.id2entry
    }
}

impl<T> From<LanguageHint> for Vocabulary<T> {
    fn from(value: LanguageHint) -> Self {
        Self::empty_from(value)
    }
}

impl<T> From<Option<LanguageHint>> for Vocabulary<T> {
    fn from(value: Option<LanguageHint>) -> Self {
        Self::empty(value)
    }
}

impl<T> From<Vec<T>> for Vocabulary<T>
where
    T: Eq + Hash + Clone
{
    fn from(value: Vec<T>) -> Self {
        Self::create_from(None, value)
    }
}

impl<T> From<(Option<LanguageHint>, Vec<T>)> for Vocabulary<T>
where
    T: Eq + Hash + Clone
{
    fn from((hint, value): (Option<LanguageHint>, Vec<T>)) -> Self {
        Self::create_from(hint, value)
    }
}

impl<T> SearchableVocabulary<T> for Vocabulary<T>
where
    T: Eq + Hash + Clone
{

    /// Retrieves the id for `value`
    fn get_id<Q: ?Sized>(&self, value: &Q) -> Option<usize>
    where
        T: Borrow<Q>,
        Q: Hash + Eq
    {
        self.entry2id.get(value).copied()
    }

    /// Retrieves the id for `value`
    fn get_value<Q: ?Sized>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq
    {
        Some(self.get_entry_by_value(value)?.0)
    }

    /// Retrieves the complete entry for `value` in the vocabulary, if it exists
    fn get_entry_by_value<Q: ?Sized>(&self, value: &Q) -> Option<(&T, &usize)>
    where
        T: Borrow<Q>,
        Q: Hash + Eq
    {
        self.entry2id.get_key_value(value)
    }

    fn contains_value<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq {

        self.entry2id.contains_key(value)
    }

    fn filter_by_id<F: Fn(usize) -> bool>(&self, filter: F) -> Self where Self: Sized {
        self.id2entry.iter().enumerate().filter_map(|(id, value)| {
            if filter(id) {
                Some(value.clone())
            } else {
                None
            }
        }).collect()
    }

    fn filter_by_value<'a, F: Fn(&'a T) -> bool>(&'a self, filter: F) -> Self where Self: Sized, T: 'a {
        self.id2entry.iter().filter_map(|value| {
            if filter(value) {
                Some(value.clone())
            } else {
                None
            }
        }).collect()
    }
}

impl<T> VocabularyMut<T> for Vocabulary<T> where T: Eq + Hash + Clone {
    /// Adds the `value` to the vocabulary and returns the associated id
    fn add_value(&mut self, value: T) -> usize {
        let found = self.entry2id.entry(value);
        match found {
            Entry::Occupied(entry) => {
                *entry.get()
            }
            Entry::Vacant(entry) => {
                let pos = self.id2entry.len();
                self.id2entry.push(entry.key().clone());
                entry.insert(pos);
                pos
            }
        }
    }


    /// Adds any `value` that can be converted into `T`
    fn add<V: Into<T>>(&mut self, value: V) -> usize {
        self.add_value(value.into())
    }

    fn add_all_value<I: IntoIterator<Item=T>>(&mut self, other: I) {
        for value in other {
            self.add_value(value);
        }
    }
}

impl<T> MappableVocabulary<T> for Vocabulary<T> where T: Eq + Hash {
    fn map<R, V, F>(self, mapping: F) -> V
    where
        F: Fn(T) -> R,
        V: BasicVocabulary<R>,
        R: Eq + Hash + Clone
    {
        V::create_from(
            self.language,
            self.id2entry.into_iter().map(mapping).collect::<Vec<_>>()
        )
    }
}

impl<T, R> Extend<R> for Vocabulary<T>
where
    T: Eq + Hash + Clone,
    R: Into<T>
{
    fn extend<I: IntoIterator<Item=R>>(&mut self, iter: I) {
        for value in iter {
            self.add(value);
        }
    }
}

impl<T> PartialEq for Vocabulary<T> where T: Eq {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() &&
            self.id2entry.iter()
                .zip_eq(other.id2entry.iter())
                .all(|(a, b)| a.eq(b))
    }
}

impl<T> Eq for Vocabulary<T>  where T: Eq {}

#[derive(Debug, Error)]
pub enum LoadVocabularyError<E: Debug> {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Had a parse error")]
    Parse(E),
}

impl<T, E> LoadableVocabulary<T, E> for  Vocabulary<T>
where
    T: Hash + Eq + Clone + FromStr<Err=E>,
    E: Debug
{
}

impl<T> StoreableVocabulary<T> for Vocabulary<T>
where
    T: AsParseableString
{
    /// Writes the vocabulary to `writer` in the list format
    fn save_to_output(&self, writer: &mut impl Write) -> std::io::Result<usize> {
        let mut written = 0;
        let mut writer = BufWriter::new(writer);
        for value in self.id2entry.iter() {
            let value = value.as_parseable_string();
            written += writer.write(value.as_bytes())?;
            written += writer.write(b"\n")?;
        }
        writer.flush()?;
        Ok(written)
    }
}

impl <T> Display for Vocabulary<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = self
            .id2entry
            .iter()
            .map(|a| a.to_string())
            .join(", ");
        write!(f, "Vocabulary<{}>[{}]", self.language.clone().unwrap_or_default(), x)
    }
}

impl<T> Serialize for Vocabulary<T> where T: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        if serializer.is_human_readable() {
            let mut st = serializer.serialize_struct("Vocabulary", 2)?;
            st.serialize_field(
                "language",
                &self.language
            )?;
            st.serialize_field(
                "id2entry",
                &self.id2entry
            )?;
            st.end()
        } else {
            let mut st = serializer.serialize_seq(Some(2))?;
            st.serialize_element(&self.language)?;
            st.serialize_element(&self.id2entry)?;
            st.end()
        }
    }
}

impl <'de, T> Deserialize<'de> for Vocabulary<T>
where
    T: Deserialize<'de> + Hash + Eq + Clone
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

        struct VocabularyVisitor<'de, T: Deserialize<'de>>{_phantom: PhantomData<(T, &'de())>}

        impl<'de, T: Deserialize<'de>> VocabularyVisitor<'de, T>  {
            fn new() -> Self {
                Self{_phantom: PhantomData}
            }
        }

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Id2Entry, Language }

        impl<'de, T> Visitor<'de> for VocabularyVisitor<'de, T>
        where T: Deserialize<'de> + Hash + Eq + Clone
        {
            type Value = Vocabulary<T>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Vocabulary")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let first = seq.next_element()?.ok_or_else(|| de::Error::missing_field("language"))?;
                let second: Vec<T> = seq.next_element()?.ok_or_else(|| de::Error::missing_field("id2entry"))?;
                Ok(Vocabulary::create_from(
                    first,
                    second
                ))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut id2entry_field = None;
                let mut language_field = None;
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Id2Entry => {
                            if id2entry_field.is_some() {
                                return Err(de::Error::duplicate_field("id2entry"));
                            }
                            id2entry_field = Some(map.next_value::<Vec<T>>()?);
                        }
                        Field::Language => {
                            if language_field.is_some() {
                                return Err(de::Error::duplicate_field("language"));
                            }
                            language_field = map.next_value::<Option<LanguageHint>>()?;
                        }
                    }
                }
                if let Some(field_value) = id2entry_field {
                    Ok(Vocabulary::create_from(language_field, field_value))
                } else {
                    Err(de::Error::missing_field("id2entry"))
                }
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_struct("Vocabulary", &["language", "id2entry"], VocabularyVisitor::<T>::new())
        } else {
            deserializer.deserialize_seq(VocabularyVisitor::<T>::new())
        }
    }
}

impl<T> IntoIterator for Vocabulary<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.id2entry.into_iter()
    }
}

impl<T> IntoParallelIterator for Vocabulary<T>
where
    T:  Sync + Send
{
    type Iter = rayon::vec::IntoIter<T>;
    type Item = T;

    fn into_par_iter(self) -> Self::Iter {
        self.id2entry.into_par_iter()
    }
}

impl<T> FromIterator<T> for Vocabulary<T> where T: Hash + Eq + Clone {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        Self::create_from(None, iter.into_iter().collect())
    }
}

impl<'a, T> FromIterator<&'a T> for Vocabulary<T> where T: Hash + Eq + Clone {
    fn from_iter<I: IntoIterator<Item=&'a T>>(iter: I) -> Self {
        Self::create_from(None, iter.into_iter().cloned().collect())
    }
}

impl<T> FromParallelIterator<T> for Vocabulary<T> where T: Hash + Eq + Clone + Send + Send {
    fn from_par_iter<I>(par_iter: I) -> Self where I: IntoParallelIterator<Item=T> {
        Self::create_from(None, par_iter.into_par_iter().collect())
    }
}

impl<'a, T> FromParallelIterator<&'a T> for Vocabulary<T> where T: Hash + Eq + Clone + Sync + Send {
    fn from_par_iter<I>(par_iter: I) -> Self where I: IntoParallelIterator<Item=&'a T> {
        Self::create_from(None, par_iter.into_par_iter().cloned().collect())
    }
}

impl<T> AnonymousVocabulary for Vocabulary<T>
where
    T: AsRef<str>
{
    fn has_entry_for(&self, word_id: usize) -> bool {
        self.contains_id(word_id)
    }

    fn id_to_entry<'a>(&'a self, word_id: usize) -> Option<&'a str> {
        self.get_value_by_id(word_id).map(|v| v.as_ref())
    }
}

impl<T> AnonymousVocabularyMut for Vocabulary<T>
where
    T: Eq + Hash + Clone + for<'a> From<&'a str>
{
    fn entry_to_id(&mut self, word: &str) -> usize {
        self.add_value(word.into())
    }
}


#[cfg(test)]
mod test {
    use crate::topicmodel::vocabulary::{EfficientStringVocabulary, BasicVocabulary, Vocabulary, VocabularyMut, SearchableVocabulary};

    #[test]
    fn can_insert_and_retrieve() {
        let mut voc = EfficientStringVocabulary::empty_from("MyLang");
        voc.add("Hello World".to_string());
        voc.add("Wasimodo".to_string());

        assert_eq!(2usize, voc.len());
        assert_eq!(Some(0usize), voc.get_id("Hello World"));
        assert_eq!(Some("Hello World"), voc.get_value_by_id(0).map(|x| x.as_str()));
        assert_eq!(Some("Wasimodo"), voc.get_value_by_id(1).map(|x| x.as_str()));

        let s = serde_json::to_string(&voc).unwrap();
        let voc2: Vocabulary<String> = serde_json::from_str(&s).unwrap();
        println!("{voc}");
        println!("{voc2}");
    }
}