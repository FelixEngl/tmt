use std::sync::{Arc, OnceLock, RwLock};
use serde::{Deserialize, Serialize};
use crate::topicmodel::dictionary::DictionaryWithVocabulary;
use crate::topicmodel::dictionary::search::impls::trie::TrieSearcher;
use crate::topicmodel::vocabulary::BasicVocabulary;
use crate::toolkit::once_serializer::OnceLockDef;
use crate::topicmodel::dictionary::direction::LanguageKind;


pub type ShareableTrieSearcher = Arc<RwLock<TrieSearcher>>;
pub type ShareableTrieSearcherRef<'a> = &'a Arc<RwLock<TrieSearcher>>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    prefix_len: Option<usize>,
    #[serde(with="OnceLockDef")]
    searcher_a: OnceLock<ShareableTrieSearcher>,
    #[serde(with="OnceLockDef")]
    searcher_b: OnceLock<ShareableTrieSearcher>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            prefix_len: None,
            searcher_a: Default::default(),
            searcher_b: Default::default(),
        }
    }

    fn get_or_init_trie_searcher_for<'a, V, T>(&self, targ: &'a OnceLock<Arc<RwLock<TrieSearcher>>>, voc: &V, language: LanguageKind) -> ShareableTrieSearcherRef<'a>
    where
        V: BasicVocabulary<T>,
        T: AsRef<str> + Send + Sync,
    {
        let provided = targ.get_or_init(|| {
            Arc::new(RwLock::new(
                TrieSearcher::new(voc, language, self.prefix_len).expect("We always create a valid trie from a dictionary provided voc.")
            ))
        });
        {
            let read = provided.read().unwrap();
            if read.is_valid_fast(self.prefix_len, voc) {
                return provided;
            }
        }
        {
            log::debug!("re-initialize the index");
            let mut write = provided.write().unwrap();
            *write = TrieSearcher::new(voc, language, self.prefix_len).expect("We always create a valid trie from a dictionary provided voc.");
        }
        provided
    }

    fn get_trie_searcher_for<'a>(&self, targ: &'a OnceLock<Arc<RwLock<TrieSearcher>>>) -> Option<ShareableTrieSearcherRef<'a>>
    {
        let provided = targ.get()?;
        {
            let read = provided.read().unwrap();
            if read.prefix_length() == self.prefix_len {
                return Some(provided);
            }
        }
        None
    }

    pub fn get_or_init_trie_searcher_a<D, V, T>(&self, dict: &D) -> ShareableTrieSearcherRef
    where
        D: DictionaryWithVocabulary<T, V> + ?Sized,
        V: BasicVocabulary<T>,
        T: AsRef<str> + Send + Sync,
    {
        self.get_or_init_trie_searcher_for(
            &self.searcher_a,
            dict.voc_a(),
            LanguageKind::A
        )
    }

    pub fn get_or_init_trie_searcher_b<D, V, T>(&self, dict: &D) -> ShareableTrieSearcherRef
    where
        D: DictionaryWithVocabulary<T, V> + ?Sized,
        V: BasicVocabulary<T>,
        T: AsRef<str> + Send + Sync,
    {
        self.get_or_init_trie_searcher_for(
            &self.searcher_b,
            dict.voc_b(),
            LanguageKind::B
        )
    }
    
    pub fn get_or_init_trie_searcher<D, V, T>(&self, dict: &D, language: LanguageKind) -> ShareableTrieSearcherRef
    where
        D: DictionaryWithVocabulary<T, V> + ?Sized,
        V: BasicVocabulary<T>,
        T: AsRef<str> + Send + Sync,
    {
        match language {
            LanguageKind::A => {
                self.get_or_init_trie_searcher_a(dict)
            }
            LanguageKind::B => {
                self.get_or_init_trie_searcher_b(dict)
            }
        }
    }

    pub fn get_or_init_both_trie_searcher<D, V, T>(&self, dict: &D) -> (ShareableTrieSearcherRef, ShareableTrieSearcherRef)
    where
        D: DictionaryWithVocabulary<T, V> + ?Sized,
        V: BasicVocabulary<T>,
        T: AsRef<str> + Send + Sync,
    {
        std::thread::scope(|scope| {
            let a = scope.spawn(|| {
                self.get_or_init_trie_searcher_a(dict)
            });
            let b = scope.spawn(|| {
                self.get_or_init_trie_searcher_b(dict)
            });
            a.join().and_then(|value_a| b.join().map(|value_b| (value_a, value_b)))
        }).expect("Calculating these values should never fail.")
    }
    
    

    pub fn get_trie_searcher_a(&self) ->Option<ShareableTrieSearcherRef>
    {
        self.get_trie_searcher_for(&self.searcher_a)
    }

    pub fn get_trie_searcher_b(&self) -> Option<ShareableTrieSearcherRef>
    {
        self.get_trie_searcher_for(&self.searcher_b)
    }

    pub fn get_trie_searcher(&self, language: LanguageKind) -> Option<ShareableTrieSearcherRef>
    {
        match language {
            LanguageKind::A => {
                self.get_trie_searcher_a()
            }
            LanguageKind::B => {
                self.get_trie_searcher_b()
            }
        }
    }

    pub fn get_both_trie_searchers(&self) -> (Option<ShareableTrieSearcherRef>, Option<ShareableTrieSearcherRef>)
    {
        (self.get_trie_searcher_a(), self.get_trie_searcher_b())
    }

    pub fn prefix_len(&self) -> Option<usize> {
        self.prefix_len
    }
}
