use std::num::NonZeroUsize;
use std::sync::Arc;
use evalexpr::{Context, context_map, EmptyContext, EmptyContextWithBuiltinFunctions, EvalexprError, HashMapContext, Value};
use itertools::{Itertools};
use nom::combinator::map;
use rayon::prelude::*;
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::toolkit::evalexpr::{CombineableContext, CombinedContextWrapper, CombinedContextWrapperMut, StaticContext};
use crate::topicmodel::topic_model::{TopicModel, WordMeta};
use crate::topicmodel::dictionary::Dictionary;
use crate::topicmodel::dictionary::direction::{AToB, BToA};
use crate::translate::AOrB::{A, B};
use crate::voting::{EmptyVotingMethod, Voting, VotingMethod, VotingResult};

#[derive(Debug)]
struct TranslateConfig {
    epsilon: Option<f64>,
    voting: String,
    voting_limit: Option<NonZeroUsize>,
    threshold: Option<f64>,
    keep_original_word: KeepOriginalWord
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
pub enum TranslateError<T> {
    #[error("The dictionary is not compatible with the topic model.")]
    InvalidDictionary(TopicModel<T>, Dictionary<T>),
    #[error(transparent)]
    EvalExpressionError(#[from] EvalexprError),
}

macro_rules! declare_variable_names {
    ($($variable_name: ident: $name: literal),+) => {
        $(
           pub const $variable_name: &str = $name;
        )+
    };
}


declare_variable_names! {
    EPSILON: "epsilon",
    VOCABULARY_SIZE_A: "n_voc",
    VOCABULARY_SIZE_B: "n_voc_target",
    TOPIC_MAX_PROBABILITY: "topic_max",
    TOPIC_MIN_PROBABILITY: "topic_min",
    TOPIC_AVG_PROBABILITY: "topic_avg",
    TOPIC_SUM_PROBABILITY: "topic_sum",
    COUNT_OF_VOTERS: "ct_voters",
    NUMBER_OF_VOTERS: "n_voters",
    HAS_VOTERS: "hasVoters",
    HAS_TRANSLATION: "has_translation",
    IS_ORIGIN_WORD: "is_origin_word",
    SCORE_CANDIDATE: "score_candidate",
    RECIPROCAL_RANK: "rr",
    RANK: "rank",
    IMPORTANCE: "importance",
    SCORE: "score"
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

    let result = topic_model.topics().iter()
        .zip_eq(topic_model.topic_metas())
        .enumerate()
        .collect_vec()
        .par_iter()
        .copied()
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
    }).collect::<Vec<_>>();

    todo!()
}


struct Candidate<'a> {
    candidate_word_id: AOrB<usize>,
    voting_result: Option<VotingResult<Value>>,
    voters: Option<Vec<&'a Arc<WordMeta>>>
}

#[derive(Copy, Clone, Debug)]
enum AOrB<T> {
    A(T),
    B(T)
}

impl<'a> Candidate<'a> {
    pub fn new(candidate_word_id: AOrB<usize>, voting_result: VotingResult<Value>, voters: Vec<&'a Arc<WordMeta>>) -> Self {
        Self {
            candidate_word_id,
            voting_result: Some(voting_result),
            voters: (!voters.is_empty()).then_some(voters)
        }
    }

    pub fn new_without_voters(candidate_word_id: AOrB<usize>) -> Candidate<'a> {
        Self {
            candidate_word_id,
            voting_result: None,
            voters: None
        }
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
) -> Result<(), TranslateError<T>> where A: Context, B: Context {
    let candidates: Vec<_> = topic
        .iter()
        .cloned()
        .enumerate()
        .collect_vec()
        .par_iter()
        .map(|(original, probability)| {
            let mut candidates = if let Some(candidates) = dictionary.translate_id_to_ids::<AToB>(*original) {
                Some(candidates.iter().cloned().map( |candidate|
                    match dictionary.translate_id_to_ids::<BToA>(candidate) {
                        None  => Candidate::new_without_voters(B(candidate)),
                        Some(voters) if voters.is_empty() => Candidate::new_without_voters(B(candidate)),
                        Some(voters) => {
                            let mapped = voters
                                .iter()
                                .filter_map(|word_id_a_retrans| {
                                    topic_model.get_word_meta(topic_id, *word_id_a_retrans)
                                })
                                .collect::<Vec<_>>();

                            let mut context = context_map! {
                                COUNT_OF_VOTERS => mapped.len() as i64,
                                HAS_VOTERS => !mapped.is_empty(),
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
                            Candidate::new(B(candidate), voting.execute(&mut context, voters.as_mut_slice()), mapped)
                        }
                    }
                ).collect_vec())
            } else {
                /// Unknown
                None
            };

            let candidates = match config.keep_original_word {
                KeepOriginalWord::Always => {
                    if let Some(mut candidates) = candidates {
                        let mut context = context_map! {
                            COUNT_OF_VOTERS => 1,
                            HAS_VOTERS => false,
                            HAS_TRANSLATION => true,
                            IS_ORIGIN_WORD => true,
                            SCORE_CANDIDATE => *probability
                        }.unwrap();

                        let mut context = context.combine_with_mut(&topic_context);

                        let original_meta = topic_model.get_word_meta(topic_id, *original).unwrap();

                        let mut voters = vec![
                            context_map! {
                                RECIPROCAL_RANK => 1./ original_meta.rank() as f64,
                                RANK => original_meta.rank() as i64,
                                IMPORTANCE => original_meta.importance_rank() as i64,
                                SCORE => original_meta.probability,
                            }.unwrap()
                        ];

                        candidates.push(Candidate::new(A(*original), voting.execute(&mut context, voters.as_mut_slice()), vec![original_meta]));

                        Some(candidates)
                    } else {
                        let mut context = context_map! {
                            COUNT_OF_VOTERS => 1,
                            HAS_VOTERS => false,
                            HAS_TRANSLATION => false,
                            IS_ORIGIN_WORD => true,
                            SCORE_CANDIDATE => *probability
                        }.unwrap();

                        let mut context = context.combine_with_mut(&topic_context);

                        let original_meta = topic_model.get_word_meta(topic_id, *original).unwrap();

                        let mut voters = vec![
                            context_map! {
                                RECIPROCAL_RANK => 1./ original_meta.rank() as f64,
                                RANK => original_meta.rank() as i64,
                                IMPORTANCE => original_meta.importance_rank() as i64,
                                SCORE => original_meta.probability,
                            }.unwrap()
                        ];

                        Some(
                            vec![
                                Candidate::new(A(*original), voting.execute(&mut context, voters.as_mut_slice()), vec![original_meta])
                            ]
                        )
                    }
                }
                KeepOriginalWord::IfNoTranslation => {
                    if let Some(candidates) = candidates {
                        Some(candidates)
                    } else {
                        let mut context = context_map! {
                                COUNT_OF_VOTERS => 1,
                                HAS_VOTERS => false,
                                HAS_TRANSLATION => false,
                                IS_ORIGIN_WORD => true,
                                SCORE_CANDIDATE => *probability
                            }.unwrap();

                        let mut context = context.combine_with_mut(&topic_context);

                        let original_meta = topic_model.get_word_meta(topic_id, *original).unwrap();

                        let mut voters = vec![
                            context_map! {
                                RECIPROCAL_RANK => 1./ original_meta.rank() as f64,
                                RANK => original_meta.rank() as i64,
                                IMPORTANCE => original_meta.importance_rank() as i64,
                                SCORE => original_meta.probability,
                            }.unwrap()
                        ];

                        Some(
                            vec![
                                Candidate::new(A(*original), voting.execute(&mut context, voters.as_mut_slice()), vec![original_meta])
                            ]
                        )
                    }
                }
                KeepOriginalWord::Never => {
                    candidates
                }
            };

            if let Some(candidates) = candidates {
                
            } else {
                None
            }
        }).collect();

    todo!()
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

