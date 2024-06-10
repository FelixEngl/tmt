use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::io::Write;
use std::ops::{Range};
use std::slice::Iter;
use pyo3::{Bound, pyclass, pymethods, PyResult};
use pyo3::prelude::{PyModule, PyModuleMethods};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{LoadableVocabulary, MappableVocabulary, StoreableVocabulary, Vocabulary, VocabularyImpl, VocabularyMut};

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyVocabulary {
    inner: VocabularyImpl<String>
}

#[pymethods]
impl PyVocabulary {
    #[new]
    pub fn new(size: Option<usize>) -> Self {
        match size {
            None => {
                Self {
                    inner: VocabularyImpl::new()
                }
            }
            Some(value) => {
                Self {
                    inner: VocabularyImpl::with_capacity(value)
                }
            }
        }
    }

    pub fn __repr__(&self) -> String {
        format!("PyVocabulary({:?})", self.inner)
    }

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    pub fn add(&mut self, word: String) -> usize {
        self.inner.add_value(word)
    }

    pub fn word_to_id(&mut self, word: String) -> Option<usize> {
        self.inner.get_id(word.as_str())
    }

    pub fn id_wo_word(&self, id: usize) -> Option<&String> {
        self.inner.get_value(id).map(|value| value.as_ref())
    }

    pub fn to_json(&self) -> PyResult<String> {
        let mut str = Vec::new();
        self.inner.save_to_output(&mut str)?;
        Ok(String::from_utf8(str)?)
    }
}

impl Vocabulary<String> for PyVocabulary {
    delegate::delegate! {
        to self.inner {
            /// The number of entries in the vocabulary
            fn len(&self) -> usize;

            /// Clear the whole thing
            fn clear(&mut self);

            /// Get the ids
            fn ids(&self) -> Range<usize>;

            /// Iterate over the words
            fn iter(&self) -> Iter<HashRef<String>>;

            fn get_id_entry(&self, id: usize) -> Option<(usize, &HashRef<String>)>;

            /// Get the HashRef for a specific `id` or none
            fn get_value(&self, id: usize) -> Option<&HashRef<String>>;

            /// Check if the `id` is contained in this
            fn contains_id(&self, id: usize) -> bool;
        }
    }
}

impl MappableVocabulary<String> for PyVocabulary {
    fn map<Q: Eq + Hash, V, F>(self, mapping: F) -> V where F: Fn(&String) -> Q, V: From<Vec<Q>> {
        self.inner.map(mapping)
    }
}

impl VocabularyMut<String> for PyVocabulary {
    delegate::delegate! {
        to self.inner {
            /// Adds the `value` to the vocabulary and returns the associated id
            fn add_hash_ref(&mut self, value: HashRef<String>) -> usize;

            fn add_value(&mut self, value: String) -> usize;

            /// Adds any `value` that can be converted into `T`
            fn add<V: Into<String>>(&mut self, value: V) -> usize;

            /// Retrieves the id for `value`
            fn get_id<Q: ?Sized>(&self, value: &Q) -> Option<usize>
                where
                    String: Borrow<Q>,
                    Q: Hash + Eq;

            /// Retrieves the id for `value`
            fn get_hash_ref<Q: ?Sized>(&self, value: &Q) -> Option<&HashRef<String>>
                where
                    String: Borrow<Q>,
                    Q: Hash + Eq;

            /// Retrieves the complete entry for `value` in the vocabulary, if it exists
            fn get_entry_id<Q: ?Sized>(&self, value: &Q) -> Option<(&HashRef<String>, &usize)>
                where
                    String: Borrow<Q>,
                    Q: Hash + Eq;

            fn contains<Q: ?Sized>(&self, value: &Q) -> bool
                where
                    String: Borrow<Q>,
                    Q: Hash + Eq;
        }
    }
}
impl StoreableVocabulary<String> for PyVocabulary {
    fn save_to_output(&self, writer: &mut impl Write) -> std::io::Result<usize> {
        self.inner.save_to_output(writer)
    }
}

impl LoadableVocabulary<String, Infallible> for PyVocabulary {}

impl Display for PyVocabulary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl<T> From<Vec<T>> for PyVocabulary where T: Into<String> {
    fn from(value: Vec<T>) -> Self {
        Self { inner: VocabularyImpl::from(value.into_iter().map(|value| value.into()).collect::<Vec<_>>()) }
    }
}

impl From<VocabularyImpl<String>> for PyVocabulary {
    fn from(inner: VocabularyImpl<String>) -> Self {
        Self { inner }
    }
}

pub(crate) fn vocabulary_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVocabulary>()?;
    Ok(())
}