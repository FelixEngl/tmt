use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;
use super::helper::gen_ms_terms_reader::iter::TermEntryElementIter;
use super::helper::gen_ms_terms_reader::*;
pub struct MSTermsReader<R> {
    iter: TermEntryElementIter<R>
}

impl<R> Iterator for MSTermsReader<R> where R: BufRead {
    type Item = Result<(), MSTermsReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum MSTermsReaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Xml(#[from] MartifReaderError),
}

pub fn read_ms_terms(path: impl AsRef<Path>) -> Result<MSTermsReader<BufReader<File>>, MSTermsReaderError> {
    let r = quick_xml::reader::Reader::from_reader(
        BufReader::with_capacity(
            128*1024,
            File::options().read(true).open(path)?
        )
    );
    let iter = iter_for_term_entry_element(r);
    Ok(
        MSTermsReader {
            iter,
        }
    )
}

#[cfg(test)]
mod test {
    use super::read_ms_terms;

    #[test]
    fn can_run(){
        let mut reader = read_ms_terms(
            "dictionaries/Microsoft TermCollection/MicrosoftTermCollection_german.tbx"
        ).unwrap();

        println!("{}", reader.count());

        let mut reader = read_ms_terms(
            "dictionaries/Microsoft TermCollection/MicrosoftTermCollectio_british_englisch.tbx"
        ).unwrap();

        println!("{}", reader.count());
    }
}