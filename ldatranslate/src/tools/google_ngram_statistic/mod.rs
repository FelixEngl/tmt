mod statistics;

use std::borrow::Borrow;
use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter};
use std::rc::Rc;
use arcstr::ArcStr;
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use pyo3::{pyclass, pymethods, PyErr};
use pyo3::exceptions::PyValueError;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ldatranslate_tokenizer::Tokenizer;
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::dictionary::DictionaryWithVocabulary;
use ldatranslate_topicmodel::dictionary::google_ngram::{load_total_counts, scan_for_voc, GoogleNGramError, NGramCount, TotalCount};
use ldatranslate_topicmodel::language_hint::LanguageHint;
use ldatranslate_topicmodel::vocabulary::BasicVocabulary;
use crate::py::dictionary::PyDictionary;
use crate::py::tokenizer::PyAlignedArticleProcessor;
pub use statistics::*;



#[derive(Debug, Error)]
pub enum GenerateNGramStatisticsError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("No tokenizer found for {0}!")]
    NoTokenizer(LanguageHint),
    #[error("No Vocabulary found for {0} in dict!")]
    NoVocabulary(LanguageHint),
    #[error("No unique count found for {0:?}!")]
    NoUnique(NGramDefinition),
    #[error(transparent)]
    NGramError(#[from] GoogleNGramError),
    #[error(transparent)]
    BinCode(#[from] bincode::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<GenerateNGramStatisticsError> for PyErr {
    fn from(value: GenerateNGramStatisticsError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}


register_python!(struct NGramDefinition;);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGramDefinition {
    language: LanguageHint,
    ngram_size: u8,
    file_max: usize,
}

impl NGramDefinition {
    pub fn identifier(&self) -> String {
        format!("{}_{}", self.language.as_str(), self.ngram_size)
    }

    pub fn new(language: impl Into<LanguageHint>, ngram_size: u8, file_max: usize) -> Self {
        Self { language: language.into(), ngram_size, file_max }
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl NGramDefinition {
    #[new]
    pub fn py_new(language: LanguageHint, ngram_size: u8, file_max: usize) -> Self {
        Self::new(language, ngram_size, file_max)
    }
}


pub fn generate_google_ngram_statistic(
    inp_root: impl AsRef<Utf8Path>,
    out_root: impl AsRef<Utf8Path>,
    proc: &PyAlignedArticleProcessor,
    v1: &PyDictionary,
    ngrams: &[NGramDefinition],
) -> Result<Vec<(Utf8PathBuf, NGramStatistics<ArcStr>)>, GenerateNGramStatisticsError> {
    generate_base(
        inp_root.as_ref(),
        out_root.as_ref(),
        proc,
        v1,
        ngrams
    )?;

    let base_path = out_root.as_ref().join("gen");
    normalize(out_root.as_ref(), base_path.as_path(), proc, v1, ngrams)?;

    generate_final(
        inp_root.as_ref(),
        base_path.as_path(),
        ngrams,
    )
}

fn save_bin_and_json<V: Serialize>(
    save_path: impl AsRef<Utf8Path>,
    file_name: impl Display,
    value: &V
) -> Result<(), GenerateNGramStatisticsError> {
    let save_path = save_path.as_ref();
    File::options().write(true).create(true).truncate(true).open(
        save_path.join(format!("{file_name}.bin"))
    ).map_err(GenerateNGramStatisticsError::IO).and_then(|file| {
        bincode::serialize_into(
            BufWriter::new(file),
            value
        ).map_err(Into::into)
    }).and_then(|_| {
        File::options().write(true).create(true).truncate(true).open(
            save_path.join(format!("{file_name}.json"))
        ).map_err(GenerateNGramStatisticsError::IO).and_then(|file| {
            serde_json::to_writer_pretty(
                BufWriter::new(file),
                value
            ).map_err(Into::into)
        })
    })
}



fn normalize_ngram_counts<T1, T2, T3>(
    word_count: HashMap<T1, HashMap<T2, NGramCount>>,
    tokenizer: Option<&Tokenizer>
) -> HashMap<T1, HashMap<T3, NGramCount>>
where
    T1: AsRef<str> + Eq + Hash + Send,
    T2: AsRef<str> + Send,
    T3: for<'a> From<&'a str> + Eq + Hash + Clone + Send,
{
    word_count.into_par_iter().map(|(k1, v1)| {
        (
            k1,
            v1.into_iter().into_group_map_by(|(k, _)| {
                if let Some(tokenizer) = tokenizer {
                    tokenizer.process_and_join_word_lemma(k.as_ref())
                } else {
                    k.as_ref().to_lowercase()
                }
            }).into_iter().map(|(k, v)| {
                (k.as_str().into(), v.into_iter().map(|(_, v)| v).sum())
            }).collect()
        )
    }).collect()
}

fn create_word_freqs<T, K1, K2>(
    voc: &impl BasicVocabulary<T>,
    n_gram_size: u8,
    word_count: &HashMap<K1, HashMap<K2, NGramCount>>,
) -> HashMap<T, NGramCount> where
    T: AsRef<str> + Clone + Eq + Hash + Send + Sync,
    K1: Eq + Hash + Borrow<str> + Send + Sync,
    K2: AsRef<str> + Send + Sync,
{
    voc.par_iter().filter(|&value| {
        value.as_ref().chars().filter(|v| ' '.eq(v)).count() + 1 == n_gram_size as usize
    }).map(|value| {
        word_count.get(value.as_ref()).and_then(|found| {
            found.iter().filter_map(|(k, v)| {
                let correct_count = k.as_ref().chars().filter(|v| ' '.eq(v)).count() + 1 == n_gram_size as usize;
                if correct_count {
                    Some(v)
                } else {
                    None
                }
            }).max().map(|a| (value.clone(), a.clone()))
        }).unwrap_or_else(|| (value.clone(), NGramCount::ZERO))
    }).collect::<HashMap<_, _>>()
}

fn generate_base(
    inp_root: impl AsRef<Utf8Path>,
    out_root: impl AsRef<Utf8Path>,
    proc: &PyAlignedArticleProcessor,
    v1: &PyDictionary,
    ngrams: &[NGramDefinition],
) -> Result<(), GenerateNGramStatisticsError> {
    log::info!("Generate base");
    let inner1 = v1.get();

    let mut cache = HashMap::new();

    for ngram in ngrams {
        log::info!("Start {} {}!", ngram.language, ngram.ngram_size);

        let tokenizer: Rc<Tokenizer> = match cache.entry(ngram.language.clone()) {
            Entry::Occupied(entry) => {
                Rc::clone(entry.get())
            }
            Entry::Vacant(entry) => {
                let tok = proc.get_tokenizers_for(entry.key()).ok_or_else(|| GenerateNGramStatisticsError::NoTokenizer(entry.key().clone()))?;
                Rc::clone(entry.insert(Rc::new(tok)))
            }
        };

        scan_for_voc(
            inp_root.as_ref(),
            out_root.as_ref(),
            ngram.language.as_str(),
            ngram.ngram_size,
            ngram.file_max,
            inner1.voc_by_hint(&ngram.language).ok_or_else(|| GenerateNGramStatisticsError::NoVocabulary(ngram.language.clone()))?,
            tokenizer.as_ref(),
            &format!("word_counts_{}.bin", ngram.identifier()),
        )?;
    }

    Ok(())
}

fn normalize(
    root: impl AsRef<Utf8Path>,
    base_path: impl AsRef<Utf8Path>,
    proc: &PyAlignedArticleProcessor,
    v1: &PyDictionary,
    ngrams: &[NGramDefinition],
) -> Result<(), GenerateNGramStatisticsError> {
    log::info!("Normalize on base path: {}", base_path.as_ref());
    for provider in [
        Some(proc),
        None
    ] {

        log::info!("Execute{}", provider.is_some().then(|| " with provider").unwrap_or(" without provider"));

        let save_path = base_path.as_ref().join(provider.is_some().to_string());
        if save_path.join("finished").exists() {
            log::info!("Already finished!");
            continue;
        }

        std::fs::create_dir_all(&save_path).unwrap();

        let mut overall: HashMap<LanguageHint, HashMap<ArcStr, NGramCount>> = HashMap::new();

        let mut unique_word_counts = HashMap::new();

        for ngram in ngrams {
            let name = format!("word_counts_{}", ngram.identifier());
            log::info!("Normalize: {}{}", name, provider.is_some().then(|| " with provider").unwrap_or(""));
            let (unique_ct, content) = bincode::deserialize_from::<_, (u128, HashMap<ArcStr, HashMap<ArcStr, NGramCount>>)>(BufReader::new(File::open(root.as_ref().join(format!(r#"{name}.bin"#))).unwrap())).unwrap();
            log::info!("Uniques: {}", unique_ct);
            let uniques_old = content.values().flat_map(|v| {
                v.keys()
            }).unique().count();
            log::info!("Uniques old: {}", uniques_old);

            let normalized: HashMap<ArcStr, HashMap<ArcStr, NGramCount>> = normalize_ngram_counts::<ArcStr, ArcStr, ArcStr>(
                content,
                provider.as_ref().and_then(|v| v.get_tokenizers_for(&ngram.language)).as_ref()
            );

            let uniques_new = normalized.values().flat_map(|v| {
                v.keys()
            }).unique().count();
            log::info!("Uniques new: {}", uniques_new);
            let unique_ct = unique_ct - uniques_old as u128 + uniques_new as u128;
            log::info!("Uniques: {}", unique_ct);
            unique_word_counts.insert(ngram.identifier(), unique_ct);

            log::info!("Save norm");
            save_bin_and_json(
                &save_path,
                format!("{name}_norm"),
                &normalized
            )?;

            log::info!("Generate word freqs");
            let targ = overall.entry(ngram.language.clone()).or_default();
            let other = create_word_freqs(
                v1.get().voc_by_hint(&ngram.language).ok_or_else(|| GenerateNGramStatisticsError::NoVocabulary(ngram.language.clone()))?,
                ngram.ngram_size,
                &normalized
            );

            log::info!("Collect word freqs");
            other.into_iter().for_each(|(k, v)| {
                targ.entry(k).and_modify(|count| *count = max(*count, v)).or_insert(v);
            })
        }


        log::info!("Save processed content");
        save_bin_and_json(
            &save_path,
            "unique_word_counts",
            &unique_word_counts
        )?;

        overall.par_iter().map(|(k, v)| {
            log::info!("Save overall for: {}", k);
            save_bin_and_json(
                &save_path,
                format!("{k}_counts_for_voc"),
                v
            )
        }).collect::<Result<(), GenerateNGramStatisticsError>>()?;


        File::options().create(true).write(true).open(save_path.join("finished"))?;
    }

    Ok(())
}



fn generate_final(
    inp_root: impl AsRef<Utf8Path>,
    base_path: impl AsRef<Utf8Path>,
    ngrams: &[NGramDefinition],
) -> Result<Vec<(Utf8PathBuf, NGramStatistics<ArcStr>)>, GenerateNGramStatisticsError> {
    log::info!("Generate final data!");
    let targets = ngrams.into_iter().into_group_map_by(|v| v.language.clone());
    log::info!("Targets: {:?}", targets);

    let mut result = Vec::new();

    for t_value in [true, false] {
        let proc = if t_value {"with_proc"} else {"without_proc"};
        let base_path_with_t_value = base_path.as_ref().join(t_value.to_string());
        let unique_word_counts: HashMap<String, u128> = bincode::deserialize_from(BufReader::new(File::open(base_path_with_t_value.join("unique_word_counts.bin"))?))?;

        let r = targets.iter().map(|(k, v)| {
            log::info!("Finalize for processed for {k} {proc}");

            v.iter().map(|&ngram| {
                match load_total_counts(
                    inp_root.as_ref().join(format!("{}-totalcounts-{}", ngram.language, ngram.ngram_size))
                ) {
                    Ok(to_sum) => {
                        let sum = to_sum.values().sum::<TotalCount>();
                        unique_word_counts.get(&ngram.identifier()).map(|&value| {
                            (ngram.ngram_size, NGramStatisticMeta::new(value, sum))
                        }).ok_or_else(|| GenerateNGramStatisticsError::NoUnique(ngram.clone()))
                    }
                    Err(err) => {
                        Err(err.into())
                    }
                }
            }).collect::<Result<HashMap<_, _>, _>>().and_then(|total| {
                match File::open(base_path_with_t_value.join(format!("{k}_counts_for_voc.bin"))) {
                    Ok(file) => {
                        bincode::deserialize_from(BufReader::new(file)).map_err(Into::into).map(|dat| {
                            (k.clone(), NGramStatisticsLangSpecific::new(
                                k.clone(),
                                dat,
                                total,
                            ))
                        })
                    }
                    Err(err) => {
                        Err(err.into())
                    }
                }
            })
        }).collect::<Result<HashMap<_, _>, _>>().map(NGramStatistics::new).and_then(|word_counts| {
            let path = base_path.as_ref().join(format!("counts_{proc}.bin"));
            save_bin_and_json(
                base_path.as_ref(),
                format!("counts_{proc}"),
                &word_counts
            ).and_then(|_| Ok((path, word_counts)))
        })?;
        result.push(r);
    }
    Ok(result)
}
