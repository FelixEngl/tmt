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
use std::rc::Rc;
use std::slice::Iter;
use std::str::FromStr;
use itertools::Itertools;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use thiserror::Error;
use crate::topicmodel::traits::ToParseableString;

pub type StringVocabulary = Vocabulary<String>;

#[derive(Clone, Debug)]
pub struct Vocabulary<T> {
    entry2id: HashMap<HashRef<T>, usize>,
    id2entry: Vec<HashRef<T>>
}

impl <T> Vocabulary<T> {
    pub fn len(&self) -> usize {
        self.id2entry.len()
    }

    pub fn clear(&mut self){
        self.id2entry.clear();
        self.entry2id.clear();
    }

    pub fn ids(&self) -> Range<usize> {
        0..self.id2entry.len()
    }

    pub fn iter_words(&self) -> VocIter<T> {
        VocIter::create(self.id2entry.iter())
    }

    pub fn get_word(&self, id: usize) -> Option<&T> {
        return self.id2entry.get(id).map(|value| value.deref())
    }

    pub fn contains_word_id(&self, id: usize) -> bool {
        self.id2entry.len() > id
    }
}


#[repr(transparent)]
pub struct VocIter<'a, T> {
    iter: Iter<'a, HashRef<T>>
}

impl<'a, T> VocIter<'a, T> {
    fn create(iter: Iter<'a, HashRef<T>>) -> Self {
        VocIter { iter }
    }
}

impl<'a, T> Iterator for VocIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let target = self.iter.next()?;
        Some(target.deref())
    }
}


#[derive(Debug, Error)]
pub enum LoadVocabularyError<E: Debug> {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Had a parse error")]
    Parse(E),
}

impl<T: Hash + Eq + FromStr<Err=E>, E: Debug> Vocabulary<T> {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, LoadVocabularyError<E>> {
        let mut reader = BufReader::new(File::open(path)?);
        Self::load_from_input(&mut reader)
    }

    pub fn load_from_input(reader: &mut impl BufRead) -> Result<Self, LoadVocabularyError<E>> {
        let mut id2entry = Vec::new();
        for line in reader.lines() {
            id2entry.push(line?.parse().map_err(LoadVocabularyError::Parse)?)
        }
        Ok(Self::build_from_id2entry(id2entry))
    }
}


impl<T: Eq> PartialEq for Vocabulary<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() &&
            self.id2entry.iter()
                .zip_eq(other.id2entry.iter())
                .all(|(a, b)| a.deref().eq(b.deref()))
    }
}

impl<T: Eq> Eq for Vocabulary<T> {}

impl<T: ToParseableString> Vocabulary<T>  {
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<usize> {
        let mut writer = File::options().create(true).truncate(true).write(true).open(path)?;
        self.save_to_output(&mut writer)
    }

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

impl <T: Eq + Hash> Vocabulary<T> {
    pub fn new() -> Vocabulary<T>{
        Default::default()
    }

    fn build_from_id2entry(values: Vec<T>) -> Self {
        let mut id2entry = values.into_iter().map(|value| HashRef::new(value)).collect_vec();
        let mut entry2id = id2entry.iter().cloned().enumerate().map(|(a, b)| (b, a)).collect();

        return Self {
            id2entry,
            entry2id
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Vocabulary<T> {
        Vocabulary {
            entry2id: HashMap::with_capacity(capacity),
            id2entry: Vec::with_capacity(capacity)
        }
    }

    pub fn add<V: Into<T>>(&mut self, to_add: V) -> usize {
        let found = self.entry2id.entry(HashRef::new(to_add.into()));
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

    pub fn get_word_id<Q: ?Sized>(&self, word: &Q) -> Option<usize>
        where
            T: Borrow<Q>,
            Q: Hash + Eq
    {
        return self.entry2id.get(Wrapper::wrap(word)).cloned()
    }

    pub fn contains_word<Q: ?Sized>(&self, word: &Q) -> bool
        where
            T: Borrow<Q>,
            Q: Hash + Eq {

        self.entry2id.contains_key(Wrapper::wrap(word))
    }
}


impl<Q: Into<T>, T: Eq + Hash> Extend<Q> for Vocabulary<T> {
    fn extend<I: IntoIterator<Item=Q>>(&mut self, iter: I) {
        for value in iter {
            self.add(value);
        }
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
        Vocabulary {
            entry2id: HashMap::default(),
            id2entry: Vec::default()
        }
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
            Ok(Vocabulary::build_from_id2entry(field_value))
        } else {
            Err(de::Error::missing_field("id2entry"))
        }
    }
}

impl <'de, T: Deserialize<'de> + Hash + Eq> Deserialize<'de> for Vocabulary<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_struct("Vocabulary", &["id2entry"], VocabularyVisitor::<T>::new())
    }
}







// Taken from https://github.com/billyrieger/bimap-rs/blob/main/src/mem.rs

#[derive(Eq, Ord)]
struct HashRef<T> {
    inner: Rc<T>
}

impl<T> HashRef<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(value)
        }
    }
}

impl<T: Hash> Hash for HashRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<T: PartialEq> PartialEq for HashRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl<T: PartialOrd> PartialOrd for HashRef<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl<T> Clone for HashRef<T> {
    fn clone(&self) -> Self {
        Self {inner: self.inner.clone()}
    }
}

impl<T> Debug for HashRef<T> where T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Deref for HashRef<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

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
        // Rc<K>: Borrow<K>
        let b: &K = self.inner.deref();
        // K: Borrow<Q>
        let b: &Q = b.borrow();
        Wrapper::wrap(b)
    }
}


#[cfg(test)]
mod test {
    use crate::topicmodel::vocabulary::{StringVocabulary, Vocabulary};

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
}