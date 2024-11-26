use crate::topicmodel::dictionary::BasicDictionaryWithVocabulary;
use crate::topicmodel::vocabulary::SearchableVocabulary;
use crate::translate::TranslatableTopicMatrix;
use crate::variable_provider::AsVariableProviderError;
use crate::variable_provider::{VariableProvider, VariableProviderResult};
use evalexpr::{Context, ContextWithMutableVariables, EvalexprNumericTypesConvert};
use std::hash::Hash;

pub trait VariableProviderOut<NumericTypes: EvalexprNumericTypesConvert>: Sync + Send {
    fn provide_global(&self, target: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) -> VariableProviderResult<(), NumericTypes>;
    fn provide_for_topic(&self, topic_id: usize, target: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) -> VariableProviderResult<(), NumericTypes>;
    fn provide_for_word_a(&self, word_id: usize, target: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) -> VariableProviderResult<(), NumericTypes>;
    fn provide_for_word_b(&self, word_id: usize, target: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) -> VariableProviderResult<(), NumericTypes>;
    fn provide_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, target: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) -> VariableProviderResult<(), NumericTypes>;
    fn provide_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, target: &mut (impl ContextWithMutableVariables + Context<NumericTypes=NumericTypes>)) -> VariableProviderResult<(), NumericTypes>;
}


pub trait AsVariableProvider<T> {
    fn as_variable_provider_for<'a, NumericTypes, Target, D, Voc>(
        &self,
        topic_model: &'a Target,
        dictionary: &'a D
    ) -> Result<VariableProvider<NumericTypes>, AsVariableProviderError>
    where
        NumericTypes: EvalexprNumericTypesConvert,
        T: Hash + Eq,
        Voc: SearchableVocabulary<T>,
        D: BasicDictionaryWithVocabulary<Voc>,
        Target: TranslatableTopicMatrix<T, Voc>;
}
