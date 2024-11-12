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

mod traits;
mod config;
mod errors;
mod language;
mod phantoms;
pub mod topic_model_specific;

use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use evalexpr::{context_map, Context, ContextWithMutableVariables, EmptyContextWithBuiltinFunctions, HashMapContext, IterateVariablesContext};
use itertools::Itertools;
use rayon::prelude::*;
pub use traits::*;
use language::*;
pub use errors::*;
pub use config::*;
use crate::toolkit::evalexpr::{CombineableContext, ContextExtender};
use crate::topicmodel::create_topic_model_specific_dictionary;
use crate::topicmodel::dictionary::{DictionaryMut, DictionaryWithVocabulary, FromVoc};
use crate::topicmodel::dictionary::direction::{AToB, BToA, B};
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::reference::HashRef;
use crate::topicmodel::vocabulary::{BasicVocabulary, MappableVocabulary, VocabularyMut};
use crate::translate::phantoms::DummyAsVariableProvider;
use crate::translate::TranslateError::IncompatibleLanguages;
use crate::variable_provider::variable_names::*;
use crate::variable_provider::{AsVariableProvider, VariableProvider, VariableProviderOut};
use crate::voting::traits::VotingMethodMarker;
use crate::voting::VotingMethod;


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

#[allow(dead_code)]
pub fn translate_topic_model_without_provider<'a, Target, D, T, Voc, V>(
    target: &'a Target,
    dictionary: &'a D,
    translate_config: &TranslateConfig<V>,
) -> Result<Target, TranslateError<'a>>
where
    T: Hash + Eq + Ord + Clone,
    V: VotingMethodMarker,
    Voc: VocabularyMut<T> + MappableVocabulary<T> + Clone + 'a + for<'b> FromIterator<&'b HashRef<T>>,
    D: DictionaryWithVocabulary<T, Voc> + DictionaryMut<T, Voc> + FromVoc<T, Voc>,
    Target: TranslatableTopicMatrixWithCreate<T, Voc>,
{
    translate_topic_model(
        target,
        dictionary,
        translate_config,
        None::<&DummyAsVariableProvider<T>>
    )
}


pub(crate) fn translate_topic_model<'a, Target, D, T, Voc, V, P>(
    target: &'a Target,
    dictionary: &'a D,
    translate_config: &TranslateConfig<V>,
    provider: Option<&P>
) -> Result<Target, TranslateError<'a>> where
    T: Hash + Eq + Ord + Clone,
    V: VotingMethodMarker,
    Voc: VocabularyMut<T> + MappableVocabulary<T> + Clone + 'a + for<'b> FromIterator<&'b HashRef<T>>,
    D: DictionaryWithVocabulary<T, Voc> + DictionaryMut<T, Voc> + FromVoc<T, Voc>,
    Target: TranslatableTopicMatrixWithCreate<T, Voc>,
    P: AsVariableProvider<T>
{

    if let Some(lang_model) = target.vocabulary().language() {
        if let (Some(lang_a), lang_b) = dictionary.language_direction() {
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
        target.vocabulary()
    );

    // TODO: make clean for rust.
    let provider = if let Some(provider) = provider {
        Some(provider.as_variable_provider_for(target, &dictionary))
    } else {
        None
    }.transpose()?;

    let epsilon = if let Some(value) = translate_config.epsilon {
        value
    } else {
        target.matrix().iter().flat_map(|value| value.iter()).fold(
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
    let result = target
        .matrix()
        .par_iter()
        .zip_eq(target.matrix_meta().par_iter())
        .enumerate()
        .map(|(topic_id, (topic, meta))| {
            let mut topic_context_2 = context_map! {
                TOPIC_MAX_PROBABILITY => meta.max_score(),
                TOPIC_MIN_PROBABILITY => meta.min_score(),
                TOPIC_AVG_PROBABILITY => meta.avg_score(),
                TOPIC_SUM_PROBABILITY => meta.sum_score(),
                TOPIC_ID => topic_id as i64
            }.unwrap();
            meta.extend_context(&mut topic_context_2);

            if let Some(provider) = provider.as_ref() {
                match provider.provide_for_topic(topic_id, &mut topic_context_2) {
                    Ok(_) => {
                        let topic_context_2 = topic_context_2
                            .to_static_with(topic_context.clone());

                        translate_topic(
                            target,
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
                    target,
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


    let mut voc_b = voc_b_col.iter().flatten().cloned().collect::<Voc>();
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

    Ok(Target::create_new_from(
        inner_topic_model,
        voc_b,
        counts,
        target
    ))
}



fn translate_topic<Target, T, V, Voc, P>(
    target: &Target,
    dictionary: &impl DictionaryWithVocabulary<T, Voc>,
    topic_id: usize,
    topic: &<Target::TopicToVoterMatrix as TopicModelLikeMatrix>::TopicLike,
    topic_context: impl Context + Send + Sync + IterateVariablesContext,
    config: &TranslateConfig<V>,
    provider: Option<&P>
) -> Result<Vec<Candidate>, TranslateErrorWithOrigin>
where V: VotingMethodMarker,
      Voc: BasicVocabulary<T> ,
      Target: TranslatableTopicMatrix<T, Voc>,
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
                                    target,
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
                    target,
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
fn translate_single_candidate<Target, T, V, Voc, P>(
    target: &Target,
    dictionary: &impl DictionaryWithVocabulary<T, Voc>,
    topic_id: usize,
    topic_context: &(impl Context + Send + Sync + IterateVariablesContext),
    config: &TranslateConfig<V>,
    original_voter_id: usize,
    probability: f64,
    provider: Option<&P>
) -> Option<Result<Vec<Candidate>, TranslateErrorWithOrigin>>
where V: VotingMethodMarker,
      Voc: BasicVocabulary<T> ,
      Target: TranslatableTopicMatrix<T, Voc>,
      P: VariableProviderOut
{
    let candidates =
        if let Some(candidates) = dictionary.translate_id_to_ids::<AToB>(original_voter_id) {
            Some(candidates.par_iter().cloned().filter_map( |candidate|
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
                            COUNT_OF_VOTERS => mapped.len() as i64,
                            HAS_TRANSLATION => true,
                            IS_ORIGIN_WORD => false,
                            SCORE_CANDIDATE => probability,
                            CANDIDATE_ID => candidate as i64
                        }.unwrap();

                        let mut context = context.combine_with_mut(topic_context);

                        let voters = mapped
                            .iter()
                            .map(|voter_a| {
                                let mut context_voter_a = context_map! {
                                    RECIPROCAL_RANK => 1./ voter_a.importance() as f64,
                                    REAL_RECIPROCAL_RANK => 1./ voter_a.rank() as f64,
                                    RANK => voter_a.rank() as i64,
                                    IMPORTANCE => voter_a.importance() as i64,
                                    SCORE => voter_a.score(),
                                    VOTER_ID => voter_a.voter_id() as i64
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
                                        (voters.len() as i64).into()
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
                                    Err(TranslateErrorWithOrigin {
                                        topic_id,
                                        word_id: original_voter_id,
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
        target: &'a impl VoterInfoProvider,
        topic_context: &(impl Context + Send + Sync + IterateVariablesContext),
        has_translation: bool,
        topic_id: usize,
        voter_id: usize,
        probability: f64,
        voting: &(impl VotingMethod + Sync + Send)
    ) -> Result<Candidate, TranslateErrorWithOrigin> {
        let mut context = context_map! {
            COUNT_OF_VOTERS => 1,
            HAS_TRANSLATION => has_translation,
            IS_ORIGIN_WORD => true,
            SCORE_CANDIDATE => probability,
            CANDIDATE_ID => voter_id as i64,
            NUMBER_OF_VOTERS => 1
        }.unwrap();

        let mut context = context.combine_with_mut(topic_context);

        let original_meta = target.get_voter_meta(topic_id, voter_id).unwrap();

        assert_eq!(original_meta.voter_id(), voter_id, "The voter ids differ!");

        let mut voters = vec![
            context_map! {
                RECIPROCAL_RANK => 1./ original_meta.importance() as f64,
                REAL_RECIPROCAL_RANK => 1./ original_meta.rank() as f64,
                RANK => original_meta.rank() as i64,
                IMPORTANCE => original_meta.importance() as i64,
                SCORE => original_meta.score(),
                VOTER_ID => voter_id as i64
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
                    target,
                    topic_context,
                    false,
                    topic_id,
                    original_voter_id,
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
                        target,
                        topic_context,
                        false,
                        topic_id,
                        original_voter_id,
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
pub(crate) mod test {
    use crate::topicmodel::dictionary::direction::Invariant;
    use crate::topicmodel::dictionary::{Dictionary, DictionaryMut};
    use crate::topicmodel::model::{FullTopicModel, TopicModel};
    use crate::topicmodel::vocabulary::{SearchableVocabulary, Vocabulary};
    use crate::translate::translate_topic_model_without_provider;
    use crate::translate::KeepOriginalWord::Never;
    use crate::translate::TranslateConfig;
    use crate::voting::spy::IntoSpy;
    use crate::voting::BuildInVoting;
    use std::num::NonZeroUsize;
    use Extend;

    pub fn create_test_data() -> (Vocabulary<String>, Vocabulary<String>, Dictionary<String, Vocabulary<String>>){
        let mut voc_a = Vocabulary::<String>::default();
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
        let mut voc_b = Vocabulary::<String>::default();
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

        let mut dict = Dictionary::new();
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Ebene").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Planum").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Platane").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Maschine").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Bremsberg").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Berg").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Fläche").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("plane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Luftfahrzeug").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Fluggerät").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("aircraft").unwrap().clone(), voc_b.get_hash_ref("Flugsystem").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Flugzeug").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airplane").unwrap().clone(), voc_b.get_hash_ref("Motorflugzeug").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("flyer").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airman").unwrap().clone(), voc_b.get_hash_ref("Flieger").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("airfoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("wing").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("deck").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("hydrofoil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("foil").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);
        dict.insert_hash_ref::<Invariant>(voc_a.get_hash_ref("bearing surface").unwrap().clone(), voc_b.get_hash_ref("Tragfläche").unwrap().clone(),);

        (voc_a, voc_b, dict)
    }

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