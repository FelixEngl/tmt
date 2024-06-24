use std::cmp::Ordering;
use std::collections::{HashMap};
use std::collections::hash_map::Entry;
use std::error::Error;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::ops::Deref;
use evalexpr::{Context, context_map, ContextWithMutableVariables, EmptyContextWithBuiltinFunctions, HashMapContext};
use itertools::{Itertools};
use rayon::prelude::*;
use strum::{AsRefStr, Display, EnumString, ParseError};
use thiserror::Error;
use crate::toolkit::evalexpr::{CombineableContext};
use crate::topicmodel::topic_model::{BasicTopicModel, TopicModel, TopicModelWithDocumentStats, TopicModelWithVocabulary};
use crate::topicmodel::dictionary::{DictionaryMut, DictionaryWithVocabulary, FromVoc};
use crate::topicmodel::dictionary::direction::{AToB, B, BToA};
use crate::topicmodel::vocabulary::{MappableVocabulary, BasicVocabulary, Vocabulary, VocabularyMut, SearchableVocabulary};
use crate::translate::LanguageOrigin::{Origin, Target};
use crate::variable_names::*;
use crate::voting::{VotingExpressionError, VotingMethod, VotingResult};
use crate::voting::traits::VotingMethodMarker;
use pyo3::{Bound, pyclass, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use crate::external_variable_provider::{VariableProvider, VariableProviderError, VariableProviderOut};
use crate::topicmodel::create_topic_model_specific_dictionary;
use crate::topicmodel::language_hint::LanguageHint;
use crate::translate::TranslateError::IncompatibleLanguages;

#[derive(Debug)]
pub struct TranslateConfig<V: VotingMethodMarker> {
    voting: V,
    epsilon: Option<f64>,
    threshold: Option<f64>,
    keep_original_word: KeepOriginalWord,
    top_candidate_limit: Option<NonZeroUsize>,
}

impl<V> TranslateConfig<V> where V: VotingMethodMarker {
    pub fn new(voting: V, epsilon: Option<f64>, threshold: Option<f64>, keep_original_word: KeepOriginalWord, top_candidate_limit: Option<NonZeroUsize>) -> Self {
        Self { epsilon, voting, threshold, keep_original_word, top_candidate_limit }
    }
}

impl<'a, V> Clone for TranslateConfig<V> where V: VotingMethodMarker + Clone {
    fn clone(&self) -> Self {
        Self {
            voting: self.voting.clone(),
            epsilon: self.epsilon,
            threshold: self.threshold,
            keep_original_word: self.keep_original_word,
            top_candidate_limit: self.top_candidate_limit
        }
    }
}




#[derive(Debug, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Default)]
#[derive(AsRefStr, Display, EnumString)]
#[pyclass]
pub enum KeepOriginalWord {
    #[strum(serialize = "ALWAYS")]
    Always,
    #[strum(serialize = "IF_NO_TRANSLATION")]
    IfNoTranslation,
    #[strum(serialize = "NEVER")]
    #[default]
    Never
}

#[pymethods]
impl KeepOriginalWord {
    pub fn __str__(&self) -> String {
        self.to_string()
    }

    #[staticmethod]
    #[pyo3(name="from_string")]
    pub fn from_string_py(value: &str) -> PyResult<Self> {
        value.parse().map_err(|value: ParseError | PyValueError::new_err(value.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum TranslateError<'a> {
    #[error(transparent)]
    VotingError(#[from] VotingExpressionError),
    #[error(transparent)]
    WithOrigin(#[from] TranslateErrorWithOrigin),
    #[error(transparent)]
    ProviderError(#[from] VariableProviderError),
    #[error("The dictionary has a translation direction from {lang_a} to {lang_b}, but the topic is in {lang_b}!")]
    IncompatibleLanguages {
        lang_a: &'a LanguageHint,
        lang_b: LanguageHint,
        lang_model: &'a LanguageHint,
    }
}

#[derive(Debug, Error)]
#[error("Failed with an error! ({topic_id}, {word_id}) {source}")]
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
        None::<&VariableProvider>
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
    P: VariableProviderOut
{

    if let Some(lang_model) = topic_model.vocabulary().language() {
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
        topic_model.vocabulary()
    );

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

    if let Some(provider) = provider {
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

            if let Some(provider) = provider {
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
                    None::<&P>
                ).map_err(TranslateError::WithOrigin)
            }


    }).collect::<Result<Vec<_>, _>>()?;


    let voc_b_col = result.par_iter().flatten().map(|value| {
        match value.candidate_word_id {
            Origin(word_id) => {
                dictionary.voc_a().get_value(word_id).unwrap()
            }
            Target(word_id) => {
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
                Origin(word_id) => {
                    dictionary.voc_a().get_value(word_id).unwrap()
                }
                Target(word_id) => {
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

    return Ok(translated)
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
    topic_context: (impl Context + Send + Sync),
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
    topic_context: &(impl Context + Send + Sync),
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
                                        Ok(Candidate::new(Target(candidate), result, original_word_id))
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

    fn vote_for_origin<'a>(topic_model: &'a impl BasicTopicModel, topic_context: &(impl Context + Send + Sync), has_translation: bool, topic_id: usize, word_id: usize, probability: f64, voting: &(impl VotingMethod + Sync + Send)) -> Result<Candidate, TranslateErrorWithOrigin> {
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
                Ok(Candidate::new(Origin(word_id), result, word_id))
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
pub(crate) mod test {
    use std::num::NonZeroUsize;
    use crate::topicmodel::dictionary::{Dictionary, DictionaryMut};
    use crate::topicmodel::dictionary::direction::Invariant;
    use crate::topicmodel::topic_model::{TopicModel};
    use crate::topicmodel::vocabulary::{SearchableVocabulary, Vocabulary};
    use crate::translate::KeepOriginalWord::Never;
    use crate::translate::{translate_topic_model_without_provider, TranslateConfig};
    use crate::voting::spy::{IntoSpy};
    use Extend;
    use crate::voting::BuildInVoting;

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
                vec![0.02, 0.002, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001],
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

        // for (id, (candidate_id, candidate_prob, result), voters) in config.voting.spy_history().lock().unwrap().iter() {
        //     println!("Topic: {id}");
        //     println!("  Candidate: {candidate_id} ({candidate_prob})");
        //     println!("  Result: {result:?}");
        //     println!("  Voters:");
        //     for (voter_id, voter_score) in voters {
        //         println!("    {voter_id} ({voter_score})")
        //     }
        // }

        // model_a.show_10().unwrap();
        model_b.show_10().unwrap();
    }
}

pub(crate) fn register_py_translate(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<KeepOriginalWord>()?;
    Ok(())
}