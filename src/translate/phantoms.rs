use crate::topicmodel::dictionary::BasicDictionaryWithVocabulary;
use crate::topicmodel::vocabulary::SearchableVocabulary;
use crate::translate::TranslatableTopicMatrix;
use crate::variable_provider::{AsVariableProvider, AsVariableProviderError, VariableProvider};
use std::hash::Hash;
use std::marker::PhantomData;
use evalexpr::EvalexprNumericTypesConvert;

pub(super) struct DummyAsVariableProvider<T> {
    _phantom: PhantomData<T>
}

impl<T> AsVariableProvider<T> for DummyAsVariableProvider<T> {
    fn as_variable_provider_for<'a, NumericTypes, Target, D, Voc>(
        &self,
        _topic_model: &'a Target,
        _dictionary: &'a D
    ) -> Result<VariableProvider<NumericTypes>, AsVariableProviderError>
    where
        NumericTypes: EvalexprNumericTypesConvert,
        T: Hash + Eq,
        Voc: SearchableVocabulary<T>,
        D: BasicDictionaryWithVocabulary<Voc>,
        Target: TranslatableTopicMatrix<T, Voc>
    {
        unreachable!("This provider should never be called!")
    }
}
