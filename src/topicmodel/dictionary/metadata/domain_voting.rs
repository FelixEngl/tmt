use std::error::Error;
use std::hash::Hash;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::sync::Arc;
use evalexpr::{context_map, Context, EmptyContextWithBuiltinFunctions, IterateVariablesContext};
use itertools::Itertools;
use rayon::prelude::*;
use thiserror::Error;
use crate::toolkit::evalexpr::{CombineableContext, ContextExtender};
use crate::topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, DictionaryFilterable, DictionaryWithMeta, DictionaryWithVocabulary};
use crate::topicmodel::dictionary::metadata::ex::{DomainCounts, MetadataManagerEx};
use crate::topicmodel::dictionary::metadata::MetadataManager;
use crate::topicmodel::language_hint::LanguageHint;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary, MappableVocabulary, SearchableVocabulary, VocabularyMut};
use crate::translate::{TopicModelLikeMatrix, TranslatableTopicMatrix, TranslatableTopicMatrixWithCreate};
use crate::variable_provider::{AsVariableProvider, AsVariableProviderError, VariableProviderError, VariableProviderOut};
use crate::variable_provider::variable_names::*;
use crate::voting::traits::VotingMethodMarker;
pub use crate::translate::*;

/// The config for a translation
#[derive(Debug)]
pub struct VoteConfig<V: VotingMethodMarker> {
    /// The voting to be used
    pub voting: V,
    /// The epsilon to be used, if it is none it is determined heuristically.
    pub epsilon: Option<f64>,
    /// The threshold of the probabilities allowed to be used as voters
    pub threshold: Option<f64>,
    /// Limits the number of accepted candidates to N. If not set keep all.
    pub top_candidate_limit: Option<NonZeroUsize>,
}

impl<V> VoteConfig<V>
where V: VotingMethodMarker {
    pub fn new(voting: V, epsilon: Option<f64>, threshold: Option<f64>, top_candidate_limit: Option<NonZeroUsize>) -> Self {
        Self { epsilon, voting, threshold, top_candidate_limit }
    }
}

impl<'a, V> Clone for VoteConfig<V>
where V: VotingMethodMarker + Clone {
    fn clone(&self) -> Self {
        Self {
            voting: self.voting.clone(),
            epsilon: self.epsilon,
            threshold: self.threshold,
            top_candidate_limit: self.top_candidate_limit
        }
    }
}



#[derive(Debug, Error)]
pub enum VoteError<'a> {
    #[error("The dictionary has a translation direction from {lang_a} to {lang_b}, but the topic is in {lang_b}!")]
    IncompatibleLanguages {
        lang_a: &'a LanguageHint,
        lang_b: LanguageHint,
        lang_model: &'a LanguageHint,
    },
    #[error(transparent)]
    AsVariableProviderFailed(#[from] AsVariableProviderError),
    #[error(transparent)]
    WithOrigin(#[from] VoteErrorWithOrigin),
    #[error(transparent)]
    VariableProvider(#[from] VariableProviderError)
}

#[derive(Debug, Error)]
#[error("Failed with an error! ({topic_id}, {word_id}) {source}")]
pub struct VoteErrorWithOrigin {
    pub topic_id: usize,
    pub word_id: usize,
    pub source: Box<dyn Error + Send + Sync>
}

#[derive(Clone)]
struct DictBridge<'a, T, V> {
    pub dictionary: &'a DictionaryWithMeta<T, V, MetadataManagerEx>,
    voc_to_dict: Arc<Vec<Option<usize>>>,
    dict_to_voc: Arc<Vec<Option<usize>>>,
}

unsafe impl<'a, T, V> Send for DictBridge<'a, T, V>{}
unsafe impl<'a, T, V> Sync for DictBridge<'a, T, V>{}
impl<'a, T, V> DictBridge<'a, T, V> where V: SearchableVocabulary<T> + AnonymousVocabulary, T: Eq + Hash {
    pub fn new<Voc>(dictionary: &'a DictionaryWithMeta<T, V, MetadataManagerEx>, voc: &Voc) -> Self
    where
        Voc: SearchableVocabulary<T>
    {
        let mut dict_to_voc = vec![None; dictionary.voc_a().len()];
        for (id, value) in dictionary.voc_a().iter_entries() {
            dict_to_voc[id] = voc.get_id(value);
        }
        let mut voc_to_dict = vec![None; voc.len()];
        for (id, value) in voc.iter_entries() {
            voc_to_dict[id] = dictionary.voc_a().get_id(value);
        }

        Self {
            dictionary,
            dict_to_voc: dict_to_voc.into(),
            voc_to_dict: voc_to_dict.into(),
        }
    }

    pub fn get_meta_for_voc_id(&self, voc_id: usize) -> Option<<MetadataManagerEx as MetadataManager>::Reference<'a>> {
        unsafe{self.voc_to_dict.get_unchecked(voc_id)}.as_ref().and_then(|value| {
            self.dictionary.get_meta_for_a(*value)
        })
    }
}

pub fn vote_for_domains<'a, Target, T, V, Voc, P>(
    target: &'a Target,
    dictionary: &'a DictionaryWithMeta<T, Voc, MetadataManagerEx>,
    translate_config: &VoteConfig<V>,
    provider: Option<&P>
) -> Result<(), VoteError<'a>>
where
    T: Hash + Eq + Ord + Clone + Send + Sync + 'a,
    V: VotingMethodMarker,
    Voc: AnonymousVocabulary + VocabularyMut<T> + SearchableVocabulary<T> + MappableVocabulary<T> + Clone + Send + Sync + for<'b> FromIterator<&'b T> + 'a,
    Target: TranslatableTopicMatrixWithCreate<T, Voc>,
    P: AsVariableProvider<T>
{
    if let Some(lang_model) = target.vocabulary().language() {
        if let (Some(lang_a), lang_b) = dictionary.language_direction_a_to_b() {
            if lang_model != lang_a {
                let lang_b = lang_b.cloned().unwrap_or_else(|| LanguageHint::new("###"));
                return Err(
                    VoteError::IncompatibleLanguages {
                        lang_a,
                        lang_b,
                        lang_model
                    }
                )
            }
        }
    }

    // let dictionary_new = dictionary.filter_by_values(
    //     |a| target.vocabulary().contains_value(a),
    //     |_| true,
    // );

    let bridge = DictBridge::new(
        dictionary,
        target.vocabulary(),
    );

    let provider = if let Some(provider) = provider {
        Some(provider.as_variable_provider_for(target, dictionary))
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
        VOCABULARY_SIZE_A => bridge.dictionary.voc_a().len() as i64,
        VOCABULARY_SIZE_B => bridge.dictionary.voc_b().len() as i64,
    }.unwrap();

    if let Some(ref provider) = provider {
        provider.provide_global(&mut topic_context)?;
    }

    let topic_context =
        topic_context.to_static_with(EmptyContextWithBuiltinFunctions);

    let (domain_vectors_sum, domain_vectors_appearance) = bridge.dictionary.metadata().domain_count();


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
                        Ok(topic_context_2.to_static_with(topic_context.clone()))
                    }
                    Err(err) => {
                        Err(VoteError::VariableProvider(err))
                    }
                }
            } else {
               Ok(topic_context_2.to_static_with(topic_context.clone()))
            }.and_then(|context| {
                vote_for_domain_in_topic(
                    target,
                    bridge.clone(),
                    topic_id,
                    topic,
                    context,
                    translate_config,
                    provider.as_ref(),
                    &domain_vectors_sum,
                    &domain_vectors_appearance,
                ).map_err(VoteError::from)
            })
        }).collect::<Result<Vec<_>, _>>()?;

    todo!()
}

fn vote_for_domain_in_topic<Target, T, V, Voc, P>(
    target: &Target,
    dictionary: DictBridge<T, Voc>,
    topic_id: usize,
    topic: &<Target::TopicToVoterMatrix as TopicModelLikeMatrix>::TopicLike,
    topic_context: impl Context + Send + Sync + IterateVariablesContext,
    config: &VoteConfig<V>,
    provider: Option<&P>,
    domain_vec_sum: &DomainCounts,
    domain_vec_count: &DomainCounts
) -> Result<(), VoteErrorWithOrigin>
where V: VotingMethodMarker,
      Voc: SearchableVocabulary<T> ,
      Target: TranslatableTopicMatrix<T, Voc>,
      P: VariableProviderOut,
      T: Hash + Eq + Ord + Clone + Send + Sync,
{
    // topic.par_iter().enumerate().filter_map(|(original_word_id, probability)| {
    //     todo!()
    // });
    todo!()
}

fn vote_for_domain_single_word(

) {

}