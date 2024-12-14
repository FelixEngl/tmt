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

pub mod config;
mod errors;
mod language;
mod phantoms;
mod candidate;
pub mod entropies;
pub mod dictionary_meta;
// pub mod topic_model_specific;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::{Duration};
use evalexpr::{context_map, Context, ContextWithMutableVariables, EmptyContextWithBuiltinFunctions, HashMapContext, IterateVariablesContext, Value};
use itertools::Itertools;
use rayon::prelude::*;
use language::*;
pub use errors::*;
pub use config::*;
use ldatranslate_toolkit::evalexpr::CombineableContext;
use ldatranslate_topicmodel::dictionary::*;
use ldatranslate_topicmodel::dictionary::direction::{AToB, BToA, DirectionMarker, B};
use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataManagerEx;
use ldatranslate_topicmodel::language_hint::LanguageHint;
use ldatranslate_topicmodel::translate::{create_topic_vocabulary_specific_dictionary, TranslatableTopicMatrix, TranslatableTopicMatrixWithCreate};
use ldatranslate_topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary, MappableVocabulary, VocabularyMut};
use ldatranslate_translate::{ContextExtender, TopicLike, TopicMeta, TopicMetas, TopicModelLikeMatrix, VoterInfoProvider, VoterMeta};
use crate::translate::phantoms::DummyAsVariableProvider;
use crate::translate::TranslateError::IncompatibleLanguages;
use ldatranslate_voting::variable_provider::variable_names::*;
use ldatranslate_voting::variable_provider::{VariableProvider, VariableProviderOut};
use ldatranslate_voting::constants::TMTNumericTypes;
use ldatranslate_voting::traits::VotingMethodMarker;
use ldatranslate_voting::VotingMethod;
use crate::tools::memory::MemoryReporter;
use crate::translate::candidate::Candidate;
use crate::translate::dictionary_meta::booster::{Booster, TopicSpecificBooster};
use crate::translate::dictionary_meta::horizontal_boost_1::HorizontalScoreBoost;
use crate::translate::dictionary_meta::vertical_boost_1::{VerticalBoostedScores};
use crate::variable_provider::AsVariableProvider;


#[allow(dead_code)]
pub fn translate_topic_model_without_provider<'a, Target, D, T, Voc, V>(
    target: &'a Target,
    dictionary: &'a D,
    translate_config: &TranslateConfig<V>,
) -> Result<Target, TranslateError<'a>>
where
    T: Hash + Eq + Ord + Clone + Sync + Send + 'a + Debug,
    V: VotingMethodMarker,
    Voc: VocabularyMut<T> + MappableVocabulary<T> + Clone + Send + Sync + for<'b> FromIterator<&'b T> + 'a + Debug + AnonymousVocabulary,
    D: DictionaryWithVocabulary<T, Voc> + DictionaryMut<T, Voc> + FromVoc<T, Voc> + Send + Sync + Debug + BasicDictionaryWithMeta<MetadataManagerEx, Voc>,
    Target: TranslatableTopicMatrixWithCreate<T, Voc>,
{
    translate_topic_model(
        target,
        dictionary,
        translate_config,
        None::<&DummyAsVariableProvider<T>>
    )
}


pub fn translate_topic_model<'a, Target, D, T, Voc, V, P>(
    target: &'a Target,
    original_dictionary: &'a D,
    translate_config: &TranslateConfig<V>,
    provider: Option<&P>
) -> Result<Target, TranslateError<'a>> where
    T: Hash + Eq + Ord + Clone + Send + Sync + 'a + Debug,
    V: VotingMethodMarker,
    Voc: VocabularyMut<T> + MappableVocabulary<T> + Clone + Send + Sync + for<'b> FromIterator<&'b T> + 'a + Debug + AnonymousVocabulary,
    D: DictionaryWithVocabulary<T, Voc> + DictionaryMut<T, Voc> + FromVoc<T, Voc> + Send + Sync + Debug + BasicDictionaryWithMeta<MetadataManagerEx, Voc>,
    Target: TranslatableTopicMatrixWithCreate<T, Voc>,
    P: AsVariableProvider<T>
{

    let reporter = MemoryReporter::new(Duration::from_millis(2000));

    if let Some(lang_model) = target.vocabulary().language() {
        if let (Some(lang_a), lang_b) = original_dictionary.language_direction_a_to_b() {
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

    if original_dictionary.map_a_to_b().is_empty() {
        return Err(TranslateError::DictionaryEmpty(DirectionMarker::AToB));
    }

    if original_dictionary.map_b_to_a().is_empty() {
        return Err(TranslateError::DictionaryEmpty(DirectionMarker::BToA));
    }

    log::info!("Create topic specific dictionary. {}", reporter.create_report_now());
    let translation_dictionary: D = create_topic_vocabulary_specific_dictionary(
        original_dictionary,
        target.vocabulary()
    );
    log::info!("After cration of topic specific dict. {}", reporter.create_report_now());

    let booster = Booster::new(
        translate_config.vertical_config.clone().map(|divergence| {
            VerticalBoostedScores::new(
                divergence,
                &translation_dictionary,
                original_dictionary,
                target
            )
        }).transpose()?,
        translate_config.horizontal_config.clone().map(|vert| {
            HorizontalScoreBoost::new(
                vert,
                &translation_dictionary,
                original_dictionary,
            )
        }).transpose()?,
    );

    if translation_dictionary.map_a_to_b().is_empty() {
        return Err(TranslateError::OptimizedDictionaryEmpty(DirectionMarker::AToB))
    }

    if translation_dictionary.map_b_to_a().is_empty() {
        return Err(TranslateError::OptimizedDictionaryEmpty(DirectionMarker::BToA))
    }

    // TODO: make clean for rust.
    let provider = if let Some(provider) = provider {
        Some(provider.as_variable_provider_for(target, &translation_dictionary))
    } else {
        None
    }.transpose()?;

    let epsilon = if let Some(value) = translate_config.epsilon {
        value
    } else {
        if let Some(vert) = booster.vertical_booster().map(|value| value.alternative_scores()) {
            TopicModelLikeMatrix::iter(vert).flat_map(|value| value.iter()).fold(
                f64::MAX,
                |old, &other| {
                    old.min(other)
                }
            ) - f64::EPSILON
        } else {
            target.matrix().iter().flat_map(|value| value.iter()).fold(
                f64::MAX,
                |old, &other| {
                    old.min(other)
                }
            ) - f64::EPSILON
        }
    };

    let mut topic_context: HashMapContext<TMTNumericTypes> = context_map! {
        EPSILON => as float epsilon,
        VOCABULARY_SIZE_A => as int translation_dictionary.voc_a().len(),
        VOCABULARY_SIZE_B => as int translation_dictionary.voc_b().len(),
    }.unwrap();

    if let Some(ref provider) = provider {
        provider.provide_global(&mut topic_context)?;
    }

    let topic_context =
        topic_context.to_static_with(EmptyContextWithBuiltinFunctions::default());


    log::info!("Memory usage before loop: {}", reporter.create_report_now());
    // topic to word id to probable translation candidates.
    let result = target
        .matrix()
        .par_iter()
        .zip_eq(target.matrix_meta().par_iter())
        .enumerate()
        .map(|(topic_id, (topic, meta))| {
            let topic_boost = booster.provide_for_topic(topic_id);
            let mut topic_context_2 = context_map! {
                TOPIC_MAX_PROBABILITY => float meta.max_score(),
                TOPIC_MIN_PROBABILITY => float meta.min_score(),
                TOPIC_AVG_PROBABILITY => float meta.avg_score(),
                TOPIC_SUM_PROBABILITY => float meta.sum_score(),
                TOPIC_ID => int topic_id as i64
            }.unwrap();
            meta.extend_context(&mut topic_context_2);
            if let Some(provider) = provider.as_ref() {
                match provider.provide_for_topic(topic_id, &mut topic_context_2) {
                    Ok(_) => {
                        let topic_context_2 = topic_context_2
                            .to_static_with(topic_context.clone());
                        translate_topic(
                            target,
                            &translation_dictionary,
                            topic_id,
                            topic,
                            topic_context_2,
                            &translate_config,
                            Some(provider),
                            topic_boost
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
                    target,
                    &translation_dictionary,
                    topic_id,
                    topic,
                    topic_context_2,
                    &translate_config,
                    None::<&VariableProvider>,
                    topic_boost
                ).map_err(TranslateError::WithOrigin)
            }
        }).collect::<Result<Vec<_>, _>>()?;


    drop(booster);


    let voc_b_col = result.par_iter().flatten().map(|value| {
        match value.candidate_word_id {
            LanguageOrigin::Origin(word_id) => {
                translation_dictionary.voc_a().get_value_by_id(word_id).unwrap()
            }
            LanguageOrigin::Target(word_id) => {
                translation_dictionary.voc_b().get_value_by_id(word_id).unwrap()
            }
        }
    }).collect_vec_list();

    log::info!("Memory usage after loop: {}", reporter.create_report_now());
    let mut voc_b = voc_b_col.iter().flatten().cloned().collect::<Voc>();
    voc_b.set_language(translation_dictionary.language::<B>().cloned());

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
                    translation_dictionary.voc_a().get_value_by_id(word_id).unwrap()
                }
                LanguageOrigin::Target(word_id) => {
                    translation_dictionary.voc_b().get_value_by_id(word_id).unwrap()
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

    log::info!("Memory usage when finished: {}", reporter.create_report_now());
    Ok(Target::create_new_from(
        inner_topic_model,
        voc_b,
        counts,
        target
    ))
}


fn translate_topic<Target, T, V, Voc, P, C>(
    target: &Target,
    dictionary: &impl DictionaryWithVocabulary<T, Voc>,
    topic_id: usize,
    topic: &<Target::TopicToVoterMatrix<'_> as TopicModelLikeMatrix>::TopicLike,
    topic_context: C,
    config: &TranslateConfig<V>,
    provider: Option<&P>,
    booster: TopicSpecificBooster
) -> Result<Vec<Candidate>, TranslateErrorWithOrigin>
where
    V: VotingMethodMarker,
    Voc: BasicVocabulary<T> ,
    Target: TranslatableTopicMatrix<T, Voc>,
    P: VariableProviderOut,
    C: Context<NumericTypes=TMTNumericTypes> + Send + Sync + IterateVariablesContext
{
    topic
        .par_iter()
        .enumerate()
        .filter_map(|(original_word_id, probability)| {
            let probability = *probability;
            if let Some(provider) = provider {
                let mut context = HashMapContext::new();
                match provider.provide_for_word_a(original_word_id, &mut context) {
                    Ok(_) => {
                        match provider.provide_for_word_a(original_word_id, &mut context) {
                            Ok(_) => {
                                let combined = topic_context.combine_with(&context);
                                translate_single_candidate(
                                    target,
                                    dictionary,
                                    topic_id,
                                    &combined,
                                    config,
                                    original_word_id,
                                    probability,
                                    Some(provider),
                                    &booster
                                )
                            }
                            Err(err) => {Some(Err(TranslateErrorWithOrigin::new(
                                err.into(),
                                original_word_id,
                                topic_id
                            )))}
                        }
                    }
                    Err(err) => {Some(Err(TranslateErrorWithOrigin::new(
                        err.into(),
                        original_word_id,
                        topic_id
                    )))}
                }
            } else {
                translate_single_candidate(
                    target,
                    dictionary,
                    topic_id,
                    &topic_context,
                    config,
                    original_word_id,
                    probability,
                    provider,
                    &booster
                )
            }
        }).collect::<Result<Vec<_>, _>>().map(|value| {
        value.into_iter().flatten().collect::<Vec<_>>()
    })
}



#[inline(always)]
fn translate_single_candidate<Target, T, V, Voc, P, C>(
    target: &Target,
    dictionary: &impl DictionaryWithVocabulary<T, Voc>,
    topic_id: usize,
    topic_context: &C,
    config: &TranslateConfig<V>,
    original_voter_id: usize,
    probability: f64,
    provider: Option<&P>,
    booster: &TopicSpecificBooster,
) -> Option<Result<Vec<Candidate>, TranslateErrorWithOrigin>>
where
    V: VotingMethodMarker,
    Voc: BasicVocabulary<T> ,
    Target: TranslatableTopicMatrix<T, Voc>,
    P: VariableProviderOut,
    C: Context<NumericTypes=TMTNumericTypes> + Send + Sync + IterateVariablesContext
{
    let candidates =
        if let Some(candidates) = dictionary.translate_id_to_ids::<AToB>(original_voter_id) {
            Some(candidates.par_iter().cloned().filter_map( |candidate| {
                match dictionary.translate_id_to_ids::<BToA>(candidate) {
                    None  => None,
                    Some(voters) if voters.is_empty() => None,
                    Some(voters) => {
                        let mapped = voters
                            .iter()
                            .filter_map(|voter_id_a_retrans| {
                                target.get_voter_meta(topic_id, *voter_id_a_retrans)
                            });

                        let mapped = if let Some(threshold) = config.threshold {
                            mapped.filter(|value| value.score() >= threshold).collect_vec()
                        } else {
                            mapped.collect_vec()
                        };


                        let mut context = context_map! {
                            COUNT_OF_VOTERS => as int mapped.len(),
                            HAS_TRANSLATION => true,
                            IS_ORIGIN_WORD => false,
                            SCORE_CANDIDATE => as float booster.boost_score(
                                probability,
                                original_voter_id,
                                candidate,
                            ),
                            CANDIDATE_ID => as int candidate
                        }.unwrap();

                        let mut context = context.combine_with_mut(topic_context);

                        let voters = mapped
                            .iter()
                            .map(|voter_a| {

                                let probability_of_voter = booster.boost_score(
                                    probability,
                                    voter_a.voter_id(),
                                    candidate,
                                );

                                let mut context_voter_a = context_map! {
                                    RECIPROCAL_RANK => float 1./ voter_a.importance() as f64,
                                    REAL_RECIPROCAL_RANK => float 1./ voter_a.rank() as f64,
                                    RANK => int voter_a.rank() as i64,
                                    IMPORTANCE => int voter_a.importance() as i64,
                                    SCORE => float probability_of_voter,
                                    VOTER_ID => int voter_a.voter_id() as i64
                                }.unwrap();
                                voter_a.extend_context(&mut context_voter_a);
                                if let Some(provider) = provider {
                                    match provider.provide_for_word_a(voter_a.voter_id(), &mut context_voter_a) {
                                        Ok(_) => {
                                            match provider.provide_for_word_in_topic_a(topic_id, voter_a.voter_id(), &mut context_voter_a) {
                                                Ok(_) => {
                                                    Ok(context_voter_a)
                                                }
                                                Err(err) => {Err(err)}
                                            }
                                        }
                                        Err(err) => {
                                            Err(err)
                                        }
                                    }
                                } else {
                                    Ok(context_voter_a)
                                }
                            })
                            .collect::<Result<Vec<_>, _>>();

                        Some(
                            match voters {
                                Ok(mut voters) => {
                                    context.set_value(
                                        NUMBER_OF_VOTERS.to_string(),
                                        Value::from_as_int(voters.len())
                                    ).expect("This should not fail!");
                                    match config.voting.execute_to_f64(&mut context, voters.as_mut_slice()) {
                                        Ok(result) => {
                                            Ok(Candidate::new(LanguageOrigin::Target(candidate), result, original_voter_id))
                                        }
                                        Err(err) => {
                                            Err(err.originates_at(topic_id, original_voter_id))
                                        }
                                    }
                                }
                                Err(err) => {
                                    Err(TranslateErrorWithOrigin::new(
                                        err.into(),
                                        original_voter_id,
                                        topic_id
                                    ))
                                }
                            }

                        )
                    }
                }
            }).collect::<Result<Vec<Candidate>, TranslateErrorWithOrigin>>())
        } else {
            // Unknown
            None
        };


    fn vote_for_origin<'a, C, V>(
        target: &'a impl VoterInfoProvider,
        topic_context: &C,
        has_translation: bool,
        topic_id: usize,
        voter_id: usize,
        probability: f64,
        voting: &V,
        booster: &TopicSpecificBooster,
    ) -> Result<Candidate, TranslateErrorWithOrigin>
    where
        C: Context<NumericTypes=TMTNumericTypes> + Send + Sync + IterateVariablesContext,
        V: VotingMethod + Sync + Send
    {
        let mut context = context_map! {
            COUNT_OF_VOTERS => as int 1,
            HAS_TRANSLATION => has_translation,
            IS_ORIGIN_WORD => true,
            SCORE_CANDIDATE => as float probability,
            CANDIDATE_ID => as int voter_id,
            NUMBER_OF_VOTERS => as int 1
        }.unwrap();

        let mut context = context.combine_with_mut(topic_context);

        let original_meta = target.get_voter_meta(topic_id, voter_id).unwrap();

        assert_eq!(original_meta.voter_id(), voter_id, "The voter ids differ!");

        let probability_of_voter =
            booster.boost_vertical(original_meta.score(), original_meta.voter_id());

        let mut voters = vec![
            context_map! {
                RECIPROCAL_RANK => as float 1./ original_meta.importance() as f64,
                REAL_RECIPROCAL_RANK => as float 1./ original_meta.rank() as f64,
                SCORE => as float probability_of_voter,
                RANK => as int original_meta.rank(),
                IMPORTANCE => as int original_meta.importance(),
                VOTER_ID => as int voter_id,
            }.and_then(|mut value| {
                original_meta.extend_context(&mut value);
                Ok(value)
            }).unwrap()
        ];

        match voting.execute_to_f64(&mut context, voters.as_mut_slice()) {
            Ok(result) => {
                Ok(Candidate::new(LanguageOrigin::Origin(voter_id), result, voter_id))
            }
            Err(err) => {
                Err(err.originates_at(topic_id, voter_id))
            }
        }
    }

    let candidates = match config.keep_original_word {
        KeepOriginalWord::Always => {
            Some(if let Some(Ok(mut candidates)) = candidates {
                match vote_for_origin(
                    target,
                    topic_context,
                    true,
                    topic_id,
                    original_voter_id,
                    probability,
                    &config.voting,
                    booster
                ) {
                    Ok(value) => {
                        candidates.push(value);
                        Ok(candidates)
                    }
                    Err(value) => {Err(value)}
                }
            } else {
                match vote_for_origin(
                    target,
                    topic_context,
                    false,
                    topic_id,
                    original_voter_id,
                    probability,
                    &config.voting,
                    booster
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
                        target,
                        topic_context,
                        false,
                        topic_id,
                        original_voter_id,
                        probability,
                        &config.voting,
                        booster
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
pub(crate) mod test {
    use ldatranslate_topicmodel::dictionary::direction::Invariant;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMutMeta, Dictionary, DictionaryMutGen, DictionaryWithMeta};
    use ldatranslate_topicmodel::model::{FullTopicModel, TopicModel};
    use ldatranslate_topicmodel::vocabulary::{SearchableVocabulary, Vocabulary};
    use crate::translate::{translate_topic_model_without_provider, FieldConfig, HorizontalScoreBootConfig, MeanMethod, VerticalScoreBoostConfig};
    use crate::translate::KeepOriginalWord::Never;
    use crate::translate::TranslateConfig;
    use ldatranslate_voting::spy::IntoSpy;
    use ldatranslate_voting::BuildInVoting;
    use std::num::NonZeroUsize;
    use Extend;
    use std::sync::Arc;
    use arcstr::ArcStr;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, Register};
    use crate::translate::dictionary_meta::vertical_boost_1::{ScoreModifierCalculator};
    use ldatranslate_topicmodel::dictionary::metadata::ex::*;
    use crate::translate::dictionary_meta::coocurrence::NormalizeMode;
    use crate::translate::entropies::{FDivergence, FDivergenceCalculator};

    pub fn create_test_data() -> (Vocabulary<ArcStr>, Vocabulary<ArcStr>, Dictionary<ArcStr, Vocabulary<ArcStr>>){
        let mut voc_a = Vocabulary::<ArcStr>::default();
        voc_a.extend(vec![
            "plane".to_string(),
            "aircraft".to_string(),
            "airplane".to_string(),
            "flyer".to_string(),
            "airman".to_string(),
            "airfoil".to_string(),
            "wing".to_string(),
            "deck".to_string(),
            "hydrofoil".to_string(),
            "foil".to_string(),
            "bearing surface".to_string()
        ]);
        let mut voc_b = Vocabulary::<ArcStr>::default();
        voc_b.extend(vec![
            "Flugzeug".to_string(),
            "Flieger".to_string(),
            "Tragfläche".to_string(),
            "Ebene".to_string(),
            "Planum".to_string(),
            "Platane".to_string(),
            "Maschine".to_string(),
            "Bremsberg".to_string(),
            "Berg".to_string(),
            "Fläche".to_string(),
            "Luftfahrzeug".to_string(),
            "Fluggerät".to_string(),
            "Flugsystem".to_string(),
            "Motorflugzeug".to_string(),
        ]);

        let mut dict = Dictionary::default();
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Flugzeug").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Flieger").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Tragfläche").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Ebene").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Planum").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Platane").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Maschine").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Bremsberg").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Berg").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Fläche").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("plane").unwrap().clone(), voc_b.get_value("Flieger").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("aircraft").unwrap().clone(), voc_b.get_value("Flugzeug").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("aircraft").unwrap().clone(), voc_b.get_value("Flieger").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("aircraft").unwrap().clone(), voc_b.get_value("Luftfahrzeug").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("aircraft").unwrap().clone(), voc_b.get_value("Fluggerät").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("aircraft").unwrap().clone(), voc_b.get_value("Flugsystem").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("airplane").unwrap().clone(), voc_b.get_value("Flugzeug").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("airplane").unwrap().clone(), voc_b.get_value("Flieger").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("airplane").unwrap().clone(), voc_b.get_value("Motorflugzeug").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("flyer").unwrap().clone(), voc_b.get_value("Flieger").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("airman").unwrap().clone(), voc_b.get_value("Flieger").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("airfoil").unwrap().clone(), voc_b.get_value("Tragfläche").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("wing").unwrap().clone(), voc_b.get_value("Tragfläche").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("deck").unwrap().clone(), voc_b.get_value("Tragfläche").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("hydrofoil").unwrap().clone(), voc_b.get_value("Tragfläche").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("foil").unwrap().clone(), voc_b.get_value("Tragfläche").unwrap().clone(),);
        dict.insert_value::<Invariant>(voc_a.get_value("bearing surface").unwrap().clone(), voc_b.get_value("Tragfläche").unwrap().clone(),);

        (voc_a, voc_b, dict)
    }

    #[test]
    fn test_complete_translation(){
        let (voc_a, _, dict) = create_test_data();

        let mut dict: DictionaryWithMeta<_, _, MetadataManagerEx> = DictionaryWithMeta::from(dict);
        dict.get_or_create_meta_a(0)
            .add_all_to_domains_default([Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict1", [Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict2", [Domain::Engin])
            .add_all_to_registers_default([Register::Techn]);

        dict.get_or_create_meta_a(1)
            .add_all_to_domains_default([Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict1", [Domain::Engin])
            .add_all_to_domains("dict2", [Domain::Engin]);


        dict.get_or_create_meta_b(0)
            .add_all_to_domains_default([Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict1", [Domain::Engin])
            .add_all_to_domains("dict2", [Domain::Aviat, Domain::Engin]);

        dict.get_or_create_meta_b(3)
            .add_all_to_domains_default([Domain::Aviat, Domain::Engin, Domain::Comm])
            .add_all_to_domains("dict2", [Domain::Aviat, Domain::Engin]);

        dict.get_or_create_meta_b(3)
            .add_all_to_domains_default([Domain::Admin, Domain::Engin])
            .add_all_to_domains("dict2", [Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict1", [Domain::Film]);




        let mut model_a = TopicModel::new(
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

        model_a.normalize_in_place();

        let config1 = TranslateConfig {
            threshold: None,
            voting: BuildInVoting::PCombSum.spy(),
            epsilon: None,
            keep_original_word: Never,
            top_candidate_limit: Some(NonZeroUsize::new(3).unwrap()),
            vertical_config: None,
            horizontal_config: None
        };

        let config2 = TranslateConfig {
            threshold: None,
            voting: BuildInVoting::PCombSum.spy(),
            epsilon: None,
            keep_original_word: Never,
            top_candidate_limit: Some(NonZeroUsize::new(3).unwrap()),
            vertical_config: Some(
                Arc::new(
                    VerticalScoreBoostConfig::new(
                        FieldConfig::new(
                            Some(vec![
                                Domain::Aviat.into(),
                                Domain::Engin.into(),
                                Domain::Film.into(),
                                Register::Techn.into(),
                                Register::Archaic.into(),
                            ]),
                            false,
                        ),
                        FDivergenceCalculator::new(
                            FDivergence::KL,
                            None,
                            ScoreModifierCalculator::WeightedSum
                        ),
                        true
                    ),
                )
            ),
            horizontal_config: None
        };

        let config3 = TranslateConfig {
            threshold: None,
            voting: BuildInVoting::PCombSum.spy(),
            epsilon: None,
            keep_original_word: Never,
            top_candidate_limit: Some(NonZeroUsize::new(3).unwrap()),
            vertical_config: None,
            horizontal_config: Some(
                Arc::new(
                    HorizontalScoreBootConfig::new(
                        FieldConfig::new(
                            Some(vec![
                                Domain::Aviat.into(),
                                Domain::Engin.into(),
                                Domain::Film.into(),
                                Register::Techn.into(),
                                Register::Archaic.into(),
                            ]),
                            false,
                        ),
                        FDivergenceCalculator::new(
                            FDivergence::KL,
                            None,
                            ScoreModifierCalculator::WeightedSum
                        ),
                        NormalizeMode::Sum,
                        Some(0.15),
                        false,
                        MeanMethod::GeometricMean
                    )
                )
            )
        };

        let config4 = TranslateConfig {
            threshold: None,
            voting: BuildInVoting::PCombSum.spy(),
            epsilon: None,
            keep_original_word: Never,
            top_candidate_limit: Some(NonZeroUsize::new(3).unwrap()),
            vertical_config: Some(
                Arc::new(
                    VerticalScoreBoostConfig::new(
                        FieldConfig::new(
                            Some(vec![
                                Domain::Aviat.into(),
                                Domain::Engin.into(),
                                Domain::Film.into(),
                                Register::Techn.into(),
                                Register::Archaic.into(),
                            ]),
                            false,
                        ),
                        FDivergenceCalculator::new(
                            FDivergence::KL,
                            None,
                            ScoreModifierCalculator::WeightedSum
                        ),
                        true
                    ),
                )
            ),
            horizontal_config: Some(
                Arc::new(
                    HorizontalScoreBootConfig::new(
                        FieldConfig::new(
                            Some(vec![
                                Domain::Aviat.into(),
                                Domain::Engin.into(),
                                Domain::Film.into(),
                                Register::Techn.into(),
                                Register::Archaic.into(),
                            ]),
                            false,
                        ),
                        FDivergenceCalculator::new(
                            FDivergence::KL,
                            None,
                            ScoreModifierCalculator::WeightedSum
                        ),
                        NormalizeMode::Sum,
                        Some(0.15),
                        false,
                        MeanMethod::GeometricMean
                    )
                )
            )
        };

        let model_b = translate_topic_model_without_provider(
            &model_a,
            &dict,
            &config1,
        ).unwrap();

        let model_c = translate_topic_model_without_provider(
            &model_a,
            &dict,
            &config2,
        ).unwrap();

        let model_d = translate_topic_model_without_provider(
            &model_a,
            &dict,
            &config3,
        ).unwrap();

        let model_e = translate_topic_model_without_provider(
            &model_a,
            &dict,
            &config4,
        ).unwrap();

        // println!("{:?}", model_b.vocabulary());

        model_a.show_10().unwrap();
        println!("----");
        model_b.show_10().unwrap();
        println!("----");
        model_c.show_10().unwrap();
        println!("----");
        model_d.show_10().unwrap();
        println!("----");
        model_e.show_10().unwrap();
    }
}