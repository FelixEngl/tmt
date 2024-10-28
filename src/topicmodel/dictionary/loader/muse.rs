use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::marker::PhantomData;
use std::path::Path;
use flate2::bufread::GzDecoder;
use thiserror::Error;
use tar::{Archive, Entries, Entry};

#[derive(Debug, Error)]
pub enum MuseError {
    #[error(transparent)]
    Io(#[from] std::io::Error)
}


pub fn read_from_archive<K, P>(path: impl AsRef<Path>, unpackers: HashMap<K, Box<dyn Fn(&str, &str) -> P>>) -> Result<(), MuseError>
    where
        K: AsRef<str>,
        P: Sized
{
    let mut archive = Archive::new(flate2::bufread::GzDecoder::new(BufReader::new(File::options().read(true).open(path)?)));
    for entry in archive.entries()? {
        let entry = entry?;
        todo!()
    }
    todo!()
}