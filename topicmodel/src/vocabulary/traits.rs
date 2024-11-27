use std::borrow::Borrow;
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Write};
use std::ops::{Range};
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use itertools::Itertools;
use trie_rs::map::{Trie, TrieBuilder};
use crate::language_hint::LanguageHint;
use crate::traits::AsParseableString;
use crate::vocabulary::LoadVocabularyError;

/// A basic vocabulary for [HashRef] elements.
pub trait BasicVocabulary<T>: AsRef<[T]> + IntoIterator<Item=T> {
    /// Gets the associated language
    fn language(&self) -> Option<&LanguageHint>;

    /// Sets a specific language
    fn set_language(&mut self, new: Option<LanguageHint>) -> Option<LanguageHint>;

    /// The number of entries in the vocabulary
    fn len(&self) -> usize;

    /// Clear the whole thing
    fn clear(&mut self);

    /// Get the ids
    fn ids(&self) -> Range<usize>;

    /// Iterates over the words in the order of the ids.
    /// To get the ids use .enumerate()
    fn iter(&self) -> Iter<T>;

    /// Iterates over the ids and associated words.
    /// Usually only a shortcut for [iter] followed by an [enumerate]
    fn iter_entries<'a>(&'a self) -> impl Iterator<Item=(usize, &'a T)> + 'a where T: 'a;

    fn get_entry_by_id(&self, id: usize) -> Option<(usize, &T)>;

    /// Get the HashRef for a specific `id` or none
    fn get_value_by_id(&self, id: usize) -> Option<&T>;

    /// Get the HashRef for a specific `id` or none
    unsafe fn get_value_unchecked(&self, id: usize) -> &T;

    /// Check if the `id` is contained in this
    fn contains_id(&self, id: usize) -> bool;

    /// Creates a new instance
    fn create(language: Option<LanguageHint>) -> Self where Self: Sized;

    /// Creates a new instance
    fn create_from(language: Option<LanguageHint>, voc: Vec<T>) -> Self where Self: Sized, T: Eq + Hash + Clone;

    /// Creates a trie from this
    fn create_trie(&self) -> Trie<u8, usize> where T: AsRef<[u8]> {
        let mut builder = TrieBuilder::new();
        for (id, entry) in self.iter().enumerate() {
            builder.push(entry.as_ref(), id);
        }
        builder.build()
    }

    /// Creates a trie from this
    fn into_trie(self) -> Trie<u8, usize> where Self: Sized, T: AsRef<[u8]> {
        let mut builder = TrieBuilder::new();
        for (id, entry) in self.iter().enumerate() {
            builder.push(entry.as_ref(), id);
        }
        builder.build()
    }
}

pub trait AlphabeticalVocabulary<T>: BasicVocabulary<T> where T: Ord {
    fn ids_in_alphabetical_order(&self) -> Vec<usize> {
        let mut sorted_entries = self.ids().collect_vec();
        sorted_entries.sort_by_key(|value| unsafe{self.get_value_unchecked(*value)});
        sorted_entries
    }
}

impl<T, V> AlphabeticalVocabulary<T> for V
where
    V: BasicVocabulary<T>,
    T: Ord
{}

/// Allows to search a vocabulary by a query
pub trait SearchableVocabulary<T>: BasicVocabulary<T> where T: Eq + Hash {

    /// Retrieves the id for `q`
    fn get_id<Q: ?Sized>(&self, q: &Q) -> Option<usize>
    where
        T: Borrow<Q>,
        Q: Hash + Eq;

    /// Retrieves the value identity for `q`
    fn get_value<Q: ?Sized>(&self, q: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq;

    /// Retrieves the complete entry for `q` in the vocabulary, if it exists
    fn get_entry_by_value<Q: ?Sized>(&self, q: &Q) -> Option<(&T, &usize)>
    where
        T: Borrow<Q>,
        Q: Hash + Eq;


    fn contains_value<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq;


    /// Returns a new vocabulary filtered by the ids
    fn filter_by_id<F: Fn(usize) -> bool>(&self, filter: F) -> Self where Self: Sized;

    /// Returns a vocabulary filtered by the values
    fn filter_by_value<'a, F: Fn(&'a T) -> bool>(&'a self, filter: F) -> Self where Self: Sized, T: 'a;
}

/// A vocabulary that can be modified
pub trait VocabularyMut<T>: SearchableVocabulary<T> where T: Eq + Hash + Clone {
    /// Adds the `value` to the vocabulary and returns the associated id
    fn add_value(&mut self, value: T) -> usize;

    /// Adds any `value` that can be converted into `T`
    fn add<V: Into<T>>(&mut self, value: V) -> usize {
        self.add_value(value.into())
    }

    fn add_all_value<I: IntoIterator<Item=T>>(&mut self, other: I);
}

/// A vocabulary that can be mapped
pub trait MappableVocabulary<T>: BasicVocabulary<T> where T: Eq + Hash {
    /// Mapps the vocabulary entries from [T] to [R]. The order of the terms stays the same.
    fn map<R, V, F>(self, mapping: F) -> V where F: Fn(T) -> R, V: BasicVocabulary<R>, R: Eq + Hash + Clone;
}

/// A vocabulary that can be stored to a file.
pub trait StoreableVocabulary<T> where T: AsParseableString
{
    /// Writes the vocabulary as a file to `path` in the list format
    fn save_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<usize> {
        let mut writer = File::options().create(true).truncate(true).write(true).open(path)?;
        self.save_to_output(&mut writer)
    }

    /// Writes the vocabulary to `writer` in the list format
    fn save_to_output(&self, writer: &mut impl Write) -> std::io::Result<usize>;
}

/// A vocabulary that can be ex.
pub trait LoadableVocabulary<T, E> where T: Hash + Eq + FromStr<Err=E>, E: Debug, Self: From<Vec<T>> {
    /// Loads from a `path` in the list format
    fn load_from_file(path: impl AsRef<Path>) -> Result<Self, LoadVocabularyError<E>> {
        let mut reader = BufReader::new(File::open(path)?);
        Self::load_from_input(&mut reader)
    }

    /// Loads from a `reader` in the list format
    fn load_from_input(reader: &mut impl BufRead) -> Result<Self, LoadVocabularyError<E>> {
        let mut id2entry = Vec::new();
        for line in reader.lines() {
            id2entry.push(line?.parse().map_err(LoadVocabularyError::Parse)?)
        }
        Ok(Self::from(id2entry))
    }
}
