//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use crate::toolkit::evalexpr::CombineableContext;
use crate::topicmodel::create_topic_model_specific_dictionary;
use crate::topicmodel::dictionary::direction::{AToB, BToA, B};
use crate::topicmodel::dictionary::{DictionaryMut, DictionaryWithVocabulary, FromVoc};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::model::{BasicTopicModel, FullTopicModel, TopicModel, TopicModelWithDocumentStats, TopicModelWithVocabulary};
use crate::topicmodel::vocabulary::{BasicVocabulary, MappableVocabulary, SearchableVocabulary, Vocabulary, VocabularyMut};
use crate::translate::errors::MapsToTranslateErrorWithOrigin;
use crate::translate::language::LanguageOrigin;
use crate::translate::phantoms::DummyAsVariableProvider;
use crate::translate::TranslateError::IncompatibleLanguages;
use crate::translate::{KeepOriginalWord, TranslateConfig, TranslateError, TranslateErrorWithOrigin};
use crate::variable_provider::variable_names::*;
use crate::variable_provider::{AsVariableProvider, VariableProvider, VariableProviderOut};
use crate::voting::traits::VotingMethodMarker;
use crate::voting::VotingMethod;
use evalexpr::{context_map, Context, ContextWithMutableVariables, EmptyContextWithBuiltinFunctions, HashMapContext, IterateVariablesContext};
use itertools::Itertools;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use crate::topicmodel::dictionary::*;

#[allow(dead_code)]
pub fn translate_topic_model_without_provider<'a, Model, D, T, Voc, V>(
    topic_model: &'a Model,
    dictionary: &'a D,
    translate_config: &TranslateConfig<V>,
) -> Result<TopicModel<T, Vocabulary<T>>, TranslateError<'a>> where
    T: Hash + Eq + Ord + Clone,
    V: VotingMethodMarker,
    Voc: VocabularyMut<T> + MappableVocabulary<T> + Clone + 'a,
    D: DictionaryWithVocabulary<T, Voc> + DictionaryMut<T, Voc> + FromVoc<T, Voc>,
    Model: TopicModelWithVocabulary<T, Voc> + TopicModelWithDocumentStats,
{
    translate_topic_model(
        topic_model,
        dictionary,
        translate_config,
        None::<&DummyAsVariableProvider<T>>
    )
}


pub(crate) fn translate_topic_model<'a, Model, D, T, Voc, V, P>(
    topic_model: &'a Model,
    dictionary: &'a D,
    translate_config: &TranslateConfig<V>,
    provider: Option<&P>
) -> Result<TopicModel<T, Vocabulary<T>>, TranslateError<'a>> where
    T: Hash + Eq + Ord + Clone,
    V: VotingMethodMarker,
    Voc: VocabularyMut<T> + MappableVocabulary<T> + Clone + 'a,
    D: DictionaryWithVocabulary<T, Voc> + DictionaryMut<T, Voc> + FromVoc<T, Voc>,
    Model: TopicModelWithVocabulary<T, Voc> + TopicModelWithDocumentStats,
    P: AsVariableProvider<T>
{

    if let Some(lang_model) = topic_model.vocabulary().language() {
        if let (Some(lang_a), lang_b) = dictionary.language_direction_a_to_b() {
            if lang_model != lang_a {
                let lang_b = lang_b.cloned().unwrap_or_else(|| LanguageHint::new("###"));
                return Err(
                    IncompatibleLanguages {
                        lang_a,
                        lang_b,
                        lang_model
                    }
                )
            }
        }
    }

    let dictionary: D = create_topic_model_specific_dictionary::<D, D, T, Voc, Voc>(
        dictionary,
        topic_model.vocabulary()
    );

    // TODO: make clean for rust.
    let provider = if let Some(provider) = provider {
        Some(provider.as_variable_provider_for(topic_model, &dictionary))
    } else {
        None
    }.transpose()?;

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

    let mut topic_context = context_map! {
        EPSILON => epsilon,
        VOCABULARY_SIZE_A => dictionary.voc_a().len() as i64,
        VOCABULARY_SIZE_B => dictionary.voc_b().len() as i64,
    }.unwrap();

    if let Some(ref provider) = provider {
        provider.provide_global(&mut topic_context)?;
    }

    let topic_context = topic_context
        .to_static_with(EmptyContextWithBuiltinFunctions);



    // topic to word id to probable translation candidates.
    let result = topic_model
        .topics()
        .par_iter()
        .zip_eq(topic_model.topic_metas())
        .enumerate()
        .map(|(topic_id, (topic, meta))| {
            let mut topic_context_2 = context_map! {
                TOPIC_MAX_PROBABILITY => meta.stats.max_value,
                TOPIC_MIN_PROBABILITY => meta.stats.min_value,
                TOPIC_AVG_PROBABILITY => meta.stats.average_value,
                TOPIC_SUM_PROBABILITY => meta.stats.sum_value,
                TOPIC_ID => topic_id as i64
            }.unwrap();

            if let Some(provider) = provider.as_ref() {
                match provider.provide_for_topic(topic_id, &mut topic_context_2) {
                    Ok(_) => {
                        let topic_context_2 = topic_context_2
                            .to_static_with(topic_context.clone());

                        translate_topic(
                            topic_model,
                            &dictionary,
                            topic_id,
                            topic,
                            topic_context_2,
                            &translate_config,
                            Some(provider)
                        ).map_err(TranslateError::WithOrigin)
                    }
                    Err(err) => {
                        Err(err.into())
                    }
                }
            } else {
                let topic_context_2 = topic_context_2
                    .to_static_with(topic_context.clone());

                translate_topic(
                    topic_model,
                    &dictionary,
                    topic_id,
                    topic,
                    topic_context_2,
                    &translate_config,
                    None::<&VariableProvider>
                ).map_err(TranslateError::WithOrigin)
            }


        }).collect::<Result<Vec<_>, _>>()?;

    let voc_b_col = result.par_iter().flatten().map(|value| {
        match value.candidate_word_id {
            LanguageOrigin::Origin(word_id) => {
                dictionary.voc_a().get_value(word_id).unwrap()
            }
            LanguageOrigin::Target(word_id) => {
                dictionary.voc_b().get_value(word_id).unwrap()
            }
        }
    }).collect_vec_list();


    let mut voc_b = voc_b_col.iter().flatten().cloned().collect::<Vocabulary<_>>();
    voc_b.set_language(dictionary.language::<B>().cloned());

    let mut counts = vec![0u64; voc_b.len()];

    for value in voc_b_col.into_iter().flatten().map(|value| voc_b.get_id(value).unwrap()) {
        unsafe {
            *counts.get_unchecked_mut(value) += 1;
        }
    }

    let inner_topic_model = result.into_par_iter().map(|topic_content| {
        let mut topic = topic_content.into_par_iter().map(|candidate| {
            let word = match candidate.candidate_word_id {
                LanguageOrigin::Origin(word_id) => {
                    dictionary.voc_a().get_value(word_id).unwrap()
                }
                LanguageOrigin::Target(word_id) => {
                    dictionary.voc_b().get_value(word_id).unwrap()
                }
            };
            (voc_b.get_id(word).unwrap(), candidate.relative_score)
        }).collect::<HashMap<_, _>>();

        voc_b.ids().for_each(|value| {
            match topic.entry(value) {
                Entry::Vacant(entry) => {
                    entry.insert(epsilon);
                }
                _ => {}
            }
        });
        assert!(voc_b.ids().all(|it| topic.contains_key(&it)));
        topic.into_iter().sorted_unstable_by_key(|value| value.0).map(|(_, b)| b).collect_vec()
    }).collect::<Vec<_>>();

    let mut translated = TopicModel::new(
        inner_topic_model,
        voc_b,
        counts,
        topic_model.doc_topic_distributions().clone(),
        topic_model.document_lengths().clone()
    );

    translated.normalize_in_place();

    Ok(translated)
}

#[derive(Debug, Clone)]
struct Candidate {
    candidate_word_id: LanguageOrigin<usize>,
    relative_score: f64,
    _origin_word_id: usize
}


impl Candidate {
    pub fn new(
        candidate_word_id: LanguageOrigin<usize>,
        relative_score: f64,
        _origin_word_id: usize,
    ) -> Self {
        Self {
            candidate_word_id,
            relative_score,
            _origin_word_id
        }
    }
}

impl PartialEq<Self> for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.relative_score == other.relative_score
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        f64::partial_cmp(&other.relative_score, &self.relative_score)
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        f64::total_cmp(&other.relative_score, &self.relative_score)
    }
}


fn translate_topic<Model, T, V, Voc, P>(
    topic_model: &Model,
    dictionary: &impl DictionaryWithVocabulary<T, Voc>,
    topic_id: usize,
    topic: &Vec<f64>,
    topic_context: impl Context + Send + Sync + IterateVariablesContext,
    config: &TranslateConfig<V>,
    provider: Option<&P>
) -> Result<Vec<Candidate>, TranslateErrorWithOrigin>
where V: VotingMethodMarker,
      Voc: BasicVocabulary<T>,
      Model: TopicModelWithVocabulary<T, Voc> + TopicModelWithDocumentStats,
      P: VariableProviderOut
{
    topic
        .par_iter()
        .enumerate()
        .filter_map(|(original_word_id, probability)| {

            if let Some(provider) = provider {
                let mut context = HashMapContext::new();
                match provider.provide_for_word_a(original_word_id, &mut context) {
                    Ok(_) => {
                        match provider.provide_for_word_a(original_word_id, &mut context) {
                            Ok(_) => {
                                let combined = topic_context.combine_with(&context);
                                translate_single_candidate(
                                    topic_model,
                                    dictionary,
                                    topic_id,
                                    &combined,
                                    config,
                                    original_word_id,
                                    *probability,
                                    Some(provider)
                                )
                            }
                            Err(err) => {Some(Err(TranslateErrorWithOrigin {
                                topic_id,
                                word_id: original_word_id,
                                source: err.into()
                            }))}
                        }
                    }
                    Err(err) => {Some(Err(TranslateErrorWithOrigin {
                        topic_id,
                        word_id: original_word_id,
                        source: err.into()
                    }))}
                }
            } else {
                translate_single_candidate(
                    topic_model,
                    dictionary,
                    topic_id,
                    &topic_context,
                    config,
                    original_word_id,
                    *probability,
                    provider
                )
            }
        }).collect::<Result<Vec<_>, _>>().map(|value| {
        value.into_iter().flatten().collect::<Vec<_>>()
    })
}

#[inline(always)]
fn translate_single_candidate<Model, T, V, Voc, P>(
    topic_model: &Model,
    dictionary: &impl DictionaryWithVocabulary<T, Voc>,
    topic_id: usize,
    topic_context: &(impl Context + Send + Sync + IterateVariablesContext),
    config: &TranslateConfig<V>,
    original_word_id: usize,
    probability: f64,
    provider: Option<&P>
) -> Option<Result<Vec<Candidate>, TranslateErrorWithOrigin>>
where V: VotingMethodMarker,
      Voc: BasicVocabulary<T> ,
      Model: TopicModelWithVocabulary<T, Voc> + TopicModelWithDocumentStats,
      P: VariableProviderOut
{
    let candidates = if let Some(candidates) = dictionary.translate_id_to_ids::<AToB>(original_word_id) {
        Some(candidates.par_iter().cloned().filter_map( |candidate|
            match dictionary.translate_id_to_ids::<BToA>(candidate) {
                None  => None,
                Some(voters) if voters.is_empty() => None,
                Some(voters) => {
                    let mapped = voters
                        .iter()
                        .filter_map(|word_id_a_retrans| {
                            topic_model.get_word_meta(topic_id, *word_id_a_retrans)
                        });

                    let mapped = if let Some(threshold) = config.threshold {
                        mapped.filter(|value| value.probability >= threshold).collect_vec()
                    } else {
                        mapped.collect_vec()
                    };


                    let mut context = context_map! {
                        COUNT_OF_VOTERS => mapped.len() as i64,
                        HAS_TRANSLATION => true,
                        IS_ORIGIN_WORD => false,
                        SCORE_CANDIDATE => probability,
                        CANDIDATE_ID => candidate as i64
                    }.unwrap();

                    let mut context = context.combine_with_mut(topic_context);

                    let voters = mapped
                        .iter()
                        .map(|value| {
                            let mut m = context_map! {
                                RECIPROCAL_RANK => 1./ value.importance_rank() as f64,
                                REAL_RECIPROCAL_RANK => 1./ value.rank() as f64,
                                RANK => value.rank() as i64,
                                IMPORTANCE => value.importance_rank() as i64,
                                SCORE => value.probability,
                                VOTER_ID => value.word_id as i64
                            }.unwrap();
                            if let Some(provider) = provider {
                                match provider.provide_for_word_a(value.word_id, &mut m) {
                                    Ok(_) => {
                                        match provider.provide_for_word_in_topic_a(topic_id, value.word_id, &mut m) {
                                            Ok(_) => {
                                                Ok(m)
                                            }
                                            Err(err) => {Err(err)}
                                        }
                                    }
                                    Err(err) => {
                                        Err(err)
                                    }
                                }
                            } else {
                                Ok(m)
                            }
                        })
                        .collect::<Result<Vec<_>, _>>();

                    Some(
                        match voters {
                            Ok(mut voters) => {
                                context.set_value(NUMBER_OF_VOTERS.to_string(), (voters.len() as i64).into()).expect("This should not fail!");
                                match config.voting.execute_to_f64(&mut context, voters.as_mut_slice()) {
                                    Ok(result) => {
                                        Ok(Candidate::new(LanguageOrigin::Target(candidate), result, original_word_id))
                                    }
                                    Err(err) => {
                                        Err(err.originates_at(topic_id, original_word_id))
                                    }
                                }
                            }
                            Err(err) => {
                                Err(TranslateErrorWithOrigin {
                                    topic_id,
                                    word_id: original_word_id,
                                    source: err.into()
                                })
                            }
                        }

                    )
                }
            }
        ).collect::<Result<Vec<Candidate>, TranslateErrorWithOrigin>>())
    } else {
        // Unknown
        None
    };


    fn vote_for_origin<'a>(
        topic_model: &'a impl BasicTopicModel,
        topic_context: &(impl Context + Send + Sync + IterateVariablesContext),
        has_translation: bool,
        topic_id: usize,
        word_id: usize,
        probability: f64,
        voting: &(impl VotingMethod + Sync + Send)
    ) -> Result<Candidate, TranslateErrorWithOrigin> {
        let mut context = context_map! {
            COUNT_OF_VOTERS => 1,
            HAS_TRANSLATION => has_translation,
            IS_ORIGIN_WORD => true,
            SCORE_CANDIDATE => probability,
            CANDIDATE_ID => word_id as i64,
            NUMBER_OF_VOTERS => 1
        }.unwrap();

        let mut context = context.combine_with_mut(topic_context);

        let original_meta = topic_model.get_word_meta(topic_id, word_id).unwrap();

        let mut voters = vec![
            context_map! {
                RECIPROCAL_RANK => 1./ original_meta.importance_rank() as f64,
                REAL_RECIPROCAL_RANK => 1./ original_meta.rank() as f64,
                RANK => original_meta.rank() as i64,
                IMPORTANCE => original_meta.importance_rank() as i64,
                SCORE => original_meta.probability,
                VOTER_ID => word_id as i64
            }.unwrap()
        ];

        match voting.execute_to_f64(&mut context, voters.as_mut_slice()) {
            Ok(result) => {
                Ok(Candidate::new(LanguageOrigin::Origin(word_id), result, word_id))
            }
            Err(err) => {
                Err(err.originates_at(topic_id, word_id))
            }
        }
    }

    let candidates = match config.keep_original_word {
        KeepOriginalWord::Always => {
            Some(if let Some(Ok(mut candidates)) = candidates {
                match vote_for_origin(
                    topic_model,
                    topic_context,
                    true,
                    topic_id,
                    original_word_id,
                    probability,
                    &config.voting
                ) {
                    Ok(value) => {
                        candidates.push(value);
                        Ok(candidates)
                    }
                    Err(value) => {Err(value)}
                }
            } else {
                match vote_for_origin(
                    topic_model,
                    topic_context,
                    false,
                    topic_id,
                    original_word_id,
                    probability,
                    &config.voting
                ) {
                    Ok(value) => {
                        Ok(vec![value])
                    }
                    Err(value) => {
                        Err(value)
                    }
                }
            })
        }
        KeepOriginalWord::IfNoTranslation => {
            if candidates.is_none() {
                Some(
                    match vote_for_origin(
                        topic_model,
                        topic_context,
                        false,
                        topic_id,
                        original_word_id,
                        probability,
                        &config.voting
                    ) {
                        Ok(value) => {
                            Ok(vec![value])
                        }
                        Err(value) => {
                            Err(value)
                        }
                    }
                )
            } else {
                candidates
            }
        }
        KeepOriginalWord::Never => {
            candidates
        }
    };

    if let Some(top_candidate_limit) = config.top_candidate_limit {
        if let Some(Ok(mut candidates)) = candidates {
            let top_candidate_limit = top_candidate_limit.get();
            Some(Ok(
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
}





#[cfg(test)]
mod test {
    use crate::topicmodel::model::{FullTopicModel, TopicModel};
    use crate::translate::test::create_test_data;
    use crate::translate::topic_model_specific::translate_topic_model_without_provider;
    use crate::translate::KeepOriginalWord::Never;
    use crate::translate::TranslateConfig;
    use crate::voting::spy::IntoSpy;
    use crate::voting::BuildInVoting;
    use std::num::NonZeroUsize;

    #[test]
    fn test_complete_translation(){
        let (voc_a, _, dict) = create_test_data();

        let model_a = TopicModel::new(
            vec![
                vec![0.019, 0.018, 0.012, 0.009, 0.008, 0.008, 0.008, 0.008, 0.008, 0.008, 0.008],
                vec![0.002, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.02, 0.0001],
            ],
            voc_a,
            vec![10, 5, 8, 1, 2, 3, 1, 1, 1, 1, 2],
            vec![
                vec![0.7, 0.2],
                vec![0.8, 0.3]
            ],
            vec![
                200,
                300
            ]
        );

        let config = TranslateConfig {
            threshold: None,
            voting: BuildInVoting::PCombSum.spy(),
            epsilon: None,
            keep_original_word: Never,
            top_candidate_limit: Some(NonZeroUsize::new(3).unwrap())
        };

        let model_b = translate_topic_model_without_provider(
            &model_a,
            &dict,
            &config,
        ).unwrap();

        model_a.show_10().unwrap();
        println!("----");
        model_b.show_10().unwrap();
    }
}