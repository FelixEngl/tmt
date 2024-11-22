use std::borrow::Borrow;
use crate::toolkit::rw_ext::RWLockUnwrapped;
use crate::topicmodel::dictionary::direction::LanguageKind;
use crate::topicmodel::dictionary::search::impls::scanning::{
    ScanAlgorithm, ScanSearcher, ScanSearcherInitError, ScanSearcherOptions,
    ScanSearcherOptionsInitError,
};
use crate::topicmodel::dictionary::search::impls::trie::TrieSearcher;
use crate::topicmodel::dictionary::search::index::{SearchIndex, ShareableTrieSearcherRef};
use crate::topicmodel::dictionary::search::{SearchInput, SearchType};
use crate::topicmodel::dictionary::DictionaryWithVocabulary;
use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary};
use either::Either;
use itertools::{EitherOrBoth, Itertools};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::RwLockReadGuard;
use pyo3::exceptions::PyValueError;
use pyo3::PyErr;
use thiserror::Error;


/// A lightweight searcher to combine the dictionary and the search index
#[derive(Debug, Copy, Clone)]
pub struct DictionarySearcher<'a, D: ?Sized, V, T, I = SearchIndex>
where
    D: DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T>
{
    dictionary: &'a D,
    index: &'a I,
    _phantom: PhantomData<fn(V) -> T>,
}

impl<'a, D: ?Sized, V, T, I> DictionarySearcher<'a, D, V, T, I>
where
    D: DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T>
{
    pub fn new(dictionary: &'a D, index: &'a I) -> Self {
        Self {
            dictionary,
            index,
            _phantom: PhantomData,
        }
    }
}

type SearchResult = Option<
    Either<
        EitherOrBoth<Vec<(usize, String)>>,
        EitherOrBoth<HashMap<String, Vec<(String, Vec<usize>)>>>,
    >,
>;

type TrieRefs<'a> = (ShareableTrieSearcherRef<'a>, ShareableTrieSearcherRef<'a>);
type ShareableTrieSearcherAccess<'a> = RwLockReadGuard<'a, TrieSearcher>;

impl<'a, D, V, T> DictionarySearcher<'a, D, V, T, SearchIndex>
where
    D: DictionaryWithVocabulary<T, V> + ?Sized,
    V: SearchableVocabulary<T>,
    T: AsRef<str> + Send + Sync + Borrow<str> + Eq + Hash + Clone,
{
    fn get_trie_searcher_a(&self) -> ShareableTrieSearcherRef<'a> {
        self.index.get_or_init_trie_searcher_a(self.dictionary)
    }

    fn get_trie_searcher_b(&self) -> ShareableTrieSearcherRef<'a> {
        self.index.get_or_init_trie_searcher_b(self.dictionary)
    }

    fn get_trie_searchers(&self) -> (ShareableTrieSearcherRef<'a>, ShareableTrieSearcherRef<'a>) {
        self.index.get_or_init_both_trie_searcher(self.dictionary)
    }

    pub fn force_init(&self) {
        self.get_trie_searchers();
    }

    /// Returns true iff the
    fn searcher_is_init_and_exact(&self) -> bool
    {
        match self.index.get_both_trie_searchers() {
            (Some(a), Some(b)) => {
                let ra = a.read_unwrapped();
                let rb = b.read_unwrapped();
                ra.is_exact()
                    && rb.is_exact()
                    && ra.is_valid_fast(self.index.prefix_len(), self.dictionary.voc_a())
                    && rb.is_valid_fast(self.index.prefix_len(), self.dictionary.voc_b())
            }
            _ => false,
        }
    }

    fn execute_with_searcher<F, R, E>(&self, method: F) -> Result<R, SearchError>
    where
        F: for<'b> FnOnce(TrieRefs<'b>) -> Result<R, E>,
        SearchError: From<E>,
    {
        Ok(method(self.get_trie_searchers())?)
    }

    fn execute_on_target<F>(
        &self,
        input: SearchInput,
        target_language: Option<LanguageKind>,
        search_function: F,
    ) -> Result<
        (
            Option<HashMap<String, Vec<(String, Vec<usize>)>>>,
            Option<HashMap<String, Vec<(String, Vec<usize>)>>>,
        ),
        SearchError,
    >
    where
        F: for<'b> Fn(
            &'b ShareableTrieSearcherAccess<'a>,
            &str,
        ) -> Result<Vec<(String, &'b [usize])>, SearchError>,
    {
        match target_language {
            None => {
                let search = self.get_trie_searchers();
                let search_a = search.0.read_unwrapped();
                let search_b = search.1.read_unwrapped();
                match input {
                    SearchInput::String(value) => {
                        let result = (&search_function)(&search_a, &value)?;
                        let result_a = if result.is_empty() {
                            None
                        } else {
                            let mut result_a = HashMap::with_capacity(1);
                            result_a.insert(
                                value.clone(),
                                result
                                    .into_iter()
                                    .map(|(a, b)| (a, b.to_vec()))
                                    .collect::<Vec<_>>(),
                            );
                            Some(result_a)
                        };
                        let result = (&search_function)(&search_b, &value)?;
                        let result_b = if result.is_empty() {
                            None
                        } else {
                            let mut result_b = HashMap::with_capacity(1);
                            result_b.insert(
                                value.clone(),
                                result
                                    .into_iter()
                                    .map(|(a, b)| (a, b.to_vec()))
                                    .collect::<Vec<_>>(),
                            );
                            Some(result_b)
                        };
                        Ok((result_a, result_b))
                    }
                    SearchInput::Many(values) => {
                        let mut result_a = HashMap::with_capacity(values.len());
                        let mut result_b = HashMap::with_capacity(values.len());
                        for value in values {
                            if !result_a.contains_key(&value) {
                                let result = (&search_function)(&search_a, &value)?;
                                if !result.is_empty() {
                                    result_a.insert(
                                        value.clone(),
                                        result
                                            .into_iter()
                                            .map(|(a, b)| (a, b.to_vec()))
                                            .collect::<Vec<_>>(),
                                    );
                                }
                            }
                            if result_b.contains_key(&value) {
                                continue;
                            }
                            let result = (&search_function)(&search_b, &value)?;
                            if !result.is_empty() {
                                result_b.insert(
                                    value,
                                    result
                                        .into_iter()
                                        .map(|(a, b)| (a, b.to_vec()))
                                        .collect::<Vec<_>>(),
                                );
                            }
                        }
                        Ok((
                            (!result_a.is_empty()).then_some(result_a),
                            (!result_b.is_empty()).then_some(result_b),
                        ))
                    }
                    _ => unreachable!(),
                }
            }
            Some(LanguageKind::A) => {
                let search = self.get_trie_searcher_a();
                let search_a = search.read_unwrapped();
                match input {
                    SearchInput::String(value) => {
                        let result = (&search_function)(&search_a, &value)?;
                        if result.is_empty() {
                            Ok((None, None))
                        } else {
                            let mut result_a = HashMap::with_capacity(1);
                            result_a.insert(
                                value,
                                result
                                    .into_iter()
                                    .map(|(a, b)| (a, b.to_vec()))
                                    .collect::<Vec<_>>(),
                            );
                            Ok((Some(result_a), None))
                        }
                    }
                    SearchInput::Many(values) => {
                        let mut result_a = HashMap::with_capacity(values.len());
                        for value in values {
                            if result_a.contains_key(&value) {
                                continue;
                            }
                            let result = (&search_function)(&search_a, &value)?;
                            if !result.is_empty() {
                                result_a.insert(
                                    value,
                                    result
                                        .into_iter()
                                        .map(|(a, b)| (a, b.to_vec()))
                                        .collect::<Vec<_>>(),
                                );
                            }
                        }
                        if result_a.is_empty() {
                            Ok((None, None))
                        } else {
                            Ok((Some(result_a), None))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Some(LanguageKind::B) => {
                let search = self.get_trie_searcher_b();
                let search_b = search.read_unwrapped();
                match input {
                    SearchInput::String(value) => {
                        let result = (&search_function)(&search_b, &value)?;
                        if result.is_empty() {
                            Ok((None, None))
                        } else {
                            let mut result_b = HashMap::with_capacity(1);
                            result_b.insert(
                                value,
                                result
                                    .into_iter()
                                    .map(|(a, b)| (a, b.to_vec()))
                                    .collect::<Vec<_>>(),
                            );
                            Ok((None, Some(result_b)))
                        }
                    }
                    SearchInput::Many(values) => {
                        let mut result_b = HashMap::with_capacity(values.len());
                        for value in values {
                            if result_b.contains_key(&value) {
                                continue;
                            }
                            let result = (&search_function)(&search_b, &value)?;
                            if !result.is_empty() {
                                result_b.insert(
                                    value,
                                    result
                                        .into_iter()
                                        .map(|(a, b)| (a, b.to_vec()))
                                        .collect::<Vec<_>>(),
                                );
                            }
                        }
                        if result_b.is_empty() {
                            Ok((None, None))
                        } else {
                            Ok((None, Some(result_b)))
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// The general search function.
    pub fn search<'py>(
        &self,
        input: impl Into<SearchInput<'py>>,
        search_type: Option<SearchType>,
        target_language: Option<LanguageKind>,
        threshold: Option<Either<usize, f64>>,
        ignores_ascii_case: bool,
    ) -> Result<SearchResult, SearchError> {
        let input = input.into();

        fn pack_as_search_result(
            value: (
                Option<HashMap<String, Vec<(String, Vec<usize>)>>>,
                Option<HashMap<String, Vec<(String, Vec<usize>)>>>,
            ),
        ) -> SearchResult {
            match value {
                (Some(a), Some(b)) => {
                    let value = if a.is_empty() && b.is_empty() {
                        return None;
                    } else if b.is_empty() {
                        EitherOrBoth::Left(a)
                    } else if a.is_empty() {
                        EitherOrBoth::Right(b)
                    } else {
                        EitherOrBoth::Both(a, b)
                    };
                    Some(Either::Right(value))
                }
                (Some(a), None) => {
                    if a.is_empty() {
                        None
                    } else {
                        Some(Either::Right(EitherOrBoth::Left(a)))
                    }
                }
                (None, Some(b)) => {
                    if b.is_empty() {
                        None
                    } else {
                        Some(Either::Right(EitherOrBoth::Right(b)))
                    }
                }
                _ => None,
            }
        }

        fn pack_as_primitive_search_result(
            value: (
                Option<HashMap<String, Vec<(String, Vec<usize>)>>>,
                Option<HashMap<String, Vec<(String, Vec<usize>)>>>,
            ),
        ) -> SearchResult {
            pack_as_search_result(value)
        }

        fn pack_primitive_search_result<T: AsRef<str>>(
            value: Vec<(LanguageKind, usize, T)>,
        ) -> SearchResult {
            let (a, b) = value
                .into_iter()
                .map(|(a, b, c)| (a, b, c.as_ref().to_string()))
                .partition::<Vec<_>, _>(|(lang, _, _)| lang.is_a());
            let result = if a.is_empty() && b.is_empty() {
                return None;
            } else if b.is_empty() {
                EitherOrBoth::Left(a)
            } else if a.is_empty() {
                EitherOrBoth::Right(b)
            } else {
                EitherOrBoth::Both(a, b)
            };
            let result = result.map_any(
                |value| value.into_iter().map(|(_, a, b)| (a, b)).collect_vec(),
                |value| value.into_iter().map(|(_, a, b)| (a, b)).collect_vec(),
            );
            Some(Either::Left(result))
        }

        let options = if input.is_method() {
            ScanSearcherOptions::new(
                ScanAlgorithm::Matcher,
                target_language,
                threshold,
                ignores_ascii_case,
            )?
        } else {
            match search_type.unwrap_or_default() {
                SearchType::Autocomplete => {
                    if ignores_ascii_case {
                        return Err(SearchError::IgnoreAsciiNotSupported("Autocompletion"));
                    } else {
                        let result =
                            self.execute_on_target(input, target_language, |search, query| {
                                Ok(search
                                    .predict_for_prefix(query)
                                    .into_iter()
                                    .map(|(a, b)| (a, b.deref()))
                                    .collect_vec())
                            })?;
                        return Ok(pack_as_search_result(result));
                    }
                }
                SearchType::ExactMatch
                    if !ignores_ascii_case && self.searcher_is_init_and_exact() =>
                {
                    let result =
                        self.execute_on_target(input, target_language, |search, query| {
                            Ok(
                                match search.search_exact(query).and_then(|value| value.exact()) {
                                    None => Vec::with_capacity(0),
                                    Some(value) => Vec::from([(query.to_string(), value)]),
                                },
                            )
                        })?;
                    return Ok(pack_as_primitive_search_result(result));
                }
                SearchType::StartsWith
                    if !ignores_ascii_case && self.searcher_is_init_and_exact() =>
                {
                    let result =
                        self.execute_on_target(input, target_language, |search, query| {
                            Ok(search
                                .predict_for_prefix(query)
                                .into_iter()
                                .map(|(a, b)| (a, b.deref()))
                                .collect_vec())
                        })?;
                    return Ok(pack_as_primitive_search_result(result));
                }
                SearchType::EndsWith
                    if !ignores_ascii_case && self.searcher_is_init_and_exact() =>
                {
                    let result =
                        self.execute_on_target(input, target_language, |search, query| {
                            Ok(search
                                .search_for_postfix(query)
                                .into_iter()
                                .map(|(a, b)| (a, b.deref()))
                                .collect_vec())
                        })?;
                    return Ok(pack_as_primitive_search_result(result));
                }
                SearchType::CommonPrefix => {
                    let result =
                        self.execute_on_target(input, target_language, |search, query| {
                            Ok(search
                                .search_for_common_prefix(query)
                                .into_iter()
                                .map(|(a, b)| (a, b.deref()))
                                .collect_vec())
                        })?;
                    return Ok(pack_as_search_result(result));
                }
                rest => ScanSearcherOptions::new(
                    rest.try_into().expect("This conversion should never fail!"),
                    target_language,
                    threshold,
                    ignores_ascii_case,
                )?,
            }
        };

        Ok(pack_primitive_search_result(
            ScanSearcher::new(input, options)?.search(self.dictionary),
        ))
    }
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    PrimitiveSearcherOption(#[from] ScanSearcherOptionsInitError),
    #[error(transparent)]
    PrimitiveSearcher(#[from] ScanSearcherInitError),
    #[error("The method {0} does not support ignoring ascii case!")]
    IgnoreAsciiNotSupported(&'static str),
}


impl From<SearchError> for PyErr {
    fn from(value: SearchError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}