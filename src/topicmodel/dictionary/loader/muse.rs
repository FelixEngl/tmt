use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Seek};
use std::marker::PhantomData;
use std::path::Path;
use flate2::bufread::GzDecoder;
use itertools::Itertools;
use thiserror::Error;
use tar::{Archive, Entries, Entry};

#[derive(Debug, Error)]
pub enum MuseError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("The file {0} is not supported!")]
    FileNotSupported(String)
}

pub struct MuseReader<R, P, S, E> {
    reader: R,
    cached_file_name: String,
    buffer: String,
    already_vistited: HashSet<String>,
    cached_file: Option<Cursor<Vec<u8>>>,
    parser: P,
    selector: S,
    had_error: bool,
    _phantom: PhantomData<E>
}

impl<R, P, S, E> MuseReader<R, P, S, E> {
    fn new(reader: R, parser: P, selector: S) -> Result<Self, std::io::Error> {
        Ok(
            Self {
                reader,
                already_vistited: HashSet::new(),
                cached_file_name: String::new(),
                buffer: String::new(),
                cached_file: None,
                parser,
                selector,
                had_error: false,
                _phantom: PhantomData
            }
        )
    }
}

impl<R, P, S, E> Iterator for  MuseReader<R, P, S, E>
where
    R: BufRead + Seek,
    P: Fn(&str, &str, &str) -> Result<(String, String), E>,
    S: Fn(&str) -> bool,
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

pub fn read_from_archive<P, S, E>(path: impl AsRef<Path>, selector: S, parser: P) -> Result<MuseReader<BufReader<File>, P, S, E>, MuseError>
where
    P: Fn(&str, &str, &str) -> Result<(String, String), E>,
    S: Fn(&str) -> bool,
    E: Error + From<std::io::Error>
{
    Ok(MuseReader::new(BufReader::new(
        File::options().read(true).open(path)?
    ), parser, selector)?)
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use crate::topicmodel::dictionary::loader::muse::{read_from_archive, MuseError};

    #[test]
    pub fn test(){
        let mut value = read_from_archive(
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