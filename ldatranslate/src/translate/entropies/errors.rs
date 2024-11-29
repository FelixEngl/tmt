use ndarray_stats::errors::{EmptyInput, MultiInputError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EntropyError<A> {
    #[error(transparent)]
    EmptyInput(#[from] EmptyInput),
    #[error(transparent)]
    MultiInputError(#[from] MultiInputError),

    #[error("Failed to cast the float {value} to {typ}.")]
    FloatCastError {
        value: f64,
        typ: &'static str,
    },
    #[error("The parameter {name} has the value {value} which is illegal! {explanation:?}")]
    IllegalParameterError {
        value: A,
        name: &'static str,
        explanation: Option<&'static str>,
    }
}

#[derive(Debug, Error)]
pub enum EntropyWithAlphaError<A1, A2> {
    #[error("Failed to cast {value} to {typ}.")]
    CastError {
        value: A2,
        typ: &'static str,
    },
    #[error(transparent)]
    EntropyError(EntropyError<A1>),
}

impl<A1, A2, T> From<T> for EntropyWithAlphaError<A1, A2> where T: Into<EntropyError<A1>> {
    fn from(value: T) -> Self {
        Self::EntropyError(value.into())
    }
}

