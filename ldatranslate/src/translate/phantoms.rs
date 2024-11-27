use ldatranslate_topicmodel::dictionary::BasicDictionaryWithVocabulary;
use ldatranslate_topicmodel::vocabulary::SearchableVocabulary;
use crate::translate::TranslatableTopicMatrix;
use crate::variable_provider::{AsVariableProvider, AsVariableProviderError};
use std::hash::Hash;
use std::marker::PhantomData;
use ldatranslate_voting::variable_provider::VariableProvider;

pub(super) struct DummyAsVariableProvider<T> {
    _phantom: PhantomData<T>
}

impl<T> AsVariableProvider<T> for DummyAsVariableProvider<T> {
    fn as_variable_provider_for<'a, Target, D, Voc>(
        &self,
        _topic_model: &'a Target,
        _dictionary: &'a D
    ) -> Result<VariableProvider, AsVariableProviderError>
    where
        T: Hash + Eq,
        Voc: SearchableVocabulary<T>,
        D: BasicDictionaryWithVocabulary<Voc>,
        Target: TranslatableTopicMatrix<T, Voc>
    {
        unreachable!("This provider should never be called!")
    }
}
