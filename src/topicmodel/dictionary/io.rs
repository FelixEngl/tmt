use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::str::FromStr;
use byte_unit::Byte;
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use sealed::sealed;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use thiserror::Error;
use crate::{define_py_literal};
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



#[derive(Debug, Error, Clone)]
pub enum WriteModeParseError {
    #[error("The value {0:?} is not valid for write mode because it needs either 'binary' or 'json'!")]
    NoTypeSet(String),
    #[error("The value {0:?} is not valid for write mode because it has both 'binary' and 'json' but only one is allowed!")]
    TooManyTypesSet(String),
    #[error("The value {0:?} contains an invalid parameter {1:?}.")]
    InvalidValue(String, String)
}

impl FromStr for WriteMode {
    type Err = WriteModeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut compressed = false;
        let mut pretty = None;
        let mut json = false;
        let mut binary = false;

        for value in s.split('+').map(str::trim) {
            match &value[0..1] {
                "j" => json = true,
                "b" => binary = true,
                "p" => pretty = Some(value),
                "c" => compressed = true,
                _ => return Err(WriteModeParseError::InvalidValue(s.to_string(), value.to_string()))
            }
        }

        if json == binary {
            if !json {
                Err(WriteModeParseError::NoTypeSet(s.to_string()))
            } else {
                Err(WriteModeParseError::TooManyTypesSet(s.to_string()))
            }
        } else {
            if json {
                Ok(
                    WriteMode::Json {
                        compressed,
                        pretty: pretty.is_some()
                    }
                )
            } else {
                if let Some(pretty) = pretty {
                    Err(WriteModeParseError::InvalidValue(s.to_string(), pretty.to_string()))
                } else {
                    Ok(
                        WriteMode::Binary {
                            compressed
                        }
                    )
                }
            }
        }
    }
}


define_py_literal!(
    pub WriteModeLiteral[
        "b",
        "binary",
        "b+c",
        "binary+compressed",
        "json",
        "j",
        "json+compressed",
        "j+c",
        "json+pretty",
        "j+p",
        "json+pretty+compressed",
        "j+p+c",
    ] into WriteMode
);


impl Default for WriteMode {
    fn default() -> Self {
        Self::Binary {
            compressed: true
        }
    }
}

// impl_py_stub! {
//     WriteMode {
//         output: {
//             builder()
//             .
//             .build_output()
//         }
//     }
// }


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
        let mut writer: Box<dyn Write> = if mode.compressed() {
            Box::new(zstd::Encoder::new(writer, 0)?)
        } else {
            Box::new(BufWriter::new(writer))
        };
        match mode {
            WriteMode::Binary { .. } => {
                bincode::serialize_into(&mut writer, self)?;
            }
            WriteMode::Json { pretty, .. } => {
                if pretty {
                    serde_json::to_writer_pretty(&mut writer, self)?;
                } else {
                    serde_json::to_writer(&mut writer, self)?;
                }
            }
        }
        writer.flush()?;
        Ok(())
    }
}

#[sealed]
impl<D> ReadableDictionary for D where D: BasicDictionary + DeserializeOwned + Sized {

    fn from_path(mode: WriteMode, path: impl AsRef<Utf8Path>) -> Result<Self, IoError> {
        let mut file = File::options().read(true).open(path.as_ref())?;

        const MAX_IN_MEMORY: u64 = match Byte::from_u64_with_unit(
            500,
            byte_unit::Unit::MB
        ) {
            Some(value) => {
                value.as_u64()
            }
            _ => unreachable!()
        };

        if mode.compressed() {
            let file_size = file.metadata().map_or(u64::MAX, |v| v.len());
            if file_size <= MAX_IN_MEMORY {
                let mut buffer = Vec::with_capacity(file_size as usize);
                file.read_to_end(&mut buffer)?;
                return <Self as ReadableDictionary>::from_reader(
                    mode,
                    Cursor::new(buffer)
                )
            }
        }
        <Self as ReadableDictionary>::from_reader(
            mode,
            file
        )

    }


    fn from_reader(mode: WriteMode, reader: impl Read) -> Result<Self, IoError> {
        let reader: Box<dyn Read> = if mode.compressed() {
            Box::new(zstd::Decoder::new(reader)?)
        } else {
            Box::new(BufReader::with_capacity(
                byte_unit::Byte::from_u64_with_unit(
                    250,
                    byte_unit::Unit::MB
                ).unwrap().as_u64() as usize,
                reader
            ))
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