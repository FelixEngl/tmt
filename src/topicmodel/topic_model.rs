use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::ops::{Deref, DerefMut, Range};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use approx::relative_eq;

use flate2::Compression;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::toolkit::normal_number::IsNormalNumber;

use crate::topicmodel::enums::{ReadError, TopicModelVersion, WriteError};
use crate::topicmodel::enums::ReadError::NotFinishedError;
use crate::topicmodel::traits::{ToParseableString};
use crate::topicmodel::io::{TopicModelFSRead, TopicModelFSWrite};
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{Vocabulary};


type TopicTo<T> = Vec<T>;
type WordTo<T> = Vec<T>;
type PositionTo<T> = Vec<T>;
type DocumentTo<T> = Vec<T>;
type ImportanceRankTo<T> = Vec<T>;
type Probability = f64;

/// The direct rank, created by the order of the probabilities and then
type Rank = usize;

/// The rank, when grouping the topic by probabilities
type ImportanceRank = usize;
type WordId = usize;
type TopicId = usize;
type Position = usize;
type Importance = usize;
type DocumentId = usize;
type WordFrequency = u64;
type DocumentLength = u64;



pub type StringTopicModel = TopicModel<String>;

/// The meta for a topic.
#[derive(Debug, Clone)]
pub struct TopicMeta {
    pub stats: TopicStats,
    pub by_words: WordTo<Arc<WordMeta>>,
    pub by_position: PositionTo<Arc<WordMeta>>,
    pub by_importance: ImportanceRankTo<Vec<Arc<WordMeta>>>
}

impl TopicMeta {
    pub fn new(
        stats: TopicStats,
        mut by_words: WordTo<Arc<WordMeta>>,
        mut by_position: PositionTo<Arc<WordMeta>>,
        mut by_importance: ImportanceRankTo<Vec<Arc<WordMeta>>>
    ) -> Self {
        by_words.shrink_to_fit();
        by_position.shrink_to_fit();
        by_importance.shrink_to_fit();

        Self {
            stats,
            by_words,
            by_position,
            by_importance
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordMeta {
    pub topic_id: TopicId,
    pub word_id: WordId,
    pub probability: Probability,
    pub position: Position,
    pub importance: Importance,
}

impl WordMeta {
    #[inline]
    pub fn rank(&self) -> Rank {
        self.position + 1
    }

    #[inline]
    pub fn importance_rank(&self) -> Rank {
        self.importance + 1
    }
}

impl Display for WordMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5}({})", self.probability, self.rank())
    }
}



#[derive(Debug)]
pub struct WordMetaWithWord<'a, T> {
    pub word: &'a T,
    inner: &'a Arc<WordMeta>
}

impl<'a, T> WordMetaWithWord<'a, T> {
    pub fn new(word: &'a T, inner: &'a Arc<WordMeta>) -> Self {
        Self {
            word,
            inner
        }
    }
}

impl<'a, T> WordMetaWithWord<'a, T> {
    pub fn into_inner(self) -> &'a Arc<WordMeta> {
        self.inner
    }
}

impl<T> Deref for WordMetaWithWord<'_, T> {
    type Target = Arc<WordMeta>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}






#[derive(Clone, Debug)]
pub struct TopicModel<T> {
    topics: TopicTo<WordTo<Probability>>,
    vocabulary: Vocabulary<T>,
    used_vocab_frequency: WordTo<WordFrequency>,
    doc_topic_distributions: DocumentTo<TopicTo<Probability>>,
    document_lengths: DocumentTo<DocumentLength>,
    topic_metas: TopicTo<TopicMeta>
}


impl<T: Hash + Eq + Ord> TopicModel<T> {
    pub fn new(
        topics: TopicTo<WordTo<Probability>>,
        vocabulary: Vocabulary<T>,
        used_vocab_frequency: WordTo<u64>,
        doc_topic_distributions: DocumentTo<TopicTo<Probability>>,
        document_lengths: Vec<u64>,
    ) -> Self {

        let topic_content = unsafe {
            Self::calculate_topic_metas(&topics, &vocabulary)
        };

        Self {
            topics,
            vocabulary,
            used_vocab_frequency,
            doc_topic_distributions,
            document_lengths,
            topic_metas: topic_content
        }
    }

    delegate::delegate! {
        to self.vocabulary {
            pub fn get_id<Q: ?Sized>(&self, word: &Q) -> Option<usize> where T: Borrow<Q>, Q: Hash + Eq;
            pub fn contains<Q: ?Sized>(&self, word: &Q) -> bool where T: Borrow<Q>, Q: Hash + Eq;
        }
    }

    pub fn get_probability_by_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<&Probability> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_probability(topic_id, self.get_id(word)?)
    }
    pub fn get_topic_probabilities_for_by_word<Q: ?Sized>(&self, word: &Q) -> Option<TopicTo<Probability>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_topic_probabilities_for(self.get_id(word)?)
    }

    pub fn get_word_meta_by_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<&Arc<WordMeta>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_word_meta(topic_id, self.vocabulary.get_id(word)?)
    }

    pub fn get_word_metas_with_word_by_word<Q: ?Sized>(&self, word: &Q) -> Option<Vec<WordMetaWithWord<HashRef<T>>>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_word_metas_with_word(self.vocabulary.get_id(word)?)
    }

    pub fn get_all_similar_important_words_for_word<Q: ?Sized>(&self, topic_id: usize, word: &Q) -> Option<&Vec<Arc<WordMeta>>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_all_similar_important(topic_id, self.vocabulary.get_id(word)?)
    }

    unsafe fn calculate_topic_metas(topics: &TopicTo<WordTo<Probability>>, vocabulary: &Vocabulary<T>) -> TopicTo<TopicMeta> {
        struct SortHelper<'a, Q>(WordId, Probability, &'a Vocabulary<Q>);

        impl<'a, Q> SortHelper<'a, Q> where Q: Hash + Eq  {
            fn word(&self) -> &HashRef<Q> {
                self.2.get_value(self.0).expect("There should be no problem with enpacking it here!")
            }
        }

        impl<Q> Eq for SortHelper<'_, Q> {}

        impl<Q> PartialEq<Self> for SortHelper<'_, Q> {
            fn eq(&self, other: &Self) -> bool {
                self.1.eq(&other.1)
            }
        }

        impl<Q> PartialOrd for SortHelper<'_, Q> where Q: Hash + Eq + Ord {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                match self.1.partial_cmp(&other.1) {
                    None => {
                        if self.1.is_normal_number() {
                            Some(Ordering::Greater)
                        } else if other.1.is_normal_number() {
                            Some(Ordering::Less)
                        } else {
                            Some(
                                other.2.get_value(other.0).unwrap().cmp(
                                    self.2.get_value(self.0).unwrap()
                                )
                            )
                        }
                    }
                    Some(Ordering::Equal) => {
                        Some(
                            other.2.get_value(other.0).unwrap().cmp(
                                self.2.get_value(self.0).unwrap()
                            )
                        )
                    }
                    otherwise => otherwise
                }
            }
        }

        impl<Q> Ord for SortHelper<'_, Q> where Q: Hash + Eq + Ord {
            fn cmp(&self, other: &Self) -> Ordering {
                self.partial_cmp(other).unwrap()
            }
        }

        let mut topics: TopicTo<WordTo<_>> = topics.iter().enumerate().map(|(topic_id, topic)| {
            let position_to_word_id_and_prob = topic
                .iter()
                .enumerate()
                .sorted_by_key(|(word_id, prob)| Reverse(SortHelper(*word_id, **prob, vocabulary)))
                .collect_vec();

            let mut current_value = position_to_word_id_and_prob.first().unwrap().1;
            let mut current_sink = Vec::new();
            let mut importance_to_word_ids: Vec<Vec<WordId>> = Vec::new();

            for (word_id, probability) in &position_to_word_id_and_prob {
                if current_value.ne(*probability) {
                    importance_to_word_ids.push(current_sink);
                    current_sink = Vec::new();
                    current_value = *probability;
                }
                current_sink.push(*word_id);
            }

            if !current_sink.is_empty() {
                importance_to_word_ids.push(current_sink);
            }



            let mut word_it_to_importance = importance_to_word_ids.into_iter().enumerate().flat_map(|(importance, words)| {
                words.into_iter().map(move |value| (importance, value))
            }).collect_vec();
            word_it_to_importance.sort_by_key(|value| value.1);

            let word_id_to_position = position_to_word_id_and_prob
                .iter()
                .enumerate()
                .map(|(position, (word_id, prob))| (word_id, prob, position))
                .sorted_by_key(|(word_id, _, _)| *word_id)
                .collect_vec();

            let mut result = word_id_to_position.into_iter().zip_eq(word_it_to_importance.into_iter()).map(|(((word_id_1, prob, position), (importance, word_id_2)))| {
                assert_eq!(*word_id_1, word_id_2, "Word ids {} {} are not compatible!", word_id_1, word_id_2);
                (word_id_1, prob, position, importance)
            }).zip_eq(topic.into_iter().enumerate()).map(|((word_id_1, probability_1, position, importance), (word_id, probability))| {
                assert_eq!(word_id, *word_id_1, "Word ids {} {} are not compatible in zipping!", word_id, word_id_1);
                assert_eq!(probability, *probability_1,
                           "Probabilities fir the ids {}({}) {}({}) are not compatible in zipping!",
                           word_id, probability,
                           word_id_1, probability_1);
                Arc::new(
                    WordMeta {
                        topic_id,
                        word_id,
                        probability: *probability,
                        position,
                        importance,
                    }
                )
            }).collect_vec();
            result.shrink_to_fit();
            result
        }).collect_vec();

        topics.shrink_to_fit();


        let topics: TopicTo<TopicMeta> = topics.into_iter().enumerate().map(|(topic_id, topic_content)| {

            let position_to_meta: PositionTo<_> = topic_content.iter().sorted_by_key(|value| value.position).cloned().collect_vec();


            let mut importance_to_meta: ImportanceRankTo<_> = Vec::new();

            for value in position_to_meta.iter() {
                while importance_to_meta.len() <= value.importance {
                    importance_to_meta.push(Vec::new())
                }
                importance_to_meta.get_unchecked_mut(value.importance).push(value.clone());
            }

            let mut max_value: f64 = f64::MIN;
            let mut min_value: f64 = f64::MAX;
            let mut sum_value: f64 = 0.0;

            for value in &topic_content {
                max_value = max_value.max(value.probability);
                min_value = min_value.min(value.probability);
                sum_value += value.probability;
            }


            let stats = TopicStats {
                topic_id,
                max_value,
                min_value,
                sum_value,
                average_value: sum_value / (topic_content.len() as f64)
            };

            TopicMeta::new(stats, topic_content, position_to_meta, importance_to_meta)
        }).collect_vec();


        return topics;
    }

    fn recalculate_statistics(&mut self) {
        self.topic_metas = unsafe {
            Self::calculate_topic_metas(&self.topics, &self.vocabulary)
        };
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

    pub fn topics(&self) -> &Vec<Vec<f64>> {
        &self.topics
    }

    pub fn topic_metas(&self) -> &[TopicMeta] {
        &self.topic_metas
    }

    delegate::delegate! {
        to self.vocabulary {
            pub fn get_value(&self, id: usize) -> Option<&HashRef<T>>;
            pub fn contains_id(&self, id: usize) -> bool;
        }
    }

    pub fn get_topic(&self, topic_id: usize) -> Option<&Vec<f64>> {
        self.topics.get(topic_id)
    }

    pub fn get_topic_meta(&self, topic_id: usize) -> Option<&TopicMeta> {
        self.topic_metas.get(topic_id)
    }

    pub fn get_probability(&self, topic_id: usize, word_id: usize) -> Option<&Probability> {
        self.topics.get(topic_id)?.get(word_id)
    }

    pub fn get_word_meta(&self, topic_id: usize, word_id: usize) -> Option<&Arc<WordMeta>> {
        self.topic_metas.get(topic_id)?.by_words.get(word_id)
    }

    pub fn get_topic_probabilities_for(&self, word_id: usize) -> Option<TopicTo<Probability>> {
        if self.contains_id(word_id) {
            Some(self.topics.iter().map(|value| unsafe{value.get_unchecked(word_id).clone()}).collect())
        } else {
            None
        }
    }

    pub fn get_word_metas_for(&self, word_id: usize) -> Option<TopicTo<&Arc<WordMeta>>> {
        if self.contains_id(word_id) {
            Some(self.topic_metas.iter().map(|value| unsafe{value.by_words.get_unchecked(word_id)}).collect())
        } else {
            None
        }
    }


    pub fn get_word_meta_with_word(&self, topic_id: usize, word_id: usize) -> Option<WordMetaWithWord<HashRef<T>>> {
        let topic_meta = self.get_topic_meta(topic_id)?;
        let word_meta = topic_meta.by_words.get(word_id)?;
        let word = self.vocabulary.get_value(word_meta.word_id)?;
        Some(WordMetaWithWord::new(word, word_meta))
    }

    pub fn get_word_metas_with_word(&self, word_id: usize) -> Option<Vec<WordMetaWithWord<HashRef<T>>>> {
        self.topic_ids().map(|topic_id| self.get_word_meta_with_word(topic_id, word_id)).collect()
    }

    pub fn get_all_similar_important(&self, topic_id: usize, word_id: usize) -> Option<&Vec<Arc<WordMeta>>> {
        let topic = self.topic_metas.get(topic_id)?;
        topic.by_importance.get(topic.by_words.get(word_id)?.importance)
    }

    pub fn get_all_similar_important_with_word_for(&self, topic_id: usize, word_id: usize) -> Option<Vec<WordMetaWithWord<HashRef<T>>>> {
        Some(
            self.get_all_similar_important(topic_id, word_id)?
                .iter()
                .map(|value| WordMetaWithWord::new(self.vocabulary.get_value(value.word_id).unwrap(), value))
                .collect()
        )
    }

    pub fn get_n_best_for_topic(&self, topic_id: usize, n: usize) -> Option<&[Arc<WordMeta>]> {
        Some(&self.topic_metas.get(topic_id)?.by_position[..n])
    }

    pub fn get_n_best_for_topics(&self, n: usize) -> Option<Vec<&[Arc<WordMeta>]>> {
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
}

impl<T: Clone + Hash + Eq + Ord> TopicModel<T> {
    pub fn normalize(&self) -> Self {
        self.clone().normalize_in_place()
    }
}

impl<T: Eq + Hash> TopicModel<T> {
    fn seems_equal_to(&self, other: &TopicModel<T>) -> bool {
        self.topic_count() == other.topic_count()
            && self.vocabulary_size() == other.vocabulary_size()
            && self.vocabulary.iter().enumerate().all(|(word_id, word)| {
            if let Some(found) = other.vocabulary.get_id(word) {
                self.used_vocab_frequency.get(word_id) == other.used_vocab_frequency.get(found)
            } else {
                false
            }
        })
            && self.topics
            .iter()
            .zip_eq(other.topics.iter())
            .all(|(topic, other_topic)| {
                self.vocabulary
                    .iter()
                    .enumerate()
                    .all(|(word_id, word)| {
                        unsafe {
                            // all accesses are already checked by the checks above!
                            let value = topic.get_unchecked(word_id);
                            let other_word_id = other.vocabulary.get_id(word).expect("All words should be known!");
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
            for it in topic_entries.iter() {
                out.write(b"\n")?;
                write!(out, "    {}: {} ({})", self.vocabulary.get_value(it.word_id).unwrap(), it.probability, it.rank())?;
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
                write!(f, "\n        '{}'({}): {}", self.vocabulary.get_value(word_id).unwrap(), word_id, probability)?;
            }
        }
        write!(f, "\n{}", self.vocabulary)
    }
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

impl<T: FromStr<Err=E> + Hash + Eq + Ord, E: Debug> TopicModel<T> {

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
                let (inp, _) = fs.create_reader_to(PATH_TO_VOCABULARY)?;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
                vec![0.019, 0.018, 0.012, 0.009, 0.008, 0.007, 0.008, 0.008, 0.008, 0.008, 0.008],
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

        let (loaded, _) = TopicModel::load_string_model(P).unwrap();

        assert!(topic_model.seems_equal_to(&loaded));

        topic_model.show_10().unwrap();
        topic_model.normalize().show_10().unwrap();

        std::fs::remove_dir_all(P).unwrap();
    }
}