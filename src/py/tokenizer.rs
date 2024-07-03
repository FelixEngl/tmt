use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::{env, io};
use std::io::{BufReader, BufWriter};
use std::iter::Map;
use std::mem::transmute;
use std::ops::Deref;
use std::path::{Path};
use std::sync::{Arc, Mutex};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, AhoCorasickKind, MatchKind, StartKind};
use charabia::{Script, SeparatorKind, Token, TokenKind};
use charabia::Language;
use charabia::normalizer::{ClassifierOption, NormalizerOption};
use charabia::segmenter::SegmenterOption;
use fst::raw::Fst;
use thiserror::Error;
use fst::Set;
use itertools::Itertools;
use pyo3::{Bound, FromPyObject, IntoPy, pyclass, pyfunction, pymethods, PyObject, PyRef, PyResult, Python, wrap_pyfunction};
use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::prelude::{PyModule, PyModuleMethods};
use rayon::prelude::*;
use rust_stemmers::{Algorithm};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::de::IoRead;
use serde_json::{Deserializer, Error, StreamDeserializer, Value};
use crate::aligned_data::{AlignedArticle, Article, IntoJsonPickleDeserializerIterator, JsonPickleDeserializerIterator};
use crate::py::enum_mapping::map_enum;
use crate::py::helpers::{LanguageHintValue, SpecialVec};
use crate::py::vocabulary::PyVocabulary;
use crate::tokenizer::{Tokenizer, TokenizerBuilder};
use crate::toolkit::with_ref_of::{SupportsWithRef, WithValue};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::vocabulary::{BasicVocabulary};


enum JsonPickleIterWrapper<'a, T> {
    Pickle(JsonPickleDeserializerIterator<StreamDeserializer<'a, IoRead<BufReader<File>>, Value>, T>),
    Unpickle(StreamDeserializer<'a, IoRead<BufReader<File>>, T>)
}

impl<'a, T> Iterator for JsonPickleIterWrapper<'a, T> where T: DeserializeOwned {
    type Item = serde_json::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            JsonPickleIterWrapper::Pickle(ref mut value) => {
                value.next()
            }
            JsonPickleIterWrapper::Unpickle(ref mut value) => {
                value.next()
            }
        }
    }
}

type DeserializeIter<'a> = JsonPickleIterWrapper<'a, PyAlignedArticle>;

#[pyclass]
#[derive(Clone)]
pub struct PyAlignedArticleIter {
    iter: Arc<Mutex<DeserializeIter<'static>>>
}

impl PyAlignedArticleIter {
    fn new(iterator: DeserializeIter) -> Self {
        Self {
            iter: Arc::new(Mutex::new(unsafe{transmute(iterator)}))
        }
    }
}

#[pymethods]
impl PyAlignedArticleIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<PyAlignedArticle>> {
        match self.iter.lock() {
            Ok(mut lock) => {
                match lock.next().transpose() {
                    Ok(value) => {
                        Ok(value)
                    }
                    Err(err) => {
                        Err(PyRuntimeError::new_err(err.to_string()))
                    }
                }
            }
            Err(err) => {
                Err(PyRuntimeError::new_err(err.to_string()))
            }
        }
    }
}

impl Iterator for PyAlignedArticleIter {
    type Item = Result<PyAlignedArticle, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.lock().unwrap().next()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyParsedAlignedArticleIter {
    iter: Arc<Mutex<TokenizingDeserializeIter<'static>>>
}

impl PyParsedAlignedArticleIter {
    fn new(iterator: TokenizingDeserializeIter) -> Self {
        Self {
            iter: Arc::new(Mutex::new(unsafe{transmute(iterator)}))
        }
    }
}

#[pymethods]
impl PyParsedAlignedArticleIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<PyTokenizedAlignedArticle>> {
        match self.iter.lock() {
            Ok(mut lock) => {
                match lock.next().transpose() {
                    Ok(value) => {
                        Ok(value)
                    }
                    Err(err) => {
                        Err(PyRuntimeError::new_err(err.to_string()))
                    }
                }
            }
            Err(err) => {
                Err(PyRuntimeError::new_err(err.to_string()))
            }
        }
    }
}

impl Iterator for PyParsedAlignedArticleIter {
    type Item = Result<PyTokenizedAlignedArticle, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.lock().unwrap().next()
    }
}


fn read_aligned_articles_impl<'a>(path: impl AsRef<Path>, with_pickle: bool) -> io::Result<DeserializeIter<'a>> {
    let iter = Deserializer::from_reader(BufReader::new(File::open(path)?));
    Ok(if with_pickle {
        DeserializeIter::Pickle(iter.into_iter().into_json_pickle_iter::<PyAlignedArticle>())
    } else {
        DeserializeIter::Unpickle(iter.into_iter())
    })
}

#[pyfunction]
pub fn read_aligned_articles(path: &str, with_pickle: bool) -> PyResult<PyAlignedArticleIter> {
    Ok(
        PyAlignedArticleIter::new(
            read_aligned_articles_impl(path, with_pickle).map_err(|value| PyValueError::new_err(value.to_string()))?
        )
    )
}

type TokenizingDeserializeIter<'a> = Map<WithValue<DeserializeIter<'a>, Arc<HashMap<LanguageHint, Tokenizer<'a>>>>, fn((Arc<HashMap<LanguageHint, Tokenizer>>, Result<PyAlignedArticle, Error>)) -> Result<PyTokenizedAlignedArticle, Error>>;

#[pyfunction]
pub fn read_and_parse_aligned_articles(path: &str, with_pickle: bool, processor: PyAlignedArticleProcessor) -> PyResult<PyParsedAlignedArticleIter>{
    let reader = read_aligned_articles_impl(path, with_pickle).map_err(|value| PyValueError::new_err(value.to_string()))?;
    let tokenizers = unsafe{processor.create_tokenizer_map()};

    let iter: TokenizingDeserializeIter = reader.with_value(Arc::new(tokenizers)).map(|(tokenizers, value)| {
        match value {
            Ok(value) => {
                let (id, articles) = value.0.into_inner();
                let articles = articles.into_par_iter().map(|(lang, art)| {
                    if let Some(tokenizer) = tokenizers.get(&lang) {
                        let tokens = tokenizer
                            .phrase_stemmed(art.0.content())
                            .map(|(original, value)| (original.to_string(), value.into()))
                            .collect_vec();
                        (lang, PyTokenizedArticleUnion::Tokenized(
                            art,
                            tokens
                        ))
                    } else {
                        (lang, PyTokenizedArticleUnion::NotTokenized(art))
                    }
                }).collect::<HashMap<_, _>>();
                Ok(
                    PyTokenizedAlignedArticle(
                        AlignedArticle::new(
                            id,
                            articles
                        )
                    )
                )
            }
            Err(value) => {
                Err(value)
            }
        }
    });

    Ok(PyParsedAlignedArticleIter::new(iter))
}

#[derive(Debug, Error)]
enum WriteIntoError {
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    IO(#[from] io::Error)
}

#[pyfunction]
pub fn read_and_parse_aligned_articles_into(path_in: &str, with_pickle: bool, path_out: &str, processor: PyAlignedArticleProcessor) -> PyResult<usize> {
    let reader = read_aligned_articles_impl(path_in, with_pickle).map_err(|value| PyValueError::new_err(value.to_string()))?;
    let tokenizers = Arc::new(unsafe{processor.create_tokenizer_map()});

    let temp_folder = env::temp_dir();

    let mut files = reader.enumerate().par_bridge().map(|(idx, value)| {
        let result = match value {
            Ok(value) => {
                let (id, articles) = value.0.into_inner();
                let articles = articles.into_par_iter().map(|(lang, art)| {
                    if let Some(tokenizer) = tokenizers.get(&lang) {
                        let tokens = tokenizer
                            .phrase_stemmed(art.0.content())
                            .map(|(original, value)| (original.to_string(), value.into()))
                            .collect_vec();
                        (lang, PyTokenizedArticleUnion::Tokenized(
                            art,
                            tokens
                        ))
                    } else {
                        (lang, PyTokenizedArticleUnion::NotTokenized(art))
                    }
                }).collect::<HashMap<_, _>>();
                Ok(
                    PyTokenizedAlignedArticle(
                        AlignedArticle::new(
                            id,
                            articles
                        )
                    )
                )
            }
            Err(value) => {
                Err(value)
            }
        };
        (idx, result)
    }).map(|(idx, value)| {
        match value {
            Ok(value) => {
                let temp_file = temp_folder.join(format!("tmp_{idx}.json"));
                match File::create_new(&temp_file) {
                    Ok(file) => {
                        match serde_json::to_writer(file, &value) {
                            Ok(_) => {
                                Ok((idx, temp_file))
                            }
                            Err(err) => Err((idx, err.into()))
                        }
                    }
                    Err(err) => Err((idx, err.into()))
                }

            }
            Err(err) => Err((idx, err.into()))
        }
    }).collect::<Vec<Result<_, (usize, WriteIntoError)>>>();

    let mut writer = BufWriter::new(File::options().append(true).create(true).open(path_out).map_err(|value| PyIOError::new_err(value.to_string()))?);

    let number_of_results = files.len();

    files.sort_by_key(|value| {
        match value {
            Ok((idx, _)) => *idx,
            Err((idx, _)) => *idx
        }
    });

    let mut error = Vec::new();
    for value in files {
        match value {
            Ok((_, value)) => {
                let mut reader = File::open(value).map_err(|value| PyIOError::new_err(value.to_string()))?;
                std::io::copy(&mut reader, &mut writer).map_err(|value| PyIOError::new_err(value.to_string()))?;
            }
            Err(err) => {
                error.push(err);
            }
        }
    }

    if let Some((idx, err)) = error.first() {
        Err(PyRuntimeError::new_err(format!("Failed with {} errors.\nFirst Error at {idx}:\n{}", error.len(), err.to_string())))
    } else {
        Ok(number_of_results)
    }
}


#[derive(Debug, FromPyObject)]
pub enum PyAlignedArticleArgUnion<TArticle> {
    Map(HashMap<LanguageHintValue, TArticle>),
    List(Vec<TArticle>)
}

#[derive(Debug, Clone)]
pub enum PyAlignedArticleResultUnion<TAlignedArticle, TArticle> {
    Normal(TAlignedArticle),
    WithDoublets(TAlignedArticle, Vec<TArticle>)
}

impl<'py, TAlignedArticle, TArticle> IntoPy<PyObject> for PyAlignedArticleResultUnion<TAlignedArticle, TArticle>
    where TAlignedArticle: IntoPy<PyObject>, TArticle: IntoPy<PyObject> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PyAlignedArticleResultUnion::Normal(value) => {
                value.into_py(py)
            }
            PyAlignedArticleResultUnion::WithDoublets(value, dubletes) => {
                (value, dubletes).into_py(py)
            }
        }
    }
}


macro_rules! impl_aligned_article_wrapper {
    ($($name: ident<$typ: ty>),+) => {
        $(
            #[pyclass]
            #[repr(transparent)]
            #[derive(Clone, Debug, Serialize, Deserialize)]
            #[serde(transparent)]
            pub struct $name(AlignedArticle<$typ>);

            #[pymethods]
            impl $name {
                #[new]
                fn new(article_id: u64, articles: HashMap<LanguageHintValue, $typ>) -> Self {
                    Self(
                        AlignedArticle::new(
                            article_id,
                            articles.into_iter().map(|(k, v)| (k.into(), v)).collect()
                        )
                    )
                }

                #[staticmethod]
                fn create(article_id: u64, articles: PyAlignedArticleArgUnion<$typ>) -> PyAlignedArticleResultUnion<$name, $typ> {
                    match articles {
                        PyAlignedArticleArgUnion::Map(value) => {
                            PyAlignedArticleResultUnion::Normal(
                                Self::new(article_id, value)
                            )
                        }
                        PyAlignedArticleArgUnion::List(value) => {
                            match AlignedArticle::from(article_id, value) {
                                Ok(value) => {
                                    PyAlignedArticleResultUnion::Normal(Self(value))
                                }
                                Err((value, doublets)) => {
                                    PyAlignedArticleResultUnion::WithDoublets(Self(value), doublets)
                                }
                            }
                        }
                    }
                }

                fn __str__(&self) -> String {
                    self.to_string()
                }

                fn __repr__(&self) -> String {
                    format!("{:?}", self)
                }

                pub fn __getitem__(&self, item: LanguageHintValue) -> Option<$typ> {
                    let lh: LanguageHint = item.into();
                    self.0.articles().get(&lh).cloned()
                }

                pub fn __contains__(&self, item: LanguageHintValue) -> bool {
                    let lh: LanguageHint = item.into();
                    self.0.articles().contains_key(&lh)
                }

                fn to_json(&self) -> PyResult<String> {
                    Ok(
                        serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
                    )
                }

                #[staticmethod]
                fn from_json(s: &str) -> PyResult<Self> {
                    Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
                }
            }

            impl Deref for $name {
                type Target = AlignedArticle<$typ>;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        )+
    };
}

#[derive(Clone, Debug, FromPyObject, Serialize, Deserialize)]
pub enum PyTokenizedArticleUnion {
    Tokenized(PyArticle, Vec<(String, PyToken)>),
    #[serde(untagged)]
    NotTokenized(PyArticle)
}


impl IntoPy<PyObject> for PyTokenizedArticleUnion {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PyTokenizedArticleUnion::Tokenized(article, values) => {
                (article, values).into_py(py)
            }
            PyTokenizedArticleUnion::NotTokenized(article) => {
                article.into_py(py)
            }
        }
    }
}

impl Display for PyTokenizedArticleUnion {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PyTokenizedArticleUnion::NotTokenized(article) => {
                Display::fmt(article, f)
            }
            PyTokenizedArticleUnion::Tokenized(article, values) => {
                write!(f, "Tokenized({article}, [{}])", values.iter().map(|(origin, token)| format!("(\"{origin}\" => {token})")).join(", "))
            }
        }
    }
}

impl Borrow<Article> for PyTokenizedArticleUnion {
    fn borrow(&self) -> &Article {
        match self {
            PyTokenizedArticleUnion::Tokenized(article, _) => {article.as_ref()}
            PyTokenizedArticleUnion::NotTokenized(article) => {article.as_ref()}
        }
    }
}

impl_aligned_article_wrapper!(
    PyAlignedArticle<PyArticle>,
    PyTokenizedAlignedArticle<PyTokenizedArticleUnion>
);



impl Display for PyAlignedArticle {
    delegate::delegate! {
        to self.0 {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
        }
    }
}

#[pyclass]
#[repr(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PyArticle(Article);

#[pymethods]
impl PyArticle {
    #[new]
    fn new(language_hint: LanguageHintValue, content: String, categories: Option<Vec<usize>>) -> Self {
        Self(Article::new(language_hint.into(), categories, content))
    }

    #[getter]
    #[pyo3(name="lang")]
    fn py_lang(&self) -> LanguageHint {
        self.lang().clone()
    }
    #[getter]
    #[pyo3(name="categories")]
    fn py_categories(&self) -> Option<Vec<usize>> {
        self.categories().clone()
    }
    #[getter]
    #[pyo3(name="content")]
    fn py_content(&self) -> String {
        self.content().to_string()
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
    }
}

impl PyArticle {
    pub fn lang(&self) -> &LanguageHint {
        self.0.lang()
    }
    pub fn categories(&self) -> &Option<Vec<usize>> {
        self.0.categories()
    }
    pub fn content(&self) -> &str {
        self.0.content()
    }
}

impl AsRef<Article> for PyArticle {
    fn as_ref(&self) -> &Article {
        &self.0
    }
}

impl Borrow<Article> for PyArticle  {
    fn borrow(&self) -> &Article {
        &self.0
    }
}

impl Display for PyArticle {
    delegate::delegate! {
        to self.0 {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
        }
    }
}

#[pyclass(get_all)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyToken {
    /// kind of the Token assigned by the classifier
    kind: PyTokenKind,
    lemma: String,
    /// index of the first and the last character of the original lemma
    char_start: usize,
    char_end: usize,
    /// index of the first and the last byte of the original lemma
    byte_start: usize,
    byte_end: usize,
    /// number of bytes used in the original string mapped to the number of bytes used in the normalized string by each char in the original string.
    /// The char_map must be the same length as the number of chars in the original lemma.
    char_map: Option<Vec<(u8, u8)>>,
    /// script of the Token
    script: PyScript,
    /// language of the Token
    language: Option<PyLanguage>
}

#[pymethods]
impl PyToken {
    pub fn byte_len(&self) -> usize {
        self.lemma.len()
    }

    fn __len__(&self) -> usize {
        self.lemma.len()
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
    }
}

impl<'a> From<Token<'a>> for PyToken {
    fn from(value: Token<'a>) -> Self {
        Self {
            kind: value.kind.into(),
            lemma: value.lemma.to_string(),
            char_start: value.char_start,
            char_end: value.char_end,
            byte_start: value.byte_start,
            byte_end: value.byte_end,
            char_map: value.char_map,
            script: value.script.into(),
            language: value.language.map(Into::into)
        }
    }
}

impl<'a> Into<Token<'a>> for PyToken {
    fn into(self) -> Token<'a> {
        Token {
            kind: self.kind.into(),
            lemma: Cow::Owned(self.lemma),
            char_start: self.char_start,
            char_end: self.char_end,
            byte_start: self.byte_start,
            byte_end: self.byte_end,
            char_map: self.char_map,
            script: self.script.into(),
            language: self.language.map(Into::into)
        }
    }
}


impl Display for PyToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"({}, {})", self.lemma, self.kind, self.kind)
    }
}

#[pyclass]
#[derive(Clone, Serialize, Deserialize)]
pub struct PyAlignedArticleProcessor {
    builders: Arc<HashMap<LanguageHint, PyTokenizerBuilder>>
}

#[pymethods]
impl PyAlignedArticleProcessor {
    #[new]
    fn new(processors: HashMap<LanguageHintValue, PyTokenizerBuilder>) -> Self {
        let provessors = processors.into_iter().map(|(k, v)| (k.into(), v)).collect();
        Self {
            builders: Arc::new(provessors)
        }
    }

    fn process(&self, value: PyAlignedArticle) -> PyResult<PyTokenizedAlignedArticle> {
        let tokenizers = unsafe{self.create_tokenizer_map()};
        let (id, articles) = value.0.into_inner();

        let articles = articles.into_iter().map(|(lang, art)| {
            if let Some(tokenizer) = tokenizers.get(&lang) {
                let tokens = tokenizer
                    .phrase_stemmed(art.0.content())
                    .map(|(original, value)| (original.to_string(), value.into()))
                    .collect_vec();
                (lang, PyTokenizedArticleUnion::Tokenized(
                    art,
                    tokens
                ))
            } else {
                (lang, PyTokenizedArticleUnion::NotTokenized(art))
            }
        }).collect();

        Ok(
            PyTokenizedAlignedArticle(
                AlignedArticle::new(
                    id,
                    articles
                )
            )
        )
    }

    fn __contains__(&self, language_hint: LanguageHintValue) -> bool {
        let lh: LanguageHint = language_hint.into();
        self.builders.contains_key(&lh)
    }

    fn process_string(&self, language_hint: LanguageHintValue, value: &str) -> Option<Vec<(String, PyToken)>> {
        let lh: LanguageHint = language_hint.into();
        let token = self.builders.get(&lh)?.build_tokenizer();
        Some(token.phrase_stemmed(value).map(|(original, value)| { (original.to_string(), value.into()) }).collect())
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
    }
}

impl PyAlignedArticleProcessor {
    pub unsafe fn create_tokenizer_map(&self) -> HashMap<LanguageHint, Tokenizer<'static>> {
        self.builders
            .iter()
            .map(|(hint, builder)| (hint.clone(), transmute(builder.build_tokenizer())))
            .collect()
    }
}


#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PyTokenizerBuilder {
    stop_words: Option<PyStopWords>,
    words_dict: Option<SpecialVec>,
    normalizer_option: PyNormalizerOption,
    segmenter_option: PySegmenterOption,
    stemmer: Option<(PyStemmingAlgorithm, bool)>,
    vocabulary: Option<PyVocabulary>
}

#[pymethods]
impl PyTokenizerBuilder {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    fn stemmer<'py>(slf: Bound<'py, Self>, stemmer: PyStemmingAlgorithm, smart: Option<bool>) -> Bound<'py, Self> {
        slf.borrow_mut().stemmer = Some((stemmer, smart.unwrap_or_default()));
        slf
    }

    fn phrase_vocabulary<'py>(slf: Bound<'py, Self>, vocabulary: PyVocabulary) -> Bound<'py, Self> {
        slf.borrow_mut().vocabulary = Some(vocabulary);
        slf
    }

    fn stop_words<'py>(slf: Bound<'py, Self>, stop_words: PyStopWords) -> Bound<'py, Self> {
        slf.borrow_mut().stop_words = Some(stop_words);
        slf
    }

    fn separators<'py>(slf: Bound<'py, Self>, separators: Vec<String>) -> PyResult<Bound<'py, Self>> {
        slf.borrow_mut().normalizer_option.classifier.set_separators(Some(separators))?;
        Ok(slf)
    }

    fn words_dict<'py>(slf: Bound<'py, Self>, words: Vec<String>) -> Bound<'py, Self> {
        slf.borrow_mut().words_dict = Some(SpecialVec::new(words));
        slf
    }

    fn create_char_map<'py>(slf: Bound<'py, Self>, create_char_map: bool) -> Bound<'py, Self> {
        slf.borrow_mut().normalizer_option.create_char_map = create_char_map;
        slf
    }

    fn lossy_normalization<'py>(slf: Bound<'py, Self>, lossy: bool) -> Bound<'py, Self> {
        slf.borrow_mut().normalizer_option.lossy = lossy;
        slf
    }

    fn allow_list<'py>(slf: Bound<'py, Self>, allow_list:  HashMap<PyScript, Vec<PyLanguage>>) -> Bound<'py, Self> {
        slf.borrow_mut().segmenter_option.set_allow_list(Some(allow_list));
        slf
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl PyTokenizerBuilder {
    pub fn as_tokenizer_builder(&self) -> TokenizerBuilder<impl AsRef<[u8]>> {
        let mut builder = TokenizerBuilder::new();
        if let Some(ref stopwords) = self.normalizer_option.classifier.stop_words {
            builder.stop_words(&stopwords.0);
        }


        if let Some(ref separators) = self.normalizer_option.classifier.separators {
            builder.separators(separators.as_slice());
        }

        if let Some(ref words_dict) = self.words_dict {
            builder.words_dict(words_dict.as_slice());
        }

        builder.create_char_map(self.normalizer_option.create_char_map);
        builder.lossy_normalization(self.normalizer_option.lossy);

        if let Some(ref allow_list) = self.segmenter_option.allow_list {
            builder.allow_list(allow_list);
        }

        builder.set_phraser(self.vocabulary.as_ref().map(PyVocabulary::create_trie));

        builder.stemmer(self.stemmer.map(|(k, v)| (k.into(), v)));

        builder
    }

    pub fn build_tokenizer(&self) -> Tokenizer {
        self.as_tokenizer_builder().into_tokenizer()
    }
}


#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "PyStopWordsSerializable")]
#[serde(into = "PyStopWordsSerializable")]
pub struct PyStopWords(Set<Vec<u8>>);

#[pymethods]
impl PyStopWords {
    #[new]
    fn new(words: Vec<String>) -> PyResult<Self> {
        match Set::from_iter(words) {
            Ok(words) => {Ok(Self(words))}
            Err(value) => {Err(PyValueError::new_err(value.to_string()))}
        }
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
    }
}

impl Display for PyStopWords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
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

impl TryFrom<PyStopWordsSerializable> for PyStopWords {
    type Error = fst::Error;

    fn try_from(value: PyStopWordsSerializable) -> Result<Self, Self::Error> {
        Ok(Self(Set::from(Fst::new(value.inner)?)))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PyStopWordsSerializable {
    inner: Vec<u8>
}

impl From<PyStopWords> for PyStopWordsSerializable {
    fn from(value: PyStopWords) -> Self {
        Self { inner: value.0.into_fst().into_inner() }
    }
}

impl Display for PyStopWordsSerializable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PyStopWordsSerializable({:?})", self.inner)
    }
}

#[pyclass(set_all, get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyNormalizerOption {
    create_char_map: bool,
    classifier: PyClassifierOption,
    lossy: bool,
}

impl Default for PyNormalizerOption {
    fn default() -> Self {
        PyNormalizerOption {
            create_char_map: false,
            classifier: Default::default(),
            lossy: true,
        }
    }
}

#[pymethods]
impl PyNormalizerOption {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PyClassifierOption {
    #[pyo3(get, set)]
    stop_words: Option<PyStopWords>,
    separators: Option<SpecialVec>,
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
                Ok(Some(value.inner().deref().clone()))
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

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(from = "PySegmenterOptionSerializer")]
#[serde(into = "PySegmenterOptionSerializer")]
pub struct PySegmenterOption {
    #[pyo3(get, set)]
    aho: Option<PyAhoCorasick>,
    allow_list: Option<HashMap<Script, Vec<Language>>>
}

impl From<PySegmenterOptionSerializer> for PySegmenterOption {
    fn from(value: PySegmenterOptionSerializer) -> Self {
        Self {
            aho: Default::default(),
            allow_list: value.allow_list.map(
                |value| {
                    value.into_iter().map(|(k, v)| (k.into(), v.into_iter().map(|lang| lang.into()).collect())).collect()
                }
            )
        }
    }
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

    fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
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


#[derive(Serialize, Deserialize)]
#[repr(transparent)]
struct PySegmenterOptionSerializer {
    allow_list: Option<HashMap<PyScript, Vec<PyLanguage>>>
}

impl From<PySegmenterOption> for PySegmenterOptionSerializer {
    fn from(value: PySegmenterOption) -> Self {
        Self {
            allow_list: value.allow_list.map(|inner| {
                inner.into_iter()
                    .map(|(k, v)| (k.into(), v.into_iter().map(Into::into).collect()))
                    .collect()
            })
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
    pub fn match_kind<'py>(slf: Bound<'py, Self>, kind: PyMatchKind) -> Bound<'py, Self> {
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
    pub fn start_kind<'py>(slf: Bound<'py, Self>, kind: PyStartKind) -> Bound<'py, Self> {
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
    pub fn ascii_case_insensitive<'py>(slf: Bound<'py, Self>, yes: bool) -> Bound<'py, Self> {
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
    pub fn kind<'py>(slf: Bound<'py, Self>, kind: Option<PyAhoCorasickKind>) -> Bound<'py, Self> {
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
    pub fn prefilter<'py>(slf: Bound<'py, Self>, yes: bool) -> Bound<'py, Self> {
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
    pub fn dense_depth<'py>(slf: Bound<'py, Self>, depth: usize) -> Bound<'py, Self> {
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
    pub fn byte_classes<'py>(slf: Bound<'py, Self>, yes: bool) -> Bound<'py, Self> {
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


#[pyo3::pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[derive(strum::EnumString, strum::IntoStaticStr, strum::Display)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum PyTokenKind {
    Word,
    StopWord,
    SeparatorHard,
    SeparatorSoft,
    Unknown
}

impl Into<TokenKind> for PyTokenKind {
    fn into(self) -> TokenKind {
        match self {
            PyTokenKind::Word => TokenKind::Word,
            PyTokenKind::StopWord => TokenKind::StopWord,
            PyTokenKind::SeparatorHard => TokenKind::Separator(SeparatorKind::Hard),
            PyTokenKind::SeparatorSoft => TokenKind::Separator(SeparatorKind::Soft),
            PyTokenKind::Unknown => TokenKind::Unknown
        }
    }
}

impl From<TokenKind> for PyTokenKind {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Word => PyTokenKind::Word,
            TokenKind::StopWord => PyTokenKind::StopWord,
            TokenKind::Separator(SeparatorKind::Hard) => PyTokenKind::SeparatorHard,
            TokenKind::Separator(SeparatorKind::Soft) => PyTokenKind::SeparatorSoft,
            TokenKind::Unknown => PyTokenKind::Unknown
        }
    }
}


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


map_enum!(
    impl PyStemmingAlgorithm for Algorithm {
        Arabic,
        Danish,
        Dutch,
        English,
        Finnish,
        French,
        German,
        Greek,
        Hungarian,
        Italian,
        Norwegian,
        Portuguese,
        Romanian,
        Russian,
        Spanish,
        Swedish,
        Tamil,
        Turkish
    }
);


pub(crate) fn tokenizer_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAhoCorasick>()?;
    m.add_class::<PyAhoCorasickBuilder>()?;
    m.add_class::<PyAhoCorasickKind>()?;
    m.add_class::<PyAlignedArticle>()?;
    m.add_class::<PyAlignedArticleProcessor>()?;
    m.add_class::<PyAlignedArticleIter>()?;
    m.add_class::<PyArticle>()?;
    m.add_class::<PyClassifierOption>()?;
    m.add_class::<PyLanguage>()?;
    m.add_class::<PyMatchKind>()?;
    m.add_class::<PyNormalizerOption>()?;
    m.add_class::<PyParsedAlignedArticleIter>()?;
    m.add_class::<PyScript>()?;
    m.add_class::<PySegmenterOption>()?;
    m.add_class::<PyStartKind>()?;
    m.add_class::<PyStopWords>()?;
    m.add_class::<PyToken>()?;
    m.add_class::<PyTokenizedAlignedArticle>()?;
    m.add_class::<PyTokenizerBuilder>()?;
    m.add_class::<PyTokenKind>()?;
    m.add_class::<PyStemmingAlgorithm>()?;
    m.add_function(wrap_pyfunction!(read_aligned_articles, m)?)?;
    m.add_function(wrap_pyfunction!(read_and_parse_aligned_articles, m)?)?;
    m.add_function(wrap_pyfunction!(read_and_parse_aligned_articles_into, m)?)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use serde_json::Deserializer;
    use crate::aligned_data::IntoJsonPickleDeserializerIterator;
    use crate::aligned_data::test::MY_TEST_DATA;
    use crate::py::tokenizer::PyAlignedArticle;

    #[test]
    fn can_deserialize() {
        let stream = Deserializer::from_str(MY_TEST_DATA).into_iter().into_json_pickle_iter::<PyAlignedArticle>();
        for value in stream {
            println!("{:}", value.unwrap());
        }
    }
}