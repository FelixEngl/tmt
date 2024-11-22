use std::borrow::Borrow;
use std::fmt::Display;
use std::hash::Hash;
use std::io;
use std::io::{ErrorKind, Write};
use std::ops::Range;
use std::sync::Arc;
use crate::topicmodel::model::{DocumentId, DocumentLength, DocumentTo, Probability, TopicId, TopicTo, WordFrequency, WordId, WordTo};
use crate::topicmodel::model::meta::{TopicMeta, WordMeta, WordMetaWithWord};
use crate::topicmodel::vocabulary::BasicVocabulary;

/// A basic topic model fulfilling the bare minimum of a topic model.
pub trait BasicTopicModel: Send + Sync {
    /// The number of topics in this model
    fn topic_count(&self) -> usize;

    /// The number of topics in this model
    #[inline]
    fn k(&self) -> usize {
        self.topic_count()
    }

    /// The size of the vocabulary for this model.
    fn vocabulary_size(&self) -> usize;

    /// A range over all topicIds
    fn topic_ids(&self) -> Range<TopicId>;

    /// Returns true if the `topic_id` is contained in self
    fn contains_topic_id(&self, topic_id: TopicId) -> bool;

    /// A range over all vocabulary ids
    fn word_ids(&self) -> Range<WordId>;

    /// Returns true if the `word_id` is contained in self
    fn contains_word_id(&self, word_id: WordId) -> bool;

    /// Returns the topics
    fn topics(&self) -> &TopicTo<WordTo<Probability>>;

    /// Get the topic for `topic_id`
    fn get_topic(&self, topic_id: TopicId) -> Option<&WordTo<Probability>>;

    /// The meta of the topic
    fn topic_metas(&self) -> &TopicTo<TopicMeta>;

    /// Get the `TopicMeta` for `topic_id`
    fn get_topic_meta(&self, topic_id: TopicId) -> Option<&TopicMeta>;

    /// Get the word freuencies for each word.
    fn used_vocab_frequency(&self) -> &WordTo<WordFrequency>;

    /// Get the probability of `word_id` of `topic_id`
    fn get_probability(&self, topic_id: TopicId, word_id: WordId) -> Option<&Probability>;

    /// Get all probabilities of `word_id`
    fn get_topic_probabilities_for(&self, word_id: WordId) -> Option<TopicTo<Probability>>;

    /// Get the [WordMeta] of `word_id` of `topic_id`
    fn get_word_meta(&self, topic_id: TopicId, word_id: WordId) -> Option<&Arc<WordMeta>>;

    /// Get all [WordMeta] for `word_id`
    fn get_word_metas_for(&self, word_id: WordId) -> Option<TopicTo<&Arc<WordMeta>>>;

    /// Get all [WordMeta] values with a similar importance in `topic_id` than `word_id`.
    /// (including the `word_id`)
    fn get_all_similar_important(&self, topic_id: TopicId, word_id: WordId) -> Option<&Vec<Arc<WordMeta>>>;

    fn get_words_for_topic_sorted(&self, topic_id: TopicId) -> Option<&[Arc<WordMeta>]>;

    /// Get the `n` best [WordMeta] in `topic_id` by their position.
    fn get_n_best_for_topic(&self, topic_id: TopicId, n: usize) -> Option<&[Arc<WordMeta>]>;

    /// Get the `n` best [WordMeta] for all topics by their position.
    fn get_n_best_for_topics(&self, n: usize) -> Option<TopicTo<&[Arc<WordMeta>]>>;
}

/// A topicmodel with document stats
pub trait TopicModelWithDocumentStats {
    /// Returns the number of documents
    fn document_count(&self) -> usize;

    /// Returns all document ids
    fn document_ids(&self) -> Range<DocumentId>;

    /// Returns the topic distributions of the topic model
    fn doc_topic_distributions(&self) -> &DocumentTo<TopicTo<Probability>>;

    /// Returns the document lengths of the documents
    fn document_lengths(&self) -> &DocumentTo<DocumentLength>;
}

/// A basic topic model with a vocabulary
pub trait BasicTopicModelWithVocabulary<T, Voc>: BasicTopicModel where Voc: BasicVocabulary<T> {
    /// The vocabulary
    fn vocabulary(&self) -> &Voc;

    /// Get the word for the `word_id`
    #[inline]
    fn get_word<'a>(&'a self, word_id: WordId) -> Option<&'a T> where Voc: 'a {
        self.vocabulary().get_value_by_id(word_id)
    }

    /// Get the [WordMetaWithWord] of `word_id` of `topic_id`
    fn get_word_meta_with_word<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<WordMetaWithWord<'a, T>>  where Voc: 'a {
        let topic_meta = self.get_topic_meta(topic_id)?;
        let word_meta = topic_meta.by_words.get(word_id)?;
        let word = self.vocabulary().get_value_by_id(word_meta.word_id)?;
        Some(WordMetaWithWord::new(word, word_meta))
    }

    /// Get the [WordMetaWithWord] of `word_id` for all topics.
    fn get_word_metas_with_word<'a>(&'a self, word_id: usize) -> Option<TopicTo<WordMetaWithWord<'a, T>>> where Voc: 'a {
        self.topic_ids().map(|topic_id| self.get_word_meta_with_word(topic_id, word_id)).collect()
    }

    /// Get all [WordMetaWithWord] values with a similar importance in `topic_id` than `word_id`.
    /// (including the `word_id`)
    fn get_all_similar_important_with_word_for<'a>(&'a self, topic_id: usize, word_id: usize) -> Option<Vec<WordMetaWithWord<'a, T>>> where Voc: 'a {
        Some(
            self.get_all_similar_important(topic_id, word_id)?
                .iter()
                .map(|value| WordMetaWithWord::new(self.vocabulary().get_value_by_id(value.word_id).unwrap(), value))
                .collect()
        )
    }
}

/// A topic model with an explicit vocabulary
pub trait TopicModelWithVocabulary<T, Voc>: BasicTopicModelWithVocabulary<T, Voc> where Voc: BasicVocabulary<T> {
    fn get_id<Q: ?Sized>(&self, word: &Q) -> Option<WordId> where T: Borrow<Q>, Q: Hash + Eq;
    fn contains<Q: ?Sized>(&self, word: &Q) -> bool where T: Borrow<Q>, Q: Hash + Eq;

    /// Get the probability of `word` of `topic_id`
    #[inline]
    fn get_probability_by_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<&Probability> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_probability(topic_id, self.get_id(word)?)
    }

    /// Get all probabilities of `word`
    #[inline]
    fn get_topic_probabilities_for_by_word<Q: ?Sized>(&self, word: &Q) -> Option<TopicTo<Probability>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_topic_probabilities_for(self.get_id(word)?)
    }

    /// Get the [WordMeta] of `word` of `topic_id`
    #[inline]
    fn get_word_meta_by_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<&Arc<WordMeta>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_word_meta(topic_id, self.get_id(word)?)
    }

    /// Get the [WordMetaWithWord] of `word` for all topics.
    #[inline]
    fn get_word_metas_with_word_by_word<'a, Q: ?Sized>(&'a self, word: &Q) -> Option<TopicTo<WordMetaWithWord<'a, T>>> where T: Borrow<Q>, Q: Hash + Eq, Voc: 'a {
        self.get_word_metas_with_word(self.get_id(word)?)
    }

    /// Get all [WordMeta] values with a similar importance in `topic_id` than `word`.
    /// (including the `word_id`)
    #[inline]
    fn get_all_similar_important_words_for_word<Q: ?Sized>(&self, topic_id: TopicId, word: &Q) -> Option<&Vec<Arc<WordMeta>>> where T: Borrow<Q>, Q: Hash + Eq {
        self.get_all_similar_important(topic_id, self.get_id(word)?)
    }

    /// Returns true iff the topic models seem similar.
    fn seems_equal_to<Q, VOther>(&self, other: &impl TopicModelWithVocabulary<Q, VOther>) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + Borrow<T>,
        VOther: BasicVocabulary<Q>
    ;
}


pub trait FullTopicModel<T, Voc>: BasicTopicModelWithVocabulary<T, Voc>
where
    Voc: BasicVocabulary<T>
{
    /// Create a new topic model
    fn new(
        topics: TopicTo<WordTo<Probability>>,
        vocabulary: Voc,
        used_vocab_frequency: WordTo<WordFrequency>,
        doc_topic_distributions: DocumentTo<TopicTo<Probability>>,
        document_lengths: DocumentTo<DocumentLength>,
    ) -> Self where Self: Sized;

    /// Normalizes the topic model in place.
    fn normalize_in_place(&mut self);
}

/// A topic model that allows basic show methods
pub trait DisplayableTopicModel<T, Voc>: BasicTopicModelWithVocabulary<T, Voc> where T: Display, Voc: BasicVocabulary<T> + Display {
    fn show_to(&self, n: usize, out: &mut impl Write) -> io::Result<()> {
        for (topic_id, topic_entries) in self.get_n_best_for_topics(n).ok_or(io::Error::from(ErrorKind::Other))?.iter().enumerate() {
            if topic_id != 0 {
                out.write(b"\n")?;
            }
            write!(out, "Topic({topic_id}):")?;
            for it in topic_entries.iter() {
                out.write(b"\n")?;
                write!(out, "    {}: {} ({})", self.get_word(it.word_id).unwrap(), it.probability, it.rank())?;
            }
        }
        Ok(())
    }

    fn show(&self, n: usize) -> io::Result<()> {
        let mut str = Vec::new();
        self.show_to(n, &mut str)?;
        println!("{}", String::from_utf8(str).unwrap());
        Ok(())
    }

    fn show_10(&self) -> io::Result<()>{
        self.show(10)
    }
}

impl<TopicModel, T, Voc> DisplayableTopicModel<T, Voc> for TopicModel
where TopicModel: BasicTopicModelWithVocabulary<T, Voc>,
      T:Display, Voc: BasicVocabulary<T> + Display
{}
