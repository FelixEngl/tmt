use either::Either;
use serde::de::{DeserializeOwned, Error, IgnoredAny};
use serde::{Deserialize, Deserializer, Serialize};
use std::any::type_name;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Read};
use std::path::Path;
use flate2::bufread::GzDecoder;
use rayon::iter::{IterBridge, Map};
use rayon::prelude::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtractedWord {
    /// the word form
    word: String,
    /// part-of-speech, such as "noun", "verb", "adj", "adv", "pron", "determiner", "prep" (preposition), "postp" (postposition), and many others. The complete list of possible values returned by the package can be found in wiktextract.PARTS_OF_SPEECH.
    #[serde(default)]
    pos: Option<String>,
    /// name of the language this word belongs to (e.g., English)
    lang: String,
    /// Wiktionary language code corresponding to lang key (e.g., en)
    lang_code: String,
    /// list of word senses (dictionaries) for this word/part-of-speech (see below)
    #[serde(default)]
    senses: Vec<Sense>,
    /// list of inflected or alternative forms specified for the word (e.g., plural, comparative, superlative, roman script version).
    /// This is a list of dictionaries, where each dictionary has a form key and a tags key.
    /// The tags identify what type of form it is. It may also contain "ipa", "roman", and "source" fields.
    /// The form can be "-" when the word is marked as not having that form
    /// (some of those will be word-specific, while others are language-specific; post-processing
    /// can drop such forms when no word has a value for that tag combination).
    #[serde(default)]
    forms: Vec<Form>,
    /// list of dictionaries containing pronunciation, hyphenation, rhyming, and related information.
    /// Each dictionary may have a tags key containing tags that clarify what kind of form that
    /// entry is. Different types of information are stored in different fields: ipa is IPA
    /// pronunciation, enPR is enPR pronunciation, audio is name of sound file in Wikimedia commons.
    #[serde(default, skip, rename = "sounds")]
    _sounds: IgnoredAny,
    /// list of non-disambiguated categories for the word
    #[serde(default)]
    categories: Vec<String>,
    /// list of non-disambiguated topics for the word
    #[serde(default)]
    topics: Vec<String>,
    /// non-disambiguated translation entries (see below)
    #[serde(default)]
    translations: Vec<Translation>,
    /// etymology section as cleaned text
    #[serde(default, skip, rename = "etymology_text")]
    _etymology_text: IgnoredAny,
    /// templates and their arguments and expansions from the etymology section. These can be used to easily parse etymological relations. Certain common templates that do not signify etymological relations are not included.
    #[serde(default, skip, rename = "etymology_templates")]
    _etymology_templates: IgnoredAny,
    /// for words with multiple numbered etymologies, this contains the number of the etymology under which this entry appeared
    #[serde(default, skip, rename = "etymology_number")]
    _etymology_number: IgnoredAny,
    /// descendants of the word (see below)
    #[serde(default, skip, rename = "descendants")]
    _descendants: IgnoredAny,
    /// non-disambiguated synonym linkages for the word (see below)
    #[serde(default)]
    synonyms: Vec<Linkage>,
    /// non-disambiguated antonym linkages for the word (see below)
    #[serde(default)]
    antonyms: Vec<Linkage>,
    /// non-disambiguated hypernym linkages for the word (see below)
    #[serde(default)]
    hypernyms: Vec<Linkage>,
    /// non-disambiguated linkages indicating being part of something (see below) (not systematically encoded)
    #[serde(default)]
    holonyms: Vec<Linkage>,
    /// non-disambiguated linkages indicating having a part (see below) (fairly rare)
    #[serde(default)]
    meronyms: Vec<Linkage>,
    /// non-disambiguated derived word linkages for the word (see below)
    #[serde(default)]
    derived: Vec<Linkage>,
    /// non-disambiguated related word linkages for the word (see below)
    #[serde(default)]
    related: Vec<Linkage>,
    /// non-disambiguated coordinate term linkages for the word (see below)
    #[serde(default)]
    coordinate_terms: Vec<Linkage>,
    /// non-disambiguated Wikidata identifer
    #[serde(default, deserialize_with = "deserialize_optional_flat_either")]
    wikidata: Option<Either<String, Vec<String>>>,
    /// non-disambiguated page title in Wikipedia (possibly prefixed by language id)
    #[serde(default)]
    wiktionary: Option<String>,
    /// part-of-speech specific head tags for the word. This basically just captures the templates (their name and arguments) as a list of dictionaries. Most applications may want to ignore this.
    #[serde(default, skip, rename = "head_templates")]
    _head_templates: IgnoredAny,
    /// conjugation and declension templates found for the word, as dictionaries. These basically capture the language-specific inflection template for the word. Note that for some languages inflection information is also contained in head_templates. XXX in the very near future, we will start parsing inflections from the inflection tables into forms, so there is usually no need to use the inflection_templates data.
    #[serde(default, skip, rename = "inflection_templates")]
    _inflection_templates: IgnoredAny,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sense {
    /// list of gloss strings for the word sense (usually only one). This has been cleaned, and should be straightforward text with no tagging.
    #[serde(default)]
    glosses: Vec<String>,
    /// list of gloss strings for the word sense, with less cleaning than
    #[serde(default)]
    raw_glosses: Vec<String>,
    /// list of qualifiers and tags for the gloss.
    /// This is a list of strings, and may include
    /// words such as "archaic", "colloquial", "present",
    /// "participle", "plural", "feminine", and many others
    /// (new words may appear arbitrarily).
    #[serde(default)]
    tags: Vec<String>,
    /// list of sense-disambiguated category names extracted from (a subset) of the Category links on the page
    #[serde(default)]
    categories: Vec<String>,
    /// list of sense-disambiguated topic names (kind of similar to categories but determined differently)
    #[serde(default)]
    topics: Vec<String>,
    /// list of words that his sense is an alternative form of; this is a list of dictionaries, with field word containing the linked word and optionally extra containing additional text
    #[serde(default, skip, rename = "alt_of")]
    _alt_of: IgnoredAny,
    /// list of words that this sense is an inflected form of; this is a list of dictionaries, with field word containing the linked word and optionally extra containing additional text
    #[serde(default, skip, rename = "form_of")]
    _form_of: IgnoredAny,
    /// sense-disambiguated translation entries (see below)
    #[serde(default)]
    translations: Vec<Translation>,
    /// sense-disambiguated synonym linkages for the word (see below)
    #[serde(default)]
    synonyms: Vec<Linkage>,
    /// sense-disambiguated antonym linkages for the word (see below)
    #[serde(default)]
    antonyms: Vec<Linkage>,
    /// sense-disambiguated hypernym linkages for the word (see below)
    #[serde(default)]
    hypernyms: Vec<Linkage>,
    /// sense-disambiguated linkages indicating being part of something (see below) (not systematically encoded)
    #[serde(default)]
    holonyms: Vec<Linkage>,
    /// sense-disambiguated linkages indicating having a part (see below) (fairly rare)
    #[serde(default)]
    meronyms: Vec<Linkage>,
    /// sense-disambiguated coordinate_terms linkages (see below)
    #[serde(default)]
    coordinate_terms: Vec<Linkage>,
    /// sense-disambiguated derived word linkages for the word (see below)
    #[serde(default)]
    derived: Vec<Linkage>,
    /// sense-disambiguated related word linkages for the word (see below)
    #[serde(default)]
    related: Vec<Linkage>,
    /// list of textual identifiers collected for the sense. If there is a QID for the entry (e.g., Q123), those are stored in the wikidata field.
    #[serde(default)]
    senseid: Vec<String>,
    /// list of QIDs (e.g., Q123) for the sense
    #[serde(default)]
    wikidata: Vec<String>,
    /// list of Wikipedia page titles (with optional language code prefix)
    #[serde(default)]
    wikipedia: Vec<String>,
    /// list of usage examples, each example being a dictionary with text field containing the example text, optional ref field containing a source reference, optional english field containing English translation, optional type field containing example type (currently example or quotation if present), optional roman field containing romanization (for some languages written in non-Latin scripts), and optional (rare) note field contains English-language parenthesized note from the beginning of a non-english example.
    #[serde(default, skip, rename = "examples")]
    _examples: IgnoredAny,
    /// if the word sense has a qualifier that could not be parsed, that qualifier is put in this field (rare). Most qualifiers are parsed into tags and/or topics. The gloss with the qualifier still present can be found in raw_glosses.
    #[serde(default)]
    english: Vec<String>,
}

/// Pronunciation -> Ignored bc sounds are ignored

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Translation {
    /// optional alternative form of the translation (e.g., in a different script)
    #[serde(default)]
    alt: Option<String>,
    /// Wiktionary's 2 or 3-letter language code for the language the translation is for
    #[serde(default)]
    code: Option<String>,
    /// English text, generally clarifying the target sense of the translation
    #[serde(default, skip, rename = "english")]
    _english: IgnoredAny,
    /// the language name that the translation is for
    lang: String,
    /// optional text describing or commenting on the translation
    #[serde(default)]
    note: Option<String>,
    /// optional romanization of the translation (when in non-Latin characters)
    #[serde(default, skip, rename = "roman")]
    _roman: IgnoredAny,
    /// optional sense indicating the meaning for which this is a translation (this is a free-text string, and may not match any gloss exactly)
    #[serde(default)]
    sense: Option<String>,
    /// optional list of qualifiers for the translations, e.g., gender
    #[serde(default)]
    tags: Vec<String>,
    /// optional taxonomic name of an organism mentioned in the translation
    #[serde(default)]
    taxonomic: Option<String>,
    /// the translation in the specified language (may be missing when note is present)
    #[serde(default)]
    word: Option<String>,
}

/// Etymologies ignored bc entymologie nicht spannend
/// Descendants ignored bc entymologie nicht spannend

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Linkage {
    /// optional alternative form of the target (e.g., in a different script)
    #[serde(default)]
    alt: Option<String>,
    /// optional English text associated with the sense, usually identifying the linked target sense
    #[serde(default, skip, rename = "english")]
    _english: IgnoredAny,
    /// optional romanization of a linked word in a non-Latin script
    #[serde(default, skip, rename = "roman")]
    _roman: IgnoredAny,
    /// text identifying the word sense or context (e.g., "to rain very heavily")
    #[serde(default, deserialize_with = "deserialize_optional_flat_either")]
    sense: Option<Either<String, Vec<String>>>,
    /// qualifiers specified for the sense (e.g., field of study, region, dialect, style)
    #[serde(default)]
    tags: Vec<String>,
    /// optional taxonomic name associated with the linkage
    #[serde(default)]
    taxonomic: Option<String>,
    /// list of topic descriptors for the linkage (e.g., military)
    #[serde(default)]
    topics: Vec<String>,
    /// the word this links to (string)
    word: String,
}

fn deserialize_optional_flat_either<'de, D, L, R>(deser: D) -> Result<Option<Either<L, R>>, D::Error>
where
    D: Deserializer<'de>,
    L: DeserializeOwned,
    R: DeserializeOwned,
{
    let v = serde_json::Value::deserialize(deser)?;
    let is_null = matches!(v, serde_json::Value::Null);
    if let Ok(l) = serde_json::from_value(v.clone()) {
        Ok(Some(Either::Left(l)))
    } else if let Ok(r) = serde_json::from_value(v) {
        Ok(Some(Either::Right(r)))
    } else if is_null {
        Ok(None)
    } else {
        Err(Error::custom(format!(
            "Failed to deserialize optional either {} or {}!",
            type_name::<L>(),
            type_name::<R>()
        )))
    }
}

fn deserialize_flat_either<'de, D, L, R>(deser: D) -> Result<Either<L, R>, D::Error>
where
    D: Deserializer<'de>,
    L: DeserializeOwned,
    R: DeserializeOwned,
{
    let v = serde_json::Value::deserialize(deser)?;
    if let Ok(l) = serde_json::from_value(v.clone()) {
        Ok(Either::Left(l))
    } else if let Ok(r) = serde_json::from_value(v) {
        Ok(Either::Right(r))
    } else {
        Err(Error::custom(format!(
            "Failed to deserialize either {} or {}!",
            type_name::<L>(),
            type_name::<R>()
        )))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Form {
    form: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    raw_tags: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Redirect {
    title: String,
    redirect: String,
    #[serde(flatten)]
    _sink: IgnoredAny
}


pub struct WiktionaryReader<R> {
    inner: Lines<BufReader<R>>
}

impl<R> WiktionaryReader<R> {
    fn process_line(line: Result<String, std::io::Error>) -> Result<Either<Redirect, ExtractedWord>, WiktionaryReaderError> {
        match line {
            Ok(value) => {
                match serde_json::from_str::<ExtractedWord>(&value) {
                    Ok(value) => Ok(Either::Right(value)),
                    Err(_) => {
                        match serde_json::from_str::<Redirect>(&value) {
                            Ok(value) => Ok(Either::Left(value)),
                            Err(err) => Err(WiktionaryReaderError::Serde(err, value))
                        }
                    }
                }
            }
            Err(err) => {
                Err(err.into())
            }
        }
    }
}

impl<R> WiktionaryReader<R> where R: Read {
    pub fn new(r: R) -> Self {
        Self {
            inner: BufReader::with_capacity((1024<<2)*100, r).lines()
        }
    }

    pub fn iter(&mut self) -> WiktionaryIter<R> {
        WiktionaryIter {
            reader: self
        }
    }
}

pub struct WiktionaryIter<'a, R> {
    reader: &'a mut WiktionaryReader<R>
}

impl<'a, R> Iterator for WiktionaryIter<'a, R> where R: Read {
    type Item = <WiktionaryReader<R> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next()
    }
}

pub type ParallelWiktionaryReader<R> = Map<IterBridge<Lines<BufReader<R>>>, fn(Result<String, std::io::Error>) -> Result<Either<Redirect, ExtractedWord>, WiktionaryReaderError>>;

impl<R> IntoParallelIterator for WiktionaryReader<R> where R: Read + 'static + Sized + Send {
    type Iter = ParallelWiktionaryReader<R>;
    type Item = <Self as Iterator>::Item;

    fn into_par_iter(self) -> Self::Iter {
        self.inner.par_bridge().map(Self::process_line)
    }
}

impl<R> Iterator for WiktionaryReader<R> where R: Read {
    type Item = Result<Either<Redirect, ExtractedWord>, WiktionaryReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Self::process_line(self.inner.next()?))
    }
}

#[derive(Debug, Error)]
pub enum WiktionaryReaderError {
    #[error("{1}\n{0:?}")]
    Serde(serde_json::Error, String),
    #[error(transparent)]
    Io(#[from] std::io::Error)
}

pub fn read_wiktionary(path: impl AsRef<Path>) -> Result<WiktionaryReader<GzDecoder<BufReader<File>>>, std::io::Error> {
    Ok(
        WiktionaryReader::new(
            GzDecoder::new(
                BufReader::with_capacity(
                    100*(1024<<2),
                    File::options().read(true).open(path)?
                )
            )
        )
    )
}




#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};
    use std::fs::File;
    use either::Either;
    use itertools::Itertools;
    use rayon::prelude::*;
    use crate::define_aho_matcher;
    use crate::topicmodel::dictionary::loader::wiktionary_reader::read_wiktionary;
    use crate::topicmodel::dictionary::word_infos::Domain;

    fn identify_possible_topics(s: &str) -> Option<Vec<(Domain, String)>> {
        let s_low = s.to_lowercase();
        let mut d = Vec::new();
        for (i, _) in s_low.char_indices().dropping(1) {
            let slice = &s_low[0..i];
            match slice.parse() {
                Ok(domain ) => {
                    d.push((domain, String::from(&s[0..i])))
                }
                _ => {}
            }
        }
        if d.is_empty() {
            None
        } else {
            Some(d)
        }
    }

    #[test]
    fn test2(){
        define_aho_matcher!(
            static LANGUAGE = "german" | "english" as ascii_case_insensitive
        );


        let topics = read_wiktionary("dictionaries/Wiktionary/raw-wiktextract-data.jsonl.gz")
            .unwrap()
            .into_par_iter()
            .filter_map(|value| {
                match value {
                    Ok(Either::Right(value)) => {
                        Some(value)
                    }
                    _ => None
                }
            })
            .map(|value| {
                value.senses.into_par_iter().flat_map(|value| {
                    value.topics
                }).chain(value.topics).collect::<Vec<_>>()
                // value.lang
            })
            .collect::<Vec<_>>();

        let mut topic_to_other = HashMap::new();

        for values in topics {
            for value in values {
                topic_to_other.entry(value.clone()).or_insert_with(HashSet::new).insert(value);
            }
        }

        let mut f = File::options().write(true).truncate(true).create(true).open("data2.json").unwrap();
        serde_json::to_writer_pretty(&mut f, &topic_to_other).unwrap()
    }

    #[test]
    fn test() {


        let topics: HashSet<String> = serde_json::from_reader(
            File::options().read(true).open("data.json").unwrap()
        ).unwrap();

        let mut mappings = indexmap::IndexMap::new();
        let mut not_recognized = HashSet::new();
        define_aho_matcher!(
            static IDENT = "aer" as ascii_case_insensitive
        );
        for value in topics {
            if value.parse::<Domain>().is_ok() {
                continue
            }
            if let Some(found) = identify_possible_topics(&value) {
                for (a, b) in found {
                    mappings.entry(a).or_insert_with(HashSet::new).insert((value.clone(), b));
                }
            } else {
                if IDENT.is_match(&value) {
                    println!("{value}")
                }
                not_recognized.insert(value);
            }
        }

        let mut not_recognized = Vec::from_iter(not_recognized.into_iter());
        not_recognized.sort();
        mappings.sort_keys();

        let mut f = File::options().write(true).create(true).truncate(true).open("data_not_rec.json").unwrap();
        serde_json::to_writer_pretty(&mut f, &not_recognized).unwrap();

        let mut f = File::options().write(true).create(true).truncate(true).open("data_rec.json").unwrap();
        serde_json::to_writer_pretty(&mut f, &mappings).unwrap();

        // define_aho_matcher!(
        //     static LANGUAGE = "german" | "english" as ascii_case_insensitive
        // );
        //
        //
        // let topics = read_wiktionary("dictionaries/Wiktionary/raw-wiktextract-data.jsonl.gz")
        //     .unwrap()
        //     .into_par_iter()
        //     .filter_map(|value| {
        //         match value {
        //             Ok(Either::Right(value)) => {
        //                 Some(value)
        //             }
        //             _ => None
        //         }
        //     })
        //     .filter(|value| LANGUAGE.is_match(&value.lang))
        //     .map(|value| {
        //         value.senses.into_par_iter().flat_map(|value| {
        //             value.topics
        //         }).chain(value.topics).collect::<Vec<_>>()
        //         // value.lang
        //     })
        //     .collect::<Vec<_>>();
        //
        // let mut topic_to_other = HashMap::new();
        //
        // for values in topics {
        //     for value in values {
        //         topic_to_other.entry(value.clone()).or_insert_with(HashSet::new).insert(value);
        //     }
        // }
        //
        // let mut f = File::options().write(true).truncate(true).create(true).open("data2.json").unwrap();
        // serde_json::to_writer_pretty(&mut f, &topic_to_other).unwrap()
    }
}
