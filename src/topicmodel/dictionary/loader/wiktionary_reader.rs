use std::path::Path;
use serde::{Deserialize, Serialize};
use serde::de::IgnoredAny;
use crate::topicmodel::dictionary::word_infos::{Language, PartOfSpeech};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtractedWord {
    /// the word form
    word: String,
    /// part-of-speech, such as "noun", "verb", "adj", "adv", "pron", "determiner", "prep" (preposition), "postp" (postposition), and many others. The complete list of possible values returned by the package can be found in wiktextract.PARTS_OF_SPEECH.
    pos: PartOfSpeech,
    /// name of the language this word belongs to (e.g., English)
    lang: String,
    /// Wiktionary language code corresponding to lang key (e.g., en)
    lang_code: String,
    /// list of word senses (dictionaries) for this word/part-of-speech (see below)
    sensed: Vec<Sense>,
    /// list of inflected or alternative forms specified for the word (e.g., plural, comparative, superlative, roman script version).
    /// This is a list of dictionaries, where each dictionary has a form key and a tags key.
    /// The tags identify what type of form it is. It may also contain "ipa", "roman", and "source" fields.
    /// The form can be "-" when the word is marked as not having that form
    /// (some of those will be word-specific, while others are language-specific; post-processing
    /// can drop such forms when no word has a value for that tag combination).
    forms: Vec<Form>,
    /// list of dictionaries containing pronunciation, hyphenation, rhyming, and related information. Each dictionary may have a tags key containing tags that clarify what kind of form that entry is. Different types of information are stored in different fields: ipa is IPA pronunciation, enPR is enPR pronunciation, audio is name of sound file in Wikimedia commons.
    #[serde(default, skip)]
    sounds: (),
    /// list of non-disambiguated categories for the word
    categories: Vec<String>,
    /// list of non-disambiguated topics for the word
    topics: Vec<String>,
    /// non-disambiguated translation entries (see below)
    translations: Vec<Translation>,
    /// etymology section as cleaned text
    #[serde(default, skip)]
    etymology_text: (),
    /// templates and their arguments and expansions from the etymology section. These can be used to easily parse etymological relations. Certain common templates that do not signify etymological relations are not included.
    #[serde(default, skip)]
    etymology_templates: (),
    /// for words with multiple numbered etymologies, this contains the number of the etymology under which this entry appeared
    #[serde(default, skip)]
    etymology_number: (),
    /// descendants of the word (see below)
    #[serde(default, skip)]
    descendants: (),
    /// non-disambiguated synonym linkages for the word (see below)
    synonyms: Vec<Linkage>,
    /// non-disambiguated antonym linkages for the word (see below)
    antonyms: Vec<Linkage>,
    /// non-disambiguated hypernym linkages for the word (see below)
    hypernyms: Vec<Linkage>,
    /// non-disambiguated linkages indicating being part of something (see below) (not systematically encoded)
    holonyms: Vec<Linkage>,
    /// non-disambiguated linkages indicating having a part (see below) (fairly rare)
    meronyms: Vec<Linkage>,
    /// non-disambiguated derived word linkages for the word (see below)
    derived: Vec<Linkage>,
    /// non-disambiguated related word linkages for the word (see below)
    related: Vec<Linkage>,
    /// non-disambiguated coordinate term linkages for the word (see below)
    coordinate_terms: Vec<Linkage>,
    /// non-disambiguated Wikidata identifer
    wikidata: String,
    /// non-disambiguated page title in Wikipedia (possibly prefixed by language id)
    wiktionary : String,
    /// part-of-speech specific head tags for the word. This basically just captures the templates (their name and arguments) as a list of dictionaries. Most applications may want to ignore this.
    #[serde(default, skip)]
    head_templates : (),
    /// conjugation and declension templates found for the word, as dictionaries. These basically capture the language-specific inflection template for the word. Note that for some languages inflection information is also contained in head_templates. XXX in the very near future, we will start parsing inflections from the inflection tables into forms, so there is usually no need to use the inflection_templates data.
    #[serde(default, skip)]
    inflection_templates: ()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sense {
    /// list of gloss strings for the word sense (usually only one). This has been cleaned, and should be straightforward text with no tagging.
    glosses: Vec<String>,
    /// list of gloss strings for the word sense, with less cleaning than
    raw_glosses: Vec<String>,
    /// list of qualifiers and tags for the gloss.
    /// This is a list of strings, and may include
    /// words such as "archaic", "colloquial", "present",
    /// "participle", "plural", "feminine", and many others
    /// (new words may appear arbitrarily).
    tags: Vec<String>,
    /// list of sense-disambiguated category names extracted from (a subset) of the Category links on the page
    categories: Vec<String>,
    /// list of sense-disambiguated topic names (kind of similar to categories but determined differently)
    topics: Vec<String>,
    /// list of words that his sense is an alternative form of; this is a list of dictionaries, with field word containing the linked word and optionally extra containing additional text
    #[serde(default, skip)]
    alt_of: (),
    /// list of words that this sense is an inflected form of; this is a list of dictionaries, with field word containing the linked word and optionally extra containing additional text
    #[serde(default, skip)]
    form_of: (),
    /// sense-disambiguated translation entries (see below)
    translations: Vec<Translation>,
    /// sense-disambiguated synonym linkages for the word (see below)
    synonyms: Vec<Linkage>,
    /// sense-disambiguated antonym linkages for the word (see below)
    antonyms: Vec<Linkage>,
    /// sense-disambiguated hypernym linkages for the word (see below)
    hypernyms: Vec<Linkage>,
    /// sense-disambiguated linkages indicating being part of something (see below) (not systematically encoded)
    holonyms: Vec<Linkage>,
    /// sense-disambiguated linkages indicating having a part (see below) (fairly rare)
    meronyms: Vec<Linkage>,
    /// sense-disambiguated coordinate_terms linkages (see below)
    coordinate_terms: Vec<Linkage>,
    /// sense-disambiguated derived word linkages for the word (see below)
    derived: Vec<Linkage>,
    /// sense-disambiguated related word linkages for the word (see below)
    related: Vec<Linkage>,
    /// list of textual identifiers collected for the sense. If there is a QID for the entry (e.g., Q123), those are stored in the wikidata field.
    senseid: Vec<String>,
    /// list of QIDs (e.g., Q123) for the sense
    wikidata: Vec<String>,
    /// list of Wikipedia page titles (with optional language code prefix)
    wikipedia: Vec<String>,
    /// list of usage examples, each example being a dictionary with text field containing the example text, optional ref field containing a source reference, optional english field containing English translation, optional type field containing example type (currently example or quotation if present), optional roman field containing romanization (for some languages written in non-Latin scripts), and optional (rare) note field contains English-language parenthesized note from the beginning of a non-english example.
    #[serde(default, skip)]
    examples: (),
    /// if the word sense has a qualifier that could not be parsed, that qualifier is put in this field (rare). Most qualifiers are parsed into tags and/or topics. The gloss with the qualifier still present can be found in raw_glosses.
    english: Vec<String>,
}

/// Pronunciation -> Ignored bc sounds are ignored

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Translation {
    /// optional alternative form of the translation (e.g., in a different script)
    alt: Option<String>,
    /// Wiktionary's 2 or 3-letter language code for the language the translation is for
    code: String,
    /// English text, generally clarifying the target sense of the translation
    #[serde(default, skip)]
    english: (),
    /// the language name that the translation is for
    lang: String,
    /// optional text describing or commenting on the translation
    note: Option<String>,
    /// optional romanization of the translation (when in non-Latin characters)
    #[serde(default, skip)]
    roman: (),
    /// optional sense indicating the meaning for which this is a translation (this is a free-text string, and may not match any gloss exactly)
    #[serde(default, skip)]
    sense: (),
    /// optional list of qualifiers for the translations, e.g., gender
    tags: Vec<String>,
    /// optional taxonomic name of an organism mentioned in the translation
    taxonomic: Option<String>,
    /// the translation in the specified language (may be missing when note is present)
    word: Option<String>
}

/// Etymologies ignored bc entymologie nicht spannend
/// Descendants ignored bc entymologie nicht spannend

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Linkage {
    /// optional alternative form of the target (e.g., in a different script)
    alt: Option<String>,
    /// optional English text associated with the sense, usually identifying the linked target sense
    #[serde(default, skip)]
    english: (),
    /// optional romanization of a linked word in a non-Latin script
    #[serde(default, skip)]
    roman: (),
    /// text identifying the word sense or context (e.g., "to rain very heavily")
    sense: String,
    /// qualifiers specified for the sense (e.g., field of study, region, dialect, style)
    tags: Vec<String>,
    /// optional taxonomic name associated with the linkage
    taxonomic: Option<String>,
    /// list of topic descriptors for the linkage (e.g., military)
    topics: Vec<String>,
    /// the word this links to (string)
    word: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Form {
    form: String,
    tags: Vec<String>
}



pub fn read_wiktionary(path: impl AsRef<Path>) {

}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};
    use flate2::bufread::GzDecoder;
    use rayon::prelude::*;
    use crate::define_aho_matcher;
    use crate::topicmodel::dictionary::loader::wiktionary_reader::ExtractedWord;

    #[test]
    fn test(){
        let mut reader = BufReader::new(
            // File::open(r#"D:\Downloads\kaikki.org-dictionary-English.jsonl"#).unwrap()
            GzDecoder::new(
                BufReader::new(
                    File::open("dictionaries/Wiktionary/raw-wiktextract-data.jsonl.gz").unwrap()
                )
            )
        );

        let ct_err = reader.lines().par_bridge().map(|value| {
            if let Ok(value) = value {
                if let Ok(value) = serde_json::from_str::<ExtractedWord>(&value) {
                    (1, 0, 0)
                } else {
                    (0, 1, 0)
                }
            } else {
                (0, 0, 1)
            }
        }).reduce(||(0usize, 0usize, 0usize), |(a, b, c), (d, e, f)| {
            (a+d, b+e, c+f)
        });

        println!("{:?}", ct_err);

        // let mut ct_err = 0usize;
        // for content in reader.lines().map_ok(|value| {
        //     serde_json::from_str::<ExtractedWord>(&value)
        // }) {
        //     match content.unwrap() {
        //         Ok(_) => {}
        //         Err(err) => {
        //             ct_err += 1;
        //         }
        //     }
        // }
        //
        // println!("{ct_err}")
    }
}