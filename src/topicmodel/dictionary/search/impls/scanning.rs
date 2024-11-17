use crate::topicmodel::dictionary::direction::LanguageKind;
use crate::topicmodel::dictionary::search::{MatchWordMethod, SearchInput, SearchType};
use crate::topicmodel::dictionary::DictionaryWithVocabulary;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::SearchableVocabulary;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, Input};
use derive_more::From;
use either::Either;
use itertools::{chain, Itertools};
use pyo3::exceptions::PyValueError;
use pyo3::PyErr;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use strsim::{
    damerau_levenshtein, hamming, jaro, jaro_winkler, levenshtein, normalized_damerau_levenshtein,
    normalized_levenshtein, osa_distance, sorensen_dice,
};
use strum::Display;
use thiserror::Error;
use ScanAlgorithm::*;

/// The kind of scan executed by the scan searcher.
#[derive(Debug, Copy, Clone, Display, Eq, PartialEq)]
pub enum ScanAlgorithm {
    Matcher,
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
}
impl TryFrom<SearchType> for ScanAlgorithm {
    type Error = SearchType;

    fn try_from(value: SearchType) -> Result<Self, Self::Error> {
        match value {
            SearchType::ExactMatch => Ok(ExactMatch),
            SearchType::Contains => Ok(Contains),
            SearchType::StartsWith => Ok(StartsWith),
            SearchType::EndsWith => Ok(EndsWith),
            SearchType::Regex => Ok(Regex),
            SearchType::Hamming => Ok(Hamming),
            SearchType::Levensthein => Ok(Levensthein),
            SearchType::OsaDistance => Ok(OsaDistance),
            SearchType::NormalizedLevensthein => Ok(NormalizedLevensthein),
            SearchType::DamerauLevensthein => Ok(DamerauLevensthein),
            SearchType::NormalizedDamerauLevensthein => Ok(NormalizedDamerauLevensthein),
            SearchType::Jaro => Ok(Jaro),
            SearchType::JaroWinkler => Ok(JaroWinkler),
            SearchType::SorensenDice => Ok(SorensenDice),
            illegal => Err(illegal),
        }
    }
}

/// A searcher with primitive scan functionality
pub struct ScanSearcher<'py> {
    input: ParsedExpression<'py>,
    kind: ScanAlgorithm,
    language: Option<LanguageKind>,
    threshold: Threshold,
    ignores_ascii_case: bool,
}
impl<'py> ScanSearcher<'py> {
    pub fn new(
        search_input: SearchInput<'py>,
        option: ScanSearcherOptions,
    ) -> Result<Self, ScanSearcherInitError> {
        let input = match search_input {
            SearchInput::String(value) => match option.kind {
                Matcher => unreachable!(),
                Contains | StartsWith | EndsWith => AhoCorasickBuilder::new()
                    .ascii_case_insensitive(option.ignores_ascii_case)
                    .build([value])?
                    .into(),
                Regex => regex::RegexBuilder::new(&value)
                    .case_insensitive(option.ignores_ascii_case)
                    .build()?
                    .into(),
                ExactMatch => StrEq::StringComp(value).into(),
                _ => {
                    if option.ignores_ascii_case {
                        StrEq::StringCompIgnCas(value.to_lowercase()).into()
                    } else {
                        StrEq::StringComp(value).into()
                    }
                }
            },
            SearchInput::Many(values) => match option.kind {
                Matcher => unreachable!(),
                Contains | StartsWith | EndsWith => AhoCorasickBuilder::new()
                    .ascii_case_insensitive(option.ignores_ascii_case)
                    .build(values)?
                    .into(),
                Regex => regex::RegexSetBuilder::new(values)
                    .case_insensitive(option.ignores_ascii_case)
                    .build()?
                    .into(),
                ExactMatch => StrEq::StringComps(HashSet::from_iter(values)).into(),
                _ => {
                    if option.ignores_ascii_case {
                        StrEq::StringCompsIgnCas(HashSet::from_iter(
                            values.into_iter().map(|value| value.to_lowercase()),
                        ))
                        .into()
                    } else {
                        StrEq::StringComps(HashSet::from_iter(values)).into()
                    }
                }
            },
            SearchInput::Method(value) => value.into(),
        };

        Ok(Self {
            input,
            kind: option.kind,
            language: option.language,
            threshold: option.threshold,
            ignores_ascii_case: option.ignores_ascii_case,
        })
    }

    pub fn search<D, T, V>(&self, dictionary: &D) -> Vec<(LanguageKind, usize, HashRef<T>)>
    where
        D: DictionaryWithVocabulary<T, V>,
        T: Borrow<str> + Hash + Eq + AsRef<str>,
        V: SearchableVocabulary<T>,
    {
        match self.kind {
            ExactMatch => {
                let matcher = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                self.get_with(dictionary, matcher)
            }
            Contains => {
                let matcher = match &self.input {
                    ParsedExpression::Aho(value) => value,
                    _ => unreachable!(),
                }
                .clone();
                self.search_with(dictionary, |_, value| matcher.is_match(value))
            }
            StartsWith => {
                let matcher = match &self.input {
                    ParsedExpression::Aho(value) => value,
                    _ => unreachable!(),
                }
                .clone();
                self.search_with(dictionary, |_, value| {
                    matcher
                        .find(Input::new(value).earliest(true))
                        .is_some_and(|v| v.start() == 0 && !v.is_empty())
                })
            }
            EndsWith => {
                let matcher = match &self.input {
                    ParsedExpression::Aho(value) => value,
                    _ => unreachable!(),
                }
                .clone();
                self.search_with(dictionary, |_, value| {
                    if value.len() < matcher.min_pattern_len() {
                        false
                    } else if value.len() <= matcher.max_pattern_len() {
                        matcher
                            .find_iter(value)
                            .last()
                            .is_some_and(|v| v.end() == value.len() && !v.is_empty())
                    } else {
                        let value = &value[value.len() - matcher.max_pattern_len()..];
                        matcher
                            .find_iter(value)
                            .last()
                            .is_some_and(|v| v.end() == value.len() && !v.is_empty())
                    }
                })
            }
            Regex => match &self.input {
                ParsedExpression::Regex(regex) => {
                    self.search_with(dictionary, |_, value| regex.is_match(value))
                }
                ParsedExpression::RegexSet(regex) => {
                    self.search_with(dictionary, |_, value| regex.is_match(value))
                }
                _ => unreachable!(),
            },
            //  Hamming | Levensthein | OsaDistance | DamerauLevensthein
            Hamming => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::new(comp, |s, pattern| {
                    hamming(s, pattern)
                        .is_ok_and(|value| value <= unsafe { self.threshold.threshold_usize })
                });
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }
            Levensthein => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::limited_matcher(
                    comp,
                    unsafe { self.threshold.threshold_usize },
                    levenshtein,
                );
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }
            OsaDistance => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::limited_matcher(
                    comp,
                    unsafe { self.threshold.threshold_usize },
                    osa_distance,
                );
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }
            DamerauLevensthein => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::limited_matcher(
                    comp,
                    unsafe { self.threshold.threshold_usize },
                    damerau_levenshtein,
                );
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }

            NormalizedLevensthein => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::limited_matcher_total(
                    comp,
                    unsafe { self.threshold.threshold_f64 },
                    normalized_levenshtein,
                );
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }
            NormalizedDamerauLevensthein => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::limited_matcher_total(
                    comp,
                    unsafe { self.threshold.threshold_f64 },
                    normalized_damerau_levenshtein,
                );
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }
            Jaro => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::limited_matcher_total(
                    comp,
                    unsafe { self.threshold.threshold_f64 },
                    jaro,
                );
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }
            JaroWinkler => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::limited_matcher_total(
                    comp,
                    unsafe { self.threshold.threshold_f64 },
                    jaro_winkler,
                );
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }
            SorensenDice => {
                let comp = match &self.input {
                    ParsedExpression::StringEq(value) => value,
                    _ => unreachable!(),
                };
                let matcher = WithMatcher::limited_matcher_total(
                    comp,
                    unsafe { self.threshold.threshold_f64 },
                    sorensen_dice,
                );
                self.search_with(dictionary, |_, value| matcher.has_match(value))
            }
            Matcher => match &self.input {
                ParsedExpression::Matcher(matcher) => {
                    self.search_with(dictionary, |lang, value| {
                        matcher
                            .call(lang, value)
                            .expect("This method should not throw an exception!")
                    })
                }
                _ => unreachable!(),
            },
        }
    }

    fn search_with<D, T, V, F>(
        &self,
        dictionary: &D,
        matcher: F,
    ) -> Vec<(LanguageKind, usize, HashRef<T>)>
    where
        D: DictionaryWithVocabulary<T, V>,
        T: Borrow<str> + Hash + Eq + AsRef<str>,
        V: SearchableVocabulary<T>,
        F: Fn(LanguageKind, &str) -> bool,
    {
        match self.language {
            None => chain!(
                dictionary.voc_a().iter().enumerate().filter_map(|value| {
                    if matcher(LanguageKind::A, value.1.as_ref()) {
                        Some((LanguageKind::A, value.0, value.1.clone()))
                    } else {
                        None
                    }
                }),
                dictionary.voc_b().iter().enumerate().filter_map(|value| {
                    if matcher(LanguageKind::B, value.1.as_ref()) {
                        Some((LanguageKind::B, value.0, value.1.clone()))
                    } else {
                        None
                    }
                })
            )
            .collect_vec(),
            Some(LanguageKind::A) => dictionary
                .voc_a()
                .iter()
                .enumerate()
                .filter_map(|value| {
                    if matcher(LanguageKind::A, value.1.as_ref()) {
                        Some((LanguageKind::A, value.0, value.1.clone()))
                    } else {
                        None
                    }
                })
                .collect_vec(),
            Some(LanguageKind::B) => dictionary
                .voc_b()
                .iter()
                .enumerate()
                .filter_map(|value| {
                    if matcher(LanguageKind::B, value.1.as_ref()) {
                        Some((LanguageKind::B, value.0, value.1.clone()))
                    } else {
                        None
                    }
                })
                .collect_vec(),
        }
    }

    fn get_with<D, T, V>(
        &self,
        dictionary: &D,
        matcher: &StrEq,
    ) -> Vec<(LanguageKind, usize, HashRef<T>)>
    where
        D: DictionaryWithVocabulary<T, V>,
        T: Hash + Eq + Borrow<str>,
        V: SearchableVocabulary<T>,
    {
        fn search_configured<D, T, V>(
            dict: &D,
            lang: LanguageKind,
            value: &str,
            ignore_ascii: bool,
            output: &mut Vec<(LanguageKind, usize, HashRef<T>)>,
        ) where
            D: DictionaryWithVocabulary<T, V>,
            T: Hash + Eq + Borrow<str>,
            V: SearchableVocabulary<T>,
        {
            match lang {
                LanguageKind::A => {
                    if let Some((a, b)) = dict.voc_a().get_entry_id(value) {
                        output.push((LanguageKind::A, *b, a.clone()))
                    }
                    if ignore_ascii {
                        if let Some((a, b)) = dict.voc_a().get_entry_id(&value.to_lowercase()) {
                            output.push((LanguageKind::A, *b, a.clone()))
                        }
                    }
                }
                LanguageKind::B => {
                    if let Some((a, b)) = dict.voc_b().get_entry_id(value) {
                        output.push((LanguageKind::B, *b, a.clone()))
                    }
                    if ignore_ascii {
                        if let Some((a, b)) = dict.voc_b().get_entry_id(&value.to_lowercase()) {
                            output.push((LanguageKind::B, *b, a.clone()))
                        }
                    }
                }
            }
        }

        match matcher {
            StrEq::StringComp(value) => match self.language {
                None => {
                    let mut result = Vec::with_capacity(4);

                    search_configured(
                        dictionary,
                        LanguageKind::A,
                        &value,
                        self.ignores_ascii_case,
                        &mut result,
                    );
                    search_configured(
                        dictionary,
                        LanguageKind::B,
                        &value,
                        self.ignores_ascii_case,
                        &mut result,
                    );
                    result
                }
                Some(targ) => {
                    let mut result = Vec::with_capacity(2);
                    search_configured(
                        dictionary,
                        targ,
                        &value,
                        self.ignores_ascii_case,
                        &mut result,
                    );
                    result
                }
            },
            StrEq::StringComps(values) => match self.language {
                None => {
                    let mut result = Vec::with_capacity(4 * values.len());
                    for s in values {
                        search_configured(
                            dictionary,
                            LanguageKind::A,
                            &s,
                            self.ignores_ascii_case,
                            &mut result,
                        );
                        search_configured(
                            dictionary,
                            LanguageKind::B,
                            &s,
                            self.ignores_ascii_case,
                            &mut result,
                        );
                    }
                    result
                }
                Some(targ) => {
                    let mut result = Vec::with_capacity(2 * values.len());
                    for s in values {
                        search_configured(
                            dictionary,
                            targ,
                            &s,
                            self.ignores_ascii_case,
                            &mut result,
                        );
                    }
                    result
                }
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ScanSearcherOptions {
    kind: ScanAlgorithm,
    language: Option<LanguageKind>,
    ignores_ascii_case: bool,
    threshold: Threshold,
}
impl ScanSearcherOptions {
    pub fn new(
        kind: ScanAlgorithm,
        language: Option<LanguageKind>,
        threshold: Option<Either<usize, f64>>,
        ignores_ascii_case: bool,
    ) -> Result<Self, ScanSearcherOptionsInitError> {
        match kind {
            Matcher => {
                if threshold.is_some() {
                    log::warn!("{kind} does not know any kinds of thresholds.");
                }
                if ignores_ascii_case {
                    log::warn!("{kind} requires the developer to ignore the thresholds.");
                }
                Ok(Self {
                    kind,
                    language,
                    threshold: Threshold::UNSET,
                    ignores_ascii_case,
                })
            }
            Regex | StartsWith | EndsWith | Contains | ExactMatch => {
                if threshold.is_some() {
                    log::warn!("{kind} does not know any kinds of thresholds.");
                }
                Ok(Self {
                    kind,
                    language,
                    threshold: Threshold::UNSET,
                    ignores_ascii_case,
                })
            }
            Hamming | Levensthein | OsaDistance | DamerauLevensthein => match threshold {
                None => Err(ScanSearcherOptionsInitError::missing_from_type::<usize>(
                    kind,
                )),
                Some(Either::Left(threshold_usize)) => Ok(Self {
                    kind,
                    language,
                    threshold: threshold_usize.into(),
                    ignores_ascii_case,
                }),
                Some(Either::Right(value)) => {
                    if value < 0.0 || value > usize::MAX as f64 {
                        Err(ScanSearcherOptionsInitError::illegal_from_type::<usize>(
                            kind, value,
                        ))
                    } else {
                        let threshold_usize = value.round_ties_even() as usize;
                        log::warn!("{kind} requires an integer as threshold. but got {value}, convert it to {threshold_usize}.");
                        Ok(Self {
                            kind,
                            language,
                            threshold: threshold_usize.into(),
                            ignores_ascii_case,
                        })
                    }
                }
            },
            NormalizedLevensthein
            | NormalizedDamerauLevensthein
            | Jaro
            | JaroWinkler
            | SorensenDice => match threshold {
                None => Err(ScanSearcherOptionsInitError::missing_from_type::<f64>(kind)),
                Some(Either::Left(value)) => {
                    let threshold_f64 = value as f64;
                    log::warn!("{kind} requires an float as threshold. but got {value}, convert it to {threshold_f64}.");
                    Ok(Self {
                        kind,
                        language,
                        threshold: threshold_f64.into(),
                        ignores_ascii_case,
                    })
                }
                Some(Either::Right(threshold_f64)) => Ok(Self {
                    kind,
                    language,
                    threshold: threshold_f64.into(),
                    ignores_ascii_case,
                }),
            },
        }
    }
}

/// An error returned when the [ScanSearcher] can not be initialized.
#[derive(Debug, Error)]
pub enum ScanSearcherInitError {
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    Aho(#[from] aho_corasick::BuildError),
}
impl Eq for ScanSearcherInitError {}
impl PartialEq for ScanSearcherInitError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (
                ScanSearcherInitError::Regex(_),
                ScanSearcherInitError::Regex(_)
            ) | (ScanSearcherInitError::Aho(_), ScanSearcherInitError::Aho(_))
        )
    }
}

/// An error returned when initializing the [ScanSearcherOptions] fails
#[derive(Debug, Error, Copy, Clone)]
pub enum ScanSearcherOptionsInitError {
    #[error("Expected a threshold of type {expected} for {kind} but got none.")]
    ThresholdMissing {
        kind: ScanAlgorithm,
        expected: &'static str,
    },
    #[error("Expected a threshold of type {expected} for {kind} in the range of 0..{max} but got {value} of type f64.", max = usize::MAX)]
    IllegalThresholdRange {
        kind: ScanAlgorithm,
        expected: &'static str,
        value: f64,
    },
}
impl Eq for ScanSearcherOptionsInitError {}
impl PartialEq for ScanSearcherOptionsInitError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ScanSearcherOptionsInitError::ThresholdMissing { kind, expected },
                ScanSearcherOptionsInitError::ThresholdMissing {
                    kind: kind2,
                    expected: expected2,
                },
            ) => kind == kind2 && expected == expected2,

            (
                ScanSearcherOptionsInitError::IllegalThresholdRange {
                    kind,
                    expected,
                    value,
                },
                ScanSearcherOptionsInitError::IllegalThresholdRange {
                    kind: kind2,
                    expected: expected2,
                    value: value2,
                },
            ) => {
                kind == kind2
                    && expected == expected2
                    && float_cmp::approx_eq!(f64, *value, *value2)
            }
            _ => false,
        }
    }
}
impl ScanSearcherOptionsInitError {
    pub fn missing_from_type<Expected>(kind: ScanAlgorithm) -> Self {
        Self::ThresholdMissing {
            kind,
            expected: std::any::type_name::<Expected>(),
        }
    }

    pub fn illegal_from_type<Expected>(kind: ScanAlgorithm, value: f64) -> Self {
        Self::IllegalThresholdRange {
            kind,
            expected: std::any::type_name::<Expected>(),
            value,
        }
    }
}
impl From<ScanSearcherOptionsInitError> for PyErr {
    fn from(value: ScanSearcherOptionsInitError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

/// Holds the threshold used, what it is depends on the selected [ScanAlgorithm]
/// We control this union so using it is safe.
#[derive(Copy, Clone)]
union Threshold {
    threshold_f64: f64,
    threshold_usize: usize,
    unset: (),
}

impl Threshold {
    pub const UNSET: Threshold = Threshold { unset: () };
}

impl From<f64> for Threshold {
    fn from(value: f64) -> Self {
        Threshold {
            threshold_f64: value,
        }
    }
}

impl From<usize> for Threshold {
    fn from(value: usize) -> Self {
        Threshold {
            threshold_usize: value,
        }
    }
}

/// The parsed expression used for the search.
#[derive(From)]
enum ParsedExpression<'py> {
    Regex(regex::Regex),
    RegexSet(regex::RegexSet),
    Aho(AhoCorasick),
    StringEq(StrEq),
    Matcher(MatchWordMethod<'py>),
}

/// Basically an enum for doing string comparisons.
enum StrEq {
    StringComp(String),
    StringComps(HashSet<String>),
    StringCompIgnCas(String),
    StringCompsIgnCas(HashSet<String>),
}
impl StrEq {
    fn has_match(&self, other: &str) -> bool {
        match self {
            StrEq::StringComp(value) => value.eq(other),
            StrEq::StringComps(values) => values.contains(other),
            StrEq::StringCompIgnCas(value) => value.eq_ignore_ascii_case(other),
            StrEq::StringCompsIgnCas(values) => values.contains(&other.to_lowercase()),
        }
    }
}

/// A decorator for a [StrEq] with the associated method.
struct WithMatcher<'a, F: Fn(&str, &str) -> bool> {
    comp: &'a StrEq,
    matcher: F,
}
impl<'a, F: Fn(&str, &str) -> bool> WithMatcher<'a, F> {
    fn has_match(&self, s: &str) -> bool {
        match &self.comp {
            StrEq::StringComp(value) => (&self.matcher)(s, value),
            StrEq::StringComps(values) => values.iter().any(|v| (&self.matcher)(s, v)),
            StrEq::StringCompIgnCas(value) => (&self.matcher)(s, &value.to_lowercase()),
            StrEq::StringCompsIgnCas(values) => {
                values.iter().any(|v| (&self.matcher)(s, &v.to_lowercase()))
            }
        }
    }

    pub fn new(comp: &'a StrEq, matcher: F) -> Self {
        Self { comp, matcher }
    }
}
impl<'a> WithMatcher<'a, Box<dyn Fn(&str, &str) -> bool>> {
    pub fn limited_matcher<M, T>(comp: &'a StrEq, limit: T, matcher: M) -> Self
    where
        M: Fn(&str, &str) -> T + 'static,
        T: Ord + Copy + 'static,
    {
        Self::new(comp, Box::new(move |a, b| matcher(a, b) <= limit))
    }

    pub fn limited_matcher_total<M>(comp: &'a StrEq, limit: f64, matcher: M) -> Self
    where
        M: Fn(&str, &str) -> f64 + 'static,
    {
        Self::new(comp, Box::new(move |a, b| matcher(a, b) >= limit))
    }
}

#[cfg(test)]
mod test {
    use super::{
        ScanAlgorithm, ScanSearcher, ScanSearcherInitError, ScanSearcherOptions,
        ScanSearcherOptionsInitError,
    };
    use crate::topicmodel::dictionary::direction::LanguageKind;
    use crate::topicmodel::dictionary::direction::LanguageKind::{A, B};
    use crate::topicmodel::dictionary::search::impls::scanning::ScanSearcherOptionsInitError::ThresholdMissing;
    use crate::topicmodel::dictionary::{MutableDictionaryWithMeta, StringDictWithMetaDefault};
    use crate::topicmodel::reference::HashRef;
    use either::Either;

    macro_rules! impl_test {
        (
            kind: $kind: expr,
            lang: $lang: expr,
            thresh: $thresh: expr,
            ascii: $ascii: expr,
            dict: $dict: expr,
            inp: $inp: expr $(,)?
        ) => {
            ScanSearcherOptions::new($kind, $lang, $thresh, $ascii).map(|search_opt| {
                ScanSearcher::new($inp, search_opt).map(|searcher| searcher.search($dict))
            })
        };
    }

    #[test]
    fn can_do_all_searched() {
        let mut dict = StringDictWithMetaDefault::default();
        dict.push_invariant("hallo", "hello").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            },
        );
        dict.push_invariant("Hallo", "hello").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            },
        );
        dict.push_invariant("Welt", "world").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            },
        );
        dict.push_invariant("Felix Engl", "Felix Engl")
            .use_consuming(
                |mut value| {
                    value.add_dictionary("test1");
                },
                |mut value| {
                    value.add_dictionary("test2");
                },
            );
        dict.push_invariant("autowaschen", "to do car washing")
            .use_consuming(
                |mut value| {
                    value.add_dictionary("test1");
                },
                |mut value| {
                    value.add_dictionary("test2");
                },
            );
        dict.push_invariant("Hallo Welt", "hello world")
            .use_consuming(
                |mut value| {
                    value.add_dictionary("test1");
                },
                |mut value| {
                    value.add_dictionary("test2");
                },
            );

        dict.push_invariant("Herz", "heart").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            },
        );

        dict.push_invariant("Gürtel", "belt").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            },
        );

        // for x in dict.iter_with_meta() {
        //     println!("{x:?}")
        // }

        fn cr<I: IntoIterator<Item = (LanguageKind, usize, T)>, T: ToString>(
            i: I,
        ) -> Result<
            Result<Vec<(LanguageKind, usize, HashRef<String>)>, ScanSearcherInitError>,
            ScanSearcherOptionsInitError,
        > {
            Ok(Ok(Vec::from_iter(
                i.into_iter()
                    .map(|(a, b, c)| (a, b, HashRef::new(c.to_string()))),
            )))
        }

        for (kind, lang, thresh, ascii, dict, inp, expected) in [
            (
                ScanAlgorithm::ExactMatch,
                None,
                None,
                false,
                &dict,
                "hallo".to_string().into(),
                cr([(A, 0, "hallo")]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                Some(A),
                None,
                false,
                &dict,
                "hallo".to_string().into(),
                cr([(A, 0, "hallo")]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                Some(B),
                None,
                false,
                &dict,
                "hallo".to_string().into(),
                cr::<_, &str>([]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                None,
                Some(Either::Left(0)),
                false,
                &dict,
                "hallo".to_string().into(),
                cr([(A, 0, "hallo")]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                None,
                Some(Either::Right(1.0)),
                false,
                &dict,
                "hallo".to_string().into(),
                cr([(A, 0, "hallo")]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                None,
                None,
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                Some(A),
                None,
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                Some(B),
                None,
                true,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                None,
                Some(Either::Left(0)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::ExactMatch,
                None,
                Some(Either::Right(1.0)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::Contains,
                None,
                None,
                false,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::Contains,
                Some(A),
                None,
                false,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::Contains,
                Some(B),
                None,
                false,
                &dict,
                "He".to_string().into(),
                cr::<_, &str>([]),
            ),
            (
                ScanAlgorithm::Contains,
                None,
                Some(Either::Left(0)),
                false,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::Contains,
                None,
                Some(Either::Right(1.0)),
                false,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::Contains,
                None,
                None,
                true,
                &dict,
                "He".to_string().into(),
                cr([
                    (A, 4, "autowaschen"),
                    (A, 6, "Herz"),
                    (B, 0, "hello"),
                    (B, 4, "hello world"),
                    (B, 5, "heart"),
                ]),
            ),
            (
                ScanAlgorithm::Contains,
                Some(A),
                None,
                true,
                &dict,
                "He".to_string().into(),
                cr([(A, 4, "autowaschen"), (A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::Contains,
                Some(B),
                None,
                true,
                &dict,
                "He".to_string().into(),
                cr([(B, 0, "hello"), (B, 4, "hello world"), (B, 5, "heart")]),
            ),
            (
                ScanAlgorithm::Contains,
                None,
                Some(Either::Left(0)),
                true,
                &dict,
                "He".to_string().into(),
                cr([
                    (A, 4, "autowaschen"),
                    (A, 6, "Herz"),
                    (B, 0, "hello"),
                    (B, 4, "hello world"),
                    (B, 5, "heart"),
                ]),
            ),
            (
                ScanAlgorithm::Contains,
                None,
                Some(Either::Right(1.0)),
                true,
                &dict,
                "He".to_string().into(),
                cr([
                    (A, 4, "autowaschen"),
                    (A, 6, "Herz"),
                    (B, 0, "hello"),
                    (B, 4, "hello world"),
                    (B, 5, "heart"),
                ]),
            ),
            (
                ScanAlgorithm::StartsWith,
                None,
                None,
                false,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::StartsWith,
                Some(A),
                None,
                false,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::StartsWith,
                Some(B),
                None,
                false,
                &dict,
                "He".to_string().into(),
                cr::<_, &str>([]),
            ),
            (
                ScanAlgorithm::StartsWith,
                None,
                Some(Either::Left(0)),
                false,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::StartsWith,
                None,
                Some(Either::Right(1.0)),
                false,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::StartsWith,
                None,
                None,
                true,
                &dict,
                "He".to_string().into(),
                cr([
                    (A, 6, "Herz"),
                    (B, 0, "hello"),
                    (B, 4, "hello world"),
                    (B, 5, "heart"),
                ]),
            ),
            (
                ScanAlgorithm::StartsWith,
                Some(A),
                None,
                true,
                &dict,
                "He".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::StartsWith,
                Some(B),
                None,
                true,
                &dict,
                "He".to_string().into(),
                cr([(B, 0, "hello"), (B, 4, "hello world"), (B, 5, "heart")]),
            ),
            (
                ScanAlgorithm::StartsWith,
                None,
                Some(Either::Left(0)),
                true,
                &dict,
                "He".to_string().into(),
                cr([
                    (A, 6, "Herz"),
                    (B, 0, "hello"),
                    (B, 4, "hello world"),
                    (B, 5, "heart"),
                ]),
            ),
            (
                ScanAlgorithm::StartsWith,
                None,
                Some(Either::Right(1.0)),
                true,
                &dict,
                "He".to_string().into(),
                cr([
                    (A, 6, "Herz"),
                    (B, 0, "hello"),
                    (B, 4, "hello world"),
                    (B, 5, "heart"),
                ]),
            ),
            (
                ScanAlgorithm::EndsWith,
                None,
                None,
                false,
                &dict,
                "lt".to_string().into(),
                cr([(A, 2, "Welt"), (A, 5, "Hallo Welt"), (B, 6, "belt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                Some(A),
                None,
                false,
                &dict,
                "lt".to_string().into(),
                cr([(A, 2, "Welt"), (A, 5, "Hallo Welt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                Some(B),
                None,
                false,
                &dict,
                "lt".to_string().into(),
                cr::<_, &str>([(B, 6, "belt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                None,
                Some(Either::Left(0)),
                false,
                &dict,
                "lt".to_string().into(),
                cr([(A, 2, "Welt"), (A, 5, "Hallo Welt"), (B, 6, "belt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                None,
                Some(Either::Right(1.0)),
                false,
                &dict,
                "lt".to_string().into(),
                cr([(A, 2, "Welt"), (A, 5, "Hallo Welt"), (B, 6, "belt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                None,
                None,
                true,
                &dict,
                "lt".to_string().into(),
                cr([(A, 2, "Welt"), (A, 5, "Hallo Welt"), (B, 6, "belt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                Some(A),
                None,
                true,
                &dict,
                "lt".to_string().into(),
                cr([(A, 2, "Welt"), (A, 5, "Hallo Welt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                Some(B),
                None,
                true,
                &dict,
                "lt".to_string().into(),
                cr([(B, 6, "belt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                None,
                Some(Either::Left(0)),
                true,
                &dict,
                "lt".to_string().into(),
                cr([(A, 2, "Welt"), (A, 5, "Hallo Welt"), (B, 6, "belt")]),
            ),
            (
                ScanAlgorithm::EndsWith,
                None,
                Some(Either::Right(1.0)),
                true,
                &dict,
                "lt".to_string().into(),
                cr([(A, 2, "Welt"), (A, 5, "Hallo Welt"), (B, 6, "belt")]),
            ),
            (
                ScanAlgorithm::Regex,
                None,
                None,
                false,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(A, 6, "Herz"), (B, 3, "to do car washing")]),
            ),
            (
                ScanAlgorithm::Regex,
                Some(A),
                None,
                false,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::Regex,
                Some(B),
                None,
                false,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr::<_, &str>([(B, 3, "to do car washing")]),
            ),
            (
                ScanAlgorithm::Regex,
                None,
                Some(Either::Left(0)),
                false,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(A, 6, "Herz"), (B, 3, "to do car washing")]),
            ),
            (
                ScanAlgorithm::Regex,
                None,
                Some(Either::Right(1.0)),
                false,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(A, 6, "Herz"), (B, 3, "to do car washing")]),
            ),
            (
                ScanAlgorithm::Regex,
                None,
                None,
                true,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(A, 6, "Herz"), (B, 3, "to do car washing")]),
            ),
            (
                ScanAlgorithm::Regex,
                Some(A),
                None,
                true,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(A, 6, "Herz")]),
            ),
            (
                ScanAlgorithm::Regex,
                Some(B),
                None,
                true,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(B, 3, "to do car washing")]),
            ),
            (
                ScanAlgorithm::Regex,
                None,
                Some(Either::Left(0)),
                true,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(A, 6, "Herz"), (B, 3, "to do car washing")]),
            ),
            (
                ScanAlgorithm::Regex,
                None,
                Some(Either::Right(1.0)),
                true,
                &dict,
                ".*(in|er).*".to_string().into(),
                cr([(A, 6, "Herz"), (B, 3, "to do car washing")]),
            ),
            (
                ScanAlgorithm::Hamming,
                None,
                Some(Either::Left(3)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo"), (B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::Hamming,
                Some(A),
                Some(Either::Left(3)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::Hamming,
                Some(B),
                Some(Either::Left(3)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([(B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::Hamming,
                None,
                Some(Either::Left(3)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo"), (B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::Hamming,
                Some(A),
                Some(Either::Left(3)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::Hamming,
                Some(B),
                Some(Either::Left(3)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([(B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::Hamming,
                None,
                None,
                false,
                &dict,
                "Hallo".to_string().into(),
                Err(ThresholdMissing {
                    expected: "usize",
                    kind: ScanAlgorithm::Hamming,
                }),
            ),
            (
                ScanAlgorithm::Levensthein,
                None,
                Some(Either::Left(1)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::Levensthein,
                Some(A),
                Some(Either::Left(1)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::Levensthein,
                Some(B),
                Some(Either::Left(1)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([]),
            ),
            (
                ScanAlgorithm::Levensthein,
                None,
                Some(Either::Left(1)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo"), (B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::Levensthein,
                Some(A),
                Some(Either::Left(1)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::Levensthein,
                Some(B),
                Some(Either::Left(1)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([(B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::Levensthein,
                None,
                None,
                false,
                &dict,
                "Hallo".to_string().into(),
                Err(ThresholdMissing {
                    expected: "usize",
                    kind: ScanAlgorithm::Levensthein,
                }),
            ),
            (
                ScanAlgorithm::OsaDistance,
                None,
                Some(Either::Left(3)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo"), (B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::OsaDistance,
                Some(A),
                Some(Either::Left(3)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::OsaDistance,
                Some(B),
                Some(Either::Left(3)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([(B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::OsaDistance,
                None,
                Some(Either::Left(3)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo"), (B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::OsaDistance,
                Some(A),
                Some(Either::Left(3)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::OsaDistance,
                Some(B),
                Some(Either::Left(3)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([(B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::OsaDistance,
                None,
                None,
                false,
                &dict,
                "Hallo".to_string().into(),
                Err(ThresholdMissing {
                    expected: "usize",
                    kind: ScanAlgorithm::OsaDistance,
                }),
            ),
            (
                ScanAlgorithm::NormalizedLevensthein,
                None,
                Some(Either::Right(0.6)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo"), (B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::NormalizedLevensthein,
                Some(A),
                Some(Either::Right(0.7)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::NormalizedLevensthein,
                Some(B),
                Some(Either::Right(0.6)),
                false,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([(B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::NormalizedLevensthein,
                None,
                Some(Either::Right(0.7)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo"), (B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::NormalizedLevensthein,
                Some(A),
                Some(Either::Right(0.7)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr([(A, 0, "hallo"), (A, 1, "Hallo")]),
            ),
            (
                ScanAlgorithm::NormalizedLevensthein,
                Some(B),
                Some(Either::Right(0.7)),
                true,
                &dict,
                "Hallo".to_string().into(),
                cr::<_, &str>([(B, 0, "hello")]),
            ),
            (
                ScanAlgorithm::NormalizedLevensthein,
                None,
                None,
                false,
                &dict,
                "Hallo".to_string().into(),
                Err(ThresholdMissing {
                    expected: "f64",
                    kind: ScanAlgorithm::NormalizedLevensthein,
                }),
            ),
        ] {
            let inp_str = format!("{inp:?}");
            let expected: Result<
                Result<Vec<(LanguageKind, usize, HashRef<String>)>, ScanSearcherInitError>,
                ScanSearcherOptionsInitError,
            > = expected;
            let result: Result<
                Result<Vec<(LanguageKind, usize, HashRef<String>)>, ScanSearcherInitError>,
                ScanSearcherOptionsInitError,
            > = impl_test!(
                kind: kind,
                lang: lang,
                thresh: thresh,
                ascii: ascii,
                dict: dict,
                inp: inp
            );

            match (expected, result) {
                (Ok(Ok(mut expected)), Ok(Ok(mut result))) => {
                    expected.sort();
                    result.sort();
                    if expected != result {
                        println!("Diff: {kind} {lang:?} {thresh:?} {ascii} {inp_str}; result={result:?} expected={expected:?}")
                    }
                }
                (a, b) => {
                    if a != b {
                        println!("Error: {kind} {lang:?} {thresh:?} {ascii} {inp_str}; result={b:?} expected={a:?}")
                    }
                }
            }
        }
    }
}
