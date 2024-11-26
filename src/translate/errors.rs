use std::error::Error;
use std::marker::PhantomData;
use evalexpr::{EvalexprNumericTypesConvert};
use thiserror::Error;
use crate::topicmodel::language_hint::LanguageHint;
use crate::variable_provider::{AsVariableProviderError, VariableProviderError};
use crate::voting::{VotingExpressionError, VotingResult};

/// An error that happened while translating
#[derive(Debug, Error)]
pub enum TranslateError<'a, NumericTypes: EvalexprNumericTypesConvert> {
    #[error(transparent)]
    VotingError(#[from] VotingExpressionError<NumericTypes>),
    #[error(transparent)]
    WithOrigin(#[from] TranslateErrorWithOrigin<NumericTypes>),
    #[error(transparent)]
    ProviderError(#[from] VariableProviderError<NumericTypes>),
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
pub struct TranslateErrorWithOrigin<NumericTypes: EvalexprNumericTypesConvert> {
    pub topic_id: usize,
    pub word_id: usize,
    pub source: Box<dyn Error + Send + Sync>,
    _phantom: PhantomData<NumericTypes>,
}

impl<NumericTypes: EvalexprNumericTypesConvert> TranslateErrorWithOrigin<NumericTypes> {
    pub fn new(source: Box<dyn Error + Send + Sync>, word_id: usize, topic_id: usize) -> Self {
        Self { topic_id, word_id, source, _phantom: PhantomData }
    }
}

unsafe impl<NumericTypes: EvalexprNumericTypesConvert> Send for TranslateErrorWithOrigin<NumericTypes> {}

/// Trait for mapping to map something to something that supports a context for topic_id and word_id
pub(super) trait MapsToTranslateErrorWithOrigin {
    type Return;
    fn originates_at(self, topic_id: usize, word_id: usize) -> Self::Return;
}

impl<T, NumericTypes: EvalexprNumericTypesConvert> MapsToTranslateErrorWithOrigin for VotingResult<T, NumericTypes> {
    type Return = Result<T, TranslateErrorWithOrigin<NumericTypes>>;

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
                        source: err.into(),
                        _phantom: PhantomData,
                    }
                )
            }
        }
    }
}

impl<NumericTypes: EvalexprNumericTypesConvert> MapsToTranslateErrorWithOrigin for VotingExpressionError<NumericTypes> {
    type Return = TranslateErrorWithOrigin<NumericTypes>;

    fn originates_at(self, topic_id: usize, word_id: usize) -> Self::Return {
        TranslateErrorWithOrigin {
            topic_id,
            word_id,
            source: self.into(),
            _phantom: PhantomData,
        }
    }
}