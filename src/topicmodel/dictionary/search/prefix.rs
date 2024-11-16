use std::collections::HashMap;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use itertools::Itertools;
use nom::AsBytes;
use rayon::prelude::*;
use trie_rs::map::TrieBuilder;
use crate::topicmodel::dictionary::DictionaryWithVocabulary;
use crate::topicmodel::reference::HashRefSlice;
use crate::topicmodel::vocabulary::{BasicVocabulary};



#[derive(Debug)]
pub struct PrefixDictSearch {
    is_complete: bool,
    search_a: trie_rs::map::Trie<u8, Vec<usize>>,
    search_b: trie_rs::map::Trie<u8, Vec<usize>>,
}

impl PrefixDictSearch {

    pub fn search_in_a_for_all_common_prefix<S, Q, M>(&self, prefix: Q) -> Vec<(S, &Vec<usize>)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>
    {
        self.search_a.common_prefix_search(prefix).collect_vec()
    }

    pub fn search_in_b_for_all_common_prefix<S, Q, M>(&self, prefix: Q) -> Vec<(S, &Vec<usize>)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>
    {
        self.search_b.common_prefix_search(prefix).collect_vec()
    }

    pub fn search_in_a_for_all_as_prediction<S, Q, M>(&self, prefix: Q) -> Vec<(S, &Vec<usize>)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>
    {
        self.search_a.predictive_search(prefix).collect_vec()
    }

    pub fn search_in_b_for_all_as_prediction<S, Q, M>(&self, prefix: Q) -> Vec<(S, &Vec<usize>)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>
    {
        self.search_b.predictive_search(prefix).collect_vec()
    }

    pub fn search_in_a_for_all_with_suffix<S, Q, M>(&self, prefix: Q) -> Vec<(S, &Vec<usize>)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>
    {
        self.search_a.postfix_search(prefix).collect_vec()
    }

    pub fn search_in_b_for_all_with_suffix<S, Q, M>(&self, prefix: Q) -> Vec<(S, &Vec<usize>)>
    where
        Q: AsRef<[u8]>,
        S: Clone + trie_rs::try_collect::TryFromIterator<u8, M>
    {
        self.search_b.postfix_search(prefix).collect_vec()
    }

    pub fn search_in_a_exact<Q>(&self, prefix: Q) -> Option<&Vec<usize>>
    where
        Q: AsRef<[u8]>
    {
        self.search_a.exact_match(prefix)
    }

    pub fn search_in_b_exact<Q>(&self, prefix: Q) ->Option<&Vec<usize>>
    where
        Q: AsRef<[u8]>
    {
        self.search_b.exact_match(prefix)
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }
}

impl PrefixDictSearch
{
    fn generate_slice_map<V: BasicVocabulary<String>>(voc: &V, prefix_length: Option<usize>) -> (bool, HashMap<HashRefSlice<String, str>, Vec<usize>>) {
        let had_a_trunkate = AtomicBool::new(false);
        let result = voc.as_ref()
            .par_iter()
            .enumerate()
            .map(|(id, value)| {
                if let Some(prefix_length) = prefix_length {
                    let slice = if let Some((pos, _)) = value.char_indices().skip(prefix_length).next() {
                        had_a_trunkate.store(true, Ordering::Relaxed);
                        value.slice_owned(..pos)
                    } else {
                        value.slice_owned(..)
                    };
                    (slice, id)
                } else {
                    (value.slice_owned(..), id)
                }
            })
            .collect_vec_list()
            .into_iter()
            .flatten()
            .into_group_map();
        (had_a_trunkate.load(Ordering::SeqCst), result)
    }

    /// If no prefix is set the whole vocabulary is indexed.
    pub fn new<D, V>(associated_dict: &D, prefix_length: Option<usize>) -> Self
    where
        D: DictionaryWithVocabulary<String, V>,
        V: BasicVocabulary<String>
    {
        let (was_trunkated_a, values_a) = Self::generate_slice_map(associated_dict.voc_a(), prefix_length);
        let (was_trunkated_b, values_b) = Self::generate_slice_map(associated_dict.voc_b(), prefix_length);

        let (a, b) = std::thread::scope(|scope|{
            let a = scope.spawn(|| {
                let mut new_a: TrieBuilder<u8, Vec<usize>> = TrieBuilder::new();
                for (label, value) in values_a {
                    new_a.push(
                        label.as_bytes(),
                        value
                    )
                }
                new_a.build()
            });

            let b = scope.spawn(|| {
                let mut new_b: TrieBuilder<u8, Vec<usize>> = TrieBuilder::new();
                for (label, value) in values_b {
                    new_b.push(
                        label.as_bytes(),
                        value
                    )
                }
                new_b.build()
            });

            (a.join(), b.join())
        });

        let a = match a {
            Ok(value) => {
                value
            }
            Err(e) => {
                panic::resume_unwind(e)
            }
        };


        let b = match b {
            Ok(value) => {
                value
            }
            Err(e) => {
                panic::resume_unwind(e)
            }
        };

        Self {
            search_a: a,
            search_b: b,
            is_complete: !(was_trunkated_a || was_trunkated_b)
        }
    }


}



