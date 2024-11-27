use std::collections::{HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Seek};
use std::path::Path;
use flate2::bufread::GzDecoder;
use itertools::Itertools;
use thiserror::Error;
use tar::{Archive};

#[derive(Debug, Error)]
pub enum MuseError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("The file {0} is not supported!")]
    FileNotSupported(String)
}

pub struct MuseReader<R, E> {
    reader: R,
    cached_file_name: String,
    buffer: String,
    already_vistited: HashSet<String>,
    cached_file: Option<Cursor<Vec<u8>>>,
    parser: Box<dyn Fn(&str, &str, &str) -> Result<(String, String), E>>,
    selector: Box<dyn Fn(&str) -> bool>,
    had_error: bool,
}

impl<R, E> MuseReader<R, E> {
    fn new<P, S>(reader: R, parser: P, selector: S) -> Result<Self, std::io::Error>
        where
            P: Fn(&str, &str, &str) -> Result<(String, String), E> + 'static,
            S: Fn(&str) -> bool + 'static
    {
        Ok(
            Self {
                reader,
                already_vistited: HashSet::new(),
                cached_file_name: String::new(),
                buffer: String::new(),
                cached_file: None,
                parser: Box::new(parser),
                selector: Box::new(selector),
                had_error: false,
            }
        )
    }
}

impl<R, E> Iterator for  MuseReader<R, E>
where
    R: BufRead + Seek,
    E: Error + From<std::io::Error>
{
    type Item = Result<(String, String), E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.had_error {
            return None
        }
        let result = 'outer: loop {
            if let Some(cached) = &mut self.cached_file {
                match cached.read_line(&mut self.buffer) {
                    Ok(0) => {}
                    Ok(_) => {
                        let input = self.buffer.trim();
                        if input.is_empty() {
                            continue
                        }
                        let (a, b) = input.split(" ").collect_tuple()?;
                        let result = Some((&self.parser)(&self.cached_file_name, a, b));
                        self.buffer.clear();
                        break result
                    }
                    Err(err) => {
                        self.buffer.clear();
                        break Some(Err(err.into()))
                    }
                }
            }

            match self.reader.rewind() {
                Ok(_) => {}
                Err(err) => {
                    self.had_error = true;
                    return Some(Err(err.into()))
                }
            }

            let mut archive = Archive::new(GzDecoder::new(&mut self.reader));
            match archive.entries() {
                Ok(mut iter) => {
                    loop {
                        match iter.next()? {
                            Ok(mut entry) => {
                                match entry.path() {
                                    Ok(value) => {
                                        self.cached_file_name = value.to_string_lossy().to_string()
                                    }
                                    Err(err) => {
                                        break 'outer Some(Err(err.into()))
                                    }
                                }
                                if !self.already_vistited.insert(self.cached_file_name.clone())
                                    || !(&self.selector)(&self.cached_file_name)
                                {
                                    continue
                                }

                                let mut in_memory_file = if let Some(file) = self.cached_file.take() {
                                    let mut inner = file.into_inner();
                                    inner.clear();
                                    inner
                                } else {
                                    Vec::new()
                                };
                                match entry.read_to_end(&mut in_memory_file) {
                                    Ok(0) => {}
                                    Ok(_) => {
                                        self.cached_file = Some(Cursor::new(in_memory_file));
                                        break
                                    }
                                    Err(err) => {
                                        break 'outer Some(Err(err.into()))
                                    }
                                }
                            }
                            Err(err) => {
                                break 'outer Some(Err(err.into()))
                            }
                        }
                    }
                }
                Err(err) => {
                    self.had_error = true;
                    break Some(Err(err.into()))
                }
            }
        };

        match result {
            None => {
                None
            }
            x @ Some(Ok(_)) => {
                x
            }
            x @ Some(Err(_)) => {
                self.had_error = true;
                x
            }
        }
    }
}

/// The selector selects if a file with a specific name should be extracted.
/// The parser converts the read words into a tuple where the word for lang a is left and the word for lang b is right.
/// The order is decided by the filename that is provided with the first argument.
///
/// S = (FileName) -> Bool
/// P = (FileName, LeftWord, RightWord) -> (String for A, String for B)
pub fn read_from_archive<P, S, E>(path: impl AsRef<Path>, selector: S, parser: P) -> Result<MuseReader<BufReader<File>, E>, MuseError>
where
    P: Fn(&str, &str, &str) -> Result<(String, String), E> + 'static,
    S: Fn(&str) -> bool + 'static,
    E: Error + From<std::io::Error>
{
    Ok(MuseReader::new(BufReader::new(
        File::options().read(true).open(path)?
    ), parser, selector)?)
}

pub fn read_single_from_archive(path: impl AsRef<Path>, name: impl Into<String>) -> Result<MuseReader<BufReader<File>, MuseError>, MuseError> {
    let name1 = name.into();
    let name2 = name1.clone();
    read_from_archive(
        path,
        move |target| { name1 == target },
        move |file, a, b| {
            if file != name2 {
                Err(MuseError::FileNotSupported(file.to_string()))
            } else {
                Ok((a.to_string(), b.to_string()))
            }
        }
    )
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use crate::topicmodel::dictionary::loader::muse::{read_from_archive, MuseError};

    #[test]
    pub fn test(){
        let value = read_from_archive(
            r#"dictionaries/MUSE/dictionaries.tar.gz"#,
            |file| {
                matches!(
                    file,
                    "dictionaries/de-en.txt" // 101997
                    |"dictionaries/en-de.txt" // 101932
                )
            },
            |file, left, right| {
                match file {
                    "dictionaries/en-de.txt" => {
                        Ok((left.to_string(), right.to_string()))
                    }
                    "dictionaries/de-en.txt" => {
                        Ok((right.to_string(), left.to_string()))
                    }
                    _ => {
                        Err(MuseError::FileNotSupported(file.to_string()))
                    }
                }
            }
        ).unwrap();

        let x = value.process_results(|value| {
            value.unique().collect_vec()
        }).unwrap();

        println!("{}", x.len());

    }
}