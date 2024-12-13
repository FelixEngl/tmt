use std::error::Error;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;
use evalexpr::{Value};
use thiserror::Error;
use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithVocabulary, DictionaryWithMeta};
use ldatranslate_topicmodel::dictionary::metadata::ex::{MetadataManagerEx};
use ldatranslate_topicmodel::dictionary::metadata::MetadataManager;
use ldatranslate_topicmodel::language_hint::LanguageHint;
use ldatranslate_topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary, SearchableVocabulary};
use ldatranslate_voting::variable_provider::{VariableProviderError};
use ldatranslate_voting::traits::VotingMethodMarker;
pub use crate::translate::*;
use ldatranslate_voting::VotingExpressionError;
use crate::variable_provider::{AsVariableProviderError};

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
    /// Declares a field that boosts the score iff present.
    pub boost_with: Option<Value>
}

impl<V> VoteConfig<V>
where V: VotingMethodMarker {
    pub fn new(voting: V, epsilon: Option<f64>, threshold: Option<f64>, top_candidate_limit: Option<NonZeroUsize>, boost_with: Option<Value>) -> Self {
        Self { epsilon, voting, threshold, top_candidate_limit, boost_with }
    }
}

impl<'a, V> Clone for VoteConfig<V>
where V: VotingMethodMarker + Clone {
    fn clone(&self) -> Self {
        Self {
            voting: self.voting.clone(),
            epsilon: self.epsilon,
            threshold: self.threshold,
            top_candidate_limit: self.top_candidate_limit,
            boost_with: self.boost_with.clone()
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
    VariableProvider(#[from] VariableProviderError),
    #[error(transparent)]
    VotingExpression(#[from] VotingExpressionError)
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
}

impl<'a, T, V> DictBridge<'a, T, V> where V: BasicVocabulary<T> + AnonymousVocabulary {
    pub fn get_meta_for_voc_id(&self, voc_id: usize) -> Option<<MetadataManagerEx as MetadataManager>::Reference<'a>> {
        unsafe{self.voc_to_dict.get_unchecked(voc_id)}.as_ref().and_then(|value| {
            self.dictionary.get_meta_for_a(*value)
        })
    }
}

// pub fn meta_to_topic_association_voting<'a, Target, T, V, Voc, P>(
//     target: &'a Target,
//     dictionary: &'a DictionaryWithMeta<T, Voc, MetadataManagerEx>,
//     translate_config: &VoteConfig<V>,
//     provider: Option<&P>
// ) -> Result<Vec<Vec<f64>>, VoteError<'a>>
// where
//     T: Hash + Eq + Ord + Clone + Send + Sync + 'a,
//     V: VotingMethodMarker,
//     Voc: AnonymousVocabulary + VocabularyMut<T> + SearchableVocabulary<T> + MappableVocabulary<T> + Clone + Send + Sync + for<'b> FromIterator<&'b T> + 'a,
//     Target: TranslatableTopicMatrixWithCreate<T, Voc>,
//     P: AsVariableProvider<T>,
// {
//     if let Some(lang_model) = target.vocabulary().language() {
//         if let (Some(lang_a), lang_b) = dictionary.language_direction_a_to_b() {
//             if lang_model != lang_a {
//                 let lang_b = lang_b.cloned().unwrap_or_else(|| LanguageHint::new("###"));
//                 return Err(
//                     VoteError::IncompatibleLanguages {
//                         lang_a,
//                         lang_b,
//                         lang_model
//                     }
//                 )
//             }
//         }
//     }
//
//     let bridge = DictBridge::new(
//         dictionary,
//         target.vocabulary(),
//     );
//
//     let provider = if let Some(provider) = provider {
//         Some(provider.as_variable_provider_for(target, dictionary))
//     } else {
//         None
//     }.transpose()?;
//
//     let epsilon = if let Some(value) = translate_config.epsilon {
//         value
//     } else {
//         target.matrix().iter().flat_map(|value| value.iter()).fold(
//             f64::MAX,
//             |old, other| {
//                 old.min(*other)
//             }
//         ) - f64::EPSILON
//     };
//
//     let mut topic_context: HashMapContext<TMTNumericTypes> = context_map! {
//         EPSILON => float epsilon,
//         VOCABULARY_SIZE_A => int bridge.dictionary.voc_a().len() as i64,
//         VOCABULARY_SIZE_B => int bridge.dictionary.voc_b().len() as i64,
//         COUNT_OF_VOTERS => int target.vocabulary().len() as i64,
//     }.unwrap();
//
//     if let Some(ref boost) = translate_config.boost_with {
//         topic_context.set_value(
//             BOOST_SCORE.to_string(),
//             boost.clone().clone().into_with().unwrap()
//         ).unwrap();
//     }
//
//     if let Some(ref provider) = provider {
//         provider.provide_global(&mut topic_context)?;
//     }
//
//     let topic_context =
//         topic_context.to_static_with(EmptyContextWithBuiltinFunctions::default());
//
//     let domain_counts = bridge.dictionary.metadata().dict_meta_counts();
//
//
//     let norm_value = domain_counts.ref_a().sum() as f64;
//     let candidate_ids_context: Vec<_> = (0..META_DICT_ARRAY_LENTH).into_iter().map(|candidate_id| {
//         context_map! {
//             CANDIDATE_ID => conv candidate_id
//         }
//     }).collect::<Result<Vec<_>, _>>().unwrap().into();
//
//     use ldatranslate_translate::TopicMetas;
//
//
//     let result = target
//         .matrix()
//         .par_iter()
//         .zip_eq(target.matrix_meta().par_iter())
//         .enumerate()
//         .map(|(topic_id, (topic, meta))| {
//             let mut topic_context_2 = context_map! {
//                 TOPIC_MAX_PROBABILITY => as float meta.max_score(),
//                 TOPIC_MIN_PROBABILITY => as float meta.min_score(),
//                 TOPIC_AVG_PROBABILITY => as float meta.avg_score(),
//                 TOPIC_SUM_PROBABILITY => as float meta.sum_score(),
//                 TOPIC_ID => as int topic_id
//             }.unwrap();
//             meta.extend_context(&mut topic_context_2);
//
//             if let Some(provider) = provider.as_ref() {
//                 match provider.provide_for_topic(topic_id, &mut topic_context_2) {
//                     Ok(_) => {
//                         Ok(topic_context_2.to_owning_with(&topic_context))
//                     }
//                     Err(err) => {
//                         Err(VoteError::VariableProvider(err))
//                     }
//                 }
//             } else {
//                 Ok(topic_context_2.to_owning_with(&topic_context))
//             }.and_then(|context| {
//
//                 let mut meta_info_to_word_to_normalized_count: [_; META_DICT_ARRAY_LENTH] = std::array::from_fn(|_| Vec::with_capacity(topic.len()));
//                 for (original_word_id, _) in topic.iter().enumerate() {
//                     if let Some(value) = bridge.get_meta_for_voc_id(original_word_id) {
//                         let domain_count = value.domain_count();
//                         for (i, value) in domain_count.iter().enumerate() {
//                             meta_info_to_word_to_normalized_count[i].push((*value) as f64 / norm_value);
//                         }
//                     }
//                 }
//                 meta_info_to_word_to_normalized_count.par_iter().enumerate().map(|(candidate_id, voters)| {
//                     vote_for_domain_in_topic(
//                         target,
//                         topic_id,
//                         topic,
//                         voters,
//                         context.combine_with(unsafe{candidate_ids_context.get_unchecked(candidate_id)}),
//                         translate_config,
//                         provider.as_ref(),
//                     ).map_err(VoteError::from)
//                 }).collect::<Result<Vec<_>, _>>()
//             })
//         }).collect::<Result<Vec<_>, _>>()?;
//
//     Ok(result)
// }

// fn vote_for_domain_in_topic<'a, Target, T, V, Voc, P>(
//     target: &'a Target,
//     topic_id: usize,
//     voters: &<Target::TopicToVoterMatrix<'_> as TopicModelLikeMatrix>::TopicLike,
//     candidate_scores: &(impl TopicLike + Send + Sync),
//     topic_context: impl Context<NumericTypes=TMTNumericTypes> + Send + Sync + IterateVariablesContext,
//     config: &VoteConfig<V>,
//     provider: Option<&P>,
// ) -> Result<f64, VoteError<'a>>
// where V: VotingMethodMarker,
//       Voc: SearchableVocabulary<T> + AnonymousVocabulary ,
//       Target: TranslatableTopicMatrix<T, Voc>,
//       P: VariableProviderOut,
//       T: Hash + Eq + Ord + Clone + Send + Sync,
// {
//     let mapped = (0..voters.len())
//         .into_iter()
//         .filter_map(|voter_id_a_retrans| {
//             target.get_voter_meta(topic_id, voter_id_a_retrans)
//         });
//     let mapped = if let Some(threshold) = config.threshold {
//         mapped.filter(|value| value.score() >= threshold).collect_vec()
//     } else {
//         mapped.collect_vec()
//     };
//
//
//     let mut voters = mapped.iter().zip_eq(candidate_scores.iter()).map(|(voter_a, domain_score_norm)|{
//         let mut context_voter_a = context_map! {
//             SCORE_CANDIDATE => as float voter_a.score(),
//             SCORE_DOMAIN => as float *domain_score_norm,
//             SCORE => as float  voter_a.score(),
//             VOTER_ID => as int  voter_a.voter_id(),
//             RECIPROCAL_RANK => as float 1./ voter_a.importance() as f64,
//             REAL_RECIPROCAL_RANK => as float 1./ voter_a.rank() as f64,
//             RANK => as int voter_a.rank(),
//             IMPORTANCE => as int voter_a.importance(),
//         }.unwrap();
//         voter_a.extend_context(&mut context_voter_a);
//         if let Some(provider) = provider {
//             match provider.provide_for_word_a(voter_a.voter_id(), &mut context_voter_a) {
//                 Ok(_) => {
//                     match provider.provide_for_word_in_topic_a(topic_id, voter_a.voter_id(), &mut context_voter_a) {
//                         Ok(_) => {
//                             Ok(context_voter_a.to_owning_with(&topic_context))
//                         }
//                         Err(err) => {Err(err)}
//                     }
//                 }
//                 Err(err) => {
//                     Err(err)
//                 }
//             }
//         } else {
//             Ok(context_voter_a.to_owning_with(&topic_context))
//         }
//     }).collect::<Result<Vec<_>, _>>()?;
//     drop(mapped);
//     let mut context = topic_context.as_empty_mutable();
//     Ok(config.voting.execute_to_f64(
//         &mut context,
//         voters.as_mut_slice()
//     )?)
// }

#[cfg(test)]
mod test {

}