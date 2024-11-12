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
use crate::topicmodel::dictionary::metadata::ex::MetadataCollectionBuilder;
use crate::topicmodel::dictionary::word_infos::{AnyWordInfo, Domain, Language, PartOfSpeech, PartOfSpeechTag};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtractedWord {
    /// the word form
    pub word: String,
    /// part-of-speech, such as "noun", "verb", "adj", "adv", "pron", "determiner", "prep" (preposition), "postp" (postposition), and many others. The complete list of possible values returned by the package can be found in wiktextract.PARTS_OF_SPEECH.
    #[serde(default)]
    pub pos: Option<String>,
    /// name of the language this word belongs to (e.g., English)
    pub lang: String,
    /// Wiktionary language code corresponding to lang key (e.g., en)
    pub lang_code: String,
    /// list of word senses (dictionaries) for this word/part-of-speech (see below)
    #[serde(default)]
    pub senses: Vec<Sense>,
    /// list of inflected or alternative forms specified for the word (e.g., plural, comparative, superlative, roman script version).
    /// This is a list of dictionaries, where each dictionary has a form key and a tags key.
    /// The tags identify what type of form it is. It may also contain "ipa", "roman", and "source" fields.
    /// The form can be "-" when the word is marked as not having that form
    /// (some of those will be word-specific, while others are language-specific; post-processing
    /// can drop such forms when no word has a value for that tag combination).
    #[serde(default)]
    pub forms: Vec<Form>,
    /// list of dictionaries containing pronunciation, hyphenation, rhyming, and related information.
    /// Each dictionary may have a tags key containing tags that clarify what kind of form that
    /// entry is. Different types of information are stored in different fields: ipa is IPA
    /// pronunciation, enPR is enPR pronunciation, audio is name of sound file in Wikimedia commons.
    #[serde(default, skip, rename = "sounds")]
    _sounds: IgnoredAny,
    /// list of non-disambiguated categories for the word
    #[serde(default)]
    pub categories: Vec<String>,
    /// list of non-disambiguated topics for the word
    #[serde(default)]
    pub topics: Vec<String>,
    /// non-disambiguated translation entries (see below)
    #[serde(default)]
    pub translations: Vec<Translation>,
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
    pub synonyms: Vec<Linkage>,
    /// non-disambiguated antonym linkages for the word (see below)
    #[serde(default)]
    pub antonyms: Vec<Linkage>,
    /// non-disambiguated hypernym linkages for the word (see below)
    #[serde(default)]
    pub hypernyms: Vec<Linkage>,
    /// non-disambiguated linkages indicating being part of something (see below) (not systematically encoded)
    #[serde(default)]
    pub holonyms: Vec<Linkage>,
    /// non-disambiguated linkages indicating having a part (see below) (fairly rare)
    #[serde(default)]
    pub meronyms: Vec<Linkage>,
    /// non-disambiguated derived word linkages for the word (see below)
    #[serde(default)]
    pub derived: Vec<Linkage>,
    /// non-disambiguated related word linkages for the word (see below)
    #[serde(default)]
    pub related: Vec<Linkage>,
    /// non-disambiguated coordinate term linkages for the word (see below)
    #[serde(default)]
    pub coordinate_terms: Vec<Linkage>,
    /// non-disambiguated Wikidata identifer
    #[serde(default, deserialize_with = "deserialize_optional_flat_either")]
    pub wikidata: Option<Either<String, Vec<String>>>,
    /// non-disambiguated page title in Wikipedia (possibly prefixed by language id)
    #[serde(default)]
    pub wiktionary: Option<String>,
    /// part-of-speech specific head tags for the word. This basically just captures the templates (their name and arguments) as a list of dictionaries. Most applications may want to ignore this.
    #[serde(default, skip, rename = "head_templates")]
    _head_templates: IgnoredAny,
    /// conjugation and declension templates found for the word, as dictionaries. These basically capture the language-specific inflection template for the word. Note that for some languages inflection information is also contained in head_templates. XXX in the very near future, we will start parsing inflections from the inflection tables into forms, so there is usually no need to use the inflection_templates data.
    #[serde(default, skip, rename = "inflection_templates")]
    _inflection_templates: IgnoredAny,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dictionary {
    /// Linked word
    word: String,
    /// additional text
    extra: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sense {
    /// list of gloss strings for the word sense (usually only one). This has been cleaned, and should be straightforward text with no tagging.
    #[serde(default)]
    pub glosses: Vec<String>,
    /// list of gloss strings for the word sense, with less cleaning than
    #[serde(default)]
    pub raw_glosses: Vec<String>,
    /// list of qualifiers and tags for the gloss.
    /// This is a list of strings, and may include
    /// words such as "archaic", "colloquial", "present",
    /// "participle", "plural", "feminine", and many others
    /// (new words may appear arbitrarily).
    #[serde(default)]
    pub tags: Vec<String>,
    /// list of sense-disambiguated category names extracted from (a subset) of the Category links on the page
    #[serde(default)]
    pub categories: Vec<String>,
    /// list of sense-disambiguated topic names (kind of similar to categories but determined differently)
    #[serde(default)]
    pub topics: Vec<String>,
    /// list of words that his sense is an alternative form of; this is a list of dictionaries, with field word containing the linked word and optionally extra containing additional text
    #[serde(default, skip)]
    alt_of: Vec<Dictionary>,
    /// list of words that this sense is an inflected form of; this is a list of dictionaries, with field word containing the linked word and optionally extra containing additional text
    #[serde(default, skip, rename = "form_of")]
    _form_of: IgnoredAny,
    /// sense-disambiguated translation entries (see below)
    #[serde(default)]
    pub translations: Vec<Translation>,
    /// sense-disambiguated synonym linkages for the word (see below)
    #[serde(default)]
    pub synonyms: Vec<Linkage>,
    /// sense-disambiguated antonym linkages for the word (see below)
    #[serde(default)]
    pub antonyms: Vec<Linkage>,
    /// sense-disambiguated hypernym linkages for the word (see below)
    #[serde(default)]
    pub hypernyms: Vec<Linkage>,
    /// sense-disambiguated linkages indicating being part of something (see below) (not systematically encoded)
    #[serde(default)]
    pub holonyms: Vec<Linkage>,
    /// sense-disambiguated linkages indicating having a part (see below) (fairly rare)
    #[serde(default)]
    pub meronyms: Vec<Linkage>,
    /// sense-disambiguated coordinate_terms linkages (see below)
    #[serde(default)]
    pub coordinate_terms: Vec<Linkage>,
    /// sense-disambiguated derived word linkages for the word (see below)
    #[serde(default)]
    pub derived: Vec<Linkage>,
    /// sense-disambiguated related word linkages for the word (see below)
    #[serde(default)]
    pub related: Vec<Linkage>,
    /// list of textual identifiers collected for the sense. If there is a QID for the entry (e.g., Q123), those are stored in the wikidata field.
    #[serde(default)]
    pub senseid: Vec<String>,
    /// list of QIDs (e.g., Q123) for the sense
    #[serde(default)]
    pub wikidata: Vec<String>,
    /// list of Wikipedia page titles (with optional language code prefix)
    #[serde(default)]
    pub wikipedia: Vec<String>,
    /// list of usage examples, each example being a dictionary with text field containing the example text, optional ref field containing a source reference, optional english field containing English translation, optional type field containing example type (currently example or quotation if present), optional roman field containing romanization (for some languages written in non-Latin scripts), and optional (rare) note field contains English-language parenthesized note from the beginning of a non-english example.
    #[serde(default, skip, rename = "examples")]
    _examples: IgnoredAny,
    /// if the word sense has a qualifier that could not be parsed, that qualifier is put in this field (rare). Most qualifiers are parsed into tags and/or topics. The gloss with the qualifier still present can be found in raw_glosses.
    #[serde(default)]
    pub english: Vec<String>,
}

/// Pronunciation -> Ignored bc sounds are ignored

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Translation {
    /// optional alternative form of the translation (e.g., in a different script)
    #[serde(default)]
    pub alt: Option<String>,
    /// Wiktionary's 2 or 3-letter language code for the language the translation is for
    #[serde(default)]
    pub code: Option<String>,
    /// English text, generally clarifying the target sense of the translation
    #[serde(default, skip, rename = "english")]
    _english: IgnoredAny,
    /// the language name that the translation is for
    pub lang: String,
    /// optional text describing or commenting on the translation
    #[serde(default)]
    pub note: Option<String>,
    /// optional romanization of the translation (when in non-Latin characters)
    #[serde(default, skip, rename = "roman")]
    _roman: IgnoredAny,
    /// optional sense indicating the meaning for which this is a translation (this is a free-text string, and may not match any gloss exactly)
    #[serde(default)]
    pub sense: Option<String>,
    /// optional list of qualifiers for the translations, e.g., gender
    #[serde(default)]
    pub tags: Vec<String>,
    /// optional taxonomic name of an organism mentioned in the translation
    #[serde(default)]
    pub taxonomic: Option<String>,
    /// the translation in the specified language (may be missing when note is present)
    #[serde(default)]
    pub word: Option<String>,
}

/// Etymologies ignored bc entymologie nicht spannend
/// Descendants ignored bc entymologie nicht spannend

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Linkage {
    /// optional alternative form of the target (e.g., in a different script)
    #[serde(default)]
    pub alt: Option<String>,
    /// optional English text associated with the sense, usually identifying the linked target sense
    #[serde(default, skip, rename = "english")]
    _english: IgnoredAny,
    /// optional romanization of a linked word in a non-Latin script
    #[serde(default, skip, rename = "roman")]
    _roman: IgnoredAny,
    /// text identifying the word sense or context (e.g., "to rain very heavily")
    #[serde(default, deserialize_with = "deserialize_optional_flat_either")]
    pub sense: Option<Either<String, Vec<String>>>,
    /// qualifiers specified for the sense (e.g., field of study, region, dialect, style)
    #[serde(default)]
    pub tags: Vec<String>,
    /// optional taxonomic name associated with the linkage
    #[serde(default)]
    pub taxonomic: Option<String>,
    /// list of topic descriptors for the linkage (e.g., military)
    #[serde(default)]
    pub topics: Vec<String>,
    /// the word this links to (string)
    pub word: String,
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


macro_rules! try_parse_without_prefix {
    ($i: ident as $ty: ty: $($l: literal),+ $(,)?) => {
        {
            #[inline(always)]
            fn parse_method(s: &str) -> Result<$ty, strum::ParseError>{
                $(
                    if s.starts_with($l) {
                        let trimmed = s.trim_start_matches($l);
                        if let Ok(value) = trimmed.parse::<$ty>() {
                            return Ok(value);
                        }
                        if let Ok(value) = trimmed.to_lowercase().parse::<$ty>() {
                            return Ok(value);
                        }
                    }
                )+
                Err(strum::ParseError::VariantNotFound)
            }
            parse_method($i.as_ref())
        }
    };
}


pub struct ExtractedWordValues<S> {
    pub main: (String, Language, MetadataCollectionBuilder<S>),
    /// Variants of the word
    pub forms: Vec<(String, MetadataCollectionBuilder<S>)>,
    /// Synonyms of the word
    pub synonyms: Vec<(String, MetadataCollectionBuilder<S>)>,
    pub antonyms: Vec<(String, MetadataCollectionBuilder<S>)>,
    pub related: Vec<(String, MetadataCollectionBuilder<S>)>,
    pub translations: Vec<(String, Language, MetadataCollectionBuilder<S>)>,
}

impl<S> ExtractedWordValues<S> {
    pub fn new(main: (String, Language, MetadataCollectionBuilder<S>)) -> Self {
        Self {
            main,
            forms: Vec::new(),
            synonyms: Vec::new(),
            antonyms: Vec::new(),
            related: Vec::new(),
            translations: Vec::new()
        }
    }
}


pub fn convert_entry_to_entries(
    ExtractedWord {
        word,
        pos,
        lang,
        // Not needed
        lang_code: _,
        senses,
        forms,
        _sounds,
        categories,
        topics,
        translations,
        _etymology_text,
        _etymology_templates,
        _etymology_number,
        _descendants,
        synonyms,
        antonyms,
        hypernyms: _,
        holonyms: _,
        meronyms: _,
        derived: _,
        related,
        coordinate_terms: _,
        wikidata: _,
        wiktionary: _,
        _head_templates,
        _inflection_templates,
    }: ExtractedWord,
    target_languages: &[Language]
) -> Result<ExtractedWordValues<String>, EntryConversionError> {


    fn parse_categories(word_meta_builder: &mut MetadataCollectionBuilder<String>, categories: Vec<String>) {
        for cat in categories {
            match cat.parse::<AnyWordInfo>() {
                Ok(value) => {
                    word_meta_builder.push_any_word_info(value);
                }
                Err(_) => {
                    if let Ok(parsed) = try_parse_without_prefix!(
                        cat as AnyWordInfo: "en:", "de:", "ger:"
                    ) {
                        word_meta_builder.push_any_word_info(parsed);
                        continue
                    }

                    if cat.starts_with("Terms with") && cat.ends_with("translations")
                        || cat.starts_with("Pages with")
                    {
                        continue
                    }
                    word_meta_builder.push_unclassified(format!("#cat:{}", cat));
                }
            }
        }
    }
    fn parse_tags(word_meta_builder: &mut MetadataCollectionBuilder<String>, tags: Vec<String>) {
        for tag in tags {
            match tag.parse::<AnyWordInfo>() {
                Ok(value) => {
                    word_meta_builder.push_any_word_info(value);
                }
                Err(_) => {
                    word_meta_builder.push_unclassified(format!("#tag:{}", tag));
                }
            }
        }
    }
    fn parse_topics(word_meta_builder: &mut MetadataCollectionBuilder<String>, topics: Vec<String>) {
        for topic in topics {
            match topic.parse::<Domain>() {
                Ok(value) => {
                    word_meta_builder.push_domains(value);
                }
                Err(_) => {
                    word_meta_builder.push_unclassified(format!("#topic:{}", topic));
                }
            }
        }
    }

    // Base and senses
    let mut result = {
        let lang = lang.parse::<Language>().map_err(|_| EntryConversionError::WrongLanguageError(lang))?;
        let mut word_meta_builder = MetadataCollectionBuilder::with_name(None);
        word_meta_builder.push_languages(lang);
        parse_categories(&mut word_meta_builder, categories);
        parse_topics(&mut word_meta_builder, topics);

        if let Some(pos) = pos {
            if let Ok(pos) = pos.parse::<PartOfSpeech>() {
                word_meta_builder.push_pos(pos);
            }
            if let Some(pos_tags) = PartOfSpeechTag::get_tags(&pos) {
                word_meta_builder.extend_pos_tag(pos_tags.into_iter().copied());
            }
        }

        for Sense {
            glosses: _,
            raw_glosses: _,
            tags,
            categories,
            topics,
            alt_of: _,
            _form_of,
            // No relevant found with a translation
            translations: _,
            synonyms:_ ,
            antonyms:_ ,
            hypernyms:_ ,
            holonyms:_ ,
            meronyms:_ ,
            coordinate_terms:_ ,
            derived:_ ,
            related:_ ,
            senseid:_ ,
            wikidata:_ ,
            wikipedia:_ ,
            _examples,
            english:_ ,
        } in senses {
            parse_topics(&mut word_meta_builder, topics);
            parse_tags(&mut word_meta_builder, tags);
            parse_categories(&mut word_meta_builder, categories);
        }

        ExtractedWordValues::new((word, lang, word_meta_builder))
    };

    // forms
    {
        for Form {
            form,
            tags,
            // Unused, this are strange tags
            raw_tags: _
        } in forms {
            let mut meta = MetadataCollectionBuilder::with_name(None);
            parse_tags(&mut meta, tags);
            result.forms.push((form, meta))
        }
    }

    // translations
    {
        for Translation {
            alt: _,
            code,
            _english,
            lang,
            note,
            _roman,
            sense,
            tags,
            taxonomic,
            word
        } in translations {
            let lang = if let Ok(lang) = lang.parse::<Language>() {
                lang
            } else if let Some(Ok(lang)) = code.map(|v| v.parse::<Language>()) {
                lang
            } else {
                continue
            };
            if !target_languages.contains(&lang) {
                continue
            }
            let mut meta = MetadataCollectionBuilder::with_name(None);
            parse_tags(&mut meta, tags);
            if let Some(sense) = sense {
                meta.push_contextual_informations(sense);
            }

            if let Some(taxonomic) = taxonomic {
                let mut copy = meta.clone();
                copy.push_domains(Domain::T);
                result.translations.push((taxonomic, lang, copy))
            }

            if let Some(word) = word {
                result.translations.push((word, lang, meta));
            } else if let Some(note) = note {
                result.translations.push((note, lang, meta));
            } else {
                unreachable!("Why was this reached? There should always be a note or a word in a translation!")
            }
        }
    }


    fn process_linkage_list(list: Vec<Linkage>) -> Option<Vec<(String, MetadataCollectionBuilder<String>)>> {
        if list.is_empty() {
            return None
        }
        let mut result = Vec::new();
        for Linkage {
            alt: _,
            _english,
            _roman,
            sense: _,
            tags,
            taxonomic,
            topics,
            word
        } in list {
            let mut meta = MetadataCollectionBuilder::with_name(None);
            parse_tags(&mut meta, tags);
            parse_topics(&mut meta, topics);
            if let Some(tax) = taxonomic {
                let mut copy = meta.clone();
                copy.push_domains(Domain::T);
                result.push((tax, copy));
            }
            result.push((word, meta));
        }
        Some(result)
    }

    // related
    {
        if let Some(r) = process_linkage_list(related) {
            result.related.extend(r);
        }
    }

    // synonyms
    {
        if let Some(r) = process_linkage_list(synonyms) {
            result.synonyms.extend(r);
        }
    }

    // antonyms
    {
        if let Some(r) = process_linkage_list(antonyms) {
            result.antonyms.extend(r);
        }
    }

    Ok(result)
}


#[derive(Debug, Error)]
pub enum EntryConversionError {
    #[error("Was not able to parse the language \"{0}\" to a known language.")]
    WrongLanguageError(String),
    #[error("The pos marker {0} is unknown!")]
    UnknownPOS(String)
}


#[cfg(test)]
mod test {
    use std::collections::{HashSet};
    use std::fs::File;
    use either::Either;
    use itertools::Itertools;
    use rayon::prelude::*;
    use crate::define_aho_matcher;
    use crate::topicmodel::dictionary::loader::wiktionary_reader::{read_wiktionary, WiktionaryReaderError};
    use crate::topicmodel::dictionary::word_infos::{Domain, AnyWordInfo};

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
            static LANGUAGE1 = "english" as ascii_case_insensitive
        );
        define_aho_matcher!(
            static LANGUAGE2 = "german" as ascii_case_insensitive
        );
        let topics1 = read_wiktionary("dictionaries/Wiktionary/raw-wiktextract-data.jsonl.gz")
            .unwrap()
            // .into_par_iter()

            .filter_map(|value| {
                match value {
                    Ok(Either::Right(value)) => {
                        Some(value)
                    }
                    _ => None
                }
            })
            .filter(|value|
                        !value.coordinate_terms.is_empty()
                    // &&
                    // value.senses.iter().any(|value| !value.translations.is_empty())
            )
            .filter(
                |value|
                    LANGUAGE1.is_match(&value.lang)
                        && value.translations.iter().any(|value| LANGUAGE2.is_match(&value.lang))
            )
            .take(5)
            .collect::<Vec<_>>();

        // let topics2 = read_wiktionary("dictionaries/Wiktionary/raw-wiktextract-data.jsonl.gz")
        //     .unwrap()
        //     // .into_par_iter()
        //     .filter_map(|value| {
        //         match value {
        //             Ok(Either::Right(value)) => {
        //                 Some(value)
        //             }
        //             _ => None
        //         }
        //     })
        //     .filter(|value| value.senses.len() > 1)
        //     .filter(
        //         |value|
        //             LANGUAGE2.is_match(&value.lang)
        //                 && value.translations.iter().any(|value| LANGUAGE1.is_match(&value.lang))
        //     )
        //     .take(5)
        //     .collect::<Vec<_>>();

        serde_json::to_writer_pretty(
            File::options().write(true).truncate(true).create(true).open("example_data1.json").unwrap(),
            &topics1
        ).unwrap();

        // serde_json::to_writer_pretty(
        //     File::options().write(true).truncate(true).create(true).open("example_data2.json").unwrap(),
        //     &topics2
        // ).unwrap();

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

    #[test]
    fn test() {


        // let topics: HashSet<String> = serde_json::from_reader(
        //     File::options().read(true).open("data.json").unwrap()
        // ).unwrap();
        //
        // let mut mappings = indexmap::IndexMap::new();
        // let mut not_recognized = HashSet::new();
        // define_aho_matcher!(
        //     static IDENT = "aer" as ascii_case_insensitive
        // );
        // for value in topics {
        //     if value.parse::<Domain>().is_ok() {
        //         continue
        //     }
        //     if let Some(found) = identify_possible_topics(&value) {
        //         for (a, b) in found {
        //             mappings.entry(a).or_insert_with(HashSet::new).insert((value.clone(), b));
        //         }
        //     } else {
        //         if IDENT.is_match(&value) {
        //             println!("{value}")
        //         }
        //         not_recognized.insert(value);
        //     }
        // }
        //
        // let mut not_recognized = Vec::from_iter(not_recognized.into_iter());
        // not_recognized.sort();
        // mappings.sort_keys();
        //
        // let mut f = File::options().write(true).create(true).truncate(true).open("data_not_rec.json").unwrap();
        // serde_json::to_writer_pretty(&mut f, &not_recognized).unwrap();
        //
        // let mut f = File::options().write(true).create(true).truncate(true).open("data_rec.json").unwrap();
        // serde_json::to_writer_pretty(&mut f, &mappings).unwrap();

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
                    Err(WiktionaryReaderError::Serde(err, s)) => {
                        panic!("{s}\n\n{err}")
                    }
                    _ => None
                }
            })
            .filter(|value| LANGUAGE.is_match(&value.lang) && value.translations.iter().any(|value| LANGUAGE.is_match(&value.lang)))
            .map(|value| {
                value.senses
                    .into_iter()
                    .map(|value| value.tags)
                    .chain(
                        value.translations.into_iter()
                            .filter(|value| LANGUAGE.is_match(&value.lang))
                            .map(|value| value.tags)
                    )
                    .flatten()
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<HashSet<_>>();

        let x = topics.into_iter().into_group_map_by(|value| value.parse::<AnyWordInfo>().ok().map(|value| value.to_string()));
        for (a, mut b) in x.into_iter() {
            b.sort();
            if let Some(a) = a {
                let mut f = File::options().write(true).truncate(true).create(true).open(format!("tags_{a}.json")).unwrap();
                serde_json::to_writer_pretty(&mut f, &b).unwrap()
            } else {
                let mut f = File::options().write(true).truncate(true).create(true).open(format!("tags_UNKNOWN.json")).unwrap();
                serde_json::to_writer_pretty(&mut f, &b).unwrap()
            }
        }


    }
}
