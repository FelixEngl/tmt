use evalexpr::EvalexprError;
use pyo3::exceptions::PyValueError;
use pyo3::PyErr;
use thiserror::Error;



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

impl From<VariableProviderError> for PyErr {
    fn from(value: VariableProviderError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}