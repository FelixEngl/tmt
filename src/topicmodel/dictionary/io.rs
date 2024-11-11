use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use sealed::sealed;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use thiserror::Error;
use crate::topicmodel::dictionary::BasicDictionary;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WriteMode {
    Binary {
        compressed: bool
    },
    Json {
        compressed: bool,
        pretty: bool
    }
}

impl Default for WriteMode {
    fn default() -> Self {
        Self::Binary {
            compressed: true
        }
    }
}


impl WriteMode {

    pub const fn json(
        compressed: bool,
        pretty: bool
    ) -> WriteMode {
        WriteMode::Json {
            pretty,
            compressed
        }
    }

    pub const fn binary(
        compressed: bool
    ) -> WriteMode {
        WriteMode::Binary {
            compressed
        }
    }

    pub const FILE_ENDINGS: [&'static str; 4] = [
        "dat.zst",
        "dat",
        "json.zst",
        "json"
    ];

    pub const fn compressed(self) -> bool {
        match self {
            WriteMode::Binary { compressed } => compressed,
            WriteMode::Json { compressed, .. } => compressed
        }
    }

    fn set_compressed(&mut self, compressed_flag: bool) {
        match self {
            WriteMode::Binary { compressed, .. } => {
                *compressed = compressed_flag
            }
            WriteMode::Json { compressed, .. } => {
                *compressed = compressed_flag
            }
        }
    }

    pub const fn associated_extension(&self) -> &'static str {
        match self {
            WriteMode::Binary { compressed: true, .. } => {
                Self::FILE_ENDINGS[0]
            }
            WriteMode::Binary { compressed: false, .. } => {
                Self::FILE_ENDINGS[1]
            }
            WriteMode::Json { compressed: true, .. } => {
                Self::FILE_ENDINGS[2]
            }
            WriteMode::Json { compressed: false, .. } => {
                Self::FILE_ENDINGS[3]
            }
        }
    }

    pub fn from_path(path: impl AsRef<Utf8Path>) -> Option<WriteMode> {
        match path.as_ref().extension()? {
            "zst" => {
                let path = path.as_ref().with_extension("");
                let mut mode = Self::from_path(path)?;
                mode.set_compressed(true);
                Some(mode)
            }
            "dat" => {
                Some(
                    WriteMode::Binary {
                        compressed: false,
                    }
                )
            }
            "json" => {
                Some(
                    WriteMode::Json {
                        compressed: false,
                        pretty: false
                    }
                )
            }
            _ => None
        }
    }
}


#[derive(Debug, Error)]
pub enum IoError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    BinaryError(#[from] bincode::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}


#[sealed]
pub trait WriteableDictionary: BasicDictionary + Serialize {
    fn write_to_path_with_extension(&self, path: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf, IoError> {
        if let Some(mode) = WriteMode::from_path(path.as_ref()) {
            self.write_to_path(mode, path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Expected a valid file extension [{}], but got {:?}",
                    WriteMode::FILE_ENDINGS.iter().join(", "),
                    path.as_ref().extension()
                )
            ).into())
        }
    }
    fn write_to_path(&self, mode: WriteMode, path: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf, IoError>;
    fn to_writer(&self, mode: WriteMode, writer: impl Write) -> Result<(), IoError>;
}

#[sealed]
pub trait ReadableDictionary: BasicDictionary + DeserializeOwned + Sized {

    fn from_path_with_extension(path: impl AsRef<Utf8Path>) -> Result<Self, IoError> {
        if let Some(mode) = WriteMode::from_path(path.as_ref()) {
            Self::from_path(mode, path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Expected a valid file extension [{}], but got {:?}",
                    WriteMode::FILE_ENDINGS.iter().join(", "),
                    path.as_ref().extension()
                )
            ).into())
        }
    }

    fn from_path(mode: WriteMode, path: impl AsRef<Utf8Path>) -> Result<Self, IoError>;
    fn from_reader(mode: WriteMode, writer: impl Read) -> Result<Self, IoError>;
}

#[sealed]
impl<D> WriteableDictionary for D where D: BasicDictionary + Serialize {

    fn write_to_path(&self, mode: WriteMode, path: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf, IoError> {
        let mut path = path.as_ref().to_path_buf();
        if !path
            .file_name()
            .map(|value| value.ends_with(mode.associated_extension()))
            .unwrap_or(false)
        {
            if path.file_name().is_none() {
                path.set_file_name("dictionary")
            }
            path.set_extension(mode.associated_extension());
        }
        let writer = File::options().write(true).create(true).truncate(true).open(&path)?;
        self.to_writer(mode, writer)?;
        Ok(path)
    }

    fn to_writer(&self, mode: WriteMode, writer: impl Write) -> Result<(), IoError> {
        let writer: Box<dyn Write> = if mode.compressed() {
            Box::new(zstd::Encoder::new(BufWriter::new(writer), 0)?)
        } else {
            Box::new(BufWriter::new(writer))
        };
        match mode {
            WriteMode::Binary { .. } => {
                bincode::serialize_into(writer, self)?;
            }
            WriteMode::Json { pretty, .. } => {
                if pretty {
                    serde_json::to_writer_pretty(writer, self)?;
                } else {
                    serde_json::to_writer(writer, self)?;
                }
            }
        }
        Ok(())
    }
}

#[sealed]
impl<D> ReadableDictionary for D where D: BasicDictionary + DeserializeOwned + Sized {

    fn from_path(mode: WriteMode, path: impl AsRef<Utf8Path>) -> Result<Self, IoError> {
        <Self as ReadableDictionary>::from_reader(
            mode,
            File::options().read(true).open(path.as_ref())?
        )
    }


    fn from_reader(mode: WriteMode, reader: impl Read) -> Result<Self, IoError> {
        let reader: Box<dyn Read> = if mode.compressed() {
            Box::new(zstd::Decoder::new(BufReader::new(reader))?)
        } else {
            Box::new(BufReader::new(reader))
        };
        match mode {
            WriteMode::Binary { .. } => {
                Ok(bincode::deserialize_from(reader)?)
            }
            WriteMode::Json { .. } => {
                Ok(serde_json::from_reader(reader)?)
            }
        }
    }
}