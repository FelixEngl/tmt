use std::num::NonZeroUsize;
use std::sync::Arc;
use evalexpr::{context_map, EmptyContext, EmptyContextWithBuiltinFunctions, EvalexprError, HashMapContext};
use itertools::{Itertools};
use nom::combinator::map;
use rayon::prelude::*;
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::toolkit::evalexpr::{CombineableContext, CombinedContextWrapper, CombinedContextWrapperMut};
use crate::topicmodel::topic_model::{TopicModel, WordMeta};
use crate::topicmodel::dictionary::Dictionary;
use crate::topicmodel::dictionary::direction::{AToB, BToA};
use crate::voting::{Voting, VotingMethod};

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
        "epsilon" => epsilon,
        "n_voc" => dictionary.voc_a().len() as i64,
        "n_voc_target" => dictionary.voc_b().len() as i64,
    }.unwrap();

    let topic_model = Arc::new(topic_model);
    let dictionary = Arc::new(dictionary);

    let voting = Arc::new(todo!());

    let result = topic_model.topics().iter()
        .zip_eq(topic_model.topic_metas())
        .enumerate()
        .collect_vec()
        .par_iter()
        .copied()
        .map(|(topic_id, (topic, meta))| {
            let topic_context_2 = context_map! {
                "topic_max" => meta.stats.max_value,
                "topic_min" => meta.stats.min_value,
                "topic_avg" => meta.stats.average_value,
                "topic_sum" => meta.stats.sum_value,
            }.unwrap();

            translate_topic(
                voting.clone(),
                topic_model.clone(),
                dictionary.clone(),
                topic_id,
                topic,
                topic_context_2.combine_with(&topic_context),
                &translate_config
            )
    }).collect::<Vec<_>>();

    todo!()
}


struct Candidate<'a> {
    candidate_word_id: usize,
    voters: Option<Vec<&'a Arc<WordMeta>>>
}

impl<'a> Candidate<'a> {
    pub fn new(candidate_word_id: usize, voters: Vec<&'a Arc<WordMeta>>) -> Self {
        Self {
            candidate_word_id,
            voters: (!voters.is_empty()).then_some(voters)
        }
    }

    pub fn new_without_voters(candidate_word_id: usize) -> Candidate<'a> {
        Self {
            candidate_word_id,
            voters: None
        }
    }
}

fn translate_topic<T, A: ?Sized, B: ?Sized>(
    voting: Arc<Voting<impl VotingMethod>>,
    topic_model: Arc<TopicModel<T>>,
    dictionary: Arc<Dictionary<T>>,
    topic_id: usize,
    topic: &Vec<f64>,
    topic_context: CombinedContextWrapper<A, B>,
    config: &TranslateConfig
) -> Result<(), TranslateError<T>> {
    let x = topic
        .iter()
        .cloned()
        .enumerate()
        .collect_vec()
        .par_iter()
        .map(|(original, probability)| {
            let value = if let Some(candidates) = dictionary.translate_id_to_ids::<AToB>(*original) {
                Some(candidates.iter().cloned().map( |candidate|
                    match dictionary.translate_id_to_ids::<BToA>(candidate) {
                        None  => Candidate::new_without_voters(candidate),
                        Some(voters) if voters.is_empty() => Candidate::new_without_voters(candidate),
                        Some(voters) => {
                            let mapped = voters
                                .iter()
                                .filter_map(|word_id_a_retrans| {
                                    topic_model.get_word_meta(topic_id, *word_id_a_retrans)
                                })
                                .collect::<Vec<_>>();

                            let context = context_map! {
                                "n_voters" => mapped.len(),
                                "hasVoters" => !mapped.is_empty(),
                                "has_translation" => true,
                                "is_origin_word" => false,
                            }.unwrap();

                            let context = context.combine_with(&topic_context);

                            let mut voters = mapped.iter().map(|value| context_map! {
                                "rr" => 1./ value.rank() as f64,
                                "rank" => value.rank(),
                                "importance" => value.importance_rank(),
                                "score" => value.probability,
                            }.unwrap()).collect_vec();
                            let mut context =(&context).combine_with_mut(&mut EmptyContextWithBuiltinFunctions);
                            voting.execute(&mut context, voters.as_mut_slice());

                            Candidate::new(candidate, mapped)
                        }
                    }
                ).collect_vec())
            } else {
                /// Unknown
                None
            };

            match config.keep_original_word {
                KeepOriginalWord::Always => {

                }
                KeepOriginalWord::IfNoTranslation => {

                }
                KeepOriginalWord::Never => {

                }
            }
        });
    //
    // for (word_id, probability) in topic.iter().enumerate() {
    //     let mut word_bound_context = context.clone();
    //     word_bound_context.set_value_direct("hasTranslation", dictionary.can_translate_id(word_id))?;
    //
    // }




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

