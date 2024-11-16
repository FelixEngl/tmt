use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use derive_more::From;
use either::Either;
use itertools::{EitherOrBoth, Itertools};
use pyo3::FromPyObject;
use strum::EnumIs;
use thiserror::Error;
use crate::define_py_method;
use crate::toolkit::sync_ext::OwnedOrArcRw;
use crate::topicmodel::dictionary::DictionaryWithVocabulary;
use crate::topicmodel::dictionary::direction::LanguageKind;
use crate::topicmodel::dictionary::search::prefix::PrefixDictSearch;
use crate::topicmodel::dictionary::search::primitive::{SearchKind, SearchOption, SearchOptionError, Searcher};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{SearchableVocabulary};

mod primitive;
mod prefix;




#[derive(Debug, FromPyObject, EnumIs)]
pub enum SearchInput<'py> {
    String(String),
    Many(Vec<String>),
    Method(MatchWordMethod<'py>),
}

impl<'py> From<MatchWordMethod<'py>> for SearchInput<'py> {
    fn from(value: MatchWordMethod<'py>) -> Self {
        Self::Method(value.to_string())
    }
}

impl<'py, S> From<S> for SearchInput<'py> where S: Into<String> {
    fn from(value: S) -> Self {
        Self::String(value.to_string())
    }
}

impl<'py, I, S> FromIterator<S> for SearchInput<'py> where S: Into<String> {
    fn from_iter<T: IntoIterator<Item=S>>(iter: T) -> Self {
        Self::Many(iter.into_iter().map(S::into).collect())
    }
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
    searcher: std::sync::OnceLock<Arc<RwLock<(SearchInfoVersion, PrefixDictSearch)>>>,
    _phantom: PhantomData<V>
}

impl<D, V> DictionarySearcher<D, V> {
    pub fn into_inner(self) -> OwnedOrArcRw<D> {
        self.dictionary
    }
}


type SearchResult = Option<Either<
    EitherOrBoth<Vec<(usize, HashRef<String>)>>,
    EitherOrBoth<HashMap<String, Vec<(String, Vec<usize>)>>>
>>;


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


    pub fn prefix_len(&self) -> Option<usize> {
        self.prefix_len
    }

    pub fn set_prefix_len(&mut self, prefix_len: Option<usize>) {
        self.prefix_len = prefix_len;
    }


    fn get_dict_searcher(&self) -> &Arc<RwLock<(SearchInfoVersion, PrefixDictSearch)>> {
        let searcher = self.searcher.get_or_init(||{
            let borrow = self.dictionary.get();
            Arc::new(
                RwLock::new(
                    (
                        self.get_current_dict_version(),
                        PrefixDictSearch::new(borrow.deref(), self.prefix_len)
                    )
                )
            )
        });
        {
            let read = searcher.read().unwrap();
            let current = self.get_current_dict_version();
            if current == read.deref().0 {
                return searcher
            }
        }
        let mut write = searcher.write().unwrap();
        write.0 = self.get_current_dict_version();
        write.1 = PrefixDictSearch::new(
            self.dictionary.get().deref(),
            self.prefix_len
        );
        searcher
    }

    pub fn init_prefix_dict_searcher(&mut self, prefix_len: Option<Option<usize>>) {
        if let Some(value) = prefix_len {
            self.prefix_len = value;
        }
        self.get_dict_searcher();
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
        F: FnOnce(&PrefixDictSearch) -> Result<R, E>,
        SearchError: From<E>
    {
        let prefix_search: &Arc<RwLock<(SearchInfoVersion, PrefixDictSearch)>> = self.get_dict_searcher();
        let read = prefix_search.read().unwrap();
        Ok(method(&read.1)?)
    }

    fn execute_on_target<F1, F2, E>(
        &self,
        input: SearchInput,
        target_language: Option<LanguageKind>,
        fna: F1,
        fnb: F2
    ) -> Result<(Option<HashMap<String, Vec<(String, Vec<usize>)>>>, Option<HashMap<String, Vec<(String, Vec<usize>)>>>), SearchError>
    where
        F1: for<'a> Fn(&'a PrefixDictSearch, &str) -> Result<Vec<(String, &'a Vec<usize>)>, E>,
        F2: for<'a> Fn(&'a PrefixDictSearch, &str) -> Result<Vec<(String, &'a Vec<usize>)>, E>,
        SearchError: From<E>
    {
        self.execute_with_searcher::<_, _, E>(
            move |search| {
                match target_language {
                    None => {
                        match input {
                            SearchInput::String(value) => {
                                let result = (&fna)(search, &value)?;
                                let result_a = if result.is_empty() {
                                    None
                                } else {
                                    let mut result_a = HashMap::with_capacity(1);
                                    result_a.insert(value.clone(), result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                    Some(result_a)
                                };
                                let result = (&fnb)(search, &value)?;
                                let result_b = if result.is_empty() {
                                    None
                                } else {
                                    let mut result_b = HashMap::with_capacity(1);
                                    result_b.insert(value.clone(), result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                    Some(result_b)
                                };
                                Ok((result_a, result_b))
                            }
                            SearchInput::Many(values) => {
                                let mut result_a = HashMap::with_capacity(values.len());
                                let mut result_b = HashMap::with_capacity(values.len());
                                for value in values {
                                    if !result_a.contains_key(&value) {
                                        let result = (&fna)(search, &value)?;
                                        if !result.is_empty() {
                                            result_a.insert(value.clone(), result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                        }
                                    }
                                    if result_b.contains_key(&value) {
                                        continue
                                    }
                                    let result = (&fnb)(search, &value)?;
                                    if !result.is_empty() {
                                        result_b.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                    }
                                }
                                Ok((
                                    (!result_a.is_empty()).then_some(result_a),
                                    (!result_b.is_empty()).then_some(result_b)
                                ))
                            }
                            _ => unreachable!()
                        }
                    }
                    Some(LanguageKind::A) => {
                        match input {
                            SearchInput::String(value) => {
                                let result = (&fna)(search, &value)?;
                                if result.is_empty() {
                                    Ok((None, None))
                                } else {
                                    let mut result_a = HashMap::with_capacity(1);
                                    result_a.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                    Ok((Some(result_a), None))
                                }
                            }
                            SearchInput::Many(values) => {
                                let mut result_a = HashMap::with_capacity(values.len());
                                for value in values {
                                    if result_a.contains_key(&value) {
                                        continue
                                    }
                                    let result = (&fna)(search, &value)?;
                                    if !result.is_empty() {
                                        result_a.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                    }
                                }
                                if result_a.is_empty() {
                                    Ok((None, None))
                                } else {
                                    Ok((Some(result_a), None))
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                    Some(LanguageKind::B) => {
                        match input {
                            SearchInput::String(value) => {
                                let result = (&fnb)(search, &value)?;
                                if result.is_empty() {
                                    Ok((None, None))
                                } else {
                                    let mut result_b = HashMap::with_capacity(1);
                                    result_b.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                    Ok((None, Some(result_b)))
                                }
                            }
                            SearchInput::Many(values) => {
                                let mut result_b = HashMap::with_capacity(values.len());
                                for value in values {
                                    if result_b.contains_key(&value) {
                                        continue
                                    }
                                    let result = (&fnb)(search, &value)?;
                                    if !result.is_empty() {
                                        result_b.insert(value, result.into_iter().map(|(a, b)| (a, b.clone())).collect::<Vec<_>>());
                                    }
                                }
                                if result_b.is_empty() {
                                    Ok((None, None))
                                } else {
                                    Ok((None, Some(result_b)))
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                }
            }
        )
    }

    /// The general search function.
    pub fn search<'a>(
        &self,
        input: impl Into<SearchInput<'a>>,
        search_type: Option<SearchType>,
        target_language: Option<LanguageKind>,
        threshold: Option<Either<usize, f64>>,
        ignores_ascii_case: bool,
    ) -> Result<SearchResult, SearchError> {

        let input = input.into();

        fn pack_as_search_result(value: (Option<HashMap<String, Vec<(String, Vec<usize>)>>>, Option<HashMap<String, Vec<(String, Vec<usize>)>>>)) -> SearchResult {
            match value {
                (Some(a), Some(b)) => {
                    let value =
                        if a.is_empty() && b.is_empty() {
                            return None
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
                _ => None
            }
        }

        fn pack_as_primitive_search_result(value: (Option<HashMap<String, Vec<(String, Vec<usize>)>>>, Option<HashMap<String, Vec<(String, Vec<usize>)>>>)) -> SearchResult {
            pack_as_search_result(value)
        }

        fn pack_primitive_search_result(value: Vec<(LanguageKind, usize, HashRef<String>)>) -> SearchResult {
            let (a, b) = value.into_iter().partition::<Vec<_>, _>(|(lang, _, _)| lang.is_a());
            let result =
                if a.is_empty() && b.is_empty() {
                    return None
                } else if b.is_empty() {
                    EitherOrBoth::Left(a)
                } else if a.is_empty() {
                    EitherOrBoth::Right(b)
                } else {
                    EitherOrBoth::Both(a, b)
                };
            let result = result.map_any(
                |value| value.into_iter().map(|(_, a, b)| (a, b)).collect_vec(),
                |value| value.into_iter().map(|(_, a, b)| (a, b)).collect_vec()
            );
            Some(Either::Left(result))
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

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::{MutableDictionaryWithMeta, StringDictWithMetaDefault};
    use crate::topicmodel::dictionary::search::DictionarySearcher;
    use crate::topicmodel::dictionary::search::SearchType::Autocomplete;

    #[test]
    fn can_exec(){
        let mut dict = StringDictWithMetaDefault::default();
        dict.push_invariant("hallo", "hello").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("Hallo", "hello").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("Welt", "world").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("Felix Engl", "Felix Engl").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("autowaschen", "to do car washing").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("Hallo Welt", "hello world").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );

        dict.push_invariant("Herz", "heart").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );

        dict.push_invariant("Gürtel", "belt").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );

        let mut searcher = DictionarySearcher::new(dict);

        let result = searcher.search(
            "Gürtel",
            None,
            None,
            None,
            false
        ).expect("This should not fail!");

        println!("{result:?}");

        searcher.init_prefix_dict_searcher(None);

        let result = searcher.search(
            "Gür",
            Some(Autocomplete),
            None,
            None,
            false
        ).expect("This should not fail!");
        println!("{result:?}");
    }
}