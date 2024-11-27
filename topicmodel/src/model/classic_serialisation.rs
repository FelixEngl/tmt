use std::convert::Infallible;
use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::ops::DerefMut;
use std::path::Path;
use std::str::FromStr;
use arcstr::ArcStr;
use flate2::Compression;
use itertools::Itertools;
use crate::enums::{ReadError, TopicModelVersion, WriteError};
use crate::enums::ReadError::NotFinishedError;
use crate::io::{TopicModelFSRead, TopicModelFSWrite};
use crate::io::TopicModelIOError::PathNotFound;
use crate::model::{FullTopicModel, TopicModel};
use crate::traits::AsParseableString;
use crate::vocabulary::{LoadableVocabulary, StoreableVocabulary, Vocabulary, VocabularyMut};

impl TopicModel<ArcStr, Vocabulary<ArcStr>> {
    pub fn load_string_model(path: impl AsRef<Path>, allow_unfinished: bool) -> Result<(Self, TopicModelVersion), ReadError<Infallible>> {
        Self::load(path, allow_unfinished)
    }
}

const MODEL_ZIP_PATH: &str = "model.zip";
const PATH_TO_DOC_LENGTHS: &str = "doc\\doc_lengths.freq";
const PATH_TO_DOC_TOPIC_DISTS: &str = "doc\\doc_topic_dists.freq";
const PATH_TO_VOCABULARY_FREQ: &str = "voc\\vocabulary.freq";
const PATH_TO_VOCABULARY: &str = "voc\\vocabulary.txt";
const PATH_TO_MODEL: &str = "model\\topic.model";
const PATH_VERSION_INFO: &str = "version.info";
const MARKER_FILE: &str = "COMPLETED_TM";

impl<T, V> TopicModel<T, V> {
    pub fn is_already_finished(path: impl AsRef<Path>) -> bool {
        println!("{:}", path.as_ref().join(MARKER_FILE).to_str().unwrap());
        path.as_ref().join(MARKER_FILE).exists()
    }
}

impl<T, E, V> TopicModel<T, V>
where
    V: LoadableVocabulary<T, E> + VocabularyMut<T> + Sync + Send,
    T: FromStr<Err=E> + Hash + Eq + Ord + Clone,
    E: Debug
{

    pub fn load(path: impl AsRef<Path>, allow_unfinished: bool) -> Result<(TopicModel<T, V>, TopicModelVersion), ReadError<E>> {
        if !allow_unfinished && !Self::is_already_finished(&path) {
            return Err(NotFinishedError(path.as_ref().to_path_buf()))
        }
        let reader = if path.as_ref().is_file() {
            TopicModelFSRead::open_zip(path)?
        } else {
            let z = path.as_ref().join(MODEL_ZIP_PATH);
            if z.exists() {
                TopicModelFSRead::open_zip(z)?
            } else {
                TopicModelFSRead::open_file_system(path)?
            }
        };
        Self::load_routine(reader)
    }

    fn load_routine(mut fs: TopicModelFSRead) -> Result<(TopicModel<T, V>, TopicModelVersion), ReadError<E>> {
        let mut buf = String::new();

        let version = match fs.create_reader_to(PATH_VERSION_INFO) {
            Ok((mut reader, _)) => {
                reader.read_to_string(&mut buf)?;
                if buf.is_empty() {
                    TopicModelVersion::V1
                } else {
                    buf.trim().parse()?
                }
            }
            Err(PathNotFound(_)) => {
                TopicModelVersion::V1
            }
            Err(other) => return Err(other.into())
        };

        match &version {
            TopicModelVersion::V1 => {
                let doc_lengths = { Self::read_vec_u64(fs.create_reader_to(PATH_TO_DOC_LENGTHS)?.0) }?;
                let (inp, deflate) = fs.create_reader_to(PATH_TO_DOC_TOPIC_DISTS)?;
                let doc_topic_distributions = Self::read_matrix_f64(inp, deflate)?;
                let used_vocab_frequency = Self::read_vec_u64(fs.create_reader_to(PATH_TO_VOCABULARY_FREQ)?.0)?;
                let (inp, _) = fs.create_reader_to(PATH_TO_VOCABULARY)?;
                let vocabulary = V::load_from_input(&mut BufReader::new(inp))?;
                let (inp, deflate) = fs.create_reader_to(PATH_TO_MODEL)?;
                let topics = Self::read_matrix_f64(inp, deflate)?;
                Ok(
                    (
                        Self::new(
                            topics,
                            vocabulary,
                            used_vocab_frequency,
                            doc_topic_distributions,
                            doc_lengths
                        ),
                        version
                    )
                )
            }
            TopicModelVersion::V2 => {
                panic!("Unsupported")
            }
        }
    }


    fn read_vec_u64(inp: impl Read) -> Result<Vec<u64>, ReadError<E>> {
        BufReader::new(inp).lines().process_results(|lines| {
            lines.enumerate().map(|(pos, line)| line.trim().parse::<u64>().map_err(|err| ReadError::ParseInt {
                line: pos,
                position: 0,
                err
            })).collect::<Result<Vec<_>, _>>()
        })?
    }

    fn read_matrix_f64(inp: impl Read, deflate: bool) -> Result<Vec<Vec<f64>>, ReadError<E>> {
        let mut reader: Box<dyn BufRead> = if deflate {
            Box::new(BufReader::new(flate2::read::DeflateDecoder::new(inp)))
        } else {
            Box::new(BufReader::new(inp))
        };

        reader.deref_mut().lines().process_results(|lines| {
            lines.enumerate().filter_map(move |(line_no, mut line)| {
                line.retain(|value| !['\n', '\r', '\t'].contains(&value));
                if line.is_empty() {
                    None
                } else {
                    Some(line.trim().split(" ").enumerate().map(
                        |(pos, it)| it.replace(",", ".").parse::<f64>().map_err(|err| ReadError::ParseFloat {
                            line: line_no,
                            position: pos,
                            err
                        })
                    ).collect::<Result<Vec<f64>, _>>())
                }
            }).collect::<Result<Vec<_>, _>>()
        })?
    }
}

impl<T: AsParseableString, V> TopicModel<T, V> where V: StoreableVocabulary<T> {

    pub fn save(&self, path: impl AsRef<Path>, save_version: TopicModelVersion, deflate: bool, replace: bool) -> Result<usize, WriteError> {
        if Self::is_already_finished(&path) {
            if !replace {
                return Err(WriteError::AlreadyFinished)
            } else {
                if path.as_ref().exists() {
                    std::fs::remove_dir_all(&path)?;
                }
            }
        } else {
            if path.as_ref().exists() {
                std::fs::remove_dir_all(&path)?;
            }
        }

        let mut fs = if deflate {
            TopicModelFSWrite::create_zip(path.as_ref().join(MODEL_ZIP_PATH))
        } else {
            TopicModelFSWrite::create_file_system(&path)
        }?;


        let result = self.save_routine(&mut fs, save_version, false)?;
        match std::fs::File::create_new(path.as_ref().join(MARKER_FILE)) {
            Ok(_) => {}
            Err(err) => {
                match err.kind() {
                    ErrorKind::AlreadyExists => {}
                    _ => {
                        return Err(WriteError::IO(err))
                    }
                }
            }
        }
        Ok(result)
    }



    fn save_routine(&self, fs: &mut TopicModelFSWrite, save_version: TopicModelVersion, deflate: bool) -> Result<usize, WriteError> {
        let mut bytes_written = fs.create_writer_to(PATH_VERSION_INFO)?.write(save_version.as_ref().as_bytes())?;
        match save_version {
            TopicModelVersion::V1 => {
                bytes_written += self.vocabulary.save_to_output(&mut fs.create_writer_to(PATH_TO_VOCABULARY)?)?;
                bytes_written += fs.create_writer_to(PATH_TO_VOCABULARY_FREQ)?.write(self.used_vocab_frequency.iter().map(|value| value.to_string()).join("\n").as_bytes())?;
                bytes_written += fs.create_writer_to(PATH_TO_DOC_LENGTHS)?.write(self.document_lengths.iter().map(|value| value.to_string()).join("\n").as_bytes())?;
                bytes_written += Self::write_matrix_f64(&mut fs.create_writer_to(PATH_TO_DOC_TOPIC_DISTS)?, &self.doc_topic_distributions, deflate)?;
                bytes_written += Self::write_matrix_f64(&mut fs.create_writer_to(PATH_TO_MODEL)?, &self.topics, deflate)?;
            }
            TopicModelVersion::V2 => {
                panic!("Unsupported!")
            }
        }
        fs.create_writer_to(MARKER_FILE)?.write(&[])?;
        Ok(bytes_written)
    }

    fn write_matrix_f64(out: &mut impl Write, target: &Vec<Vec<f64>>, deflate: bool) -> io::Result<usize> {
        let mut write: Box<dyn Write> = if deflate {
            Box::new(BufWriter::new(flate2::write::DeflateEncoder::new(out, Compression::default())))
        } else {
            Box::new(BufWriter::new(out))
        };
        let mut bytes = 0usize;
        for doubles in target {
            let t = doubles.iter().map(|value| format!("{:.20}", value).replace(',', ".")).join(" ");
            bytes += write.write(t.as_bytes())?;
            bytes += write.write(b"\n")?;
        }
        Ok(bytes)
    }
}
