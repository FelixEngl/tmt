//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::borrow::{Borrow, Cow};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::{env, io};
use std::hash::Hash;
use std::io::{BufReader, BufWriter, IoSliceMut, Read, Write};
use std::iter::Map;
use std::mem::transmute;
use std::num::NonZeroUsize;
use std::ops::{Deref};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, AhoCorasickKind, MatchKind, StartKind};
use charabia::{Script, SeparatorKind, Token, TokenKind};
use charabia::Language;
use charabia::normalizer::{ClassifierOption, NormalizerOption};
use charabia::segmenter::SegmenterOption;
use file_format::{FileFormat, Kind};
use fst::raw::Fst;
use thiserror::Error;
use fst::Set;
use itertools::Itertools;
use pyo3::{Bound, FromPyObject, IntoPy, pyclass, pyfunction, pymethods, PyObject, PyRef, PyResult, Python};
use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use rayon::prelude::*;
use rust_stemmers::{Algorithm};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::de::IoRead;
use serde_json::{Deserializer, Error, Serializer, StreamDeserializer, Value};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use zip::read::ZipFile;
use zip::result::ZipResult;
use crate::aligned_data::{AlignedArticle, Article, IntoJsonPickleDeserializerIterator, JsonPickleDeserializerIterator};
use crate::{impl_py_stub, register_python};
use crate::py::enum_mapping::map_enum;
use crate::py::helpers::{LanguageHintValue, SpecialVec};
use crate::py::vocabulary::PyVocabulary;
use crate::tokenizer::{Tokenizer, TokenizerBuilder};
use crate::toolkit::special_python_values::{VecOrSet};
use crate::toolkit::with_ref_of::{SupportsWithRef, WithValue};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::vocabulary::{BasicVocabulary};

#[cfg(feature = "gen_python_api")]
use crate::toolkit::pystub::TypeInfoBuilder;


enum JsonPickleIterWrapper<'a, T> {
    Pickle(JsonPickleDeserializerIterator<StreamDeserializer<'a, IoRead<BufReader<AlignedArticlesImplReader>>, Value>, T>),
    Unpickle(StreamDeserializer<'a, IoRead<BufReader<AlignedArticlesImplReader>>, T>)
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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


type ParsedDeserializeIter<'a> = JsonPickleIterWrapper<'a, PyTokenizedAlignedArticle>;

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone)]
pub struct PyAlignedArticleParsedIter {
    iter: Arc<Mutex<ParsedDeserializeIter<'static>>>
}

impl PyAlignedArticleParsedIter {
    fn new(iterator: ParsedDeserializeIter) -> Self {
        Self {
            iter: Arc::new(Mutex::new(unsafe{transmute(iterator)}))
        }
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyAlignedArticleParsedIter {
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

impl Iterator for PyAlignedArticleParsedIter {
    type Item = Result<PyTokenizedAlignedArticle, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.lock().unwrap().next()
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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
    let iter = Deserializer::from_reader(BufReader::new(AlignedArticlesImplReader::Plain(File::open(path)?)));
    Ok(if with_pickle {
        DeserializeIter::Pickle(iter.into_iter().into_json_pickle_iter::<PyAlignedArticle>())
    } else {
        DeserializeIter::Unpickle(iter.into_iter())
    })
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
#[pyo3(signature = (path, with_pickle=None))]
pub fn read_aligned_articles(path: PathBuf, with_pickle: Option<bool>) -> PyResult<PyAlignedArticleIter> {
    Ok(
        PyAlignedArticleIter::new(
            read_aligned_articles_impl(path, with_pickle.unwrap_or_default()).map_err(|value| PyValueError::new_err(value.to_string()))?
        )
    )
}

#[cfg(test)]
mod test_read {
    use std::path::PathBuf;
    use crate::py::tokenizer::{read_aligned_articles};

    #[test]
    fn test(){
        let mut last = None;
        for value in read_aligned_articles(PathBuf::from("E:\\git\\ldatranslation\\bambergdictionary\\lda_translate\\data\\preprocessed\\wikicomp-2014_deen.bulkjson"), Some(true)).unwrap() {
            match value {
                Ok(value) => {
                    last.replace(value);
                }
                Err(value) => {
                    println!("Failed with {} after {}", value, last.unwrap());
                    break
                }
            }
        }
    }
}


enum AlignedArticlesImplReader {
    Plain(File),
    Compressed {
        archive: ZipArchive<File>,
        reader: ZipFile<'static>,
    }
}

unsafe impl Sync for AlignedArticlesImplReader{}
unsafe impl Send for AlignedArticlesImplReader{}

impl AlignedArticlesImplReader {
    pub fn new_compressed<'a>(mut archive: ZipArchive<File>, file_number: Option<usize>) -> ZipResult<Self> {
        let reader = archive.by_index(file_number.unwrap_or_default())?;
        let reader: ZipFile<'static> = unsafe{transmute(reader)};
        Ok(
            AlignedArticlesImplReader::Compressed {
                archive,
                reader
            }
        )
    }

    #[allow(dead_code)]
    pub fn into_inner(self) -> File {
        match self {
            AlignedArticlesImplReader::Plain(value) => {value}
            AlignedArticlesImplReader::Compressed { archive, reader } => {
                drop(reader);
                archive.into_inner()
            }
        }
    }
}

impl Read for AlignedArticlesImplReader {
    delegate::delegate! {
        to match self {
            AlignedArticlesImplReader::Plain(ref mut value) => value,
            AlignedArticlesImplReader::Compressed{ref mut reader, ..} => reader
        } {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

            fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize>;

            fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize>;

            fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize>;

            fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()>;
        }
    }
}


#[derive(Debug, Error)]
pub enum ReaderError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    ZIP(#[from] zip::result::ZipError)
}

fn read_aligned_parsed_articles_impl<'a>(path: impl AsRef<Path>, with_pickle: Option<bool>) -> Result<ParsedDeserializeIter<'a>, ReaderError> {
    let compressed = match FileFormat::from_file(path.as_ref())? {
        FileFormat::Zip | FileFormat::Gzip => {
            true
        }
        FileFormat::JsonFeed => {
            false
        }
        other => {
            match other.kind() {
                Kind::Compressed | Kind::Archive => true,
                _ => false
            }
        }
    };

    let reader = if compressed {
        let zipped = ZipArchive::new(File::open(path)?)?;
        let idx_default = zipped.index_for_name("data.bulkjson");
        AlignedArticlesImplReader::new_compressed(zipped, idx_default).expect("This should not fail! Why can't we open the file?")
    } else {
        AlignedArticlesImplReader::Plain(File::open(path)?)
    };

    let reader = BufReader::with_capacity(32 * 1024, reader);

    let iter = Deserializer::from_reader(reader);
    Ok(if with_pickle.unwrap_or_default() {
        ParsedDeserializeIter::Pickle(iter.into_iter().into_json_pickle_iter::<PyTokenizedAlignedArticle>())
    } else {
        ParsedDeserializeIter::Unpickle(iter.into_iter())
    })
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
#[pyo3(signature = (path, with_pickle=None))]
pub fn read_aligned_parsed_articles(path: PathBuf, with_pickle: Option<bool>) -> PyResult<PyAlignedArticleParsedIter> {
    Ok(
        PyAlignedArticleParsedIter::new(
            read_aligned_parsed_articles_impl(path, with_pickle).map_err(|value| PyValueError::new_err(value.to_string()))?
        )
    )
}


type TokenizingDeserializeIter<'a> = Map<WithValue<DeserializeIter<'a>, Arc<HashMap<LanguageHint, Tokenizer<'a>>>>, fn((Arc<HashMap<LanguageHint, Tokenizer>>, Result<PyAlignedArticle, Error>)) -> Result<PyTokenizedAlignedArticle, Error>>;


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
#[pyo3(signature = (path, processor, with_pickle=None))]
pub fn read_and_parse_aligned_articles(path: PathBuf, processor: PyAlignedArticleProcessor, with_pickle: Option<bool>) -> PyResult<PyParsedAlignedArticleIter>{
    let reader = read_aligned_articles_impl(path, with_pickle.unwrap_or_default()).map_err(|value| PyValueError::new_err(value.to_string()))?;
    let tokenizers = unsafe{processor.create_tokenizer_map()};

    let iter: TokenizingDeserializeIter = reader.with_value(Arc::new(tokenizers)).map(|(tokenizers, value)| {
        match value {
            Ok(value) => {
                Ok(
                    PyAlignedArticleProcessor::process_article_with(
                        value,
                        &tokenizers
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
    IO(#[from] io::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError)
}



#[derive(Debug, Copy, Clone, Error)]
#[error("The max value {1} is smaller than the min value (0)!")]
pub struct TokenCountFilterError(usize, usize);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(get_all)]
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct TokenCountFilter {
    min: Option<usize>,
    max: Option<usize>,
}

impl TokenCountFilter {
    pub fn new(
        min: Option<usize>,
        max: Option<usize>,
    ) -> Result<Self, TokenCountFilterError> {
        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                return Err(TokenCountFilterError(min, max))
            }
        }
        Ok(
            Self {
                min,
                max,
            }
        )
    }

    // pub fn as_range(&self) -> impl RangeBounds<usize> {
    //     match (self.min, self.max) {
    //         (None, None) => ..,
    //         (Some(min), None) => min..,
    //         (None, Some(max)) => ..max,
    //         (Some(min), Some(max)) => min..max
    //     }
    // }

    pub fn is_in_count_range(&self, token_len: usize) -> bool {
        if let Some(min) = self.min {
            if min > token_len {
                return false;
            }
        }

        if let Some(max) = self.max  {
            if max < token_len {
                return false;
            }
        }

        return true;
    }

    pub fn set_min(&mut self, min: Option<usize>) -> Result<(), TokenCountFilterError>{
        if let Some(min_value) = min {
            if let Some(max) = self.max {
                if min_value > max {
                    return Err(TokenCountFilterError(min_value, max));
                }
            }
            self.min = min;
        } else {
            self.min = None;
        }
        Ok(())
    }

    pub fn set_max(&mut self, max: Option<usize>) -> Result<(), TokenCountFilterError>{
        if let Some(max_value) = max {
            if let Some(min) = self.min {
                if max_value < min {
                    return Err(TokenCountFilterError(min, max_value));
                }
            }
            self.max = max;
        } else {
            self.max = None;
        }
        Ok(())
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl TokenCountFilter {
    #[new]
    #[pyo3(signature = (min=None, max=None))]
    fn py_new(
        min: Option<usize>,
        max: Option<usize>,
    ) -> PyResult<Self> {
        Ok(Self::new(min, max).map_err(|value| PyValueError::new_err(value.to_string()))?)
    }

    #[setter]
    #[pyo3(name="set_min")]
    pub fn py_set_min(&mut self, min: Option<usize>) -> PyResult<()>{
        Ok(self.set_min(min).map_err(|value| PyValueError::new_err(value.to_string()))?)
    }

    #[setter]
    #[pyo3(name="set_max")]
    pub fn py_set_max(&mut self, max: Option<usize>) -> PyResult<()>{
        Ok(self.set_max(max).map_err(|value| PyValueError::new_err(value.to_string()))?)
    }

    pub fn to_json(&self) -> PyResult<String> {
        Ok(
            serde_json::to_string(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        )
    }

    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(serde_json::from_str(s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?)
    }

    fn __contains__(&self, value: usize) -> bool {
        self.is_in_count_range(value)
    }
}

#[cfg(test)]
mod test_filter {
    use crate::py::tokenizer::TokenCountFilter;

    #[test]
    fn sets_correctly() {
        let mut filter = TokenCountFilter::new(4.into(), 9.into()).unwrap();
        assert!(filter.set_min(None).is_ok());
        assert!(filter.set_min(9.into()).is_ok());
        assert!(filter.set_min(5.into()).is_ok());
        assert!(filter.set_min(10.into()).is_err());

        assert!(filter.set_max(None).is_ok());
        assert!(filter.set_max(5.into()).is_ok());
        assert!(filter.set_max(4.into()).is_err());
        assert!(filter.set_min(100.into()).is_ok());

        assert_eq!(
            TokenCountFilter::new(10.into(), 100.into()).unwrap(),
            filter
        )
    }

    #[test]
    fn filters_correctly() {
        let filter = TokenCountFilter::new(4.into(), 9.into()).unwrap();
        assert!(!filter.is_in_count_range(3));
        assert!(filter.is_in_count_range(4));
        assert!(filter.is_in_count_range(5));
        assert!(filter.is_in_count_range(9));
        assert!(!filter.is_in_count_range(10));

        let filter = TokenCountFilter::new(4.into(), None).unwrap();
        assert!(!filter.is_in_count_range(3));
        assert!(filter.is_in_count_range(4));
        assert!(filter.is_in_count_range(5));
        assert!(filter.is_in_count_range(9));
        assert!(filter.is_in_count_range(10));

        let filter = TokenCountFilter::new(None, 9.into()).unwrap();
        assert!(filter.is_in_count_range(3));
        assert!(filter.is_in_count_range(4));
        assert!(filter.is_in_count_range(5));
        assert!(filter.is_in_count_range(9));
        assert!(!filter.is_in_count_range(10));

        let filter = TokenCountFilter::new(None, None).unwrap();
        assert!(filter.is_in_count_range(3));
        assert!(filter.is_in_count_range(4));
        assert!(filter.is_in_count_range(5));
        assert!(filter.is_in_count_range(9));
        assert!(filter.is_in_count_range(10));

        assert!(TokenCountFilter::new(9.into(), 1.into()).is_err())
    }
}

/// The options for storing.
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Default, Debug)]
pub struct StoreOptions {
    #[pyo3(get, set)]
    deflate_temp_files: bool,
    #[pyo3(get, set)]
    delete_temp_files_immediately: bool,
    #[pyo3(get, set)]
    compress_result: bool,
    temp_folder: Option<PathBuf>,
    show_progress_after: Option<NonZeroUsize>,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl StoreOptions {
    #[new]
    #[pyo3(signature = (deflate_temp_files=None, delete_temp_files_immediately=None, compress_result=None, temp_folder=None, show_progress_after=None))]
    pub fn new(
        deflate_temp_files: Option<bool>,
        delete_temp_files_immediately: Option<bool>,
        compress_result: Option<bool>,
        temp_folder: Option<PathBuf>,
        show_progress_after: Option<usize>
    ) -> Self {
        Self {
            deflate_temp_files: deflate_temp_files.unwrap_or_default(),
            delete_temp_files_immediately: delete_temp_files_immediately.unwrap_or_default(),
            compress_result: compress_result.unwrap_or_default(),
            temp_folder,
            show_progress_after: show_progress_after.and_then(|value| NonZeroUsize::new(value))
        }
    }

    #[setter]
    fn show_progress_after(&mut self, show_progress_after: usize) {self.show_progress_after = NonZeroUsize::new(show_progress_after)}

    #[getter]
    fn get_show_progress_after(&self) -> Option<usize> {self.show_progress_after.and_then(|value| Some(value.get()))}

    #[setter]
    fn temp_folder(&mut self, temp_folder: Option<PathBuf>) {
        self.temp_folder = temp_folder
    }

    #[getter]
    fn get_temp_folder(&self) -> Option<String> {
        Some(self.temp_folder.as_ref()?.to_str().unwrap().to_string())
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
#[pyo3(signature = (path_in, path_out, processor, filter=None, store_options=None, with_pickle=None))]
pub fn read_and_parse_aligned_articles_into(
    path_in: PathBuf,
    path_out: PathBuf,
    processor: PyAlignedArticleProcessor,
    filter: Option<TokenCountFilter>,
    store_options: Option<StoreOptions>,
    with_pickle: Option<bool>,
) -> PyResult<usize> {
    let store_options = store_options.unwrap_or_default();
    if let Some(file_name) = path_out.file_name() {
        if path_out.exists() {
            return Err(PyIOError::new_err(format!("The file at {path_out:?} already exists!")));
        }
        if let Some(name) = file_name.to_str() {
            if let Some((name, _)) = name.split_once('.'){
                name.to_string()
            } else {
                name.to_string()
            }
        } else{
            return Err(PyIOError::new_err(format!("The filename {file_name:?} should only contain unicode!")));
        }
    } else{
        return Err(PyIOError::new_err(format!("The path {path_out:?} does not lead to a file!")));
    };


    let reader = read_aligned_articles_impl(path_in, with_pickle.unwrap_or_default()).map_err(|value| PyValueError::new_err(value.to_string()))?;
    let tokenizers = Arc::new(unsafe{processor.create_tokenizer_map()});

    let temp_folder = (&store_options.temp_folder).as_ref().cloned().unwrap_or_else(|| env::temp_dir());
    let now = std::time::SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards!");
    let temp_folder = temp_folder.join(format!("processing_{}", now.as_millis()));
    std::fs::create_dir_all(&temp_folder)?;
    eprintln!("Storing data in temp folder {}", temp_folder.to_string_lossy());

    let mut files = reader.enumerate().par_bridge().filter_map(|(idx, value)| {
        if let Some(after) = store_options.show_progress_after {
            if  idx % after.get() == 0 {
                eprintln!("Processed {idx} entries.");
            }
        }
        let result = match value {
            Ok(value) => {
                let original_length = value.0.len();
                if let Some(filter) = (&filter).as_ref() {
                    let result = PyAlignedArticleProcessor::process_article_with_filter(
                        value,
                        &tokenizers,
                        filter
                    );
                    if result.len() != original_length {
                        None
                    } else {
                        Some(Ok(result))
                    }
                } else {
                    Some(
                        Ok(
                            PyAlignedArticleProcessor::process_article_with(
                                value,
                                &tokenizers
                            )
                        )
                    )
                }
            }
            Err(value) => {
                Some(Err(value))
            }
        };

        if let Some(result) = result {
            Some((idx, result))
        } else {
            None
        }
    }).map(|(idx, value)| {
        match value {
            Ok(value) => {
                fn save_plain(temp_folder: &Path, idx: usize, value: PyTokenizedAlignedArticle) -> Result<(usize, PathBuf), (usize, WriteIntoError)> {
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

                fn save_compressed(temp_folder: &Path, idx: usize, value: PyTokenizedAlignedArticle) -> Result<(usize, PathBuf), (usize, WriteIntoError)> {
                    let filename = format!("tmp_{idx}.zip");
                    let temp_file = temp_folder.join(&filename);
                    match File::create_new(&temp_file) {
                        Ok(file) => {
                            let mut writer = ZipWriter::new(file);
                            let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
                            writer.start_file("data.json", options).map_err(|err|(idx, err.into()))?;
                            let mut writer = Serializer::new(writer);
                            match value.serialize(&mut writer) {
                                Ok(_) => {
                                    writer.into_inner().finish().map_err(|err|(idx, err.into()))?;
                                    Ok((idx, temp_file))
                                }
                                Err(err) => Err((idx, err.into()))
                            }
                        }
                        Err(err) => Err((idx, err.into()))
                    }
                }

                if (&store_options).deflate_temp_files {
                    save_compressed(&temp_folder, idx, value)
                } else {
                    save_plain(&temp_folder, idx, value)
                }

            }
            Err(err) => Err((idx, err.into()))
        }
    }).collect::<Vec<Result<_, (usize, WriteIntoError)>>>();

    let mut writer: Box<dyn Write> = if (&store_options).compress_result {
        let bulk_name = format!("data.bulkjson");
        let file = File::options().append(true).create(true).open(path_out).map_err(|value| PyIOError::new_err(value.to_string()))?;
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Lzma);
        writer.start_file(bulk_name, options).map_err(|value| PyIOError::new_err(value.to_string()))?;
        Box::new(BufWriter::new(writer))
    } else {
        let file = File::options().append(true).create(true).open(path_out).map_err(|value| PyIOError::new_err(value.to_string()))?;
        Box::new(BufWriter::new(file))
    };

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
                let mut reader = File::open(&value).map_err(|value| PyIOError::new_err(value.to_string()))?;
                if (&store_options).deflate_temp_files {
                    let mut reader = zip::ZipArchive::new(reader).map_err(|value| PyIOError::new_err(value.to_string()))?;
                    let mut entry = reader.by_name("data.json").map_err(|value| PyIOError::new_err(value.to_string()))?;
                    std::io::copy(&mut entry, &mut writer).map_err(|value| PyIOError::new_err(value.to_string()))?;
                } else {
                    std::io::copy(&mut reader, &mut writer).map_err(|value| PyIOError::new_err(value.to_string()))?;
                }
                write!(writer, "\n")?;
                if (&store_options).delete_temp_files_immediately {
                    std::fs::remove_file(value)?;
                }
            }
            Err(err) => {
                error.push(err);
            }
        }
    }

    writer.flush()?;

    drop(writer);

    std::fs::remove_dir_all(temp_folder)?;

    if let Some((idx, err)) = error.first() {
        Err(PyRuntimeError::new_err(format!("Failed with {} errors.\nFirst Error at {idx}:\n{}", error.len(), err.to_string())))
    } else {
        Ok(number_of_results)
    }
}


#[derive(Debug, FromPyObject, Clone)]
pub enum PyAlignedArticleArgUnion<TArticle> {
    Map(HashMap<LanguageHintValue, TArticle>),
    List(Vec<TArticle>)
}

#[cfg(feature = "gen_python_api")]
impl<T> pyo3_stub_gen::PyStubType for PyAlignedArticleArgUnion<T> where T: pyo3_stub_gen::PyStubType {
    fn type_output() -> pyo3_stub_gen::TypeInfo {
        TypeInfoBuilder::new()
            .with::<HashMap<LanguageHintValue, T>>()
            .with::<Vec<T>>()
            .build_output()
    }

    fn type_input() -> pyo3_stub_gen::TypeInfo {
        TypeInfoBuilder::new()
            .with::<HashMap<LanguageHintValue, T>>()
            .with::<Vec<T>>()
            .build_input()
    }
}

#[derive(Debug, Clone, FromPyObject)]
pub enum PyAlignedArticleResultUnion<TAlignedArticle, TArticle> {
    Normal(TAlignedArticle),
    WithDoublets(TAlignedArticle, Vec<TArticle>)
}

#[cfg(feature = "gen_python_api")]
impl<TAlignedArticle, TArticle> pyo3_stub_gen::PyStubType for PyAlignedArticleResultUnion<TAlignedArticle, TArticle>
where
    TAlignedArticle: pyo3_stub_gen::PyStubType,
    TArticle: pyo3_stub_gen::PyStubType
{
    fn type_output() -> pyo3_stub_gen::TypeInfo {
        TypeInfoBuilder::new()
            .with::<TAlignedArticle>()
            .with::<(TAlignedArticle, Vec<TArticle>)>()
            .build_output()
    }

    fn type_input() -> pyo3_stub_gen::TypeInfo {
        TypeInfoBuilder::new()
            .with::<TAlignedArticle>()
            .with::<(TAlignedArticle, Vec<TArticle>)>()
            .build_input()
    }
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
            #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
            #[pyclass]
            #[repr(transparent)]
            #[derive(Clone, Debug, Serialize, Deserialize)]
            #[serde(transparent)]
            pub struct $name(AlignedArticle<$typ>);

            #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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

                #[getter]
                fn article_id(&self) -> u64 {
                    self.0.article_id()
                }

                #[getter]
                fn language_hints(&self) -> Vec<LanguageHint> {
                    self.0.get_language_hints().into_iter().cloned().collect()
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

impl_py_stub!(PyTokenizedArticleUnion: PyArticle, Vec<(String, PyToken)>);


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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[repr(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PyArticle(Article);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyArticle {
    #[new]
    #[pyo3(signature = (language_hint, content, categories=None, is_list=None))]
    fn new(language_hint: LanguageHintValue, content: String, categories: Option<Vec<usize>>, is_list: Option<bool>) -> Self {
        Self(Article::new(language_hint.into(), categories, Some(content), is_list.unwrap_or_default()))
    }

    #[getter]
    #[pyo3(name="is_list")]
    fn py_is_list(&self) -> bool {
        self.0.is_list()
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
    fn py_content(&self) -> Option<String> {
        self.content().as_ref().cloned()
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
    #[inline(always)]
    pub fn lang(&self) -> &LanguageHint {
        self.0.lang()
    }
    #[inline(always)]
    pub fn is_list(&self) -> bool {
        self.0.is_list()
    }
    #[inline(always)]
    pub fn categories(&self) -> &Option<Vec<usize>> {
        self.0.categories()
    }
    #[inline(always)]
    pub fn content(&self) -> &Option<String> {
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Serialize, Deserialize)]
pub struct PyAlignedArticleProcessor {
    builders: Arc<HashMap<LanguageHint, PyTokenizerBuilder>>
}

impl PyAlignedArticleProcessor {
    pub fn process_article_with(
        value: PyAlignedArticle,
        tokenizers: &HashMap<LanguageHint, Tokenizer>,
    ) -> PyTokenizedAlignedArticle {
        let (id, articles) = value.0.into_inner();
        let articles = articles.into_par_iter().map(|(lang, art)| {
            if let Some(content) = art.0.content() {
                if let Some(tokenizer) = tokenizers.get(&lang) {
                    let tokens = tokenizer
                        .process(content.as_str())
                        .map(|(original, value)| (original.to_string(), value.into()))
                        .collect_vec();
                    (lang, PyTokenizedArticleUnion::Tokenized(
                        art,
                        tokens
                    ))
                } else {
                    (lang, PyTokenizedArticleUnion::NotTokenized(art))
                }
            } else {
                (lang, PyTokenizedArticleUnion::NotTokenized(art))
            }
        }).collect();

        PyTokenizedAlignedArticle(
            AlignedArticle::new(
                id,
                articles
            )
        )
    }

    pub fn process_article_with_filter(
        value: PyAlignedArticle,
        tokenizers: &HashMap<LanguageHint, Tokenizer>,
        filter: &TokenCountFilter
    ) -> PyTokenizedAlignedArticle {
        let (id, articles) = value.0.into_inner();
        let articles = articles.into_par_iter().filter_map(|(lang, art)| {
            if let Some(content) = art.0.content() {
                if let Some(tokenizer) = tokenizers.get(&lang) {
                    let tokens = tokenizer
                        .process(content.as_str())
                        .map(|(original, value)| (original.to_string(), value.into()))
                        .collect_vec();
                    if filter.is_in_count_range(tokens.len()) {
                        Some((lang, PyTokenizedArticleUnion::Tokenized(
                            art,
                            tokens
                        )))
                    } else {
                        None
                    }
                } else {
                    Some((lang, PyTokenizedArticleUnion::NotTokenized(art)))
                }
            } else {
                Some((lang, PyTokenizedArticleUnion::NotTokenized(art)))
            }

        }).collect();

        PyTokenizedAlignedArticle(
            AlignedArticle::new(
                id,
                articles
            )
        )
    }

    pub fn process_article(&self, value: PyAlignedArticle, tokenizers: Option<&HashMap<LanguageHint, Tokenizer>>) -> PyTokenizedAlignedArticle {
        if let Some(created) = tokenizers {
            Self::process_article_with(value, created)
        } else {
            Self::process_article_with(value, &unsafe{self.create_tokenizer_map()})
        }
    }

    pub fn get_tokenizers_for(&self, hint: &LanguageHint) -> Option<Tokenizer> {
        Some(self.builders.get(hint)?.build_tokenizer())
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyAlignedArticleProcessor {
    #[new]
    fn new(processors: HashMap<LanguageHintValue, PyTokenizerBuilder>) -> Self {
        Self {
            builders: Arc::new(processors.into_iter().map(|(k, v)| (k.into(), v)).collect())
        }
    }

    fn __getitem__(&self, value: LanguageHintValue) -> Option<PyTokenizerBuilder> {
        let lh: LanguageHint = value.into();
        self.builders.get(&lh).cloned()
    }

    fn process(&self, value: PyAlignedArticle) -> PyResult<PyTokenizedAlignedArticle> {
        Ok(self.process_article(value, None))
    }

    fn __contains__(&self, language_hint: LanguageHintValue) -> bool {
        let lh: LanguageHint = language_hint.into();
        self.builders.contains_key(&lh)
    }

    fn process_string(&self, language_hint: LanguageHintValue, value: &str) -> Option<Vec<(String, PyToken)>> {
        let lh: LanguageHint = language_hint.into();
        let token = self.builders.get(&lh)?.build_tokenizer();
        Some(token.process(value).map(|(original, value)| { (original.to_string(), value.into()) }).collect())
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


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PyTokenizerBuilder {
    unicode: bool,
    words_dict: Option<SpecialVec>,
    normalizer_option: PyNormalizerOption,
    segmenter_option: PySegmenterOption,
    stemmer: Option<(PyStemmingAlgorithm, bool)>,
    vocabulary: Option<PyVocabulary>
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyTokenizerBuilder {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[pyo3(signature = (stemmer, smart=None), text_signature = "stemmer: PyStemmingAlgorithm, smart: None | bool = None")]
    fn stemmer<'py>(slf: Bound<'py, Self>, stemmer: PyStemmingAlgorithm, smart: Option<bool>) -> Bound<'py, Self> {
        slf.borrow_mut().stemmer = Some((stemmer, smart.unwrap_or_default()));
        slf
    }

    #[pyo3(signature = (vocabulary), text_signature = "vocabulary: PyVocabulary")]
    fn phrase_vocabulary<'py>(slf: Bound<'py, Self>, vocabulary: PyVocabulary) -> Bound<'py, Self> {
        slf.borrow_mut().vocabulary = Some(vocabulary);
        slf
    }

    #[pyo3(signature = (stop_words), text_signature = "stop_words: PyStopWordsArg")]
    fn stop_words<'py>(slf: Bound<'py, Self>, stop_words: PyStopWordsArg) -> PyResult<Bound<'py, Self>> {
        slf.borrow_mut().normalizer_option.classifier.stop_words = Some(stop_words.to_stop_words()?);
        Ok(slf)
    }

    #[pyo3(signature = (separators), text_signature = "separators: list[str] | set[str]")]
    fn separators<'py>(slf: Bound<'py, Self>, separators: VecOrSet<String>) -> PyResult<Bound<'py, Self>> {
        slf.borrow_mut().normalizer_option.classifier.set_separators(Some(separators.to_vec()))?;
        Ok(slf)
    }

    #[pyo3(signature = (words), text_signature = "words: list[str] | set[str]")]
    fn words_dict<'py>(slf: Bound<'py, Self>, words: VecOrSet<String>) -> Bound<'py, Self> {
        slf.borrow_mut().words_dict = Some(SpecialVec::new(words.to_vec()));
        slf
    }

    #[pyo3(signature = (create_char_map), text_signature = "create_char_map: bool")]
    fn create_char_map<'py>(slf: Bound<'py, Self>, create_char_map: bool) -> Bound<'py, Self> {
        slf.borrow_mut().normalizer_option.create_char_map = create_char_map;
        slf
    }

    #[pyo3(signature = (lossy), text_signature = "lossy: bool")]
    fn lossy_normalization<'py>(slf: Bound<'py, Self>, lossy: bool) -> Bound<'py, Self> {
        slf.borrow_mut().normalizer_option.lossy = lossy;
        slf
    }

    #[pyo3(signature = (unicode), text_signature = "unicode: bool")]
    fn unicode_segmentation<'py>(slf: Bound<'py, Self>, unicode: bool) -> Bound<'py, Self> {
        slf.borrow_mut().unicode = unicode;
        slf
    }

    #[pyo3(signature = (allow_list), text_signature = "allow_list: dict[PyScript, list[PyLanguage]]")]
    fn allow_list<'py>(slf: Bound<'py, Self>, allow_list: HashMap<PyScript, Vec<PyLanguage>>) -> Bound<'py, Self> {
        slf.borrow_mut().segmenter_option.set_allow_list(Some(allow_list));
        slf
    }

    fn create_stopword_filter(&self) -> Option<PyStopWords> {
        self.normalizer_option.classifier.stop_words.clone()
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
        builder.unicode(self.normalizer_option.lossy);

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


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "PyStopWordsSerializable")]
#[serde(into = "PyStopWordsSerializable")]
pub struct PyStopWords(Set<Vec<u8>>);

impl PyStopWords {
    pub fn new(field0: Set<Vec<u8>>) -> Self {
        Self(field0)
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyStopWords {
    #[new]
    fn py_new(words: VecOrSet<String>) -> PyResult<Self> {
        let mut words = words.to_vec();
        words.sort();
        match Set::from_iter(words) {
            Ok(words) => {Ok(Self(words))}
            Err(value) => {Err(PyValueError::new_err(value.to_string()))}
        }
    }

    fn __contains__(&self, value: &str) -> bool {
        self.0.contains(value)
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

#[derive(Clone, Debug, FromPyObject)]
pub enum PyStopWordsArg {
    List(Vec<String>),
    Set(HashSet<String>),
    StopWords(PyStopWords)
}

impl_py_stub!(PyStopWordsArg: Vec<String>, HashSet<String>, PyStopWords);

impl PyStopWordsArg {
    pub fn to_stop_words(self) -> PyResult<PyStopWords> {
        match self {
            PyStopWordsArg::List(value) => {
                PyStopWords::py_new(VecOrSet::from(value))
            }
            PyStopWordsArg::Set(value) => {
                PyStopWords::py_new(VecOrSet::from(value))
            }
            PyStopWordsArg::StopWords(value) => {
                Ok(value)
            }
        }
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PyClassifierOption {
    #[pyo3(get, set)]
    stop_words: Option<PyStopWords>,
    separators: Option<SpecialVec>,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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


/// An automaton for searching multiple strings in linear time.
/// This is only used to hold the results.
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyAhoCorasick(AhoCorasick);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyAhoCorasick {
    #[staticmethod]
    pub fn builder() -> PyAhoCorasickBuilder {
        PyAhoCorasickBuilder::new()
    }
}

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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone, Default)]
#[repr(transparent)]
pub struct PyAhoCorasickBuilder(AhoCorasickBuilder);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
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
    /// ```plaintext
    /// use ldatranslate::py::tokenizer::{PyAhoCorasick, PyMatchKind};
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
    /// ```plaintext
    /// use ldatranslate::py::tokenizer::{PyAhoCorasick, PyMatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = PyAhoCorasick::builder()
    ///     .match_kind(PyMatchKind::LeftmostFirst)
    ///     .build(patterns)
    ///     .unwrap();
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("abc", &haystack[mat.start()..mat.end()]);
    /// ```
    ///
    /// Leftmost-longest semantics:
    ///
    /// ```plaintext
    /// use ldatranslate::py::tokenizer::{PyAhoCorasick, PyMatchKind};
    ///
    /// let patterns = &["b", "abc", "abcd"];
    /// let haystack = "abcd";
    ///
    /// let ac = PyAhoCorasick::builder()
    ///     .match_kind(PyMatchKind::LeftmostLongest)
    ///     .build(patterns)
    ///     .unwrap();
    /// let mat = ac.find(haystack).expect("should have a match");
    /// assert_eq!("abcd", &haystack[mat.start()..mat.end()]);
    /// ```
    #[pyo3(signature = (kind), text_signature = "kind: PyMatchKind")]
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
    #[pyo3(signature = (kind), text_signature = "kind: PyStartKind")]
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
    #[pyo3(signature = (yes), text_signature = "yes: bool")]
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
    #[pyo3(signature = (kind=None), text_signature = "kind: None | PyAhoCorasickKind")]
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
    #[pyo3(signature = (yes), text_signature = "yes: bool")]
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
    #[pyo3(signature = (depth), text_signature = "depth: int")]
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
    #[pyo3(signature = (yes), text_signature = "yes: bool")]
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


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyo3::pyclass(eq, eq_int, hash, frozen)]
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

register_python! {
    struct PyAhoCorasick;
    struct PyAhoCorasickBuilder;
    struct PyAhoCorasickKind;
    struct PyAlignedArticle;
    struct PyAlignedArticleProcessor;
    struct PyAlignedArticleIter;
    struct PyArticle;
    struct PyClassifierOption;
    struct PyLanguage;
    struct PyMatchKind;
    struct PyNormalizerOption;
    struct PyParsedAlignedArticleIter;
    struct PyScript;
    struct PySegmenterOption;
    struct PyStartKind;
    struct PyStopWords;
    struct PyToken;
    struct PyTokenizedAlignedArticle;
    struct PyTokenizerBuilder;
    struct PyTokenKind;
    struct PyStemmingAlgorithm;
    struct TokenCountFilter;
    struct StoreOptions;
    fn read_aligned_articles;
    fn read_aligned_parsed_articles;
    fn read_and_parse_aligned_articles;
    fn read_and_parse_aligned_articles_into;
}


// pub(crate) fn tokenizer_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     // m.add_class::<PyAhoCorasick>()?;
//     // m.add_class::<PyAhoCorasickBuilder>()?;
//     // m.add_class::<PyAhoCorasickKind>()?;
//     // m.add_class::<PyAlignedArticle>()?;
//     // m.add_class::<PyAlignedArticleProcessor>()?;
//     // m.add_class::<PyAlignedArticleIter>()?;
//     // m.add_class::<PyArticle>()?;
//     // m.add_class::<PyClassifierOption>()?;
//     // m.add_class::<PyLanguage>()?;
//     // m.add_class::<PyMatchKind>()?;
//     // m.add_class::<PyNormalizerOption>()?;
//     // m.add_class::<PyParsedAlignedArticleIter>()?;
//     // m.add_class::<PyScript>()?;
//     // m.add_class::<PySegmenterOption>()?;
//     // m.add_class::<PyStartKind>()?;
//     // m.add_class::<PyStopWords>()?;
//     // m.add_class::<PyToken>()?;
//     // m.add_class::<PyTokenizedAlignedArticle>()?;
//     // m.add_class::<PyTokenizerBuilder>()?;
//     // m.add_class::<PyTokenKind>()?;
//     // m.add_class::<PyStemmingAlgorithm>()?;
//     // m.add_class::<TokenCountFilter>()?;
//     // m.add_class::<StoreOptions>()?;
//     m.add_function(wrap_pyfunction!(read_aligned_articles, m)?)?;
//     m.add_function(wrap_pyfunction!(read_aligned_parsed_articles, m)?)?;
//     m.add_function(wrap_pyfunction!(read_and_parse_aligned_articles, m)?)?;
//     m.add_function(wrap_pyfunction!(read_and_parse_aligned_articles_into, m)?)?;
//     Ok(())
// }

#[cfg(test)]
mod test {
    use rust_stemmers::Algorithm;
    use serde_json::Deserializer;
    use crate::aligned_data::IntoJsonPickleDeserializerIterator;
    use crate::aligned_data::test::MY_TEST_DATA;
    use crate::py::tokenizer::{PyAlignedArticle, PyStopWordsArg};
    use crate::tokenizer::TokenizerBuilder;
    use crate::topicmodel::language_hint::LanguageHint;

    #[test]
    fn can_deserialize() {
        let stream = Deserializer::from_str(MY_TEST_DATA).into_iter().into_json_pickle_iter::<PyAlignedArticle>();
        for value in stream {
            println!("{:}", value.unwrap());
        }
    }
    
    #[test]
    fn can_tokenize(){
        let mut builder = TokenizerBuilder::default();
        let args =  PyStopWordsArg::List(vec!["to".to_string(), "do".to_string(), "a".to_string()]).to_stop_words().unwrap();
        builder.stop_words(&args.0);
        builder.lossy_normalization(false);
        builder.stemmer(Some((Algorithm::English, false)));
        builder.unicode(true);
        builder.lossy_normalization(true);
        let tokenizer = builder.into_tokenizer();
        let stream = Deserializer::from_str(MY_TEST_DATA).into_iter().into_json_pickle_iter::<PyAlignedArticle>();
        for value in stream {
            let x = value.unwrap();
            let artivle = x.0.articles().get(&LanguageHint::new("en")).unwrap();
            for (origin, value) in tokenizer.process(artivle.0.content().as_ref().unwrap()) {
                println!("{origin} -- {value:?}")
            }
            println!("########")
        }
    }
}