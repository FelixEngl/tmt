//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::fmt::Debug;
use std::path::PathBuf;
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;
use crate::topicmodel::io::TopicModelIOError;
use crate::topicmodel::vocabulary::LoadVocabularyError;

/// The model storeing version.
#[derive(Debug, Copy, Clone, Display, AsRefStr, EnumString)]
pub enum TopicModelVersion {
    V1,
    V2
}

/// The errors while writing
#[derive(Debug, Error)]
pub enum WriteError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    WriterError(#[from] TopicModelIOError),
    #[error("The topic model is already finished and saved!")]
    AlreadyFinished
}

/// The errors while reading
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