use std::hash::Hash;
use evalexpr::ContextWithMutableVariables;
use crate::topicmodel::dictionary::{DictionaryMut, DictionaryWithVocabulary, FromVoc};
use crate::topicmodel::model::{TopicModelWithDocumentStats, TopicModelWithVocabulary};
use crate::topicmodel::vocabulary::{MappableVocabulary, VocabularyMut};
use crate::variable_provider::AsVariableProviderError;
use crate::variable_provider::{VariableProvider, VariableProviderResult};

pub trait VariableProviderOut: Sync + Send {
    fn provide_global(&self, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_topic(&self, topic_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_word_a(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_word_b(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
}


pub trait AsVariableProvider<T> {
    fn as_variable_provider_for<'a, Model, D, Voc>(&self, topic_model: &'a Model, dictionary: &'a D) -> Result<VariableProvider, AsVariableProviderError> where
        T: Hash + Eq + Ord + Clone,
        Voc: VocabularyMut<T> + MappableVocabulary<T> + Clone + 'a,
        D: DictionaryWithVocabulary<T, Voc> + DictionaryMut<T, Voc> + FromVoc<T, Voc>,
        Model: TopicModelWithVocabulary<T, Voc> + TopicModelWithDocumentStats;
}
