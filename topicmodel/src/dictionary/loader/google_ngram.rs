use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Lines, Read, Write};
use std::iter::Sum;
use std::num::{ParseIntError};
use std::ops::{Add, AddAssign};
use std::path::{Path};
use arcstr::ArcStr;
use camino::{Utf8Path};
use curl::easy::{Handler, List, WriteError};
use flate2::bufread::MultiGzDecoder;
use itertools::{Itertools};
use num::cast::AsPrimitive;
use num::traits::ConstZero;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use thiserror::Error;
use ldatranslate_tokenizer::Tokenizer;
use ldatranslate_toolkit::from_str_ex::{ParseErrorEx, ParseEx};
use ldatranslate_toolkit::fs::retry_copy;
use crate::dictionary::word_infos::PartOfSpeech;
use crate::vocabulary::{SearchableVocabulary, Vocabulary};


// Google NGrams: https://storage.googleapis.com/books/ngrams/books/datasetsv3.html

#[derive(Debug, Error)]
pub enum GoogleNGramError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("The word {0} is not splittable by {1}!")]
    NotSplittable(String, &'static str),
    #[error(transparent)]
    IntParseError(#[from] ParseErrorEx<ParseIntError>),
    #[error("The order of the ngrams is not in naturlal order")]
    WrongOrder,
    #[error(transparent)]
    SerialisationError(#[from] bincode::Error),
    #[error("More than one prefix len!")]
    NotUnifiable,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactNGramCounts {
    sum: NGramCount<u128>,
    raw: HashMap<u16, NGramCount<u64>>
}



impl CompactNGramCounts {
    fn new(raw: HashMap<u16, NGramCount<u64>>) -> CompactNGramCounts {
        Self {
            sum: raw.values().copied().sum::<NGramCount<u128>>(),
            raw: raw.into_iter().collect()
        }
    }
}


impl Display for CompactNGramCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(sum:{}, years:{})", self.sum, self.raw.values().len())
    }
}


#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct NGramCount<T = u128> {
    pub frequency: T,
    pub volumes: T
}

impl<T> NGramCount<T> {
    pub const fn new(frequency: T, volumes: T) -> Self {
        Self { frequency, volumes }
    }
}

impl<T> Ord for NGramCount<T> where T: Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.frequency.cmp(&other.frequency) {
            Ordering::Equal => self.volumes.cmp(&other.volumes),
            other => other
        }
    }
}

impl<T> PartialOrd for NGramCount<T> where T: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.frequency.partial_cmp(&other.frequency) {
            Some(Ordering::Equal) | None => {
                self.volumes.partial_cmp(&other.volumes)
            }
            other => other
        }
    }
}

impl<T> NGramCount<T> {
    #[allow(clippy::wrong_self_convention)]
    pub fn as_<O>(self) -> NGramCount<O>
        where
            T: AsPrimitive<O>,
            O: Copy + 'static
    {
        NGramCount {
            frequency: self.frequency.as_(),
            volumes: self.volumes.as_(),
        }
    }
}




impl<T> NGramCount<T> where T: ConstZero {
    pub const ZERO: NGramCount<T> = NGramCount::new(T::ZERO, T::ZERO);
}

impl<T> Add for NGramCount<T> where T: Add<Output=T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            frequency: self.frequency + rhs.frequency,
            volumes: self.volumes + rhs.volumes,
        }
    }
}

impl<T> AddAssign for NGramCount<T> where T: AddAssign<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.frequency += rhs.frequency;
        self.volumes += rhs.volumes;
    }
}

impl<T, O> Sum<NGramCount<O>> for NGramCount<T>
where
    T: Add<Output=T> + ConstZero + Copy + 'static,
    O: AsPrimitive<T>
{
    fn sum<I: Iterator<Item=NGramCount<O>>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + x.as_::<T>())
    }
}

impl<T> Display for NGramCount<T> where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(frequency:{}, volume number:{})", self.frequency, self.volumes)
    }
}

fn parse_line(line: &str) -> Result<(String, Option<PartOfSpeech>, CompactNGramCounts), GoogleNGramError> {
    // v3: https://github.com/orgtre/google-books-words/blob/main/src/google-books-words.py

    // split line by tab
    let mut contents = line.split('\t');

    //  first element is always the ngram
    let (word, tag) = if let Some(word_and_tag) = contents.next() {
        if word_and_tag.len() > 1 && word_and_tag.contains('_') {
            let (word, tag) = word_and_tag.rsplit_once("_").ok_or_else(|| GoogleNGramError::NotSplittable(word_and_tag.to_string(), "_"))?;
            if tag.trim().is_empty() {
                (word_and_tag.to_string(), None)
            } else {
                let tag = match tag.parse_ex::<PartOfSpeech>() {
                    Ok(value) => {
                        Ok(value)
                    }
                    Err(_) => {
                        tag.to_lowercase().parse_ex::<PartOfSpeech>()
                    }
                }.ok();
                if tag.is_some() {
                    (word.trim().to_string(), tag)
                } else {
                    (word_and_tag.trim().to_string(), None)
                }
            }
        } else {
            (word_and_tag.trim().to_string(), None)
        }
    } else {
        return Err(GoogleNGramError::NotSplittable(line.to_string(), "\t"));
    };

    // remainder is always a list with elements of form
    // "year,frequency,number_of_volumes"
    // sorted increasingly by year but with gaps
    let years_and_values = contents.map(|value| {
        value.split(',').collect_tuple().ok_or(GoogleNGramError::NotSplittable(value.to_string(), ",")).and_then(|(year, frequency, number_of_volumes)| {
            year.parse_ex_tagged::<u16>("year").and_then(|year| {
                frequency.parse_ex_tagged::<u64>("frequency").and_then(|frequency| {
                    number_of_volumes.parse_ex_tagged::<u64>("number_of_volumes").map(|number_of_volumes| {
                        (year, NGramCount::new(frequency, number_of_volumes))
                    })
                })
            }).map_err(Into::into)
        })
    }).collect::<Result<HashMap<_, _>, _>>()?;

    Ok((word, tag, CompactNGramCounts::new(years_and_values)))
}

pub struct NGramIter {
    line_iter: Lines<BufReader<MultiGzDecoder<BufReader<Box<dyn Read>>>>>,
    line_count: u128
}

impl NGramIter {
    pub fn new(reader: impl Read + Sized + 'static) -> Self {
        let read: Box<dyn Read> = Box::new(reader);
        Self {
            line_iter: BufReader::new(MultiGzDecoder::new(BufReader::new(read))).lines(),
            line_count: 0
        }
    }

    pub fn line_count(&self) -> u128 {
        self.line_count
    }
}

impl Iterator for NGramIter {
    type Item = Result<(String, Option<PartOfSpeech>, CompactNGramCounts), GoogleNGramError>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.line_iter.next()?;
        match next {
            Ok(value) => {
                self.line_count += 1;
                Some(parse_line(&value))
            }
            Err(err) => {
                Some(Err(GoogleNGramError::IoError(err)))
            }
        }
    }
}

impl<R> From<R> for NGramIter where R: Read + 'static {
    fn from(input: R) -> Self {
        Self::new(input)
    }
}

pub fn read_google_ngram_file(file: impl AsRef<Utf8Path>, load_into_memory: bool) -> Result<NGramIter, GoogleNGramError> {
    let mut file = File::options().read(true).open(file.as_ref())?;
    if load_into_memory {
        let mut contents = Vec::with_capacity(file.metadata()?.len() as usize);
        file.read_to_end(&mut contents)?;
        Ok(Cursor::new(contents).into())
    } else {
        Ok(file.into())
    }
}


fn process_data<T>(par_iter: impl ParallelIterator<Item=Result<(u128, HashMap<T, HashMap<String, NGramCount<u128>>>), GoogleNGramError>>) -> Result<(u128, HashMap<T, HashMap<String, NGramCount<u128>>>), GoogleNGramError>
where T: AsRef<str> + Eq + Hash + Clone + Send + Borrow<str>
{
    par_iter.fold(|| Ok((0, HashMap::new())), |acc, value| {
        value.and_then(|(ct, other)| {
            acc.map(|mut acc| {
                acc.0 += ct;
                for (k, v) in other {
                    match acc.1.entry(k) {
                        Entry::Occupied(mut value) => {
                            let inner: &mut HashMap<String, NGramCount<u128>> = value.get_mut();
                            for (k, v) in v {
                                inner.entry(k).and_modify(|e| *e += v).or_insert(v);
                            }
                        }
                        Entry::Vacant(value) => {
                            value.insert(v);
                        }
                    }
                }
                acc
            })
        })
    }).reduce(|| Ok((0, HashMap::new())), |acc, value| {
        value.and_then(|(ct, other)| {
            acc.map(|mut acc| {
                acc.0 += ct;
                for (k, v) in other {
                    match acc.1.entry(k) {
                        Entry::Occupied(mut value) => {
                            let inner: &mut HashMap<String, NGramCount<u128>> = value.get_mut();
                            for (k, v) in v {
                                inner.entry(k).and_modify(|e| *e += v).or_insert(v);
                            }
                        }
                        Entry::Vacant(value) => {
                            value.insert(v);
                        }
                    }
                }
                acc
            })
        })
    })
}



fn process_ngram_iter<T>(mut iter: NGramIter, voc: &Vocabulary<T>, tokenizer: &Tokenizer) -> Result<(u128, HashMap<T, HashMap<String, NGramCount<u128>>>), GoogleNGramError>
where T: AsRef<str> + Eq + Hash + Clone + Borrow<str>
{
    let result = (&mut iter).filter_map(|ngram| {
        ngram.map(|(word, _, ct)| {
            let word_proc = tokenizer.process_and_join_word_lemma(&word);
            if let Some(k) = voc.get_value(&word_proc) {
                Some((k.clone(), word, ct.sum))
            } else {
                None
            }
        }).transpose()
    }).fold_ok(HashMap::<T, HashMap<String, NGramCount<u128>>>::new(), |mut value, (k, word, v)|{
        value.entry(k).or_insert(HashMap::new()).entry(word).and_modify(|t| *t += v).or_insert(v);
        value
    })?;

    Ok((iter.line_count(), result))
}

pub fn load_scan_for_voc(
    out_root: impl AsRef<Utf8Path>,
    target: &str,
    n_gram_size: u8,
) -> Result<Option<(u128, HashMap<ArcStr, HashMap<String, NGramCount<u128>>>)>, GoogleNGramError> {
    let idx_file = out_root.as_ref().join(format!("word_counts_{target}_{n_gram_size}.bin"));
    if idx_file.exists() {
        log::info!("{idx_file} exists!");
        match bincode::deserialize_from::<_, (u128, HashMap<ArcStr, HashMap<String, NGramCount<u128>>>)>(BufReader::new(File::open(idx_file)?)) {
            Ok(value) => {
                Ok(Some(value))
            }
            Err(err) => {
                Err(err.into())
            }
        }
    } else {
        Ok(None)
    }
}

/// Returns the number of unique words
pub fn scan_for_voc<T>(
    inp_root: impl AsRef<Utf8Path>,
    out_root: impl AsRef<Utf8Path>,
    target: &str,
    n_gram_size: u8,
    file_max: usize,
    voc: &Vocabulary<T>,
    tokenizer: &Tokenizer
) -> Result<(), GoogleNGramError>
where T: AsRef<str> + Eq + Hash + Clone + DeserializeOwned + Serialize + Send + Borrow<str>
{

    let inp_root = inp_root.as_ref();
    let out_root = out_root.as_ref();

    log::info!("Start processing for {target}_{n_gram_size}!");

    let idx_file = out_root.join(format!("word_counts_{target}_{n_gram_size}.bin"));
    if idx_file.exists() {
        log::info!("{idx_file} exists!");
        return Ok(())
    }

    fn scan_single_for_voc<T>(
        inp: impl AsRef<Utf8Path>,
        outp_dir: impl AsRef<Utf8Path>,
        voc: &Vocabulary<T>,
        tokenizer: &Tokenizer
    ) -> Result<(u128, HashMap<T, HashMap<String, NGramCount<u128>>>), GoogleNGramError>
    where T: AsRef<str> + Eq + Hash + Clone + Borrow<str>
    {
        let inp = inp.as_ref();
        let outp_dir = outp_dir.as_ref();
        let file_name = inp.file_name().expect("Input file name missing!");
        let mut builder = tempfile::Builder::new();
        builder.prefix(file_name);
        builder.tempfile_in(&outp_dir).map_err(GoogleNGramError::IoError).and_then(|temp_file| {
            retry_copy(
                inp,
                &temp_file,
                10
            ).map_err(Into::into).and_then(|_| {
                process_ngram_iter(NGramIter::new(temp_file), voc, tokenizer)
            })
        })
    }

    let outp_dir = out_root.join(format!("{target}_{n_gram_size}"));
    std::fs::create_dir_all(&outp_dir).map_err(GoogleNGramError::IoError)?;
    let result = process_data((0..file_max).into_par_iter().map(|i|{
        let inp_file = inp_root.join(format!("{n_gram_size}-{i:0>5}-of-{file_max:0>5}.gz"));
        log::info!("Process: {inp_file}");
        scan_single_for_voc(
            inp_file,
            outp_dir.as_path(),
            voc,
            tokenizer
        )
    }))?;

    let idx_file = out_root.join(format!("word_counts_{target}_{n_gram_size}.bin"));
    log::info!("Write: {idx_file}");
    bincode::serialize_into(BufWriter::new(File::options().write(true).truncate(true).create(true).open(idx_file)?), &result)?;
    Ok(())
}


pub fn scan_for_voc_online<T: AsRef<str> + Eq + Hash + Clone + DeserializeOwned + Serialize + Send + Borrow<str>>(
    out_root: impl AsRef<Utf8Path>,
    base_url: &str,
    target: &str,
    n_gram_size: u8,
    file_max: usize,
    voc: &Vocabulary<T>,
    tokenizer: &Tokenizer,
    id_filter: fn(usize) -> bool
) -> Result<(u128, HashMap<T, HashMap<String, NGramCount<u128>>>), GoogleNGramError> {

    let out_root = out_root.as_ref();

    log::info!("Start processing for {target}_{n_gram_size}!");

    let idx_file = out_root.join(format!("word_counts_{target}_{n_gram_size}.bin"));
    if idx_file.exists() {
        log::info!("{idx_file} exists!");
        return bincode::deserialize_from::<_, (u128, HashMap<T, HashMap<String, NGramCount<u128>>>)>(BufReader::new(File::open(idx_file)?)).map_err(Into::into)
    }

    struct Collector<W: Write> {
        writer: BufWriter<W>
    }

    impl<W> Handler for Collector<W> where W: Write {
        fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
            Ok(self.writer.write(data).unwrap())
        }
    }

    fn scan_single_for_voc<T>(
        base_url: &str,
        file_name: &str,
        outp_dir: impl AsRef<Utf8Path>,
        voc: &Vocabulary<T>,
        tokenizer: &Tokenizer
    ) -> Result<(u128, HashMap<T, HashMap<String, NGramCount<u128>>>), GoogleNGramError>
    where T: AsRef<str> + Eq + Hash + Clone + Borrow<str>
    {
        let outp_dir = outp_dir.as_ref();
        let mut builder = tempfile::Builder::new();
        builder.prefix(file_name);
        builder.tempfile_in(&outp_dir).map_err(GoogleNGramError::IoError).and_then(|temp_file| {
            let mut dl = curl::easy::Easy2::new(Collector {
                writer: BufWriter::new(&temp_file)
            });
            let mut x = List::new();
            x.append("User-Agent: NGram-API-Downloader").unwrap();
            dl.http_headers(x).unwrap();
            dl.url(&format!("{base_url}/{file_name}")).unwrap();
            dl.perform().unwrap();
            drop(dl);
            process_ngram_iter(NGramIter::new(temp_file), voc, tokenizer)
        })
    }
    let outp_dir = out_root.join(format!("{target}_{n_gram_size}"));
    std::fs::create_dir_all(&outp_dir).map_err(GoogleNGramError::IoError)?;
    let result = process_data((0..file_max).into_par_iter().filter(|&x| id_filter(x)).map(|i|{
        let file_name = format!("{n_gram_size}-{i:0>5}-of-{file_max:0>5}.gz");
        log::info!("Process: {outp_dir} -> {file_name} after DL");
        scan_single_for_voc(
            base_url,
            &file_name,
            outp_dir.as_path(),
            voc,
            tokenizer
        )
    }))?;

    let idx_file = out_root.join(format!("word_counts_{target}_{n_gram_size}.bin"));
    log::info!("Write: {idx_file}");
    bincode::serialize_into(BufWriter::new(File::options().write(true).truncate(true).create(true).open(idx_file)?), &result)?;
    Ok(result)
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TotalCount {
    pub match_count: u128,
    pub page_count: u128,
    pub volume_count: u128
}

impl TotalCount {
    pub const ZERO: TotalCount = TotalCount {
        page_count: 0, match_count: 0, volume_count: 0
    };

    pub const fn new(match_count: u128, page_count: u128, volume_count: u128) -> Self {
        Self {
            page_count,
            volume_count,
            match_count
        }
    }

    pub const fn overflowing_add(self, other: Self) -> (Self, bool) {
        let (match_count, match_count_overflow) = self.match_count.overflowing_add(other.match_count);
        let (page_count, page_count_overflow) = self.page_count.overflowing_add(other.page_count);
        let (volume_count, volume_count_overflow) = self.volume_count.overflowing_add(other.volume_count);
        (
            Self {
                match_count,
                page_count,
                volume_count
            },
            match_count_overflow || page_count_overflow || volume_count_overflow
        )
    }
}

impl Add for TotalCount {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            match_count: self.match_count + other.match_count,
            page_count: self.page_count + other.page_count,
            volume_count: self.volume_count + other.volume_count,
        }
    }
}

impl Sum for TotalCount {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(TotalCount::ZERO, |a, b| a + b)
    }
}

impl<'a> Sum<&'a TotalCount> for TotalCount {
    fn sum<I: Iterator<Item=&'a TotalCount>>(iter: I) -> Self {
        iter.copied().fold(TotalCount::ZERO, |a, b| a + b)
    }
}

pub fn load_total_counts<P: AsRef<Path>>(file: P) -> Result<HashMap<u16, TotalCount>, GoogleNGramError> {
    let mut s = String::new();
    BufReader::new(File::open(file)?).read_to_string(&mut s)?;
    s.split('\t').map(|value| {

        let (year, match_count, page_count, volume_count) = value.split(' ').collect_tuple().expect("This should never fail");
        year.parse_ex_tagged::<u16>("year").and_then(|year| {
            match_count.parse_ex_tagged::<u128>("match_count").and_then(|match_count| {
                page_count.parse_ex_tagged::<u128>("page_count").and_then(|page_count| {
                    volume_count.parse_ex_tagged::<u128>("volume_count").map(|volume_count| {
                        (year, TotalCount::new(
                            match_count,
                            page_count,
                            volume_count,
                        ))
                    })
                })
            })
        }).map_err(Into::into)
    }).collect::<Result<HashMap<u16, TotalCount>, _>>()
}
