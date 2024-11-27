use std::hash::Hash;
use thiserror::Error;
use ldatranslate_voting::variable_provider::{VariableProvider};
use ldatranslate_topicmodel::dictionary::BasicDictionaryWithVocabulary;
use ldatranslate_topicmodel::vocabulary::SearchableVocabulary;
use crate::translate::TranslatableTopicMatrix;

pub trait AsVariableProvider<T> {
    fn as_variable_provider_for<'a, Target, D, Voc>(
        &self,
        topic_model: &'a Target,
        dictionary: &'a D
    ) -> Result<VariableProvider, AsVariableProviderError>
    where
        T: Hash + Eq,
        Voc: SearchableVocabulary<T>,
        D: BasicDictionaryWithVocabulary<Voc>,
        Target: TranslatableTopicMatrix<T, Voc>;
}

#[derive(Debug, Error)]
#[error("AsVariableProviderError({0})")]
#[repr(transparent)]
pub struct AsVariableProviderError(pub String);
