use evalexpr::EvalexprError;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("AsVariableProviderError({0})")]
#[repr(transparent)]
pub struct AsVariableProviderError(pub String);


#[derive(Debug, Clone, Error)]
pub enum VariableProviderError {
    #[error("{topic_id} is not in 0..{topic_count}")]
    TopicNotFound {
        topic_id: usize,
        topic_count: usize
    },
    #[error("{word_id} is not in 0..{word_count}")]
    WordNotFound {
        word_id: usize,
        word_count: usize
    },
    #[error(transparent)]
    EvalExpressionError(#[from] EvalexprError)
}
