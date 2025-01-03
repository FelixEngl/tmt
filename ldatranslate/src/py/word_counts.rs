use crate::py::dictionary::PyDictionary;
use crate::py::tokenizer::PyAlignedArticleProcessor;
use arcstr::ArcStr;
use camino::{Utf8PathBuf};
use ldatranslate_topicmodel::dictionary::{DictionaryWithVocabulary};
use ldatranslate_topicmodel::language_hint::LanguageHint;
use ldatranslate_topicmodel::vocabulary::BasicVocabulary;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use pyo3::{pyclass, pyfunction, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use ldatranslate_toolkit::register_python;
use crate::tools::google_ngram_statistic::{generate_google_ngram_statistic, IdfProviderError, NGramDefinition, NGramStatistics, NGramStatisticsLangSpecific};
use crate::tools::tf_idf::{IdfAlgorithm};



#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyNGramStatistics {
    inner: Arc<NGramStatistics<ArcStr>>
}

impl PyNGramStatistics {
    pub fn new(inner: HashMap<LanguageHint, NGramStatisticsLangSpecific<ArcStr>>) -> Self {
        Self { inner: Arc::new(NGramStatistics::new(inner)) }
    }

    delegate::delegate! {
        to self.inner {
            pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&NGramStatisticsLangSpecific<ArcStr>>
            where
                LanguageHint: Borrow<Q>,
                Q: Hash + Eq;

            pub fn get_idf_voc<Idf, D, V>(&self, idf: &Idf, dict: &D) -> Result<HashMap<LanguageHint, (usize, Vec<f64>)>, IdfProviderError<Idf>>
            where
                D: DictionaryWithVocabulary<ArcStr, V>,
                V: BasicVocabulary<ArcStr>,
                Idf: IdfAlgorithm + Send + Sync,
                Idf::Error: Send;
        }
    }
}

impl AsRef<NGramStatistics<ArcStr>> for PyNGramStatistics {
    fn as_ref(&self) -> &NGramStatistics<ArcStr> {
        self.inner.as_ref()
    }
}

impl From<NGramStatistics<ArcStr>> for PyNGramStatistics {
    fn from(value: NGramStatistics<ArcStr>) -> Self {
        Self {
            inner: Arc::new(value)
        }
    }
}

impl Serialize for PyNGramStatistics {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        self.inner.as_ref().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PyNGramStatistics {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        Ok(
            Self {
                inner: Arc::new(Deserialize::deserialize(deserializer)?)
            }
        )
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyNGramStatistics {
    #[staticmethod]
    pub fn load(path: PathBuf) -> PyResult<PyNGramStatistics> {
        bincode::deserialize_from(BufReader::new(File::open(path)?)).map_err(|err| {
            PyValueError::new_err(err.to_string())
        })
    }
}

register_python!(struct PyNGramStatistics;);


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyfunction)]
#[pyfunction]
pub fn generate_word_counts(
    inp_root: PathBuf,
    out_root: PathBuf,
    proc: PyAlignedArticleProcessor,
    v1: PyDictionary,
    ngrams: Vec<NGramDefinition>,
) -> PyResult<Vec<(PathBuf, PyNGramStatistics)>> {
    Ok(
        generate_google_ngram_statistic(
            Utf8PathBuf::from_path_buf(inp_root).map_err(|p| PyValueError::new_err(format!("Illegal inp path {p:?} (only utf-8)")))?,
            Utf8PathBuf::from_path_buf(out_root).map_err(|p| PyValueError::new_err(format!("Illegal outp path {p:?} (only utf-8)")))?,
            &proc,
            &v1,
            &ngrams,
        )?.into_iter().map(|(p, v)| (p.into_std_path_buf(), v.into())).collect()
    )
}

register_python!(fn generate_word_counts;);



#[cfg(test)]
mod test {
    use crate::py::dictionary::PyDictionary;
    use crate::py::tokenizer::PyAlignedArticleProcessor;
    use crate::py::word_counts::{NGramDefinition};
    use log::LevelFilter;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};
    use std::path::PathBuf;
    use arcstr::ArcStr;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use crate::tools::google_ngram_statistic::{generate_google_ngram_statistic, NGramStatistics};
    use crate::tools::tf_idf::Idf;

    #[test]
    fn make_counts(){
        let _ = env_logger::builder().filter_level(LevelFilter::Info).try_init();

        let proc = PyAlignedArticleProcessor::from_json(
            r#"{"builders":{"de":{"unicode":true,"words_dict":null,"normalizer_option":{"create_char_map":false,"classifier":{"stop_words":{"inner":[3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,16,135,194,0,0,0,0,115,114,110,109,16,68,194,0,111,16,65,1,5,115,108,16,2,0,0,0,0,12,115,114,110,109,101,16,69,199,194,1,100,16,65,0,16,142,0,0,1,115,102,99,16,3,1,12,0,30,51,117,110,109,108,98,16,5,0,16,136,0,116,16,65,1,0,115,110,16,2,1,11,105,101,16,2,0,16,129,200,0,16,139,0,16,130,218,207,1,101,16,65,1,115,16,65,0,16,147,0,16,159,128,1,5,8,21,24,195,122,115,110,109,16,69,111,101,16,65,203,38,16,130,218,207,194,1,115,16,65,5,0,115,110,16,66,0,110,16,65,194,218,207,194,1,151,115,101,16,66,0,27,110,108,16,2,194,1,115,16,65,1,12,26,32,42,115,114,110,109,105,16,5,0,0,0,0,32,115,114,110,109,108,16,69,1,101,16,65,1,115,16,65,0,1,157,114,101,99,16,3,123,165,114,99,16,2,171,16,138,1,114,16,65,1,8,14,42,100,117,111,105,101,97,16,5,108,16,151,0,16,143,197,1,5,229,109,105,101,16,67,203,0,16,134,1,97,16,65,204,245,0,1,1,214,0,114,101,99,32,3,1,12,0,0,20,117,116,115,114,105,16,5,24,1,32,188,128,195,192,158,16,134,194,1,162,119,103,16,2,194,152,101,16,65,155,116,16,65,1,5,116,98,16,2,55,1,116,32,65,1,0,61,1,110,101,32,2,1,14,105,97,16,2,242,101,16,65,210,1,0,114,110,109,16,3,0,16,144,194,0,1,115,100,16,66,12,1,1,0,0,0,11,0,54,1,115,110,109,104,99,32,5,29,1,32,182,1,0,106,1,106,1,116,110,100,32,3,40,16,146,1,4,43,109,101,100,16,3,51,1,32,139,18,1,32,136,56,1,59,1,116,101,32,2,203,203,182,192,1,13,17,195,101,97,16,3,39,1,32,142,160,1,32,142,1,99,16,65,1,9,110,99,16,2,0,0,0,0,143,1,116,114,99,32,3,105,1,116,32,65,198,198,1,8,57,19,117,105,101,97,16,4,0,115,16,65,193,206,202,0,0,114,110,16,2,1,0,11,1,7,0,11,1,117,111,105,97,32,4,149,1,32,139,1,0,244,1,0,0,104,100,98,32,3,171,1,32,134,218,1,0,135,1,5,2,108,105,104,32,3,0,16,146,1,0,0,0,232,1,110,101,99,32,3,85,111,108,99,16,2,203,1,32,135,194,212,1,1,0,115,100,32,2,1,14,110,108,16,66,1,26,40,111,105,101,16,3,0,0,0,115,110,109,16,67,1,101,16,65,199,1,101,16,65,79,2,1,0,0,0,116,115,100,32,3,1,0,110,109,16,2,115,1,32,130,0,0,0,114,110,109,16,3,1,9,111,105,16,2,30,2,26,2,115,101,32,66,0,1,115,114,16,2,128,2,0,0,116,108,32,2,218,16,138,249,1,32,146,1,0,55,2,5,0,8,0,0,0,114,110,108,105,103,32,5,160,2,100,32,65,81,2,0,0,115,100,32,66,1,0,192,1,9,0,114,108,101,32,3,30,1,32,143,1,108,16,65,175,16,139,194,199,206,56,16,135,1,4,188,164,16,2,1,16,24,48,80,195,111,105,101,97,16,5,0,0,114,109,16,66,55,1,32,138,198,1,0,231,2,105,97,32,2,1,14,119,117,16,2,244,2,32,154,188,192,1,0,7,0,32,0,137,0,155,0,189,0,252,0,11,1,38,1,84,1,112,1,138,1,177,1,210,1,221,1,228,1,19,2,169,2,188,2,195,122,119,118,117,115,111,110,109,107,106,105,104,103,102,101,100,98,97,32,19,235,0,0,0,0,0,0,0,71,3,0,0,0,0,0,0,240,0,183,189]},"separators":[" ",",",":",".","\n","\r\n","(","[","{",")","]","}","!","\t","?","\"","'","|","`","-","_"]},"lossy":true},"segmenter_option":{"allow_list":null},"stemmer":["German",false],"vocabulary":null},"en":{"unicode":true,"words_dict":null,"normalizer_option":{"create_char_map":false,"classifier":{"stop_words":{"inner":[3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,16,129,0,16,130,1,4,118,117,16,2,196,0,16,135,194,193,16,115,16,65,203,200,197,0,16,139,0,16,143,0,0,121,100,16,66,35,39,16,65,1,110,16,65,194,0,0,1,10,0,16,19,22,29,34,116,115,114,110,109,108,105,103,102,98,16,74,63,16,134,211,197,68,16,135,196,0,16,151,203,0,16,140,196,58,16,130,194,204,1,6,10,14,63,18,116,108,105,102,101,99,16,70,0,16,142,193,0,106,1,5,121,117,111,101,16,4,78,16,139,210,207,211,1,97,111,97,16,2,86,16,146,89,16,134,109,96,56,1,119,110,105,101,16,68,66,16,136,199,1,5,18,117,111,105,16,67,52,16,138,197,0,16,144,196,150,16,142,193,199,1,6,156,88,117,114,111,101,16,4,101,137,105,101,16,2,1,143,143,118,115,100,16,3,0,16,155,207,1,101,16,65,1,0,115,101,16,66,1,114,16,65,15,16,130,1,115,16,65,0,1,115,109,16,2,143,1,14,32,111,105,101,97,16,4,0,16,132,1,116,16,65,0,16,134,44,1,115,39,16,66,1,208,10,0,116,115,110,102,16,68,4,1,32,134,211,147,16,129,206,215,14,1,11,1,115,114,32,2,11,16,134,64,1,4,12,0,0,121,117,111,105,101,97,16,70,176,16,130,0,0,0,119,116,114,16,67,1,9,111,101,16,2,0,102,16,65,0,16,157,1,0,60,1,108,99,32,66,84,16,130,226,207,1,101,16,65,1,115,16,65,0,1,116,114,16,2,65,1,76,1,1,0,180,0,0,0,20,0,31,0,119,118,117,116,114,110,102,32,71,110,1,32,144,130,39,16,65,118,1,32,162,87,1,1,0,110,39,32,66,210,207,211,1,0,16,0,18,1,111,101,97,32,3,144,1,109,32,65,250,0,48,1,1,0,6,0,0,0,36,0,117,116,111,104,98,97,32,70,144,1,32,143,1,39,16,65,1,0,116,110,16,2,0,115,16,65,199,116,16,130,1,115,16,65,0,0,195,1,195,1,0,0,1,0,8,0,121,115,114,110,109,105,32,70,114,1,32,151,211,196,1,0,156,1,237,0,7,0,39,0,114,111,105,101,97,32,5,0,111,16,65,1,5,111,104,16,66,223,1,32,136,1,0,241,1,116,100,32,2,0,1,112,110,16,2,207,114,16,65,194,231,1,114,32,65,20,2,0,0,114,110,32,2,28,2,183,1,108,99,32,2,0,109,16,65,0,0,1,0,5,0,13,0,43,2,121,111,105,101,97,32,5,212,1,32,2,116,108,32,2,204,1,30,2,117,110,32,2,1,0,9,0,17,0,54,0,199,1,111,105,104,101,97,32,5,90,2,90,2,65,2,0,0,118,114,108,100,32,4,34,1,0,0,118,102,32,2,207,1,101,16,65,1,115,16,65,1,18,114,39,16,66,211,1,111,16,65,1,0,43,0,118,0,123,0,141,0,220,0,132,2,23,1,80,1,97,1,107,2,127,1,132,1,158,1,213,1,232,1,236,1,8,2,20,2,70,2,121,119,118,117,116,115,114,111,110,109,108,106,105,104,102,101,100,99,98,97,32,20,181,0,0,0,0,0,0,0,214,2,0,0,0,0,0,0,175,79,249,173]},"separators":[" ",",",":",".","\n","\r\n","(","[","{",")","]","}","!","\t","?","\"","'","|","`","-","_"]},"lossy":true},"segmenter_option":{"allow_list":null},"stemmer":["English",false],"vocabulary":null}}}"#
        ).unwrap();

        let v1 = PyDictionary::load(
            PathBuf::from(r#"E:\git\ptmt\data\final_dict\dictionary_20241130_proc3.dat.zst"#)
        ).unwrap();

        generate_google_ngram_statistic(
            r#"Z:\NGrams"#,
            r#"E:\tmp\google_ngrams2"#,
            &proc,
            &v1,
            &[
                NGramDefinition::new(
                    "de",
                    1,
                    8
                ),
                NGramDefinition::new(
                    "de",
                    2,
                    181
                ),
                NGramDefinition::new(
                    "en",
                    1,
                    24
                ),
                NGramDefinition::new(
                    "en",
                    2,
                    589
                )
            ]
        ).expect("Should not fail!");
    }

    #[test]
    fn see_idf(){
        for k in [
            "with_proc",
            "without_proc",
        ] {
            bincode::deserialize_from::<_, NGramStatistics<ArcStr>>(
                BufReader::new(
                    File::options().read(true).open(format!(r#"E:\tmp\google_ngrams2\gen\counts_{k}.bin"#)).unwrap()
                )
            ).unwrap().into_inner().into_iter().for_each(|(lang, entry)| {
                for adj in [true, false] {
                    serde_json::to_writer_pretty(
                        BufWriter::new(File::options().write(true).create(true).truncate(true).open(format!(r#"E:\tmp\google_ngrams2\gen\{lang}_{k}_{}.json"#, if adj {"adj"} else {"not_adj"})).unwrap()),
                        &entry.all_idf_with_freq(&Idf::InverseDocumentFrequency, adj).unwrap().into_iter().sorted_by_key(|(k, _)| k.clone()).collect::<IndexMap<ArcStr, _>>()
                    ).unwrap()
                }
            });
        }
    }

    #[test]
    fn load(){
        // en1 152050
        // en2 680038
        // de1 468070
        // de2 579447

        for targ in [
            "word_counts_en_1",
            "word_counts_en_2",
            "word_counts_de_1",
            "word_counts_de_2",
        ] {
            let data: HashMap<String, HashMap<String, u128>> = bincode::deserialize_from(
                File::open(format!(r#"E:\tmp\google_ngams\Test1\{targ}.bin"#)).map(BufReader::new).unwrap()
            ).unwrap();
            let col = data.values().flat_map(|v| v.values().copied()).collect::<Vec<_>>();

            println!("Name {}", targ);
            println!("Min: {}", col.iter().min().unwrap());
            println!("Max: {}", col.iter().max().unwrap());
            println!("Max: {}", col.iter().sum::<u128>());
            println!("Len: {}", data.len());
        }

        /*


        Name word_counts_en_1
        Min: 40
        Max: 68877126
        Max: 233421132918
        Len: 152050

        Name word_counts_en_2
        Min: 40
        Max: 50004715
        Max: 1077358227835
        Len: 680038

        Name word_counts_de_1
        Min: 40
        Max: 10824267
        Max: 54356049594
        Len: 468070

        Name word_counts_de_2
        Min: 40
        Max: 8170745
        Max: 181495046469
        Len: 579447

         */


        // serde_json::to_writer_pretty(
        //     File::options().create(true).write(true).open(format!(r#"E:\tmp\google_ngams\{targ}.json"#)).map(BufWriter::new).unwrap(),
        //     &data
        // ).unwrap()
    }
}