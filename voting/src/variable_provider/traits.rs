use crate::topicmodel::dictionary::BasicDictionaryWithVocabulary;
use crate::topicmodel::vocabulary::SearchableVocabulary;
use crate::translate::TranslatableTopicMatrix;
use crate::variable_provider::AsVariableProviderError;
use crate::variable_provider::{VariableProvider, VariableProviderResult};
use evalexpr::ContextWithMutableVariables;
use std::hash::Hash;
use crate::voting::constants::TMTNumericTypes;

pub trait VariableProviderOut: Sync + Send {
    fn provide_global(&self, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
    fn provide_for_topic(&self, topic_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
    fn provide_for_word_a(&self, word_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
    fn provide_for_word_b(&self, word_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
    fn provide_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
    fn provide_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
}


