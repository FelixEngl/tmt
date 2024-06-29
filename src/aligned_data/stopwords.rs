use std::borrow::{Borrow, Cow};
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;
use compact_str::{CompactString, ToCompactString};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct StopWordList {
    raw: HashSet<CompactString>,
    normalized: HashSet<CompactString>
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ContainsKind {
    Raw,
    Normalized,
    Both
}

impl StopWordList {

    pub fn new(mut raw: HashSet<CompactString>, mut normalized: HashSet<CompactString>) -> Self {
        raw.shrink_to_fit();
        normalized.shrink_to_fit();
        Self { raw, normalized }
    }

    pub fn from(raw: HashSet<CompactString>) -> Self {
        let normalized = raw
            .iter()
            .map(|value| value.nfc().collect::<CompactString>())
            .collect::<HashSet<_>>();
        Self::new(raw, normalized)
    }

    pub fn extend_with(&mut self, other: Self) {
        self.raw.extend(other.raw);
        self.normalized.extend(other.normalized);
        self.raw.shrink_to_fit();
        self.normalized.shrink_to_fit();
    }

    #[inline]
    pub fn contains<Q: ?Sized>(&self, kind: ContainsKind, value: &Q) -> bool
        where
            CompactString: Borrow<Q>,
            Q: Hash + Eq, {
        match kind {
            ContainsKind::Raw => {self.contains_raw(value)}
            ContainsKind::Normalized => {self.contains_normalized(value)}
            ContainsKind::Both => {self.contains_both(value)}
        }
    }

    #[inline]
    pub fn contains_both<Q: ?Sized>(&self, value: &Q) -> bool
        where
            CompactString: Borrow<Q>,
            Q: Hash + Eq, {
        self.contains_raw(value) || self.contains_normalized(value)
    }

    #[inline]
    pub fn contains_raw<Q: ?Sized>(&self, value: &Q) -> bool
        where
            CompactString: Borrow<Q>,
            Q: Hash + Eq, {
        self.raw.contains(value)
    }

    #[inline]
    pub fn contains_normalized<Q: ?Sized>(&self, value: &Q) -> bool
        where
            CompactString: Borrow<Q>,
            Q: Hash + Eq, {
        self.normalized.contains(value)
    }
}

impl<Q> Extend<Q> for StopWordList where Q: ToCompactString {
    fn extend<T: IntoIterator<Item=Q>>(&mut self, iter: T) {
        for value in iter.into_iter() {
            let word = value.to_compact_string();
            let normalized = word.nfc().to_compact_string();
            self.raw.insert(word);
            self.normalized.insert(normalized);
        }
        self.raw.shrink_to_fit();
        self.normalized.shrink_to_fit();
    }
}

