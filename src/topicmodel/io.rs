use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, IoSlice, IoSliceMut, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::read::ZipFile;
use zip::{ZipArchive, ZipWriter};
use zip::write::{FileOptions};

#[derive(Debug, Error)]
pub enum TopicModelIOError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error("The path {0} was not found!")]
    PathNotFound(PathBuf),
    #[error("The path {0} is illegal!")]
    IllegalPath(PathBuf),
    #[error("The path {0} is unsupported!")]
    UnsupportedFileType(PathBuf),
    #[error("The path {0} already exists!")]
    AlreadyExists(PathBuf),
}


pub enum TopicModelFSRead {
    Zip {
        zip_archive: ZipArchive<BufReader<File>>
    },
    System {
        path_on_disc: PathBuf
    }
}


impl TopicModelFSRead {
    // pub fn open<P: AsRef<Path>>(path: P) -> Result<TopicModelFSRead, TopicModelIOError> {
    //     if path.as_ref().is_dir() {
    //         Ok(TopicModelFSRead::System { path_on_disc: path.as_ref().to_path_buf()})
    //     } else {
    //         let zip_archive = ZipArchive::new(BufReader::new(File::options().read(true).open(path)?))?;
    //         Ok(TopicModelFSRead::Zip {zip_archive})
    //     }
    // }

    pub fn open_zip<P: AsRef<Path>>(path: P) -> Result<TopicModelFSRead, TopicModelIOError> {
        if path.as_ref().is_dir() {
            Err(TopicModelIOError::IllegalPath(path.as_ref().to_path_buf()))
        } else {
            let zip_archive = ZipArchive::new(BufReader::new(File::options().read(true).open(path)?))?;
            Ok(TopicModelFSRead::Zip {zip_archive})
        }
    }

    pub fn open_file_system<P: AsRef<Path>>(path: P) -> Result<TopicModelFSRead, TopicModelIOError> {
        if path.as_ref().is_dir() {
            Ok(TopicModelFSRead::System { path_on_disc: path.as_ref().to_path_buf()})
        } else {
            Err(TopicModelIOError::IllegalPath(path.as_ref().to_path_buf()))
        }
    }

    // pub fn can_create_reader<P: AsRef<Path>>(&self, path: P) -> bool {
    //     match self {
    //         TopicModelFSRead::Zip { zip_archive } => {
    //             return zip_archive.index_for_path(path).is_some()
    //         }
    //         TopicModelFSRead::System { path_on_disc } => {
    //             path_on_disc.join(path).exists()
    //         }
    //     }
    // }


    pub fn create_reader_to<P: AsRef<Path>>(&mut self, path: P) -> Result<(TopicModelReader, bool), TopicModelIOError> {
        match self {
            TopicModelFSRead::Zip { zip_archive } => {
                let (pos, deflated) = if let Some(pos) = zip_archive.index_for_path(&path) {
                    (pos, false)
                } else {
                    let mut p = path.as_ref().to_path_buf();
                    if let Some(ext) = p.extension() {
                        p.set_extension(format!("{}.deflated", ext.to_ascii_lowercase().to_string_lossy()));
                    } else {
                        return Err(TopicModelIOError::IllegalPath(p.clone()));
                    }

                    if let Some(pos) = zip_archive.index_for_path(&p) {
                        (pos, true)
                    } else {
                        return Err(TopicModelIOError::PathNotFound(p));
                    }
                };

                let found = zip_archive.by_index(pos)?;
                Ok((TopicModelReader::Zip(found), deflated))
            }
            TopicModelFSRead::System { path_on_disc } => {
                let mut p = path_on_disc.join(path);
                if p.exists() {
                    return Ok((TopicModelReader::File(File::options().read(true).open(p)?), false))
                }
                if let Some(ext) = p.extension() {
                    p.set_extension(format!("{}.deflated", ext.to_ascii_lowercase().to_string_lossy()));
                } else {
                    return Err(TopicModelIOError::IllegalPath(p))
                }
                if p.exists() {
                    return Ok((TopicModelReader::File(File::options().read(true).open(p)?), true))
                } else {
                    Err(TopicModelIOError::PathNotFound(p))
                }
            }
        }
    }
}


pub enum TopicModelReader<'a> {
    Zip(ZipFile<'a>),
    File(File)
}

impl Read for TopicModelReader<'_> {
    delegate::delegate! {
        to match self {
            TopicModelReader::Zip(a) => a,
            TopicModelReader::File(b) => b,
        } {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
            fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize>;
            fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize>;
            fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize>;
            fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()>;
        }
    }
}


pub enum TopicModelFSWrite {
    Zip {
        zip_archive: ZipWriter<BufWriter<File>>,
        registered_files: HashSet<OsString>
    },
    System {
        path_on_disc: PathBuf
    }
}

pub enum TopicModelWriter<'a> {
    Zip {
        outp: &'a mut ZipWriter<BufWriter<File>>
    },
    File(File)
}

impl Write for TopicModelWriter<'_> {
    delegate::delegate! {
        to match self {
            TopicModelWriter::Zip{outp} => outp,
            TopicModelWriter::File(b) => b,
        } {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
            fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
            fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()>;
        }
    }
}



impl TopicModelFSWrite {
    // pub fn create(path: impl AsRef<Path>) -> Result<Self, TopicModelIOError> {
    //     let p = path.as_ref();
    //
    //     if let Some(ext) = p.extension() {
    //         if ext.eq(".zip") {
    //             if let Some(parent) = p.parent() {
    //                 std::fs::create_dir_all(parent)?;
    //             }
    //             Ok(
    //                 TopicModelFSWrite::Zip {
    //                     zip_archive: ZipWriter::new(BufWriter::new(File::options().create(true).truncate(true).open(p)?)),
    //                     registered_files: Default::default()
    //                 }
    //             )
    //         } else {
    //             Err(TopicModelIOError::UnsupportedFileType(p.to_path_buf()))
    //         }
    //     } else {
    //         std::fs::create_dir_all(p)?;
    //         Ok(
    //             TopicModelFSWrite::System {
    //                 path_on_disc: p.to_path_buf()
    //             }
    //         )
    //     }
    // }

    pub fn create_zip(path: impl AsRef<Path>) -> Result<Self, TopicModelIOError> {
        let p = path.as_ref();
        if let Some(parents) = p.parent() {
            std::fs::create_dir_all(parents)?;
        }
        if let Some(ext) = p.extension() {
            if ext.eq("zip") {
                if let Some(parent) = p.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                Ok(
                    TopicModelFSWrite::Zip {
                        zip_archive: ZipWriter::new(BufWriter::new(File::options().create(true).write(true).truncate(true).open(p)?)),
                        registered_files: Default::default()
                    }
                )
            } else {
                Err(TopicModelIOError::UnsupportedFileType(p.to_path_buf()))
            }
        } else {
            Err(TopicModelIOError::UnsupportedFileType(p.to_path_buf()))
        }
    }

    pub fn create_file_system(path: impl AsRef<Path>) -> Result<Self, TopicModelIOError> {
        std::fs::create_dir_all(&path)?;
        Ok(
            TopicModelFSWrite::System {
                path_on_disc: path.as_ref().to_path_buf()
            }
        )
    }

    pub fn can_create_writer<P: AsRef<Path>>(&self, path: P) -> bool {
        match self {
            TopicModelFSWrite::Zip { registered_files, .. } => {
                !registered_files.contains(path.as_ref().as_os_str())
            }
            TopicModelFSWrite::System { path_on_disc } => {
                !path_on_disc.join(path).exists()
            }
        }
    }

    pub fn create_writer_to<P: AsRef<Path>>(&mut self, path: P) -> Result<TopicModelWriter, TopicModelIOError> {
        if !self.can_create_writer(&path) {
            return Err(TopicModelIOError::AlreadyExists(path.as_ref().to_path_buf()))
        }

        match self {
            TopicModelFSWrite::Zip { ref mut zip_archive, .. } => {
                zip_archive.start_file_from_path::<(), _>(path, FileOptions::default())?;
                Ok(TopicModelWriter::Zip { outp: zip_archive })
            }
            TopicModelFSWrite::System { path_on_disc } => {
                let p = path_on_disc.join(path);
                if let Some(v) = p.parent() {
                    std::fs::create_dir_all(v)?;
                }
                Ok(TopicModelWriter::File(File::options().create(true).write(true).truncate(true).open(p)?))
            }
        }
    }
}

