#![allow(dead_code)]

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash};
use std::collections::hash_map::{Entry};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::marker::PhantomData;
use std::ops::{Bound, Deref, Range};
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use std::vec::IntoIter;
use itertools::Itertools;
use rayon::prelude::{FromParallelIterator, IntoParallelIterator, ParallelIterator};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;
use thiserror::Error;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::traits::ToParseableString;

pub type StringVocabulary = VocabularyImpl<String>;

#[macro_export]
macro_rules! voc {
    () => {
        VocabularyImpl::new()
    };
    ($($value: tt),+) => {
        {
            let mut __voc = $crate::topicmodel::vocabulary::VocabularyImpl::new();
            $(
                $crate::topicmodel::vocabulary::VocabularyMut::add_value(&mut __voc, $value);
            )+
            __voc
        }
    };
}

pub trait Vocabulary<T>: Send + Sync {
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

    /// Check if the `id` is contained in this
    fn contains_id(&self, id: usize) -> bool;

}


pub trait VocabularyMut<T>: Vocabulary<T> where T: Eq + Hash {
    /// Adds the `value` to the vocabulary and returns the associated id
    fn add_hash_ref(&mut self, value: HashRef<T>) -> usize;

    fn add_value(&mut self, value: T) -> usize;

    /// Adds any `value` that can be converted into `T`
    fn add<V: Into<T>>(&mut self, value: V) -> usize;

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

    fn filter_by_value<'a, F: Fn(&'a HashRef<T>) -> bool>(&'a self, filter: F) -> Self where Self: Sized, T: 'a;
}

pub trait MappableVocabulary<T>: Vocabulary<T> where T: Eq + Hash {
    fn map<Q: Eq + Hash, V, F>(self, mapping: F) -> V where F: Fn(&T) -> Q, V: From<Vec<Q>>;
}

pub trait StoreableVocabulary<T> where T: ToParseableString {
    /// Writes the vocabulary as a file to `path` in the list format
    fn save_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<usize> {
        let mut writer = File::options().create(true).truncate(true).write(true).open(path)?;
        self.save_to_output(&mut writer)
    }

    /// Writes the vocabulary to `writer` in the list format
    fn save_to_output(&self, writer: &mut impl Write) -> std::io::Result<usize>;
}

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



/// A vocabulary mapping between an usize id and a specific object (word)
#[derive(Clone, Debug)]
pub struct VocabularyImpl<T> {
    entry2id: HashMap<HashRef<T>, usize>,
    id2entry: Vec<HashRef<T>>
}



impl <T> VocabularyImpl<T> {
    /// Create a new vocabulary with the default sizes
    pub fn new() -> Self {
        Self {
            entry2id: Default::default(),
            id2entry: Default::default()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entry2id: HashMap::with_capacity(capacity),
            id2entry: Vec::with_capacity(capacity)
        }
    }
}


impl <T> Vocabulary<T> for VocabularyImpl<T> {
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
    fn iter(&self) -> Iter<HashRef<T>> {
        self.id2entry.iter()
    }

    fn get_id_entry(&self, id: usize) -> Option<(usize, &HashRef<T>)> {
        self.get_value(id).map(|value| (id, value))
    }

    /// Get the HashRef for a specific `id` or none
    fn get_value(&self, id: usize) -> Option<&HashRef<T>> {
        self.id2entry.get(id)
    }

    /// Check if the `id` is contained in this
    fn contains_id(&self, id: usize) -> bool {
        self.id2entry.len() > id
    }

}


impl<T> AsRef<Vec<HashRef<T>>> for VocabularyImpl<T> {
    fn as_ref(&self) -> &Vec<HashRef<T>> {
        &self.id2entry
    }
}

impl<T: Eq + Hash> From<Vec<T>> for VocabularyImpl<T>  {
    fn from(value: Vec<T>) -> Self {
        let id2entry = value.into_iter().map(|value| HashRef::new(value)).collect_vec();
        let entry2id = id2entry.iter().cloned().enumerate().map(|(a, b)| (b, a)).collect();

        return Self {
            id2entry,
            entry2id
        }
    }
}


impl<T> VocabularyMut<T> for VocabularyImpl<T> where T: Eq + Hash {
    /// Adds the `value` to the vocabulary and returns the associated id
    fn add_hash_ref(&mut self, value: HashRef<T>) -> usize {
        let found = self.entry2id.entry(value);
        match found {
            Entry::Occupied(entry) => {
                *entry.get()
            }
            Entry::Vacant(entry) => {
                let pos = self.id2entry.len();
                self.id2entry.push(entry.key().clone());
                entry.insert(pos);
                return pos
            }
        }
    }

    fn add_value(&mut self, value: T) -> usize {
        self.add_hash_ref(value.into())
    }

    /// Adds any `value` that can be converted into `T`
    fn add<V: Into<T>>(&mut self, value: V) -> usize {
        self.add_hash_ref(value.into().into())
    }

    /// Retrieves the id for `value`
    fn get_id<Q: ?Sized>(&self, value: &Q) -> Option<usize>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        return self.entry2id.get(Wrapper::wrap(value)).cloned()
    }

    /// Retrieves the id for `value`
    fn get_hash_ref<Q: ?Sized>(&self, value: &Q) -> Option<&HashRef<T>>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        Some(self.get_entry_id(value)?.0)
    }

    /// Retrieves the complete entry for `value` in the vocabulary, if it exists
    fn get_entry_id<Q: ?Sized>(&self, value: &Q) -> Option<(&HashRef<T>, &usize)>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        self.entry2id.get_key_value(Wrapper::wrap(value))
    }

    fn contains<Q: ?Sized>(&self, value: &Q) -> bool
        where
            T: Borrow<Q>,
            Q: Hash + Eq {

        self.entry2id.contains_key(Wrapper::wrap(value))
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

    fn filter_by_value<'a, F: Fn(&'a HashRef<T>) -> bool>(&'a self, filter: F) -> Self where Self: Sized, T: 'a {
        self.id2entry.iter().filter_map(|value| {
            if filter(value) {
                Some(value.clone())
            } else {
                None
            }
        }).collect()
    }
}


impl<T> MappableVocabulary<T> for VocabularyImpl<T> where T: Eq + Hash {
    fn map<Q: Eq + Hash, V, F>(self, mapping: F) -> V where F: Fn(&T) -> Q, V: From<Vec<Q>> {
        V::from(self.id2entry.into_iter().map(|value| mapping(value.as_ref())).collect::<Vec<_>>())
    }
}

impl<T, Q: Into<T>> Extend<Q> for VocabularyImpl<T> where T: Eq + Hash {
    fn extend<I: IntoIterator<Item=Q>>(&mut self, iter: I) {
        for value in iter {
            self.add(value);
        }
    }
}


impl<T: Eq> PartialEq for VocabularyImpl<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() &&
            self.id2entry.iter()
                .zip_eq(other.id2entry.iter())
                .all(|(a, b)| a.eq(b))
    }
}

impl<T: Eq> Eq for VocabularyImpl<T> {}


#[derive(Debug, Error)]
pub enum LoadVocabularyError<E: Debug> {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Had a parse error")]
    Parse(E),
}

impl<T: Hash + Eq + FromStr<Err=E>, E: Debug> LoadableVocabulary<T, E> for  VocabularyImpl<T> {
}


impl<T: ToParseableString> StoreableVocabulary<T> for VocabularyImpl<T>  {
    /// Writes the vocabulary to `writer` in the list format
    fn save_to_output(&self, writer: &mut impl Write) -> std::io::Result<usize> {
        let mut written = 0;
        let mut writer = BufWriter::new(writer);
        for value in self.id2entry.iter() {
            let value = value.to_parseable_string();
            written += writer.write(value.as_bytes())?;
            written += writer.write(b"\n")?;
        }
        writer.flush()?;
        Ok(written)
    }
}

impl <T: ToString> Display for VocabularyImpl<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = self
            .id2entry
            .iter()
            .map(|a| a.to_string())
            .join(", ");
        write!(f, "Vocabulary[{}]", x)
    }
}

impl<T> Default for VocabularyImpl<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}


impl<T: Serialize> Serialize for VocabularyImpl<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut st = serializer.serialize_struct("Vocabulary", 1)?;
        st.serialize_field(
            "id2entry",
            &self.id2entry.iter().map(|it| it.as_ref()).collect_vec()
        )?;
        st.end()
    }
}

impl <'de, T: Deserialize<'de> + Hash + Eq> Deserialize<'de> for VocabularyImpl<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

        struct VocabularyVisitor<'de, T: Deserialize<'de>>{_phantom: PhantomData<(T, &'de())>}

        impl<'de, T: Deserialize<'de>> VocabularyVisitor<'de, T>  {
            fn new() -> Self {
                Self{_phantom: PhantomData}
            }
        }

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Id2Entry }

        impl<'de, T: Deserialize<'de> + Hash + Eq> Visitor<'de> for VocabularyVisitor<'de, T> {
            type Value = VocabularyImpl<T>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Vocabulary")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut field = None;
                while let Some(key) = map.next_key::<Field>()? {
                    match key { Field::Id2Entry => {
                        if field.is_some() {
                            return Err(de::Error::duplicate_field("id2entry"));
                        }
                        field = Some(map.next_value::<Vec<T>>()?);
                    } }
                }
                if let Some(field_value) = field {
                    Ok(VocabularyImpl::from(field_value))
                } else {
                    Err(de::Error::missing_field("id2entry"))
                }
            }
        }

        deserializer.deserialize_struct("Vocabulary", &["id2entry"], VocabularyVisitor::<T>::new())
    }
}


impl<T> IntoIterator for VocabularyImpl<T> {
    type Item = HashRef<T>;
    type IntoIter = IntoIter<HashRef<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.id2entry.into_iter()
    }
}


impl<T> IntoParallelIterator for VocabularyImpl<T> {
    type Iter = rayon::vec::IntoIter<HashRef<T>>;
    type Item = HashRef<T>;

    fn into_par_iter(self) -> Self::Iter {
        self.id2entry.into_par_iter()
    }
}


impl<T> FromIterator<HashRef<T>> for VocabularyImpl<T> where T: Hash + Eq {
    fn from_iter<I: IntoIterator<Item=HashRef<T>>>(iter: I) -> Self {
        let mut new = Self::new();
        for value in iter {
            new.add_hash_ref(value);
        }
        return new;
    }
}

impl<'a, T> FromIterator<&'a HashRef<T>> for VocabularyImpl<T> where T: Hash + Eq {
    fn from_iter<I: IntoIterator<Item=&'a HashRef<T>>>(iter: I) -> Self {
        let mut new = Self::new();
        for value in iter {
            new.add_hash_ref(value.clone());
        }
        return new;
    }
}

impl<T> FromParallelIterator<HashRef<T>> for VocabularyImpl<T> where T: Hash + Eq {
    fn from_par_iter<I>(par_iter: I) -> Self where I: IntoParallelIterator<Item=HashRef<T>> {
        let mut new = Self::new();
        for value in par_iter.into_par_iter().collect_vec_list() {
            for value in value.into_iter() {
                new.add_hash_ref(value);
            }
        }
        return new;
    }
}

impl<'a, T> FromParallelIterator<&'a HashRef<T>> for VocabularyImpl<T> where T: Hash + Eq {
    fn from_par_iter<I>(par_iter: I) -> Self where I: IntoParallelIterator<Item=&'a HashRef<T>> {
        let mut new = Self::new();
        for value in par_iter.into_par_iter().collect_vec_list() {
            for value in value.into_iter() {
                new.add_hash_ref(value.clone());
            }
        }
        return new;
    }
}



/// Used for hash lookup
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
struct Wrapper<T: ?Sized> {
    inner: T
}

impl<T: ?Sized> Wrapper<T> {
    #[inline]
    pub fn wrap(value: &T) -> &Self {
        // safe because Wrapper<T> is #[repr(transparent)]
        unsafe { &*(value as *const T as *const Self) }
    }

    pub fn wrap_bound(bound: Bound<&T>) -> Bound<&Self> {
        match bound {
            Bound::Included(t) => Bound::Included(Self::wrap(t)),
            Bound::Excluded(t) => Bound::Excluded(Self::wrap(t)),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

impl<K, Q> Borrow<Wrapper<Q>> for HashRef<K>
    where
        K: Borrow<Q>,
        Q: ?Sized,
{
    fn borrow(&self) -> &Wrapper<Q> {
        let b: &K = self.deref();
        let b: &Q = b.borrow();
        Wrapper::wrap(b)
    }
}



#[cfg(test)]
mod test {
    use crate::topicmodel::vocabulary::{HashRef, StringVocabulary, Vocabulary, VocabularyImpl, VocabularyMut};

    #[test]
    fn can_insert_and_retrieve() {
        let mut voc = StringVocabulary::new();
        voc.add("Hello World".to_string());
        voc.add("Wasimodo".to_string());

        assert_eq!(2usize, voc.len());
        assert_eq!(Some(0usize), voc.get_id("Hello World"));
        assert_eq!(Some("Hello World"), voc.get_value(0).map(|x| x.as_str()));
        assert_eq!(Some("Wasimodo"), voc.get_value(1).map(|x| x.as_str()));

        let s = serde_json::to_string(&voc).unwrap();
        let voc2: VocabularyImpl<String> = serde_json::from_str(&s).unwrap();
        println!("{voc}");
        println!("{voc2}");
    }

    #[test]
    fn equals_behaves_normally() {
        let a = HashRef::new("Test1");
        let b = a.clone();
        let c = HashRef::new("Test1");
        let d = HashRef::new("Test2");

        assert_eq!(a, a);
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(b, c);
        assert_ne!(d, a);
        assert_ne!(d, b);
        assert_ne!(d, c);
    }
}