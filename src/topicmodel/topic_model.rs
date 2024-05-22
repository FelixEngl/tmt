use std::borrow::Borrow;
use std::cmp::min;
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::ops::{DerefMut, Range};
use std::path::Path;
use std::str::FromStr;
use approx::relative_eq;

use flate2::Compression;
use itertools::Itertools;
use serde::Serialize;

use crate::topicmodel::enums::{ReadError, TopicModelVersion, WriteError};
use crate::topicmodel::enums::ReadError::NotFinishedError;
use crate::topicmodel::traits::{ToParseableString};
use crate::topicmodel::io::{TopicModelFSRead, TopicModelFSWrite, TopicModelIOError, TopicModelWriter};
use crate::topicmodel::vocabulary::Vocabulary;

type WordToProbability = Vec<f64>;
type TopicToProbability = Vec<f64>;
type TopicToWordProbability = Vec<WordToProbability>;
type WordToTopicProbability = Vec<TopicToProbability>;
type DocumentTopicToProbability = Vec<TopicToProbability>;


pub type StringTopicModel = TopicModel<String>;

#[derive(Clone, Debug)]
pub struct TopicModel<T> {
    topics: TopicToWordProbability,
    vocabulary: Vocabulary<T>,
    used_vocab_frequency: Vec<u64>,
    doc_topic_distributions: DocumentTopicToProbability,
    document_lengths: Vec<u64>,
    topic_stats: Vec<TopicStats>,
    topics_to_probability_sorted_word_ids: Vec<Vec<usize>>,
    topics_to_importance_to_word_ids: Vec<Vec<Vec<usize>>>,
    topics_to_word_id_to_importance: Vec<Vec<usize>>
}





impl<T: Hash + Eq> TopicModel<T> {
    pub fn new(
        topics: TopicToWordProbability,
        vocabulary: Vocabulary<T>,
        used_vocab_frequency: Vec<u64>,
        doc_topic_distributions: DocumentTopicToProbability,
        document_lengths: Vec<u64>,
    ) -> Self {
        let topic_stats =
            Self::calculate_topic_stats(&topics);

        let topics_to_probability_sorted_word_ids: Vec<Vec<usize>> =
            unsafe {
                Self::calculate_topics_to_probability_sorted_word_ids(&topics)
            };

        let topics_to_importance_to_word_ids =
            unsafe {
                Self::calculate_topics_to_importance_to_word_ids(
                    &topics,
                    &topics_to_probability_sorted_word_ids
                )
            };

        let topics_to_word_id_to_importance =
            unsafe {
                Self::calculate_topics_to_word_id_to_importance(
                    &topics,
                    &topics_to_importance_to_word_ids
                )
            };


        Self {
            topics,
            vocabulary,
            used_vocab_frequency,
            doc_topic_distributions,
            document_lengths,
            topic_stats,
            topics_to_probability_sorted_word_ids,
            topics_to_importance_to_word_ids,
            topics_to_word_id_to_importance
        }
    }

    delegate::delegate! {
        to self.vocabulary {
            pub fn get_word_id<Q: ?Sized>(&self, word: &Q) -> Option<usize> where T: Borrow<Q>, Q: Hash + Eq;
            pub fn contains<Q: ?Sized>(&self, word: &Q) -> bool where T: Borrow<Q>, Q: Hash + Eq;
        }
    }

    pub fn get_probability_by_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<&f64> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_probability(topic_id, self.get_word_id(word)?)
    }
    pub fn get_word_to_topic_probabilities_by_word<Q: ?Sized>(&self, word: &Q) -> Option<TopicToProbability> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_word_to_topic_probabilities(self.get_word_id(word)?)
    }

    pub fn get_probability_and_position_for_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<WordProbabilityWithPosition<T>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_probability_and_position_for(topic_id, self.vocabulary.get_word_id(word)?)
    }

    pub fn get_probabilities_and_positions_for_word<Q: ?Sized>(&self, word: &Q) -> Option<Vec<WordProbabilityWithPosition<T>>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_probabilities_and_positions_for(self.vocabulary.get_word_id(word)?)
    }

    pub fn get_all_similar_important_words_for_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<Vec<WordProbabilityWithPosition<T>>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_all_similar_important_words_for(topic_id, self.vocabulary.get_word_id(word)?)
    }
}

impl<T> TopicModel<T> {
    pub fn is_already_finished(path: impl AsRef<Path>) -> bool {
        path.as_ref().join(MARKER_FILE).exists()
    }

    /// The number of topics in this model
    pub fn topic_count(&self) -> usize {
        self.topics.len()
    }

    pub fn vocabulary(&self) -> &Vocabulary<T> {
        &self.vocabulary
    }

    /// The size of the vocabulary for this model.
    pub fn vocabulary_size(&self) -> usize {
        self.vocabulary.len()
    }

    /// A range over all topicIds
    pub fn topic_ids(&self) -> Range<usize> {
        0..self.topics.len()
    }

    pub fn stats(&self) -> &[TopicStats] {
        &self.topic_stats
    }

    pub fn topics(&self) -> &Vec<Vec<f64>> {
        &self.topics
    }

    delegate::delegate! {
        to self.vocabulary {
            pub fn get_word(&self, id: usize) -> Option<&T>;
            pub fn contains_id(&self, id: usize) -> bool;
        }
    }

    pub fn get_topic(&self, topic_id: usize) -> Option<&Vec<f64>> {
        self.topics.get(topic_id)
    }

    pub fn get_probability(&self, topic_id: usize, word_id: usize) -> Option<&f64> {
        self.topics.get(topic_id)?.get(word_id)
    }


    pub fn get_word_to_topic_probabilities(&self, word_id: usize) -> Option<TopicToProbability> {
        if self.contains_id(word_id) {
            Some(self.topics.iter().map(|value| unsafe{value.get_unchecked(word_id).clone()}).collect())
        } else {
            None
        }
    }

    pub fn get_word_at(&self, topic_id: usize, index: usize) -> Option<&T> {
        self.vocabulary.get_word(
            self.topics_to_probability_sorted_word_ids
                .get(topic_id)?
                .get(index)?
                .clone()
        )
    }

    pub fn get_probability_and_position_for(&self, topic_id: usize, word_id: usize) -> Option<WordProbabilityWithPosition<T>> {
        let probability = *self.get_probability(topic_id, word_id)?;
        let importance = *self.topics_to_word_id_to_importance.get(topic_id)?.get(word_id)?;
        let index = self.topics_to_probability_sorted_word_ids.get(topic_id)?.iter().position(|value| word_id.eq(value))?;
        Some(
            WordProbabilityWithPosition {
                topic_id,
                word: self.vocabulary.get_word(word_id)?,
                word_id,
                probability,
                index_in_topic: index,
                importance
            }
        )
    }

    pub fn get_probabilities_and_positions_for(&self, word_id: usize) -> Option<Vec<WordProbabilityWithPosition<T>>> {
        self.topic_ids().map(|topic_id| self.get_probability_and_position_for(topic_id, word_id)).collect()
    }

    pub fn get_all_similar_important_words_for(&self, topic_id: usize, word_id: usize) -> Option<Vec<WordProbabilityWithPosition<T>>> {
        self.topics_to_importance_to_word_ids
            .get(topic_id)?
            .get(*self.topics_to_word_id_to_importance.get(topic_id)?.get(word_id)?)?
            .iter()
            .map(|word_id| self.get_probability_and_position_for(topic_id, *word_id))
            .collect()
    }

    pub fn get_n_best_for_topic(&self, topic_id: usize, n: usize) -> Option<Vec<WordIdAndProbability>> {
        let topic = self.topics.get(topic_id)?;
        let probability_sorted_word_ids = self.topics_to_probability_sorted_word_ids.get(topic_id)?;
        Some(
            (0..min(n, topic.len())).map(|it| unsafe {
                let word_id = *probability_sorted_word_ids.get_unchecked(it);
                WordIdAndProbability {
                    word_id,
                    probability: *topic.get_unchecked(word_id)
                }
            }).collect()
        )
    }

    pub fn get_n_best_for_topics(&self, n: usize) -> Option<Vec<Vec<WordIdAndProbability>>> {
        self.topic_ids().map(|topic_id| self.get_n_best_for_topic(topic_id, n)).collect()
    }

    fn calculate_topic_stats(topics: &Vec<Vec<f64>>) -> Vec<TopicStats> {
        topics.iter().enumerate().map(|(topic_id, topic)| {
            let mut max_value: f64 = f64::MIN;
            let mut min_value: f64 = f64::MAX;
            let mut sum_value: f64 = 0.0;

            for &value in topic {
                max_value = max_value.max(value);
                min_value = min_value.min(value);
                sum_value += value;
            }


            TopicStats {
                topic_id,
                max_value,
                min_value,
                sum_value,
                average_value: sum_value / (topic.len() as f64)
            }
        }).collect()
    }

    unsafe fn calculate_topics_to_probability_sorted_word_ids(topics: &Vec<Vec<f64>>) -> Vec<Vec<usize>> {
        topics.iter().map(|topic| {
            let mut created = (0..topic.len())
                .sorted_by(|a, b|
                    topic.get_unchecked(*a)
                        .partial_cmp(topic.get_unchecked(*b))
                        .unwrap()
                )
                .collect_vec();
            created.reverse();
            created
        }).collect()
    }

    unsafe fn calculate_topics_to_importance_to_word_ids(topics: &Vec<Vec<f64>>, topics_to_probability_sorted_word_ids: &Vec<Vec<usize>>) -> Vec<Vec<Vec<usize>>> {
        topics.iter().enumerate().map(|(topic_id, topic)| {
            let probability_sorted_word_ids: &Vec<usize> = topics_to_probability_sorted_word_ids.get_unchecked(topic_id);
            let mut current_value = topic.get_unchecked(probability_sorted_word_ids[0]);
            let mut current_sink = Vec::new();
            let mut collection = Vec::new();
            for word_id in probability_sorted_word_ids {
                let probability = topic.get_unchecked(*word_id);
                if current_value != probability {
                    current_sink.shrink_to_fit();
                    collection.push(current_sink);
                    current_sink = Vec::new();
                    current_value = probability;
                }
                current_sink.push(*word_id);
            }
            current_sink.shrink_to_fit();
            collection.push(current_sink);
            collection
        }).collect()
    }

    unsafe fn calculate_topics_to_word_id_to_importance(
        topics: &Vec<Vec<f64>>,
        topics_to_importance_to_word_ids: &Vec<Vec<Vec<usize>>>
    ) -> Vec<Vec<usize>> {
        topics.iter().enumerate().map(|(topic_id, topic)| {
            let groupings = topics_to_importance_to_word_ids.get_unchecked(topic_id);
            let mut result = vec![0usize; topic.len()];

            for (importance, importances) in groupings.iter().enumerate() {
                for idx in importances {
                    result.insert(*idx, importance);
                }
            }
            result
        }).collect()
    }

    fn recalculate_statistics(&mut self) {
        self.topic_stats = Self::calculate_topic_stats(&self.topics);

        unsafe {
            self.topics_to_probability_sorted_word_ids =
                Self::calculate_topics_to_probability_sorted_word_ids(&self.topics);

            self.topics_to_importance_to_word_ids =
                Self::calculate_topics_to_importance_to_word_ids(
                    &self.topics,
                    &self.topics_to_probability_sorted_word_ids
                );

            self.topics_to_word_id_to_importance =
                Self::calculate_topics_to_word_id_to_importance(
                    &self.topics,
                    &self.topics_to_importance_to_word_ids
                )
        }
    }

    pub fn normalize_in_place(mut self) -> Self {
        for topic in self.topics.iter_mut() {
            let sum: f64 = topic.iter().sum();
            topic.iter_mut().for_each(|value| {
                *value /= sum
            });
        }

        for probabilities in self.doc_topic_distributions.iter_mut() {
            let sum: f64 = probabilities.iter().sum();
            probabilities.iter_mut().for_each(|value| {
                *value /= sum
            });
        }

        self.recalculate_statistics();

        self
    }
}

impl<T: Clone> TopicModel<T> {
    pub fn normalize(&self) -> Self {
        self.clone().normalize_in_place()
    }
}

impl<T: Eq + Hash> TopicModel<T> {
    fn seems_equal_to(&self, other: &TopicModel<T>) -> bool {
        self.topic_count() == other.topic_count()
            && self.vocabulary_size() == other.vocabulary_size()
            && self.vocabulary.iter().enumerate().all(|(word_id, word)| {
            if let Some(found) = other.vocabulary.get_word_id(word) {
                self.used_vocab_frequency.get(word_id) == other.used_vocab_frequency.get(found)
            } else {
                false
            }
        })
            && self.topics
            .iter()
            .zip_eq(other.topics.iter())
            .enumerate()
            .all(|(topic_id, (topic, other_topic))| {
                self.vocabulary
                    .iter()
                    .enumerate()
                    .all(|(word_id, word)| {
                        unsafe {
                            // all accesses are already checked by the checks above!
                            let value = topic.get_unchecked(word_id);
                            let other_word_id = other.vocabulary.get_word_id(word).expect("All words should be known!");
                            let value_other = other_topic.get_unchecked(other_word_id);
                            relative_eq!(*value, *value_other)
                        }
                    })
            })
    }
}

impl<T: Display> TopicModel<T> {

    pub fn show_to(&self, n: usize, out: &mut impl Write) -> io::Result<()> {
        for (topic_id, topic_entries) in self.get_n_best_for_topics(n).ok_or(io::Error::from(ErrorKind::Other))?.iter().enumerate() {
            if topic_id != 0 {
                out.write(b"\n")?;
            }
            write!(out, "Topic({topic_id}):")?;
            for it in topic_entries {
                out.write(b"\n")?;
                write!(out, "    {}: {}", self.vocabulary.get_word(it.word_id).unwrap(), it.probability)?;
            }
        }
        Ok(())
    }

    pub fn show(&self, n: usize) -> io::Result<()> {
        let mut str = Vec::new();
        self.show_to(n, &mut str)?;
        println!("{}", String::from_utf8(str).unwrap());
        Ok(())
    }

    pub fn show_10(&self) -> io::Result<()>{
        self.show(10)
    }
}

impl<T: Display> Display for TopicModel<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Topic Model:")?;
        for (topic_id, topic) in self.topics.iter().enumerate() {
            write!(f, "\n    Topic({topic_id})")?;
            for (word_id, probability) in topic.iter().enumerate() {
                write!(f, "\n        '{}'({}): {}", self.vocabulary.get_word(word_id).unwrap(), word_id, probability)?;
            }
        }
        write!(f, "\n{}", self.vocabulary)
    }
}

pub struct WordIdAndProbability {
    pub word_id: usize,
    pub probability: f64
}


pub struct WordProbabilityWithPosition<'a, T> {
    pub topic_id: usize,
    pub word: &'a T,
    pub word_id: usize,
    pub probability: f64,
    pub index_in_topic: usize,
    pub importance: usize
}


impl TopicModel<String> {
    pub fn load_string_model(path: impl AsRef<Path>) -> Result<(Self, TopicModelVersion), ReadError<Infallible>> {
        Self::load(path)
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

impl<T: FromStr<Err=E> + Hash + Eq, E: Debug> TopicModel<T> {

    pub fn load(path: impl AsRef<Path>) -> Result<(Self, TopicModelVersion), ReadError<E>> {
        if !Self::is_already_finished(&path) {
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

    fn load_routine(mut fs: TopicModelFSRead) -> Result<(Self, TopicModelVersion), ReadError<E>> {
        let mut buf = String::new();
        fs.create_reader_to(PATH_VERSION_INFO)?.0.read_to_string(&mut buf)?;
        let version = buf.trim().parse()?;
        match &version {
            TopicModelVersion::V1 => {
                let doc_lengths = { Self::read_vec_u64(fs.create_reader_to(PATH_TO_DOC_LENGTHS)?.0) }?;
                let (inp, deflate) = fs.create_reader_to(PATH_TO_DOC_TOPIC_DISTS)?;
                let doc_topic_distributions = Self::read_matrix_f64(inp, deflate)?;
                let used_vocab_frequency = Self::read_vec_u64(fs.create_reader_to(PATH_TO_VOCABULARY_FREQ)?.0)?;
                let (inp, deflate) = fs.create_reader_to(PATH_TO_VOCABULARY)?;
                let vocabulary = Vocabulary::load_from_input(&mut BufReader::new(inp))?;
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

impl<T: ToParseableString> TopicModel<T> {

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

#[derive(Debug, Clone, Serialize)]
pub struct TopicStats {
    pub topic_id: usize,
    pub max_value: f64,
    pub min_value: f64,
    pub average_value: f64,
    pub sum_value: f64,
}

#[cfg(test)]
mod test {
    use crate::topicmodel::enums::TopicModelVersion;
    use crate::topicmodel::topic_model::{StringTopicModel, TopicModel};
    use crate::topicmodel::vocabulary::{StringVocabulary, Vocabulary};

    pub fn create_test_data() -> StringTopicModel {
        let mut voc: StringVocabulary = Vocabulary::new();
        voc.add("plane");
        voc.add("aircraft");
        voc.add("airplane");
        voc.add("flyer");
        voc.add("airman");
        voc.add("airfoil");
        voc.add("wing");
        voc.add("deck");
        voc.add("hydrofoil");
        voc.add("foil");
        voc.add("bearing surface");

        TopicModel::new(
            vec![
                vec![0.019, 0.018, 0.012, 0.009, 0.008, 0.008, 0.008, 0.008, 0.008, 0.008, 0.008],
                vec![0.02, 0.002, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001],
            ],
            voc,
            vec![10, 5, 8, 1, 2, 3, 1, 1, 1, 1, 2],
            vec![
                vec![0.7, 0.2],
                vec![0.8, 0.3],
            ],
            vec![
                200,
                300
            ]
        )
    }

    #[test]
    fn can_load_and_unload(){
        let topic_model = create_test_data();
        const P: &str = "test\\def";

        std::fs::create_dir("test");
        topic_model.save(P, TopicModelVersion::V1, true, true).unwrap();

        let (loaded, version) = TopicModel::load_string_model(P).unwrap();

        assert!(topic_model.seems_equal_to(&loaded));

        topic_model.normalize().show_10().unwrap();

        std::fs::remove_dir_all(P).unwrap();
    }
}