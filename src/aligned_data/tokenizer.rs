use std::sync::Arc;
use itertools::Itertools;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;
use crate::aligned_data::stopwords::{StopWordList};

pub struct Tokenizer {
    normalize: bool,
    lower_case: bool,
    filter_spaces: bool,
    stop_words: Option<Arc<StopWordList>>,
    stemmer: Option<rust_stemmers::Algorithm>,
}

impl Tokenizer {

    pub fn new(
        normalize: bool,
        lower_case: bool,
        filter_spaces: bool,
        stop_words: Option<Arc<StopWordList>>,
        stemmer: Option<rust_stemmers::Algorithm>
    ) -> Self {
        Self {
            normalize,
            lower_case,
            filter_spaces,
            stop_words,
            stemmer
        }
    }

    /// Preprocesses a text
    pub fn tokenize<'a>(&self, text: &'a str) -> Vec<(Option<String>, (usize, &'a str))> {
        let text = text.split_word_bound_indices();
        let mut text = if self.normalize {
            text.map(|value|(Some(value.1.nfc().to_string()), value)).collect_vec()
        } else {
            text.map(|value|(Some(value.1.to_string()), value)).collect_vec()
        };

        let stemmer = self.stemmer.map(rust_stemmers::Stemmer::create);

        for (value, _) in text.iter_mut() {
            if let Some(mut found) = value.take() {
                if self.filter_spaces && found.chars().all(char::is_whitespace) {
                    continue;
                }
                if let Some(ref stop_words) = self.stop_words {
                    if self.normalize && stop_words.contains_normalized(found.as_str()) {
                        continue;
                    } else if stop_words.contains_raw(found.as_str()) {
                        continue;
                    }
                }
                if self.lower_case {
                    let lower_case = found.to_lowercase();
                    let _ = std::mem::replace(&mut found, lower_case);
                }
                if let Some(ref stemmer) = stemmer {
                    let stemmed = stemmer.stem(&found).to_string();
                    let _ = std::mem::replace(&mut found, stemmed);
                }
                let _ = value.insert(found);
            }
        }

        text
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn can_tokenize(){
        const TEXT: &'static str = "Hallo Welt!";

    }
}