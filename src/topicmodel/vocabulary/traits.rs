use std::borrow::Borrow;
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Write};
use std::ops::Range;
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use trie_rs::map::{Trie, TrieBuilder};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::traits::ToParseableString;
use crate::topicmodel::vocabulary::LoadVocabularyError;

/// A basic vocabulary for [HashRef] elements.
pub trait BasicVocabulary<T>: Send + Sync + AsRef<Vec<HashRef<T>>> + IntoIterator<Item=HashRef<T>> {
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

    /// Iterate over the words
    fn iter(&self) -> Iter<HashRef<T>>;

    fn get_id_entry(&self, id: usize) -> Option<(usize, &HashRef<T>)>;

    /// Get the HashRef for a specific `id` or none
    fn get_value(&self, id: usize) -> Option<&HashRef<T>>;

    /// Get the HashRef for a specific `id` or none
    unsafe fn get_value_unchecked(&self, id: usize) -> &HashRef<T>;

    /// Check if the `id` is contained in this
    fn contains_id(&self, id: usize) -> bool;

    /// Creates a new instance
    fn create(language: Option<LanguageHint>) -> Self where Self: Sized;

    /// Creates a new instance
    fn create_from(language: Option<LanguageHint>, voc: Vec<T>) -> Self where Self: Sized, T: Eq + Hash;

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

/// Allows to search a vocabulary by a query
pub trait SearchableVocabulary<T>: BasicVocabulary<T> where T: Eq + Hash {

    /// Retrieves the id for `value`
    fn get_id<Q: ?Sized>(&self, value: &Q) -> Option<usize>
    where
        T: Borrow<Q>,
        Q: Hash + Eq;

    /// Retrieves the id for `value`
    fn get_hash_ref<Q: ?Sized>(&self, value: &Q) -> Option<&HashRef<T>>
    where
        T: Borrow<Q>,
        Q: Hash + Eq;

    /// Retrieves the complete entry for `value` in the vocabulary, if it exists
    fn get_entry_id<Q: ?Sized>(&self, value: &Q) -> Option<(&HashRef<T>, &usize)>
    where
        T: Borrow<Q>,
        Q: Hash + Eq;

    fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq;



    /// Returns a new vocabulary filtered by the ids
    fn filter_by_id<F: Fn(usize) -> bool>(&self, filter: F) -> Self where Self: Sized;

    /// Returns a vocabulary filtered by the values
    fn filter_by_value<'a, F: Fn(&'a HashRef<T>) -> bool>(&'a self, filter: F) -> Self where Self: Sized, T: 'a;
}

/// A vocabulary that can be modified
pub trait VocabularyMut<T>: SearchableVocabulary<T> where T: Eq + Hash {
    /// Adds the `value` to the vocabulary and returns the associated id
    fn add_hash_ref(&mut self, value: HashRef<T>) -> usize;

    fn add_value(&mut self, value: T) -> usize;

    /// Adds any `value` that can be converted into `T`
    fn add<V: Into<T>>(&mut self, value: V) -> usize;

    fn add_all_hash_ref<I: IntoIterator<Item=HashRef<T>>>(&mut self, other: I);
}

/// A vocabulary that can be mapped
pub trait MappableVocabulary<T>: BasicVocabulary<T> where T: Eq + Hash {
    /// Mapps the vocabulary entries from [T] to [Q]. The order of the terms stays the same.
    fn map<Q: Eq + Hash, V, F>(self, mapping: F) -> V where F: Fn(&T) -> Q, V: BasicVocabulary<Q>;
}

/// A vocabulary that can be stored to a file.
pub trait StoreableVocabulary<T> where T: ToParseableString {
    /// Writes the vocabulary as a file to `path` in the list format
    fn save_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<usize> {
        let mut writer = File::options().create(true).truncate(true).write(true).open(path)?;
        self.save_to_output(&mut writer)
    }

    /// Writes the vocabulary to `writer` in the list format
    fn save_to_output(&self, writer: &mut impl Write) -> std::io::Result<usize>;
}

/// A vocabulary that can be loaded.
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

