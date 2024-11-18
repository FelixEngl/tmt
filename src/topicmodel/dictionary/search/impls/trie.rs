use std::cmp::{min};
use crate::topicmodel::dictionary::direction::LanguageKind;
use crate::topicmodel::vocabulary::BasicVocabulary;
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet};
use std::ops::{Deref, Range};
use thiserror::Error;
use trie_rs::map::TrieBuilder;

/// Uses a trie to search in the vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrieSearcher {
    is_exact: bool,
    target_voc: LanguageKind,
    voc_len: usize,
    prefix_length: Option<usize>,
    search: trie_rs::map::Trie<u8, Entry>,
}

impl TrieSearcher {
    /// If no prefix is set the whole vocabulary is indexed.
    pub fn new<V>(
        voc: &V,
        language: LanguageKind,
        prefix_length: Option<usize>,
    ) -> Result<Self, EmptyEntryBuilderError>
    where
        V: BasicVocabulary<String>,
    {
        let result = voc
            .as_ref()
            .par_iter()
            .enumerate()
            .filter(|(_, value)| !value.is_empty())
            .map(|(id, value)| {
                if let Some(prefix_length) = prefix_length {
                    if let Some((pos, _)) = value.char_indices().skip(prefix_length).next() {
                        (value.slice_owned(..pos), (id, false))
                    } else {
                        (value.slice_owned(..), (id, true))
                    }
                } else {
                    (value.slice_owned(..), (id, true))
                }
            })
            .collect_vec_list()
            .into_iter()
            .flatten()
            .into_grouping_map()
            .fold_with(
                |_, _| Entry::builder(),
                |acc, _, (value, is_exact)| acc.insert(value, is_exact),
            );

        let mut new: TrieBuilder<u8, Entry> = TrieBuilder::new();
        let mut is_exact = true;
        for (label, builder) in result {
            let entry = builder.build()?;
            is_exact = is_exact && entry.is_exclusive_exact();
            new.push(label.as_bytes(), entry)
        }
        Ok(Self {
            search: new.build(),
            voc_len: voc.len(),
            is_exact,
            prefix_length,
            target_voc: language,
        })
    }

    /// Returns true if the searcher is valid for the provided prefix and vocabulary.
    /// This check is relatively simple, it only checks for vocabulary lengths.
    ///
    /// For a better test use [is_valid].
    pub fn is_valid_fast<V, T>(&self, prefix_length: Option<usize>, voc: &V) -> bool
    where
        V: BasicVocabulary<T>
    {
        self.prefix_length == prefix_length && self.voc_len == voc.len()
    }


    /// Returns true iff the searcher is valid for the provided prefix and vocabulary.
    /// This check is relatively simple, it only checks for vocabulary lengths.
    pub fn is_valid<V>(&self, prefix_length: Option<usize>, voc: &V) -> bool
    where
        V: BasicVocabulary<String>
    {
        self.is_valid_fast(prefix_length, voc) && {
            voc.as_ref().par_iter().enumerate().all(|(idx, value)| {
                if let Some(prefix_length) = prefix_length {
                    self.search.exact_match(&value[..min(prefix_length, value.len())])
                } else {
                    self.search.exact_match(value.as_str())
                }.is_some_and(|value| value.contains(&idx))
            })
        }
    }


    pub fn search_for_common_prefix<S, Q, M>(&self, prefix: Q) -> Vec<(S, &Entry)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>,
    {
        self.search.common_prefix_search(prefix).collect_vec()
    }

    pub fn predict_for_prefix<S, Q, M>(&self, prefix: Q) -> Vec<(S, &Entry)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>,
    {
        self.search.predictive_search(prefix).collect_vec()
    }

    pub fn search_for_postfix<S, Q, M>(&self, postfix: Q) -> Vec<(S, &Entry)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>,
    {
        self.search.postfix_search(postfix).collect_vec()
    }

    pub fn search_exact<Q>(&self, prefix: Q) -> Option<&Entry>
    where
        Q: AsRef<[u8]>,
    {
        self.search.exact_match(prefix)
    }

    pub fn is_exact(&self) -> bool {
        self.is_exact
    }
    
    pub fn target_voc(&self) -> LanguageKind {
        self.target_voc
    }
    
    pub fn prefix_length(&self) -> Option<usize> {
        self.prefix_length
    }

    pub fn voc_len(&self) -> usize {
        self.voc_len
    }
}

/// An entry in a trie searcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    values: Vec<usize>,
    first_index_of_prefix: usize,
}

impl Entry {
    pub fn builder() -> EntryBuilder {
        EntryBuilder::default()
    }

    fn new<I1, I2>(
        exact: impl IntoIterator<Item = usize, IntoIter = I1>,
        prefix: impl IntoIterator<Item = usize, IntoIter = I2>,
    ) -> Entry
    where
        I1: ExactSizeIterator + Iterator<Item = usize>,
        I2: ExactSizeIterator + Iterator<Item = usize>,
    {
        let exact = exact.into_iter();
        let prefix = prefix.into_iter();
        let first_index_of_prefix = exact.len();
        let mut values = Vec::with_capacity(exact.len() + prefix.len());
        values.extend(exact);
        values.extend(prefix);
        assert!(
            !values.is_empty(),
            "Tried to initialize an empty Entry for a search trie. That is illegal!"
        );
        assert!(
            first_index_of_prefix <= values.len(),
            "The partition can NOT be grater than the contained values!"
        );
        Self {
            values,
            first_index_of_prefix,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn len_exact(&self) -> usize {
        self.first_index_of_prefix
    }

    pub fn len_prefix(&self) -> usize {
        self.values.len() - self.first_index_of_prefix
    }
    
    /// True if the entry has one or more exact value
    pub fn has_exact(&self) -> bool {
        self.first_index_of_prefix != 0
    }

    /// True if it is ONLY exact
    pub fn is_exclusive_exact(&self) -> bool {
        self.values.len() == self.first_index_of_prefix
    }

    /// True if it is ONLY prefix
    pub fn is_exclusive_prefix(&self) -> bool {
        0 == self.first_index_of_prefix
    }
    
    /// True if the entry has prefix values
    pub fn has_prefix(&self) -> bool {
        self.values.len() != self.first_index_of_prefix
    }

    /// Returns the exact entries associated in the trie. Usually 1, but we want to make sure,
    /// that we can support future changes like ignoring "to" etc.
    pub fn exact(&self) -> Option<&[usize]> {
        self.has_exact().then(|| &self.values[..self.first_index_of_prefix])
    }

    /// All values where this entry is only a prefix.
    pub fn prefixed(&self) -> Option<&[usize]> {
        self.has_prefix().then(|| &self.values[self.first_index_of_prefix..])
    }

    pub fn as_slice(&self) -> &[usize] {
        self.values.as_slice()
    }

    pub fn indices_exact(&self) -> Range<usize> {
        0..self.first_index_of_prefix
    }

    pub fn indices_prefix(&self) -> Range<usize> {
        self.first_index_of_prefix..self.values.len()
    }
}

impl Deref for Entry {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

/// A builder for an entry.
#[derive(Default)]
pub struct EntryBuilder {
    exact: HashSet<usize>,
    prefix: HashSet<usize>,
}

impl EntryBuilder {
    pub fn add(&mut self, value: usize, is_exact: bool) {
        if is_exact {
            self.exact.insert(value);
        } else {
            self.prefix.insert(value);
        }
    }

    pub fn insert(mut self, value: usize, is_exact: bool) -> Self {
        self.add(value, is_exact);
        self
    }

    pub fn build(self) -> Result<Entry, EmptyEntryBuilderError> {
        if self.exact.is_empty() && self.prefix.is_empty() {
            Err(EmptyEntryBuilderError)
        } else {
            Ok(Entry::new(self.exact.into_iter(), self.prefix.into_iter()))
        }
    }
}

#[derive(Debug, Error)]
#[error("Can not build an entry from an empty entry builder!")]
pub struct EmptyEntryBuilderError;
