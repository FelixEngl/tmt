use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use derive_more::From;
use either::Either;
use itertools::EitherOrBoth;
use pyo3::FromPyObject;
use strum::EnumIs;
use thiserror::Error;
use crate::define_py_method;
use crate::toolkit::sync_ext::OwnedOrArcRw;
use crate::topicmodel::dictionary::DictionaryWithVocabulary;
use crate::topicmodel::dictionary::direction::LanguageKind;
use crate::topicmodel::dictionary::search::primitive::{SearchKind, SearchOption, SearchOptionError, Searcher};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{BasicVocabulary, SearchableVocabulary};

mod primitive;
mod prefix;




#[derive(From, Debug, FromPyObject, EnumIs)]
pub enum SearchInput<'py> {
    String(String),
    Many(Vec<String>),
    Method(MatchWordMethod<'py>),
}

define_py_method! {
    MatchWord(language_hing: LanguageKind, word: &str) -> bool
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct SearchInfoVersion(usize, usize, Option<usize>);

#[derive(Debug)]
pub struct DictionarySearcher<D, V> {
    dictionary: OwnedOrArcRw<D>,
    prefix_len: Option<usize>,
    searcher: std::sync::OnceLock<Arc<RwLock<(SearchInfoVersion, prefix::PrefixDictSearch)>>>,
    _phantom: PhantomData<V>
}


type SearchResult = Option<EitherOrBoth<SingleSearchResult, SingleSearchResult>>;
type SingleSearchResult = ();

impl<D, V> DictionarySearcher<D, V>
where
    D: DictionaryWithVocabulary<String, V>,
    V: SearchableVocabulary<String>
{
    pub fn new(value: impl Into<OwnedOrArcRw<D>>) -> Self {
        Self {
            dictionary: value.into(),
            prefix_len: None,
            searcher: Default::default(),
            _phantom: PhantomData
        }
    }

    fn searcher_is_init_and_up_to_date_and_complete(&self) ->bool {
        self.searcher.get().is_some_and(|value| {
            let read = value.read().unwrap();
            read.0 == self.get_current_dict_version() && read.1.is_complete()
        })
    }


    fn get_current_dict_version(&self) -> SearchInfoVersion {
        let read = self.dictionary.get();
        SearchInfoVersion(read.voc_a().len(), read.voc_b().len(), self.prefix_len)
    }

    fn execute_with_searcher<F, R, E>(&self, method: F) -> Result<R, SearchError>
    where
        F: FnOnce(&prefix::PrefixDictSearch) -> Result<R, E>,
        SearchError: From<E>
    {
        let prefix_search = self.searcher.get_or_init(||{
            let borrow = self.dictionary.get();
            Arc::new(
                RwLock::new(
                    (
                        self.get_current_dict_version(),
                        prefix::PrefixDictSearch::new(borrow.deref(), self.prefix_len)
                    )
                )
            )
        });

        let current = {
            let read = prefix_search.read().unwrap();
            let current = self.get_current_dict_version();
            if current == read.deref().0 {
                return Ok(method(&read.1)?)
            }
            drop(read);
            current
        };
        let mut write = prefix_search.write().unwrap();
        write.0 = current;
        write.1 = prefix::PrefixDictSearch::new(
            self.dictionary.get().deref(),
            self.prefix_len
        );
        Ok(method(&write.1)?)
    }

    fn execute_on_target<F1, F2, E>(
        &self,
        input: SearchInput,
        target_language: Option<LanguageKind>,
        fna: F1,
        fnb: F2
    ) -> Result<(Option<HashMap<String, Vec<(String, Vec<usize>)>>>, Option<HashMap<String, Vec<(String, Vec<usize>)>>>), SearchError>
    where
        F1: for<'a> Fn(&'a prefix::PrefixDictSearch, &str) -> Result<Vec<(String, &'a Vec<usize>)>, E>,
        F2: for<'a> Fn(&'a prefix::PrefixDictSearch, &str) -> Result<Vec<(String, &'a Vec<usize>)>, E>,
        SearchError: From<E>
    {
        self.execute_with_searcher::<_, _, E>(
            move |search| {
                match target_language {
                    None => {
                        match input {
                            SearchInput::String(value) => {
                                let mut result_a = HashMap::with_capacity(1);
                                let result = (&fna)(search, &value)?;
                                result_a.insert(value.clone(), result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());

                                let mut result_b = HashMap::with_capacity(1);
                                let result = (&fnb)(search, &value)?;
                                result_b.insert(value.clone(), result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());

                                Ok((Some(result_a), Some(result_b)))
                            }
                            SearchInput::Many(values) => {
                                let mut result_a = HashMap::with_capacity(values.len());
                                for value in &values {
                                    if result_a.contains_key(value) {
                                        continue
                                    }
                                    let result = (&fna)(search, value)?;
                                    result_a.insert(value.clone(), result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                }

                                let mut result_b = HashMap::with_capacity(values.len());
                                for value in values {
                                    if result_b.contains_key(&value) {
                                        continue
                                    }
                                    let result = (&fnb)(search, &value)?;
                                    result_b.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                }

                                Ok((Some(result_a), Some(result_b)))
                            }
                            _ => unreachable!()
                        }
                    }
                    Some(LanguageKind::A) => {
                        match input {
                            SearchInput::String(value) => {
                                let mut result_a = HashMap::with_capacity(1);
                                let result = (&fna)(search, &value)?;
                                result_a.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                Ok((Some(result_a), None))
                            }
                            SearchInput::Many(values) => {
                                let mut result_a = HashMap::with_capacity(values.len());
                                for value in values {
                                    if result_a.contains_key(&value) {
                                        continue
                                    }
                                    let result = (&fna)(search, &value)?;
                                    result_a.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                }
                                Ok((Some(result_a), None))
                            }
                            _ => unreachable!()
                        }
                    }
                    Some(LanguageKind::B) => {
                        match input {
                            SearchInput::String(value) => {
                                let mut result_b = HashMap::with_capacity(1);
                                let result = (&fnb)(search, &value)?;
                                result_b.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());

                                Ok((None, Some(result_b)))
                            }
                            SearchInput::Many(values) => {
                                let mut result_b = HashMap::with_capacity(values.len());
                                for value in values {
                                    if result_b.contains_key(&value) {
                                        continue
                                    }
                                    let result = (&fnb)(search, &value)?;
                                    result_b.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                }

                                Ok((None, Some(result_b)))
                            }
                            _ => unreachable!()
                        }
                    }
                }
            }
        )
    }

    /// The general search function.
    pub fn search(
        &self,
        input: SearchInput,
        search_type: Option<SearchType>,
        target_language: Option<LanguageKind>,
        threshold: Option<Either<usize, f64>>,
        ignores_ascii_case: bool,
    ) -> Result<SearchResult, SearchError> {

        fn pack_as_search_result(value: (Option<HashMap<String, Vec<(String, Vec<usize>)>>>, Option<HashMap<String, Vec<(String, Vec<usize>)>>>)) -> SearchResult {
            todo!()
        }

        fn pack_as_primitive_search_result(value: (Option<HashMap<String, Vec<(String, Vec<usize>)>>>, Option<HashMap<String, Vec<(String, Vec<usize>)>>>)) -> SearchResult {
            todo!()
        }

        fn pack_primitive_search_result(value: Vec<(LanguageKind, usize, HashRef<String>)>) -> SearchResult {
            todo!()
        }


        let options = if input.is_method() {
            SearchOption::new(
                SearchKind::Matcher,
                target_language,
                threshold,
                ignores_ascii_case
            )?
        } else {
            match search_type.unwrap_or_default() {
                SearchType::Autocomplete => {
                    if ignores_ascii_case {
                        return Err(SearchError::IgnoreAsciiNotSupported("Autocompletion"))
                    } else {
                        let result = self.execute_on_target(
                            input,
                            target_language,
                            |search, query| {
                                Ok::<_, SearchError>(search.search_in_a_for_all_as_prediction(query))
                            },
                            |search, query| {
                                Ok::<_, SearchError>(search.search_in_b_for_all_as_prediction(query))
                            }
                        )?;
                        return Ok(pack_as_search_result(result))
                    }
                }
                SearchType::ExactMatch if !ignores_ascii_case && self.searcher_is_init_and_up_to_date_and_complete() => {
                    let result = self.execute_on_target(
                        input,
                        target_language,
                        |search, query| {
                            Ok::<_, SearchError>(Vec::from_iter(search.search_in_a_exact(query).map(|v| (query.to_string(), v))))
                        },
                        |search, query| {
                            Ok::<_, SearchError>(Vec::from_iter(search.search_in_b_exact(query).map(|v| (query.to_string(), v))))
                        }
                    )?;
                    return Ok(pack_as_primitive_search_result(result))
                }
                SearchType::StartsWith if !ignores_ascii_case && self.searcher_is_init_and_up_to_date_and_complete() => {
                    let result = self.execute_on_target(
                        input,
                        target_language,
                        |search, query| {
                            Ok::<_, SearchError>(search.search_in_a_for_all_as_prediction(query))
                        },
                        |search, query| {
                            Ok::<_, SearchError>(search.search_in_b_for_all_as_prediction(query))
                        }
                    )?;
                    return Ok(pack_as_primitive_search_result(result))
                }
                SearchType::EndsWith if !ignores_ascii_case && self.searcher_is_init_and_up_to_date_and_complete() => {
                    let result = self.execute_on_target(
                        input,
                        target_language,
                        |search, query| {
                            Ok::<_, SearchError>(search.search_in_a_for_all_with_suffix(query))
                        },
                        |search, query| {
                            Ok::<_, SearchError>(search.search_in_b_for_all_with_suffix(query))
                        }
                    )?;
                    return Ok(pack_as_primitive_search_result(result))
                }
                SearchType::CommonPrefix => {
                    let result = self.execute_on_target(
                        input,
                        target_language,
                        |search, query| {
                            Ok::<_, SearchError>(search.search_in_a_for_all_common_prefix(query))
                        },
                        |search, query| {
                            Ok::<_, SearchError>(search.search_in_b_for_all_common_prefix(query))
                        }
                    )?;
                    return Ok(pack_as_search_result(result))
                }
                rest => {
                    SearchOption::new(
                        rest.try_into().expect("This conversion should never fail!"),
                        target_language,
                        threshold,
                        ignores_ascii_case
                    )?
                }
            }
        };

        Ok(
            pack_primitive_search_result(
                Searcher::new(input, options)?
                    .search(self.dictionary.get().deref())
            )
        )
    }

    pub fn prefix_len(&self) -> Option<usize> {
        self.prefix_len
    }

    pub fn set_prefix_len(&mut self, prefix_len: Option<usize>) {
        self.prefix_len = prefix_len;
    }
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    PrimitiveSearcherOption(#[from] SearchOptionError),
    #[error(transparent)]
    PrimitiveSearcher(#[from] primitive::SearcherError),
    #[error("The method {0} does not support ignoring ascii case!")]
    IgnoreAsciiNotSupported(&'static str),
}


#[derive(Default, Debug)]
pub enum SearchType {
    Autocomplete,
    #[default]
    ExactMatch,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    Hamming,
    Levensthein,
    OsaDistance,
    NormalizedLevensthein,
    DamerauLevensthein,
    NormalizedDamerauLevensthein,
    Jaro,
    JaroWinkler,
    SorensenDice,
    CommonPrefix
}