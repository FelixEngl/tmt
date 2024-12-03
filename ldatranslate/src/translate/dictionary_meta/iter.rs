use std::ops::Range;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, META_DICT_ARRAY_LENTH};
use crate::translate::dictionary_meta::dict_meta::SparseMetaVector;

pub struct IterSorted<'a> {
    vector: &'a SparseMetaVector,
    pos: Range<usize>
}

impl<'a> IterSorted<'a> {
    pub fn new(vector: &'a SparseMetaVector) -> Self {
        Self { vector, pos: 0..META_DICT_ARRAY_LENTH }
    }
}

impl<'a> Iterator for IterSorted<'a> {
    type Item = (DictMetaTagIndex, f64);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.pos.next()?;
            if let Some(idx) = self.vector.template.mapping[next] {
                let key = self.vector.template[idx];
                let value = self.vector.inner[idx];
                break Some((key, value))
            }
        }
    }
}

pub struct Iter<'a> {
    vector: &'a SparseMetaVector,
    pos: Range<usize>
}

impl<'a> Iter<'a> {
    pub fn new(vector: &'a SparseMetaVector) -> Self {
        Self { vector, pos: 0..vector.template.len() }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (DictMetaTagIndex, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.pos.next()?;
        let key = self.vector.template[next];
        let value = self.vector.inner[next];
        Some((key, value))
    }
}

