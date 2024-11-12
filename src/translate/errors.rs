use std::error::Error;
use thiserror::Error;
use crate::topicmodel::language_hint::LanguageHint;
use crate::variable_provider::{AsVariableProviderError, VariableProviderError};
use crate::voting::{VotingExpressionError, VotingResult};

/// An error that happened while translating
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
    },
    #[error(transparent)]
    AsVariableProviderFailed(#[from] AsVariableProviderError)
}

#[derive(Debug, Error)]
#[error("Failed with an error! ({topic_id}, {word_id}) {source}")]
pub struct TranslateErrorWithOrigin {
    pub topic_id: usize,
    pub word_id: usize,
    pub source: Box<dyn Error + Send + Sync>
}

/// Trait for mapping to map something to something that supports a context for topic_id and word_id
pub(super) trait MapsToTranslateErrorWithOrigin {
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