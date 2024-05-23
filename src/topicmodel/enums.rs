use std::fmt::Debug;
use std::path::PathBuf;
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::topicmodel::io::TopicModelIOError;
use crate::topicmodel::vocabulary::LoadVocabularyError;

#[derive(Debug, Copy, Clone, Display, AsRefStr, EnumString)]
pub enum TopicModelVersion {
    V1,
    V2
}


#[derive(Debug, Error)]
pub enum WriteError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    WriterError(#[from] TopicModelIOError),
    #[error("The topic model is already finished and saved!")]
    AlreadyFinished
}

#[derive(Debug, Error)]
pub enum ReadError<E: Debug> {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Failed at {line}:{position} with {err:?}")]
    ParseFloat {
        line: usize,
        position: usize,
        #[source]
        err: std::num::ParseFloatError
    },
    #[error("Failed at {line}:{position} with {err:?}")]
    ParseInt {
        line: usize,
        position: usize,
        #[source]
        err: std::num::ParseIntError
    },
    #[error(transparent)]
    StrumParse(#[from] strum::ParseError),
    #[error("Some kind of error in the vocabulary")]
    VocabularyError(#[from] LoadVocabularyError<E>),
    #[error(transparent)]
    ReaderError(#[from] TopicModelIOError),
    #[error("The model at {0} is not finished!")]
    NotFinishedError(PathBuf)
}