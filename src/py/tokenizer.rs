use std::collections::HashMap;
use std::mem::transmute;
use std::sync::Arc;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, AhoCorasickKind, BuildError, dfa, MatchKind, StartKind};
use aho_corasick::nfa::{contiguous, noncontiguous};
use aho_corasick::nfa::noncontiguous::{Builder, NFA};
use charabia::{Script, TokenizerBuilder};
use charabia::Language;
use charabia::normalizer::{ClassifierOption, NormalizerOption};
use charabia::segmenter::SegmenterOption;
use fst::Set;
use itertools::Itertools;
use pyo3::{Bound, FromPyObject, pyclass, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use crate::py::enum_mapping::map_enum;

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyTokenizerBuilder {
    stop_words: Option<PyStopWords>,
    words_dict: Option<Vec<String>>,
    normalizer_option: PyNormalizerOption,
    segmenter_option: PySegmenterOption,
}

#[pymethods]
impl PyTokenizerBuilder {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stop_words<'py>(mut slf: Bound<'py, Self>, stop_words: PyStopWords) -> Bound<'py, Self> {
        slf.borrow_mut().stop_words = Some(stop_words);
        slf
    }

    pub fn separators<'py>(mut slf: Bound<'py, Self>, separators: Vec<String>) -> PyResult<Bound<'py, Self>> {
        slf.borrow_mut().normalizer_option.classifier.set_separators(Some(separators))?;
        Ok(slf)
    }

    pub fn words_dict<'py>(mut slf: Bound<'py, Self>, words: Vec<String>) -> Bound<'py, Self> {
        slf.borrow_mut().words_dict = Some(words);
        slf
    }

    pub fn create_char_map<'py>(mut slf: Bound<'py, Self>, create_char_map: bool) -> Bound<'py, Self> {
        slf.borrow_mut().normalizer_option.create_char_map = create_char_map;
        slf
    }

    pub fn lossy_normalization<'py>(mut slf: Bound<'py, Self>, lossy: bool) -> Bound<'py, Self> {
        slf.borrow_mut().normalizer_option.lossy = lossy;
        slf
    }

    pub fn allow_list<'py>(mut slf: Bound<'py, Self>, allow_list:  HashMap<PyScript, Vec<PyLanguage>>) -> Bound<'py, Self> {
        slf.borrow_mut().segmenter_option.set_allow_list(Some(allow_list));
        slf
    }
}

impl PyTokenizerBuilder {
    pub fn as_tokenizer_builder(&self) -> TokenizerBuilder<> {
        TokenizerBuilder::new()
    }
}


#[pyclass]
#[derive(Clone, Debug, Default)]
#[repr(transparent)]
pub struct PyStopWords(Set<Vec<u8>>);

#[pymethods]
impl PyStopWords {
    #[new]
    pub fn new(value: Vec<String>) -> PyResult<Self> {
        match Set::from_iter(value) {
            Ok(value) => {Ok(Self(value))}
            Err(value) => {Err(PyValueError::new_err(value.to_string()))}
        }
    }
}

impl PyStopWords {
    fn as_classifier_stopwords(&self) -> Set<&[u8]> {
        Set::new(self.0.as_fst().as_bytes()).unwrap()
    }
}

impl AsRef<Set<Vec<u8>>> for PyStopWords {
    fn as_ref(&self) -> &Set<Vec<u8>> {
        &self.0
    }
}

#[pyclass(set_all, get_all)]
#[derive(Debug, Clone, Default)]
pub struct PyNormalizerOption {
    create_char_map: bool,
    classifier: PyClassifierOption,
    lossy: bool,
}

#[pymethods]
impl PyNormalizerOption {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }
}

impl PyNormalizerOption {
    pub fn as_normalizer_option<'a>(&'a self) -> NormalizerOption<'a> {
        NormalizerOption {
            create_char_map: self.create_char_map,
            classifier: self.classifier.as_classifier_option(),
            lossy: self.lossy
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct PyClassifierOption {
    #[pyo3(get, set)]
    stop_words: Option<PyStopWords>,
    separators: Option<SpecialVec>,
}

#[derive(Debug, Clone)]
struct SpecialVec {
    inner: Arc<Vec<String>>,
    references: Arc<Vec<*const str>>
}

unsafe impl Send for SpecialVec{}
unsafe impl Sync for SpecialVec{}

impl SpecialVec {
    pub fn new(inner: Vec<String>) -> Self {
        let references = inner.iter().map(|value| value.as_str() as *const str).collect_vec();
        Self {
            inner: Arc::new(inner),
            references: Arc::new(references)
        }
    }

    pub fn as_slice(&self) -> &[&str] {
        // A &str is basically a *const str but with a safe livetime.
        unsafe {transmute(self.references.as_slice())}
    }

    pub fn copy_string_vec(&self) -> Vec<String> {
        (*self.inner).clone()
    }
}

#[cfg(test)]
mod special_vec_test {
    use crate::py::tokenizer::SpecialVec;

    #[test]
    fn can_be_used_safely(){
        let v = SpecialVec::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let r = v.as_slice();

        println!("{:?}", v.as_ref());
        println!("{:?}", r);
    }
}


impl AsRef<[String]> for SpecialVec {
    fn as_ref(&self) -> &[String] {
        self.inner.as_slice()
    }
}



#[pymethods]
impl PyClassifierOption {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    #[getter]
    pub fn get_separators(&self) -> PyResult<Option<Vec<String>>> {
        match &self.separators {
            None => {Ok(None)}
            Some(value) => {
                Ok(Some(value.copy_string_vec()))
            }
        }
    }

    #[setter]
    pub fn set_separators(&mut self, value: Option<Vec<String>>) -> PyResult<()> {
        match value {
            None => {
                self.separators = None;
            }
            Some(value) => {
                self.separators = Some(SpecialVec::new(value));
            }
        }
        Ok(())
    }
}


impl PyClassifierOption {
    pub fn as_classifier_option<'a>(&'a self) -> ClassifierOption<'a> {
        let stop_words = match &self.stop_words {
            None => {None}
            Some(value) => {Some(value.as_classifier_stopwords())}
        };
        let separators = match &self.separators {
            None => {None}
            Some(value) => {Some(value.as_slice())}
        };
        ClassifierOption {
            stop_words,
            separators
        }
    }
}


#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct PySegmenterOption {
    #[pyo3(get, set)]
    aho: Option<PyAhoCorasick>,
    allow_list: Option<HashMap<Script, Vec<Language>>>
}

#[pymethods]
impl PySegmenterOption {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[setter]
    pub fn set_allow_list(&mut self, allow_list: Option<HashMap<PyScript, Vec<PyLanguage>>>) {
        self.allow_list = allow_list.map(
            |value| {
                value.into_iter().map(|(k, v)| (k.into(), v.into_iter().map(|lang| lang.into()).collect())).collect()
            }
        )
    }

    #[getter]
    pub fn get_allow_list(&self) -> Option<HashMap<PyScript, Vec<PyLanguage>>> {
        self.allow_list.as_ref().map(|value| {
            value.iter().map(|(k, v)| {
                (k.clone().into(), v.iter().map(|lang| lang.clone().into()).collect())
            }).collect()
        })
    }
}

impl PySegmenterOption {
    pub fn as_segmenter_option(&self) -> SegmenterOption {
        SegmenterOption {
            aho: self.aho.clone().map(|value| value.into()),
            allow_list: self.allow_list.as_ref()
        }
    }
}


#[pyclass]
#[derive(Debug, Clone)]
pub struct PyAhoCorasick(AhoCorasick);

impl From<AhoCorasick> for PyAhoCorasick {
    #[inline(always)]
    fn from(value: AhoCorasick) -> Self {
        Self(value)
    }
}

impl Into<AhoCorasick> for PyAhoCorasick {
    #[inline(always)]
    fn into(self) -> AhoCorasick {
        self.0
    }
}

#[pyclass]
#[derive(Debug, Clone, Default)]
#[repr(transparent)]
pub struct PyAhoCorasickBuilder(AhoCorasickBuilder);

#[pymethods]
impl PyAhoCorasickBuilder {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an Aho-Corasick automaton using the configuration set on this
    /// builder.
    ///
    /// A builder may be reused to create more automatons.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::{AhoCorasickBuilder, PatternID};
    ///
    /// let patterns = &["foo", "bar", "baz"];
    /// let ac = AhoCorasickBuilder::new().build(patterns).unwrap();
    /// assert_eq!(
    ///     Some(PatternID::must(1)),
    ///     ac.find("xxx bar xxx").map(|m| m.pattern()),
    /// );
    /// ```
    pub fn build(&self, patterns: Vec<String>) -> PyResult<PyAhoCorasick> {
        match self.0.build(patterns) {
            Ok(value) => {
                Ok(value.into())
            }
            Err(err) => {
                Err(PyValueError::new_err(err.to_string()))
            }
        }
    }

    /// Set the desired match semantics.
    ///
    /// The default is [`PyMatchKind::Standard`], which corresponds to the match
    /// semantics supported by the standard textbook description of the
    /// Aho-Corasick algorithm. Namely, matches are reported as soon as they
    /// are found. Moreover, this is the only way to get overlapping matches
    /// or do stream searching.
    ///
    /// The other kinds of match semantics that are supported are
    /// [`PyMatchKind::LeftmostFirst`] and [`PyMatchKind::LeftmostLongest`]. The
    /// former corresponds to the match you would get if you were to try to
    /// match each pattern at each position in the haystack in the same order
    /// that you give to the automaton. That is, it returns the leftmost match
    /// corresponding to the earliest pattern given to the automaton. The
    /// latter corresponds to finding the longest possible match among all
    /// leftmost matches.
    ///
    /// For more details on match semantics, see the [documentation for
    /// `MatchKind`](MatchKind).
    ///
    /// Note that setting this to [`PyMatchKind::LeftmostFirst`] or
    /// [`PyMatchKind::LeftmostLongest`] will cause some search routines on
    /// [`PyAhoCorasick`] to return an error (or panic if you're using the
    /// infallible API). Notably, this includes stream and overlapping
    /// searches.
    ///
    /// # Examples
    ///
    /// In these examples, we demonstrate the differences between match
    /// semantics for a particular set of patterns in a specific order:
    /// `b`, `abc`, `abcd`.
    ///
    /// Standard semantics:
    ///
    /// ```
    /// use ldatranslate::py::{PyAhoCorasick, PyMatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = PyAhoCorasick::builder()
    ///     .match_kind(PyMatchKind::Standard) // default, not necessary
    ///     .build(patterns)
    ///     .unwrap();
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("b", &haystack[mat.start()..mat.end()]);
    /// ```
    ///
    /// Leftmost-first semantics:
    ///
    /// ```
    /// use ldatranslate::py::{PyAhoCorasick, PyMatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = PyAhoCorasick::builder()
    ///     .match_kind(PyAhoCorasick::LeftmostFirst)
    ///     .build(patterns)
    ///     .unwrap();
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("abc", &haystack[mat.start()..mat.end()]);
    /// ```
    ///
    /// Leftmost-longest semantics:
    ///
    /// ```
    /// use ldatranslate::py::{PyAhoCorasick, PyMatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = PyAhoCorasick::builder()
    ///     .match_kind(PyAhoCorasick::LeftmostLongest)
    ///     .build(patterns)
    ///     .unwrap();
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("abcd", &haystack[mat.start()..mat.end()]);
    /// ```
    pub fn match_kind<'py>(mut slf: Bound<'py, Self>, kind: PyMatchKind) -> Bound<'py, Self> {
        slf.borrow_mut().0.match_kind(kind.into());
        slf
    }

    /// Sets the starting state configuration for the automaton.
    ///
    /// Every Aho-Corasick automaton is capable of having two start states: one
    /// that is used for unanchored searches and one that is used for anchored
    /// searches. Some automatons, like the NFAs, support this with almost zero
    /// additional cost. Other automatons, like the DFA, require two copies of
    /// the underlying transition table to support both simultaneously.
    ///
    /// Because there may be an added non-trivial cost to supporting both, it
    /// is possible to configure which starting state configuration is needed.
    ///
    /// Indeed, since anchored searches tend to be somewhat more rare,
    /// _only_ unanchored searches are supported by default. Thus,
    /// [`PyStartKind::Unanchored`] is the default.
    ///
    /// Note that when this is set to [`PyStartKind::Unanchored`], then
    /// running an anchored search will result in an error (or a panic
    /// if using the infallible APIs). Similarly, when this is set to
    /// [`PyStartKind::Anchored`], then running an unanchored search will
    /// result in an error (or a panic if using the infallible APIs). When
    /// [`PyStartKind::Both`] is used, then both unanchored and anchored searches
    /// are always supported.
    ///
    /// Also note that even if an `PyAhoCorasick` searcher is using an NFA
    /// internally (which always supports both unanchored and anchored
    /// searches), an error will still be reported for a search that isn't
    /// supported by the configuration set via this method. This means,
    /// for example, that an error is never dependent on which internal
    /// implementation of Aho-Corasick is used.
    ///
    /// # Example: anchored search
    ///
    /// This shows how to build a searcher that only supports anchored
    /// searches:
    ///
    /// ```
    /// use aho_corasick::{
    ///     AhoCorasick, Anchored, Input, Match, MatchKind, StartKind,
    /// };
    ///
    /// let ac = AhoCorasick::builder()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .start_kind(StartKind::Anchored)
    ///     .build(&["b", "abc", "abcd"])
    ///     .unwrap();
    ///
    /// // An unanchored search is not supported! An error here is guaranteed
    /// // given the configuration above regardless of which kind of
    /// // Aho-Corasick implementation ends up being used internally.
    /// let input = Input::new("foo abcd").anchored(Anchored::No);
    /// assert!(ac.try_find(input).is_err());
    ///
    /// let input = Input::new("foo abcd").anchored(Anchored::Yes);
    /// assert_eq!(None, ac.try_find(input)?);
    ///
    /// let input = Input::new("abcd").anchored(Anchored::Yes);
    /// assert_eq!(Some(Match::must(1, 0..3)), ac.try_find(input)?);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Example: unanchored and anchored searches
    ///
    /// This shows how to build a searcher that supports both unanchored and
    /// anchored searches:
    ///
    /// ```
    /// use aho_corasick::{
    ///     AhoCorasick, Anchored, Input, Match, MatchKind, StartKind,
    /// };
    ///
    /// let ac = AhoCorasick::builder()
    ///     .match_kind(MatchKind::LeftmostFirst)
    ///     .start_kind(StartKind::Both)
    ///     .build(&["b", "abc", "abcd"])
    ///     .unwrap();
    ///
    /// let input = Input::new("foo abcd").anchored(Anchored::No);
    /// assert_eq!(Some(Match::must(1, 4..7)), ac.try_find(input)?);
    ///
    /// let input = Input::new("foo abcd").anchored(Anchored::Yes);
    /// assert_eq!(None, ac.try_find(input)?);
    ///
    /// let input = Input::new("abcd").anchored(Anchored::Yes);
    /// assert_eq!(Some(Match::must(1, 0..3)), ac.try_find(input)?);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn start_kind<'py>(mut slf: Bound<'py, Self>, kind: PyStartKind) -> Bound<'py, Self> {
        slf.borrow_mut().0.start_kind(kind.into());
        slf
    }

    /// Enable ASCII-aware case insensitive matching.
    ///
    /// When this option is enabled, searching will be performed without
    /// respect to case for ASCII letters (`a-z` and `A-Z`) only.
    ///
    /// Enabling this option does not change the search algorithm, but it may
    /// increase the size of the automaton.
    ///
    /// **NOTE:** It is unlikely that support for Unicode case folding will
    /// be added in the future. The ASCII case works via a simple hack to the
    /// underlying automaton, but full Unicode handling requires a fair bit of
    /// sophistication. If you do need Unicode handling, you might consider
    /// using the [`regex` crate](https://docs.rs/regex) or the lower level
    /// [`regex-automata` crate](https://docs.rs/regex-automata).
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use aho_corasick::AhoCorasick;
    ///
    /// let patterns = &["FOO", "bAr", "BaZ"];
    /// let haystack = "foo bar baz";
    ///
    /// let ac = AhoCorasick::builder()
    ///     .ascii_case_insensitive(true)
    ///     .build(patterns)
    ///     .unwrap();
    /// assert_eq!(3, ac.find_iter(haystack).count());
    /// ```
    pub fn ascii_case_insensitive<'py>(mut slf: Bound<'py, Self>, yes: bool) -> Bound<'py, Self> {
        slf.borrow_mut().0.ascii_case_insensitive(yes);
        slf
    }

    /// Choose the type of underlying automaton to use.
    ///
    /// Currently, there are four choices:
    ///
    /// * [`PyAhoCorasickKind::NoncontiguousNFA`] instructs the searcher to
    /// use a [`noncontiguous::NFA`]. A noncontiguous NFA is the fastest to
    /// be built, has moderate memory usage and is typically the slowest to
    /// execute a search.
    /// * [`PyAhoCorasickKind::ContiguousNFA`] instructs the searcher to use a
    /// [`contiguous::NFA`]. A contiguous NFA is a little slower to build than
    /// a noncontiguous NFA, has excellent memory usage and is typically a
    /// little slower than a DFA for a search.
    /// * [`PyAhoCorasickKind::DFA`] instructs the searcher to use a
    /// [`dfa::DFA`]. A DFA is very slow to build, uses exorbitant amounts of
    /// memory, but will typically execute searches the fastest.
    /// * `None` (the default) instructs the searcher to choose the "best"
    /// Aho-Corasick implementation. This choice is typically based primarily
    /// on the number of patterns.
    ///
    /// Setting this configuration does not change the time complexity for
    /// constructing the Aho-Corasick automaton (which is `O(p)` where `p`
    /// is the total number of patterns being compiled). Setting this to
    /// [`PyAhoCorasickKind::DFA`] does however reduce the time complexity of
    /// non-overlapping searches from `O(n + p)` to `O(n)`, where `n` is the
    /// length of the haystack.
    ///
    /// In general, you should probably stick to the default unless you have
    /// some kind of reason to use a specific Aho-Corasick implementation. For
    /// example, you might choose `PyAhoCorasickKind::DFA` if you don't care
    /// about memory usage and want the fastest possible search times.
    ///
    /// Setting this guarantees that the searcher returned uses the chosen
    /// implementation. If that implementation could not be constructed, then
    /// an error will be returned. In contrast, when `None` is used, it is
    /// possible for it to attempt to construct, for example, a contiguous
    /// NFA and have it fail. In which case, it will fall back to using a
    /// noncontiguous NFA.
    ///
    /// If `None` is given, then one may use [`PyAhoCorasickKind::kind`] to determine
    /// which Aho-Corasick implementation was chosen.
    ///
    /// Note that the heuristics used for choosing which `PyAhoCorasickKind`
    /// may be changed in a semver compatible release.
    pub fn kind<'py>(mut slf: Bound<'py, Self>, kind: Option<PyAhoCorasickKind>) -> Bound<'py, Self> {
        slf.borrow_mut().0.kind(kind.map(Into::into));
        slf
    }

    /// Enable heuristic prefilter optimizations.
    ///
    /// When enabled, searching will attempt to quickly skip to match
    /// candidates using specialized literal search routines. A prefilter
    /// cannot always be used, and is generally treated as a heuristic. It
    /// can be useful to disable this if the prefilter is observed to be
    /// sub-optimal for a particular workload.
    ///
    /// Currently, prefilters are typically only active when building searchers
    /// with a small (less than 100) number of patterns.
    ///
    /// This is enabled by default.
    pub fn prefilter<'py>(mut slf: Bound<'py, Self>, yes: bool) -> Bound<'py, Self> {
        slf.borrow_mut().0.prefilter(yes);
        slf
    }

    /// Set the limit on how many states use a dense representation for their
    /// transitions. Other states will generally use a sparse representation.
    ///
    /// A dense representation uses more memory but is generally faster, since
    /// the next transition in a dense representation can be computed in a
    /// constant number of instructions. A sparse representation uses less
    /// memory but is generally slower, since the next transition in a sparse
    /// representation requires executing a variable number of instructions.
    ///
    /// This setting is only used when an Aho-Corasick implementation is used
    /// that supports the dense versus sparse representation trade off. Not all
    /// do.
    ///
    /// This limit is expressed in terms of the depth of a state, i.e., the
    /// number of transitions from the starting state of the automaton. The
    /// idea is that most of the time searching will be spent near the starting
    /// state of the automaton, so states near the start state should use a
    /// dense representation. States further away from the start state would
    /// then use a sparse representation.
    ///
    /// By default, this is set to a low but non-zero number. Setting this to
    /// `0` is almost never what you want, since it is likely to make searches
    /// very slow due to the start state itself being forced to use a sparse
    /// representation. However, it is unlikely that increasing this number
    /// will help things much, since the most active states have a small depth.
    /// More to the point, the memory usage increases superlinearly as this
    /// number increases.
    pub fn dense_depth<'py>(mut slf: Bound<'py, Self>, depth: usize) -> Bound<'py, Self> {
        slf.borrow_mut().0.dense_depth(depth);
        slf
    }

    /// A debug setting for whether to attempt to shrink the size of the
    /// automaton's alphabet or not.
    ///
    /// This option is enabled by default and should never be disabled unless
    /// one is debugging the underlying automaton.
    ///
    /// When enabled, some (but not all) Aho-Corasick automatons will use a map
    /// from all possible bytes to their corresponding equivalence class. Each
    /// equivalence class represents a set of bytes that does not discriminate
    /// between a match and a non-match in the automaton.
    ///
    /// The advantage of this map is that the size of the transition table can
    /// be reduced drastically from `#states * 256 * sizeof(u32)` to
    /// `#states * k * sizeof(u32)` where `k` is the number of equivalence
    /// classes (rounded up to the nearest power of 2). As a result, total
    /// space usage can decrease substantially. Moreover, since a smaller
    /// alphabet is used, automaton compilation becomes faster as well.
    ///
    /// **WARNING:** This is only useful for debugging automatons. Disabling
    /// this does not yield any speed advantages. Namely, even when this is
    /// disabled, a byte class map is still used while searching. The only
    /// difference is that every byte will be forced into its own distinct
    /// equivalence class. This is useful for debugging the actual generated
    /// transitions because it lets one see the transitions defined on actual
    /// bytes instead of the equivalence classes.
    pub fn byte_classes<'py>(mut slf: Bound<'py, Self>, yes: bool) -> Bound<'py, Self> {
        slf.borrow_mut().0.byte_classes(yes);
        slf
    }


}

map_enum!(
    impl PyScript for Script {
        Arabic,
        Armenian,
        Bengali,
        Cyrillic,
        Devanagari,
        Ethiopic,
        Georgian,
        Greek,
        Gujarati,
        Gurmukhi,
        Hangul,
        Hebrew,
        Kannada,
        Khmer,
        Latin,
        Malayalam,
        Myanmar,
        Oriya,
        Sinhala,
        Tamil,
        Telugu,
        Thai,
        Cj,
        Other
    }
);

map_enum!(
    impl PyLanguage for Language {
        Epo,
        Eng,
        Rus,
        Cmn,
        Spa,
        Por,
        Ita,
        Ben,
        Fra,
        Deu,
        Ukr,
        Kat,
        Ara,
        Hin,
        Jpn,
        Heb,
        Yid,
        Pol,
        Amh,
        Jav,
        Kor,
        Nob,
        Dan,
        Swe,
        Fin,
        Tur,
        Nld,
        Hun,
        Ces,
        Ell,
        Bul,
        Bel,
        Mar,
        Kan,
        Ron,
        Slv,
        Hrv,
        Srp,
        Mkd,
        Lit,
        Lav,
        Est,
        Tam,
        Vie,
        Urd,
        Tha,
        Guj,
        Uzb,
        Pan,
        Aze,
        Ind,
        Tel,
        Pes,
        Mal,
        Ori,
        Mya,
        Nep,
        Sin,
        Khm,
        Tuk,
        Aka,
        Zul,
        Sna,
        Afr,
        Lat,
        Slk,
        Cat,
        Tgl,
        Hye,
        Other
    }
);



map_enum!(
    impl PyMatchKind for non_exhaustive MatchKind {
        Standard,
        LeftmostFirst,
        LeftmostLongest
    }
);

map_enum!(
    impl PyAhoCorasickKind for non_exhaustive AhoCorasickKind {
        NoncontiguousNFA,
        ContiguousNFA,
        DFA
    }
);

map_enum!(
    impl PyStartKind for StartKind {
        Both,
        Unanchored,
        Anchored
    }
);