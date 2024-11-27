use crate::dictionary::direction::LanguageKind;
use derive_more::From;
use pyo3::{pyclass, FromPyObject};
use strum::{Display, EnumIs};

mod index;
mod impls;
mod searcher;

pub use index::*;
use ldatranslate_toolkit::{define_py_literal, define_py_method, impl_py_stub, register_python};
pub use searcher::*;

register_python! {
    enum SearchType;
}

#[derive(Debug, FromPyObject, EnumIs)]
pub enum SearchInput<'py> {
    String(String),
    Many(Vec<String>),
    Method(MatchWordMethod<'py>),
}

impl_py_stub!(SearchInput<'_>: String, Vec<String>, MatchWordMethod<'_>);


impl<'py> From<MatchWordMethod<'py>> for SearchInput<'py> {
    fn from(value: MatchWordMethod<'py>) -> Self {
        Self::Method(value)
    }
}

impl<'py, S> From<S> for SearchInput<'py> where S: Into<String> {
    fn from(value: S) -> Self {
        Self::String(value.into())
    }
}

impl<'py, S> FromIterator<S> for SearchInput<'py> where S: Into<String> {
    fn from_iter<T: IntoIterator<Item=S>>(iter: T) -> Self {
        Self::Many(iter.into_iter().map(S::into).collect())
    }
}

define_py_method!(MatchWord(language_hint: LanguageKind, word: &str) -> bool);

/// The modes supported by the search.
/// - ExactMatch (aliases: "e", "exact")
///     Matches the word or words provided in an exact manner.
///
/// - Autocomplete (aliases: "a", "auto")
///     Uses a prefix trie to search for words with a common prefix.
///
/// - Contains (aliases: "c", "contains")
///     Returns all words that contain the same string
///
/// - StartsWith (aliases: "start", "starts_with")
///     Returns all words that start with the same string.
///
/// - EndsWith (aliases: "end", "ends_with")
///     Returns all words that end with the same string
///
/// - Regex (aliases: "r", "reg", "regex")
///     Matches one or multiple regex expressions, returns all words that match.
///
/// - Hamming (aliases: "h", "ham", "hamming")
///     Returns everything that is <= threshold for a hamming distance between two words.
///
/// - Levensthein (aliases: "l", "lev", "levensthein")
///     Returns everything that is <= threshold for a levensthein distance between two words.
///
/// - OsaDistance (aliases: "o", "osa", "osa_dist", "osa_distance")
///     Returns everything that is <= threshold for a osa distance between two words.
///     Like Levenshtein but allows for adjacent transpositions. Each substring can only be edited once.
///
/// - NormalizedLevensthein (aliases: "l+n", "lev_norm", "lev_normalized")
///     Returns everything that is >= threshold for a normalized levensthein distance between two words.
///     The normalized value is between 0.0 and 1.0, where 1.0 means that strings are the same.
///
/// - DamerauLevensthein (aliases: "dl", "dam_lev", "damerau_levensthein")
///     Returns everything that is <= threshold for a damerau levensthein distance between two words.
///
/// - NormalizedDamerauLevensthein (aliases: "dl+n", "dam_lev_norm", "damerau_levensthein_norm", "damerau_levensthein_normalized"):
///     Returns everything that is >= threshold for a normalized damerau levensthein distance between two words.
///     The normalized value is between 0.0 and 1.0, where 1.0 means that strings are the same.
///
/// - Jaro (aliases: "j", "jaro")
///     Returns everything that is >= threshold for a jaro distance between two words.
///     The returned value is between 0.0 and 1.0 (higher value means more similar).
///
/// - JaroWinkler (aliases: "jw", "jaro_winkler")
///     Returns everything that is >= threshold for a jaro winkler distance between two words.
///     The returned value is between 0.0 and 1.0 (higher value means more similar).
///     Gives a boost to strings where a common prefix exists.
///
/// - SorensenDice (aliases: "s", "soren", "sorensen_dice")
///     Returns everything that is >= threshold for a sorensen dice distance between two words.
///     The returned value is between 0.0 and 1.0 (higher value means more similar), where 1.0 means that strings are the same.
///     Calculates a Sørensen-Dice similarity distance using bigrams. See https:// en. wikipedia. org/ wiki/ S%C3%B8rensen%E2%80%93Dice_coefficient
///
/// - CommonPrefix (aliases: "cp", "com_pre", "common_prefix")
///     Returns every word that has a common prefix (the provided string/strings).
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Default, Debug, Display, Eq, PartialEq, Hash, Copy, Clone)]
pub enum SearchType {
    #[default]
    ExactMatch,
    Autocomplete,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    Hamming,
    Levensthein,
    OsaDistance,
    NormalizedLevensthein,
    DamerauLevensthein,
    NormalizedDamerauLevensthein,
    Jaro,
    JaroWinkler,
    SorensenDice,
    CommonPrefix
}

define_py_literal!(
    pub SearchTypeLiteral for SearchType [
        "a" => SearchType::Autocomplete,
        "auto" => SearchType::Autocomplete,
        "e" => SearchType::ExactMatch,
        "exact" => SearchType::ExactMatch,
        "c" => SearchType::Contains,
        "contains" => SearchType::Contains,
        "start" => SearchType::StartsWith,
        "starts_with" => SearchType::StartsWith,
        "end" => SearchType::EndsWith,
        "ends_with" => SearchType::EndsWith,
        "r" => SearchType::Regex,
        "reg" => SearchType::Regex,
        "regex" => SearchType::Regex,
        "h" => SearchType::Hamming,
        "ham" => SearchType::Hamming,
        "hamming" => SearchType::Hamming,
        "l" => SearchType::Levensthein,
        "lev" => SearchType::Levensthein,
        "levensthein" => SearchType::Levensthein,
        "o" => SearchType::OsaDistance,
        "osa" => SearchType::OsaDistance,
        "osa_dist" => SearchType::OsaDistance,
        "osa_distance" => SearchType::OsaDistance,
        "l+n" => SearchType::NormalizedLevensthein,
        "lev_norm" => SearchType::NormalizedLevensthein,
        "lev_normalized" => SearchType::NormalizedLevensthein,
        "dl" => SearchType::DamerauLevensthein,
        "dam_lev" => SearchType::DamerauLevensthein,
        "damerau_levensthein" => SearchType::DamerauLevensthein,
        "dl+n" => SearchType::NormalizedDamerauLevensthein,
        "dam_lev_norm" => SearchType::NormalizedDamerauLevensthein,
        "damerau_levensthein_norm" => SearchType::NormalizedDamerauLevensthein,
        "damerau_levensthein_normalized" => SearchType::NormalizedDamerauLevensthein,
        "j" => SearchType::Jaro,
        "jaro" => SearchType::Jaro,
        "jw" => SearchType::JaroWinkler,
        "jaro_winkler" => SearchType::JaroWinkler,
        "s" => SearchType::SorensenDice,
        "soren" => SearchType::SorensenDice,
        "sorensen_dice" => SearchType::SorensenDice,
        "cp" => SearchType::CommonPrefix,
        "com_pre" => SearchType::CommonPrefix,
        "common_prefix" => SearchType::CommonPrefix,
    ]
);

#[cfg(test)]
mod test {
    use crate::dictionary::search::index::SearchIndex;
    use crate::dictionary::search::searcher::DictionarySearcher;
    use crate::dictionary::io::ReadableDictionary;

    #[test]
    fn can_exec(){
        // let mut dict = StringDictWithMetaDefault::default();
        // dict.push_invariant("hallo", "hello").use_consuming(
        //     |mut value| {
        //         value.add_dictionary("test1");
        //     },
        //     |mut value| {
        //         value.add_dictionary("test2");
        //     }
        // );
        // dict.push_invariant("Hallo", "hello").use_consuming(
        //     |mut value| {
        //         value.add_dictionary("test1");
        //     },
        //     |mut value| {
        //         value.add_dictionary("test2");
        //     }
        // );
        // dict.push_invariant("Welt", "world").use_consuming(
        //     |mut value| {
        //         value.add_dictionary("test1");
        //     },
        //     |mut value| {
        //         value.add_dictionary("test2");
        //     }
        // );
        // dict.push_invariant("Felix Engl", "Felix Engl").use_consuming(
        //     |mut value| {
        //         value.add_dictionary("test1");
        //     },
        //     |mut value| {
        //         value.add_dictionary("test2");
        //     }
        // );
        // dict.push_invariant("autowaschen", "to do car washing").use_consuming(
        //     |mut value| {
        //         value.add_dictionary("test1");
        //     },
        //     |mut value| {
        //         value.add_dictionary("test2");
        //     }
        // );
        // dict.push_invariant("Hallo Welt", "hello world").use_consuming(
        //     |mut value| {
        //         value.add_dictionary("test1");
        //     },
        //     |mut value| {
        //         value.add_dictionary("test2");
        //     }
        // );
        //
        // dict.push_invariant("Herz", "heart").use_consuming(
        //     |mut value| {
        //         value.add_dictionary("test1");
        //     },
        //     |mut value| {
        //         value.add_dictionary("test2");
        //     }
        // );
        //
        // dict.push_invariant("Gürtel", "belt").use_consuming(
        //     |mut value| {
        //         value.add_dictionary("test1");
        //     },
        //     |mut value| {
        //         value.add_dictionary("test2");
        //     }
        // );


        // let dict = DefaultDict::from_path_with_extension(r#"E:\git\tmt\test\dictionary3.dat.zst"#).unwrap();
        //
        //
        // let index = SearchIndex::new();
        //
        // let searcher = DictionarySearcher::new(&dict, &index);
        //
        // searcher.force_init();

        // let result = searcher.search(
        //     "Dog",
        //     None,
        //     None,
        //     None,
        //     false
        // ).expect("This should not fail!");
        //
        // println!("{result:?}");
        //
        // // searcher.init_prefix_dict_searcher(None);
        //
        // let result = searcher.search(
        //     "Gür",
        //     Some(Autocomplete),
        //     None,
        //     None,
        //     false
        // ).expect("This should not fail!");
        // println!("{result:?}");
    }
}