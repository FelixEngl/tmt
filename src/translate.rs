use std::cmp::Ordering;
use std::error::Error;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::Arc;
use evalexpr::{Context, context_map, EmptyContextWithBuiltinFunctions};
use itertools::{Itertools};
use rayon::prelude::*;
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::toolkit::evalexpr::{CombineableContext, StaticContext};
use crate::topicmodel::topic_model::{BasicTopicModel, BasicTopicModelWithVocabulary, TopicModel, WordMeta};
use crate::topicmodel::dictionary::Dictionary;
use crate::topicmodel::dictionary::direction::{AToB, BToA};
use crate::translate::LanguageOrigin::{Origin, Target};
use crate::voting::{EmptyVotingMethod, Voting, VotingExpressionError, VotingMethod, VotingResult};

#[derive(Debug)]
struct TranslateConfig {
    epsilon: Option<f64>,
    voting: String,
    voting_limit: Option<NonZeroUsize>,
    threshold: Option<f64>,
    keep_original_word: KeepOriginalWord,
    top_candidate_limit: Option<NonZeroUsize>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Default)]
#[derive(AsRefStr, Display, EnumString)]
pub enum KeepOriginalWord {
    #[strum(serialize = "ALWAYS")]
    Always,
    #[strum(serialize = "IF_NO_TRANSLATION")]
    IfNoTranslation,
    #[strum(serialize = "NEVER")]
    #[default]
    Never
}

#[derive(Debug, Error)]
pub enum TranslateError<L> {
    #[error("The dictionary is not compatible with the topic model.")]
    InvalidDictionary(TopicModel<L>, Dictionary<L>),
    #[error(transparent)]
    VotingError(#[from] VotingExpressionError),
    #[error(transparent)]
    WithOrigin(#[from] TranslateErrorWithOrigin)
}

#[derive(Debug, Error)]
#[error("Failedwith an error!")]
pub struct TranslateErrorWithOrigin {
    topic_id: usize,
    word_id: usize,
    source: Box<dyn Error + Send + Sync>
}

trait MapsToTranslateErrorWithOrigin {
    type Return;
    fn originates_at(self, topic_id: usize, word_id: usize) -> Self::Return;
}

impl<T> MapsToTranslateErrorWithOrigin for VotingResult<T> {
    type Return = Result<T, TranslateErrorWithOrigin>;

    fn originates_at(self, topic_id: usize, word_id: usize) -> Self::Return {
        match self {
            Ok(value) => {
                Ok(value)
            }
            Err(err) => {
                Err(
                    TranslateErrorWithOrigin {
                        topic_id,
                        word_id,
                        source: err.into()
                    }
                )
            }
        }
    }
}

impl MapsToTranslateErrorWithOrigin for VotingExpressionError {
    type Return = TranslateErrorWithOrigin;

    fn originates_at(self, topic_id: usize, word_id: usize) -> Self::Return {
        TranslateErrorWithOrigin {
            topic_id,
            word_id,
            source: self.into()
        }
    }
}


/// Allows to differentiate the source of the object regarding a language
#[derive(Copy, Clone, Debug)]
enum LanguageOrigin<T> {
    Origin(T),
    Target(T)
}

impl<T> Deref for LanguageOrigin<T> {
    type Target = T;

    fn deref(&self) -> &<Self as Deref>::Target {
        match self {
            Origin(value) => {value}
            Target(value) => {value}
        }
    }
}



macro_rules! declare_variable_names {
    () => {};
    ($variable_name: ident: $name: literal, $($tt:tt)*) => {
        pub const $variable_name: &str = $name;
        declare_variable_names!($($tt)*);
    };

    (doc = $doc: literal $variable_name: ident: $name: literal, $($tt:tt)*) => {
        #[doc = $doc]
        pub const $variable_name: &str = $name;
        declare_variable_names!($($tt)*);
    };
}


declare_variable_names! {
    doc = "The epsilon of the calculation."
    EPSILON: "epsilon",
    doc = "The size of the vocabulary in language a."
    VOCABULARY_SIZE_A: "n_voc",
    doc = "The size of the vocabulary in language b."
    VOCABULARY_SIZE_B: "n_voc_target",
    doc = "The max probability of the topic."
    TOPIC_MAX_PROBABILITY: "topic_max",
    doc = "The min probability of the topic."
    TOPIC_MIN_PROBABILITY: "topic_min",
    doc = "The avg probability of the topic."
    TOPIC_AVG_PROBABILITY: "topic_avg",
    doc = "The sum of all probabilities of the topic."
    TOPIC_SUM_PROBABILITY: "topic_sum",
    doc = "The number of available voters"
    COUNT_OF_VOTERS: "ct_voters",
    doc = "The number of used voters."
    NUMBER_OF_VOTERS: "n_voters",
    doc = "True if the word in language A has translations to language B."
    HAS_TRANSLATION: "has_translation",
    doc = "True if this is the original word in language A"
    IS_ORIGIN_WORD: "is_origin_word",
    doc = "The original score of the candidate."
    SCORE_CANDIDATE: "score_candidate",
    doc = "The reciprocal rank of the word."
    RECIPROCAL_RANK: "rr",
    doc = "The rank of the word."
    RANK: "rank",
    doc = "The importance rank of the word."
    IMPORTANCE: "importance",
    doc = "The score of the word in the topic model."
    SCORE: "score",
}



fn translate_impl<T>(
    topic_model: TopicModel<T>,
    dictionary: Dictionary<T>,
    translate_config: TranslateConfig
) -> Result<TopicModel<T>, TranslateError<T>> {
    if topic_model.vocabulary().len() != dictionary.voc_a().len() {
        return Err(TranslateError::InvalidDictionary(topic_model, dictionary));
    }

    let epsilon = if let Some(value) = translate_config.epsilon {
        value
    } else {
        topic_model.topics().iter().flatten().fold(
            f64::MAX,
            |old, other| {
                old.min(*other)
            }
        ) - f64::EPSILON
    };

    let topic_context = context_map! {
        EPSILON => epsilon,
        VOCABULARY_SIZE_A => dictionary.voc_a().len() as i64,
        VOCABULARY_SIZE_B => dictionary.voc_b().len() as i64,
    }.unwrap().to_static_with(EmptyContextWithBuiltinFunctions);

    let topic_model = Arc::new(topic_model);
    let dictionary = Arc::new(dictionary);

    let voting = Arc::new(Voting::new(None, EmptyVotingMethod));

    // topic to word id to probable translation candidates.
    let result = topic_model.topics().par_iter()
        .zip_eq(topic_model.topic_metas())
        .enumerate()
        .map(|(topic_id, (topic, meta))| {
            let topic_context_2 = context_map! {
                TOPIC_MAX_PROBABILITY => meta.stats.max_value,
                TOPIC_MIN_PROBABILITY => meta.stats.min_value,
                TOPIC_AVG_PROBABILITY => meta.stats.average_value,
                TOPIC_SUM_PROBABILITY => meta.stats.sum_value,
            }.unwrap().to_static_with(topic_context.clone());

            translate_topic(
                voting.clone(),
                topic_model.clone(),
                dictionary.clone(),
                topic_id,
                topic,
                topic_context_2,
                &translate_config
            )
    }).collect::<Result<Vec<_>, _>>()?;

    todo!()
}

#[derive(Debug, Clone)]
struct Candidate<'a> {
    candidate_word_id: LanguageOrigin<usize>,
    relative_score: f64,
    voters: Vec<&'a Arc<WordMeta>>,
    origin_word_id: usize
}


impl<'a> Candidate<'a> {
    pub fn new(
        candidate_word_id: LanguageOrigin<usize>,
        relative_score: f64,
        voters: Vec<&'a Arc<WordMeta>>,
        origin_word_id: usize,
    ) -> Self {
        Self {
            candidate_word_id,
            relative_score,
            voters,
            origin_word_id
        }
    }
}

impl PartialEq<Self> for Candidate<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.relative_score == other.relative_score
    }
}

impl PartialOrd for Candidate<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        f64::partial_cmp(&other.relative_score, &self.relative_score)
    }
}

impl Eq for Candidate<'_> {}

impl Ord for Candidate<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        f64::total_cmp(&other.relative_score, &self.relative_score)
    }
}

fn translate_topic<T, A, B>(
    voting: Arc<Voting<impl VotingMethod + Sync + Send>>,
    topic_model: Arc<TopicModel<T>>,
    dictionary: Arc<Dictionary<T>>,
    topic_id: usize,
    topic: &Vec<f64>,
    topic_context: StaticContext<A, B>,
    config: &TranslateConfig
) -> Result<Vec<Option<Vec<Candidate>>>, TranslateErrorWithOrigin> where A: Context, B: Context {
    topic
        .par_iter()
        .enumerate()
        .map(|(original_word_id, probability)| {
            let mut candidates = if let Some(candidates) = dictionary.translate_id_to_ids::<AToB>(original_word_id) {
                Some(candidates.par_iter().cloned().filter_map( |candidate|
                    match dictionary.translate_id_to_ids::<BToA>(candidate) {
                        None  => None,
                        Some(voters) if voters.is_empty() => None,
                        Some(voters) => {
                            let mapped = voters
                                .iter()
                                .filter_map(|word_id_a_retrans| {
                                    topic_model.get_word_meta(topic_id, *word_id_a_retrans)
                                })
                                .collect::<Vec<_>>();

                            let mut context = context_map! {
                                COUNT_OF_VOTERS => mapped.len() as i64,
                                HAS_TRANSLATION => true,
                                IS_ORIGIN_WORD => false,
                                SCORE_CANDIDATE => *probability
                            }.unwrap();

                            let mut context = context.combine_with_mut(&topic_context);

                            let mut voters = mapped.iter().map(|value| context_map! {
                                RECIPROCAL_RANK => 1./ value.rank() as f64,
                                RANK => value.rank() as i64,
                                IMPORTANCE => value.importance_rank() as i64,
                                SCORE => value.probability,
                            }.unwrap()).collect_vec();
                            Some(
                                match voting.execute_to_f64(&mut context, voters.as_mut_slice()) {
                                    Ok(result) => {
                                        Ok(Candidate::new(Target(candidate), result, mapped, original_word_id))
                                    }
                                    Err(err) => {
                                        Err(err.originates_at(topic_id, original_word_id))
                                    }
                                }
                            )
                        }
                    }
                ).collect::<Result<Vec<Candidate>, TranslateErrorWithOrigin>>())
            } else {
                // Unknown
                None
            }.transpose();

            fn vote_for_origin<'a, A, B>(topic_model: &'a impl BasicTopicModel, topic_context: &StaticContext<A, B>, has_translation: bool, topic_id: usize, word_id: usize, probability: f64, voting: &Voting<impl VotingMethod + Sync + Send>) -> Result<Candidate<'a>, TranslateErrorWithOrigin> where A: Context, B: Context {
                let mut context = context_map! {
                    COUNT_OF_VOTERS => 1,
                    HAS_TRANSLATION => has_translation,
                    IS_ORIGIN_WORD => true,
                    SCORE_CANDIDATE => probability
                }.unwrap();

                let mut context = context.combine_with_mut(topic_context);

                let original_meta = topic_model.get_word_meta(topic_id, word_id).unwrap();

                let mut voters = vec![
                    context_map! {
                        RECIPROCAL_RANK => 1./ original_meta.rank() as f64,
                        RANK => original_meta.rank() as i64,
                        IMPORTANCE => original_meta.importance_rank() as i64,
                        SCORE => original_meta.probability,
                    }.unwrap()
                ];

                match voting.execute_to_f64(&mut context, voters.as_mut_slice()) {
                    Ok(result) => {
                        Ok(Candidate::new(Origin(word_id), result, vec![original_meta], word_id))
                    }
                    Err(err) => {
                        Err(err.originates_at(topic_id, word_id))
                    }
                }
            }

            let candidates = match config.keep_original_word {
                KeepOriginalWord::Always => {
                    if let Ok(Some(mut candidates)) = candidates {
                        match vote_for_origin(
                            topic_model.as_ref(),
                            &topic_context,
                            true,
                            topic_id,
                            original_word_id,
                            *probability,
                            &voting
                        ) {
                            Ok(value) => {
                                candidates.push(value);
                                Ok(Some(candidates))
                            }
                            Err(value) => {Err(value)}
                        }
                    } else {
                        match vote_for_origin(
                            topic_model.as_ref(),
                            &topic_context,
                            false,
                            topic_id,
                            original_word_id,
                            *probability,
                            &voting
                        ) {
                            Ok(value) => {
                                Ok(Some(vec![value]))
                            }
                            Err(value) => {
                                Err(value)
                            }
                        }
                    }
                }
                KeepOriginalWord::IfNoTranslation => {
                    if let Ok(None) = candidates {
                        match vote_for_origin(
                            topic_model.as_ref(),
                            &topic_context,
                            false,
                            topic_id,
                            original_word_id,
                            *probability,
                            &voting
                        ) {
                            Ok(value) => {
                                Ok(Some(vec![value]))
                            }
                            Err(value) => {
                                Err(value)
                            }
                        }
                    } else {
                        candidates
                    }
                }
                KeepOriginalWord::Never => {
                    candidates
                }
            };

            if let Some(top_candidate_limit) = config.top_candidate_limit {
                if let Ok(Some(mut candidates)) = candidates {
                    let top_candidate_limit = top_candidate_limit.get();
                    Ok(Some(
                        if top_candidate_limit < candidates.len() {
                            candidates.sort();
                            candidates.truncate(top_candidate_limit);
                            candidates
                        } else {
                            candidates
                        }
                    ))
                } else {
                    candidates
                }
            } else {
                candidates
            }
        }).collect()
}



#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let a = evalexpr::context_map! {
            "epsilon" => 0.7
        }.unwrap();

        let mut b = evalexpr::context_map! {
            "katze" => 1
        }.unwrap();

        println!("{b:?}")
    }
}

