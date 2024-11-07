use std::path::Path;

pub struct ExtractedWord {
    word: String,

}

pub fn read_wiktionary(path: impl AsRef<Path>) {

}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use flate2::bufread::GzDecoder;

    #[test]
    fn test(){
        let mut reader = BufReader::new(
            GzDecoder::new(
                BufReader::new(
                    File::open("dictionaries/Wiktionary/raw-wiktextract-data.jsonl.gz").unwrap()
                )
            )
        );

        for content in reader.lines() {
            let content = content.unwrap();
            let value = serde_json::from_str::<serde_json::Value>(&content).unwrap();
            println!("{}", value);
            break
        }
    }
}