use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Lines, Read};
use std::iter::Enumerate;
use std::num::{NonZero, NonZeroUsize, ParseIntError};
use std::path::PathBuf;
use camino::{Utf8Path, Utf8PathBuf};
use either::Either;
use flate2::bufread::MultiGzDecoder;
use itertools::{ExactlyOneError, Itertools};
use rayon::iter::{IterBridge, Map};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;
use trie_rs::map::TrieBuilder;
use ldatranslate_toolkit::create_interned_typesafe_symbol;
use ldatranslate_toolkit::from_str_ex::{ParseErrorEx, ParseEx};
use crate::dictionary::word_infos::PartOfSpeech;
use crate::vocabulary::{Vocabulary, VocabularyMut};

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


type MatchCountType = u64;
type ValueCountType = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactNGramCount {
    match_count_sum: u128,
    value_count_sum: u128,
    raw: HashMap<u16, (MatchCountType, ValueCountType)>
}

impl CompactNGramCount {
    fn new(raw: Vec<(u16, MatchCountType, ValueCountType)>) -> CompactNGramCount {
        let mut match_count_sum = 0;
        let mut value_count_sum = 0;
        for (_, m, v) in raw.iter().copied() {
            match_count_sum += m as u128;
            value_count_sum += v as u128;
        }
        Self {
            match_count_sum,
            value_count_sum,
            raw: raw.into_iter().map(|(a, b, c)| (a, (b, c))).collect()
        }
    }
}

impl Display for CompactNGramCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(match_counts:{}, value_counts:{}, years:{})", self.match_count_sum, self.value_count_sum, self.raw.values().len())
    }
}

fn parse_line(line: &str) -> Result<(String, Option<PartOfSpeech>, CompactNGramCount), GoogleNGramError> {
    let mut contents = line.split('\t');
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
                    (word.to_string(), tag)
                } else {
                    (word_and_tag.to_string(), None)
                }
            }
        } else {
            (word_and_tag.to_string(), None)
        }
    } else {
        return Err(GoogleNGramError::NotSplittable(line.to_string(), "\t"));
    };
    let years_and_values = contents.map(|value| {
        value.split(',').collect_tuple().ok_or(GoogleNGramError::NotSplittable(value.to_string(), ",")).and_then(|(year, match_count, volume_count)| {
            year.parse_ex::<u16>().and_then(|year| {
                match_count.parse_ex::<MatchCountType>().and_then(|match_count| {
                    volume_count.parse_ex::<ValueCountType>().map(|volume_count| {
                        (year, match_count, volume_count)
                    })
                })
            }).map_err(Into::into)
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((word, tag, CompactNGramCount::new(years_and_values)))
}

pub struct NGramIter {
    line_iter: Lines<BufReader<MultiGzDecoder<BufReader<Box<dyn Read>>>>>
}

impl NGramIter {
    pub fn new(reader: impl Read + Sized + 'static) -> Self {
        let read: Box<dyn Read> = Box::new(reader);
        Self {
            line_iter: BufReader::new(MultiGzDecoder::new(BufReader::new(read))).lines()
        }
    }

    pub fn skip_n(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        // advance_by
        /*
        Advances the iterator by n elements.

        This method will eagerly skip n elements by calling next up to n times until None is encountered.
        advance_by(n) will return Ok(()) if the iterator successfully advances by n elements, or a
        Err(NonZero<usize>) with value k if None is encountered, where k is remaining number of
        steps that could not be advanced because the iterator ran out. If self is empty and n is
        non-zero, then this returns Err(n). Otherwise, k is always less than n.

        Calling advance_by(0) can do meaningful work, for example Flatten can advance its outer
        iterator until it finds an inner iterator that is not empty, which then often allows it
        to return a more accurate size_hint() than in its initial state.
         */
        for i in 0..n {
            if self.line_iter.next().is_none() {
                // SAFETY: `i` is always less than `n`.
                return Err(unsafe { NonZero::new_unchecked(n - i) });
            }
        }
        Ok(())
    }
}

impl Iterator for NGramIter {
    type Item = Result<(String, Option<PartOfSpeech>, CompactNGramCount), GoogleNGramError>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.line_iter.next()?;
        match next {
            Ok(value) => {
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

create_interned_typesafe_symbol! {
    FileName
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGramIndex {
    trie: trie_rs::map::Trie<u8, (FileNameSymbol, usize, usize)>,
    path_interner: FileNameStringInterner,
    prefix_len: NonZeroUsize
}

impl NGramIndex {
    pub fn builder(prefix_len: NonZeroUsize) -> NGramIndexBuilder {
        NGramIndexBuilder::new(prefix_len)
    }
}

#[derive(Debug)]
pub struct NGramIndexBuilder {
    registry: Vec<(String, (FileNameSymbol, usize, usize))>,
    interner: FileNameStringInterner,
    prefix_len: NonZeroUsize
}

impl NGramIndexBuilder {
    pub fn new(prefix_len: NonZeroUsize) -> Self {
        Self {
            registry: Vec::new(),
            interner: FileNameStringInterner::new(),
            prefix_len
        }
    }

    pub fn register_file(&mut self, file: impl AsRef<Utf8Path>) -> Result<(usize, usize), GoogleNGramError> {
        let path_id = self.interner.get_or_intern(file.as_ref().file_name().expect("Expected a file name!").to_string());
        let mut reader = read_google_ngram_file(
            file,
            true
        )?;

        let mut ct_all = 0usize;
        let mut ct_added = 0usize;
        let mut last = 0usize;
        for (line_no, value) in reader.enumerate() {
            ct_all += 1;
            let (word, _, _) = value?;
            let prefix =  if let Some((pos, _)) = word.char_indices().skip(self.prefix_len.get()).next() {
                &word[..pos]
            } else {
                word.as_str()
            };
            if let Some(last) = self.registry.last() {
                if prefix == last.0 {
                    continue;
                }
            }
            ct_added += 1;
            self.registry.push((prefix.to_string(), (path_id, last, line_no)));
            last = line_no;
        }

        Ok((ct_all, ct_added))
    }

    pub fn build(mut self) -> NGramIndex {
        self.registry.sort_by(|a, b| a.0.cmp(&b.0));
        let mut new: TrieBuilder<u8, (FileNameSymbol, usize, usize)> = TrieBuilder::new();
        for (prefix, entry) in self.registry {
            new.push(prefix, entry);
        }
        NGramIndex {
            path_interner: self.interner,
            prefix_len: self.prefix_len,
            trie: new.build()
        }
    }
}

fn read_to_file(inp: impl AsRef<Utf8Path>, outp_dir: impl AsRef<Utf8Path>, prefix_len: NonZeroUsize) -> Result<Utf8PathBuf, GoogleNGramError> {
    let inp = inp.as_ref();
    std::fs::create_dir_all(outp_dir.as_ref())?;

    let targ_name = inp.file_name().expect("Input file name missing!");
    let output = outp_dir.as_ref().join(targ_name);
    if output.exists() && output.is_file() {
        log::info!("{targ_name}: Already finished {inp} -> {output}!");
        return Ok(output);
    }

    let temp_copy =  outp_dir.as_ref().join(format!("{targ_name}_temp"));

    struct FreeOnDrop {
        dir: Utf8PathBuf
    }

    impl Drop for FreeOnDrop {
        fn drop(&mut self) {
            std::fs::remove_file(&self.dir).unwrap()
        }
    }

    log::info!("{targ_name}: Copy to temp: {inp} -> {temp_copy}");

    std::fs::copy(
        &inp,
        &temp_copy,
    )?;

    let temp_copy = FreeOnDrop {
        dir: temp_copy
    };

    let mut reader = read_google_ngram_file(
        inp,
        true
    )?;


    let mut collection: Vec<(String, (usize, usize))> = Vec::new();
    let mut last = 0usize;
    let mut skipped_because_not_alphabetic = 0usize;
    log::info!("{targ_name}: Start processing!");
    for (line_no, value) in reader.enumerate() {
        if line_no % 1000 == 0 {
            log::info!("{targ_name}: {}", line_no);
        }
        let (word, _, _) = value?;
        if !word.starts_with(|c: char| c.is_alphabetic()) {
            skipped_because_not_alphabetic += 1;
            continue;
        }
        let prefix =  if let Some((pos, _)) = word.char_indices().skip(prefix_len.get()).next() {
            &word[..pos]
        } else {
            word.as_str()
        };
        if let Some(last) = collection.last() {
            if prefix == last.0 {
                continue;
            }
        }

        collection.push((prefix.to_string(), (last, line_no)));
        last = line_no;
    }
    log::info!("{targ_name}: Finished processing: Skipped:{skipped_because_not_alphabetic}");

    drop(temp_copy);

    log::info!("{targ_name}: Write to {output}");
    let outp =  File::options().write(true).create(true).open(&output)?;
    bincode::serialize_into(
        BufWriter::new(outp),
        &(targ_name, prefix_len, collection)
    )?;

    Ok(output)
}

fn reconstruct_from_files<I: IntoIterator<Item=P>, P: AsRef<Utf8Path>>(paths: I) -> Result<NGramIndex, GoogleNGramError> {
    let mut interner = FileNameStringInterner::new();
    let data = paths.into_iter().map(|path| {
        let path = path.as_ref();
        File::open(path).and_then(|mut file| {
            let mut buf: Vec<u8> = Vec::with_capacity(file.metadata().expect("Needs to be a file!").len() as usize);
            file.read_to_end(&mut buf).expect("Was not able to read!");
            let (file, prefix_len, values): (String, NonZeroUsize, Vec<(String, (usize, usize))>) = bincode::deserialize(&buf).expect("Was not able to deserialize!");
            Ok((prefix_len, values.into_iter().filter(|value| value.0.starts_with(|a:char| a.is_alphabetic())).map(|(mut a, b)| {
                a.shrink_to_fit();
                (a, (interner.get_or_intern(&file), b.0, b.1))
            }).collect_vec()))
        })
    }).collect::<Result<Vec<_>, _>>()?;

    let prefix = data.iter().map(|v| v.0.clone()).unique().exactly_one().map_err(|_| GoogleNGramError::NotUnifiable)?;
    let mut data = data.into_iter().flat_map(|value| value.1).collect_vec();
    data.sort_by(|a, b| a.0.cmp(&b.0));
    let mut trie_builder = TrieBuilder::new();
    for (k, v) in data.into_iter() {
        trie_builder.push(k, v)
    }
    Ok(
        NGramIndex {
            prefix_len: prefix,
            trie: trie_builder.build(),
            path_interner: interner
        }
    )
}

fn read_filed(inp_root: impl AsRef<Utf8Path>, out_root: impl AsRef<Utf8Path>, target: &str, n_gram_size: u8, file_max: usize, prefix_len: usize) -> Result<(), GoogleNGramError> {
    let inp_root = inp_root.as_ref();
    let out_root = out_root.as_ref();
    let files = (0..file_max).into_par_iter().map(|i|{
        let inp_file = inp_root.join(format!("{n_gram_size}-{i:0>5}-of-{file_max:0>5}.gz"));
        let outp_dir = out_root.join(format!("{target}_{n_gram_size}"));
        log::info!("Process: {inp_file} into {outp_dir}");
        read_to_file(
            inp_file,
            outp_dir,
            unsafe{NonZeroUsize::new_unchecked(prefix_len)}
        )
    }).collect::<Result<Vec<_>, _>>()?;
    let idx_file = out_root.join(format!("ngram_index_{target}_{n_gram_size}.idx"));
    if idx_file.exists() {
        log::info!("{idx_file} exists!");
        return Ok(())
    }
    let idx = reconstruct_from_files(files)?;
    bincode::serialize_into(
        BufWriter::new(File::options().write(true).create(true).truncate(true).open(idx_file).unwrap()),
        &idx
    )?;
    Ok(())
}


#[cfg(test)]
mod test {
    use log::LevelFilter;
    use crate::dictionary::loader::google_ngram::{read_filed};

    #[test]
    fn can_read(){
        env_logger::builder().filter_level(LevelFilter::Info).init();
        read_filed(
            r#"Z:\NGrams"#,
            r#"E:\tmp\google_ngams"#,
            "de",
            1,
            8,
            4
        ).unwrap();

        read_filed(
            r#"Z:\NGrams"#,
            r#"E:\tmp\google_ngams"#,
            "en",
            1,
            24,
            4
        ).unwrap();

        read_filed(
            r#"Z:\NGrams"#,
            r#"E:\tmp\google_ngams"#,
            "de",
            2,
            181,
            4
        ).unwrap();

        read_filed(
            r#"Z:\NGrams"#,
            r#"E:\tmp\google_ngams"#,
            "en",
            2,
            589,
            4
        ).unwrap();
    }
}