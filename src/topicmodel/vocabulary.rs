use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::{Entry};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::marker::PhantomData;
use std::ops::{Bound, Deref, DerefMut, Range};
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use std::sync::Arc;
use itertools::Itertools;
use rayon::prelude::IntoParallelIterator;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use thiserror::Error;
use crate::topicmodel::traits::ToParseableString;

pub type StringVocabulary = Vocabulary<String>;

/// A vocabulary mapping between an usize id and a specific object (word)
#[derive(Clone, Debug)]
pub struct Vocabulary<T> {
    entry2id: HashMap<HashRef<T>, usize>,
    id2entry: Vec<HashRef<T>>
}

impl <T> Vocabulary<T> {

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

    /// The number of entries in the vocabulary
    pub fn len(&self) -> usize {
        self.id2entry.len()
    }

    /// Clear the whole thing
    pub fn clear(&mut self){
        self.id2entry.clear();
        self.entry2id.clear();
    }

    /// Get the ids
    pub fn ids(&self) -> Range<usize> {
        0..self.id2entry.len()
    }

    /// Iterate over the words
    pub fn iter(&self) -> Iter<HashRef<T>> {
        self.id2entry.iter()
    }

    /// Get the word for a specific `id` or none
    pub fn get_word(&self, id: usize) -> Option<&T> {
        return self.id2entry.get(id).map(|value| &value)
    }

    /// Get the HashRef for a specific `id` or none
    pub fn get_hash_ref(&self, id: usize) -> Option<&HashRef<T>> {
        return self.id2entry.get(id)
    }

    /// Check if the `id` is contained in this
    pub fn contains_id(&self, id: usize) -> bool {
        self.id2entry.len() > id
    }
}

impl<T: Eq + Hash> From<Vec<T>> for Vocabulary<T>  {
    fn from(value: Vec<T>) -> Self {
        let mut id2entry = value.into_iter().map(|value| HashRef::new(value)).collect_vec();
        let mut entry2id = id2entry.iter().cloned().enumerate().map(|(a, b)| (b, a)).collect();

        return Self {
            id2entry,
            entry2id
        }
    }
}

impl <T: Eq + Hash> Vocabulary<T> {

    /// Adds the `value` to the vocabulary and returns the associated id
    pub fn add_hash_ref(&mut self, value: HashRef<T>) -> usize {
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

    /// Adds any `value` that can be converted into `T`
    pub fn add<V: Into<T>>(&mut self, value: V) -> usize {
        self.add_hash_ref(value.into().into())
    }

    /// Retrieves the id for `value`
    pub fn get_word_id<Q: ?Sized>(&self, value: &Q) -> Option<usize>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        return self.entry2id.get(Wrapper::wrap(value)).cloned()
    }

    /// Retrieves the complete entry for `value` in the vocabulary, if it exists
    pub fn get_entry_for<Q: ?Sized>(&self, value: &Q) -> Option<(&HashRef<T>, &usize)>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        self.entry2id.get_key_value(Wrapper::wrap(value))
    }

    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
        where
            T: Borrow<Q>,
            Q: Hash + Eq {

        self.entry2id.contains_key(Wrapper::wrap(value))
    }
}

impl<Q: Into<T>, T: Eq + Hash> Extend<Q> for Vocabulary<T> {
    fn extend<I: IntoIterator<Item=Q>>(&mut self, iter: I) {
        for value in iter {
            self.add(value);
        }
    }
}


impl<T: Eq> PartialEq for Vocabulary<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() &&
            self.id2entry.iter()
                .zip_eq(other.id2entry.iter())
                .all(|(a, b)| a.eq(b))
    }
}

impl<T: Eq> Eq for Vocabulary<T> {}


#[derive(Debug, Error)]
pub enum LoadVocabularyError<E: Debug> {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Had a parse error")]
    Parse(E),
}

impl<T: Hash + Eq + FromStr<Err=E>, E: Debug> Vocabulary<T> {

    /// Loads from a `path` in the list format
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, LoadVocabularyError<E>> {
        let mut reader = BufReader::new(File::open(path)?);
        Self::load_from_input(&mut reader)
    }

    /// Loads from a `reader` in the list format
    pub fn load_from_input(reader: &mut impl BufRead) -> Result<Self, LoadVocabularyError<E>> {
        let mut id2entry = Vec::new();
        for line in reader.lines() {
            id2entry.push(line?.parse().map_err(LoadVocabularyError::Parse)?)
        }
        Ok(Self::from(id2entry))
    }
}

impl<T: ToParseableString> Vocabulary<T>  {
    /// Writes the vocabulary as a file to `path` in the list format
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<usize> {
        let mut writer = File::options().create(true).truncate(true).write(true).open(path)?;
        self.save_to_output(&mut writer)
    }

    /// Writes the vocabulary to `writer` in the list format
    pub fn save_to_output(&self, writer: &mut impl Write) -> std::io::Result<usize> {
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

impl <T: ToString> Display for Vocabulary<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = self
            .id2entry
            .iter()
            .map(|a| a.to_string())
            .join(", ");
        write!(f, "Vocabulary[{}]", x)
    }
}

impl<T> Default for Vocabulary<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}


impl<T: Serialize> Serialize for Vocabulary<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut st = serializer.serialize_struct("Vocabulary", 1)?;
        st.serialize_field(
            "id2entry",
            &self.id2entry.iter().map(|it| it.inner.deref()).collect_vec()
        )?;
        st.end()
    }
}

impl <'de, T: Deserialize<'de> + Hash + Eq> Deserialize<'de> for Vocabulary<T> {
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
            type Value = Vocabulary<T>;

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
                    Ok(Vocabulary::from(field_value))
                } else {
                    Err(de::Error::missing_field("id2entry"))
                }
            }
        }

        deserializer.deserialize_struct("Vocabulary", &["id2entry"], VocabularyVisitor::<T>::new())
    }
}


impl<T> IntoParallelIterator for Vocabulary<T> {
    type Iter = rayon::vec::IntoIter<HashRef<T>>;
    type Item = HashRef<T>;

    fn into_par_iter(self) -> Self::Iter {
        self.id2entry.into_par_iter()
    }
}





// Taken from https://github.com/billyrieger/bimap-rs/blob/main/src/mem.rs

/// A ref that supplies the Hash and Eq method of the underlying struct.
/// It is threadsafe and allows a simple cloning as well as ordering
/// and dereferencing of the underlying value.
#[derive(Debug)]
#[repr(transparent)]
pub struct HashRef<T: ?Sized> {
    inner: Arc<T>
}

unsafe impl<T> Sync for HashRef<T>{}
unsafe impl<T> Send for HashRef<T>{}

impl<T> HashRef<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value)
        }
    }
}

impl<T: Hash> Hash for HashRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<T: ?Sized + PartialEq> PartialEq for HashRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}
impl<T: ?Sized + Eq> Eq for HashRef<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for HashRef<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T: ?Sized + Ord> Ord for HashRef<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T> Clone for HashRef<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T> Deref for HashRef<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T> From<T> for HashRef<T>  {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Used for hash lookup
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
struct Wrapper<T: ?Sized>(T);

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
        let b: &K = self.inner.deref();
        let b: &Q = b.borrow();
        Wrapper::wrap(b)
    }
}


#[cfg(test)]
mod test {
    use crate::topicmodel::vocabulary::{HashRef, StringVocabulary, Vocabulary};

    #[test]
    fn can_insert_and_retrieve() {
        let mut voc = StringVocabulary::new();
        voc.add("Hello World".to_string());
        voc.add("Wasimodo".to_string());

        assert_eq!(2usize, voc.len());
        assert_eq!(Some(0usize), voc.get_word_id("Hello World"));
        assert_eq!(Some("Hello World"), voc.get_word(0).map(|x| x.as_str()));
        assert_eq!(Some("Wasimodo"), voc.get_word(1).map(|x| x.as_str()));

        let s = serde_json::to_string(&voc).unwrap();
        let voc2: Vocabulary<String> = serde_json::from_str(&s).unwrap();
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