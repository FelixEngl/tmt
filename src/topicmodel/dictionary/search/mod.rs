use crate::{define_py_literal, define_py_method};
use crate::topicmodel::dictionary::direction::LanguageKind;
use derive_more::From;
use pyo3::FromPyObject;
use strum::{EnumIs, EnumString};

mod index;
mod impls;
mod searcher;

#[derive(Debug, FromPyObject, EnumIs)]
pub enum SearchInput<'py> {
    String(String),
    Many(Vec<String>),
    Method(MatchWordMethod<'py>),
}

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

#[derive(Default, Debug)]
pub enum SearchType {
    Autocomplete,
    #[default]
    ExactMatch,
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

// define_py_literal!(
//     
// );

#[cfg(test)]
mod test {
    use crate::topicmodel::dictionary::search::index::SearchIndex;
    use crate::topicmodel::dictionary::search::searcher::DictionarySearch;
    use crate::topicmodel::dictionary::search::SearchType::Autocomplete;
    use crate::topicmodel::dictionary::{MutableDictionaryWithMeta, StringDictWithMetaDefault};

    #[test]
    fn can_exec(){
        let mut dict = StringDictWithMetaDefault::default();
        dict.push_invariant("hallo", "hello").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("Hallo", "hello").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("Welt", "world").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("Felix Engl", "Felix Engl").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("autowaschen", "to do car washing").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        dict.push_invariant("Hallo Welt", "hello world").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );

        dict.push_invariant("Herz", "heart").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );

        dict.push_invariant("Gürtel", "belt").use_consuming(
            |mut value| {
                value.add_dictionary("test1");
            },
            |mut value| {
                value.add_dictionary("test2");
            }
        );
        
        let index = SearchIndex::new();

        let searcher = DictionarySearch::new(&dict, &index);

        let result = searcher.search(
            "Gürtel",
            None,
            None,
            None,
            false
        ).expect("This should not fail!");

        println!("{result:?}");

        // searcher.init_prefix_dict_searcher(None);

        let result = searcher.search(
            "Gür",
            Some(Autocomplete),
            None,
            None,
            false
        ).expect("This should not fail!");
        println!("{result:?}");
    }
}