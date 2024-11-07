use num_enum::*;
use pyo3::{pyclass, pymethods};
use strum::*;
use serde::*;
use tinyset::Fits64;
use crate::register_python;
use crate::topicmodel::dictionary::loader::helper::gen_freedict_tei_reader::EPosElement;
use crate::topicmodel::dictionary::loader::helper::gen_ms_terms_reader::ETermNoteElement;
use crate::topicmodel::dictionary::metadata::loaded::impl_try_from_as_unpack;

register_python!(
    enum PartOfSpeech;
);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumIter)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum PartOfSpeech {
    /// Associated Tags:
    /// ```plaintext
    ///    - verbal
    ///    - ideophone
    ///    - dependent
    /// ```
    #[strum(to_string = "noun", serialize = "Noun")]
    #[strum(serialize = "verbal noun")]
    #[strum(serialize = "noum")]
    #[strum(serialize = "nouns")]
    #[strum(serialize = "νoun")]
    #[strum(serialize = "noun form")]
    #[strum(serialize = "noun")]
    #[strum(serialize = "ideophone")]
    #[strum(serialize = "nouɲ")]
    #[strum(serialize = "dependent noun")]
    Noun = 0,

    /// Associated Tags:
    /// ```plaintext
    ///    - present
    ///    - past
    ///    - perfect
    ///    - participle
    ///    - transitive
    ///    - infinitive
    ///    - intransitive
    ///    - gerund
    /// ```
    #[strum(to_string = "verb", serialize = "Verb")]
    #[strum(serialize = "verbs")]
    #[strum(serialize = "perfect expression")]
    #[strum(serialize = "present participle")]
    #[strum(serialize = "verb")]
    #[strum(serialize = "verb form")]
    #[strum(serialize = "instransitive verb")]
    #[strum(serialize = "perfect participle")]
    #[strum(serialize = "v")]
    #[strum(serialize = "perfection expression")]
    #[strum(serialize = "past-p")]
    #[strum(serialize = "past participle")]
    #[strum(serialize = "participle")]
    #[strum(serialize = "pres-p")]
    #[strum(serialize = "intransitive verb")]
    #[strum(serialize = "infinitive")]
    #[strum(serialize = "transitive verb")]
    #[strum(serialize = "gerund")]
    Verb = 1,

    #[strum(to_string = "adv", serialize = "Adv")]
    #[strum(serialize = "adverb")]
    #[strum(serialize = "adverbs")]
    #[strum(serialize = "adv")]
    #[strum(serialize = "adv.")]
    Adv = 2,

    /// Associated Tags:
    /// ```plaintext
    ///    - person
    ///    - interrogative
    ///    - prepositional
    ///    - relative
    /// ```
    #[strum(to_string = "pron", serialize = "Pron")]
    #[strum(serialize = "prepositional pronoun")]
    #[strum(serialize = "pronoun")]
    #[strum(serialize = "pron")]
    #[strum(serialize = "relativ.pron")]
    #[strum(serialize = "personal pronoun")]
    #[strum(serialize = "interrogative pronoun")]
    #[strum(serialize = "pron interrog")]
    #[strum(serialize = "ppron")]
    Pron = 3,

    /// Associated Tags:
    /// ```plaintext
    ///    - relative
    /// ```
    #[strum(to_string = "conj", serialize = "Conj")]
    #[strum(serialize = "relative")]
    #[strum(serialize = "conj")]
    #[strum(serialize = "conjunction")]
    #[strum(serialize = "conjuntion")]
    Conj = 4,

    #[strum(to_string = "prep", serialize = "Prep")]
    #[strum(serialize = "prepositions")]
    #[strum(serialize = "preposition")]
    #[strum(serialize = "prepositional expressions")]
    #[strum(serialize = "proposition")]
    #[strum(serialize = "prep")]
    #[strum(serialize = "prp")]
    Prep = 5,

    /// Associated Tags:
    /// ```plaintext
    ///    - possessive
    /// ```
    #[strum(to_string = "det", serialize = "Det")]
    #[strum(serialize = "possessive determiner")]
    #[strum(serialize = "det")]
    #[strum(serialize = "determiner")]
    #[strum(serialize = "possessive pronoun")]
    Det = 6,

    #[strum(to_string = "intj", serialize = "Intj")]
    #[strum(serialize = "interj")]
    #[strum(serialize = "interjection")]
    #[strum(serialize = "int")]
    Intj = 7,

    /// Associated Tags:
    /// ```plaintext
    ///    - number
    /// ```
    #[strum(to_string = "num", serialize = "Num")]
    #[strum(serialize = "number")]
    #[strum(serialize = "numeral")]
    Num = 8,

    /// Associated Tags:
    /// ```plaintext
    ///    - indefinite
    /// ```
    #[strum(to_string = "article", serialize = "Article")]
    #[strum(serialize = "article")]
    #[strum(serialize = "art")]
    #[strum(serialize = "indart")]
    #[strum(serialize = "indefinite article")]
    Article = 9,

    #[strum(to_string = "name", serialize = "Name")]
    #[strum(serialize = "pnoun")]
    #[strum(serialize = "proper noun")]
    #[strum(serialize = "proper oun")]
    Name = 10,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "prefix", serialize = "Prefix")]
    #[strum(serialize = "prefix")]
    Prefix = 11,

    /// Associated Tags:
    /// ```plaintext
    ///    - clitic
    ///    - morpheme
    /// ```
    #[strum(to_string = "suffix", serialize = "Suffix")]
    #[strum(serialize = "adjective suffix")]
    #[strum(serialize = "enclitic particle")]
    #[strum(serialize = "clitic")]
    #[strum(serialize = "suffix form")]
    #[strum(serialize = "enclitic")]
    #[strum(serialize = "suffix")]
    Suffix = 12,

    /// Associated Tags:
    /// ```plaintext
    ///    - han
    ///    - diacritic
    ///    - Hanja
    ///    - kanji
    ///    - letter
    ///    - ligature
    ///    - hanzi
    /// ```
    #[strum(to_string = "character", serialize = "Character")]
    #[strum(serialize = "han characters")]
    #[strum(serialize = "diacritical mark")]
    #[strum(serialize = "character")]
    #[strum(serialize = "hanja")]
    #[strum(serialize = "kanji")]
    #[strum(serialize = "letter")]
    #[strum(serialize = "ligature")]
    #[strum(serialize = "definitions")]
    #[strum(serialize = "hanzi")]
    #[strum(serialize = "han character")]
    Character = 13,

    #[strum(to_string = "particle", serialize = "Particle")]
    #[strum(serialize = "ptcl")]
    #[strum(serialize = "particle")]
    #[strum(serialize = "Partikel")]
    Particle = 14,

    #[strum(to_string = "other", serialize = "Other")]
    #[strum(serialize = "misc")]
    #[strum(serialize = "other")]
    Other = 15,

    /// Associated Tags:
    /// ```plaintext
    ///    - abbreviation
    /// ```
    #[strum(to_string = "abbrev", serialize = "Abbrev")]
    #[strum(serialize = "acronym")]
    #[strum(serialize = "clipping")]
    #[strum(serialize = "initialism")]
    #[strum(serialize = "abbreviation")]
    Abbrev = 16,

    /// Associated Tags:
    /// ```plaintext
    ///    - comparative
    ///    - ordinal
    ///    - predicative
    /// ```
    #[strum(to_string = "adj", serialize = "Adj")]
    #[strum(serialize = "comparative")]
    #[strum(serialize = "adjectives")]
    #[strum(serialize = "ordinal number")]
    #[strum(serialize = "adjectuve")]
    #[strum(serialize = "adj")]
    #[strum(serialize = "adjective")]
    #[strum(serialize = "predicative")]
    Adj = 17,

    #[strum(to_string = "adj_noun", serialize = "Adj_noun")]
    #[strum(serialize = "adjectival noun")]
    #[strum(serialize = "adjectival")]
    AdjNoun = 18,

    #[strum(to_string = "adj_verb", serialize = "Adj_verb")]
    #[strum(serialize = "adjectival verb")]
    AdjVerb = 19,

    #[strum(to_string = "adnominal", serialize = "Adnominal")]
    #[strum(serialize = "adnominal")]
    Adnominal = 20,

    #[strum(to_string = "adv_phrase", serialize = "Adv_phrase")]
    #[strum(serialize = "adverbial phrase")]
    AdvPhrase = 21,

    #[strum(to_string = "affix", serialize = "Affix")]
    #[strum(serialize = "affix")]
    Affix = 22,

    #[strum(to_string = "ambiposition", serialize = "Ambiposition")]
    #[strum(serialize = "ambiposition")]
    Ambiposition = 23,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "circumfix", serialize = "Circumfix")]
    #[strum(serialize = "circumfix")]
    Circumfix = 24,

    #[strum(to_string = "circumpos", serialize = "Circumpos")]
    #[strum(serialize = "circumposition")]
    Circumpos = 25,

    #[strum(to_string = "classifier", serialize = "Classifier")]
    #[strum(serialize = "classifier")]
    Classifier = 26,

    #[strum(to_string = "clause", serialize = "Clause")]
    #[strum(serialize = "nominal nuclear clause")]
    Clause = 27,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "combining_form", serialize = "Combining_form")]
    #[strum(serialize = "combining form")]
    CombiningForm = 28,

    /// Associated Tags:
    /// ```plaintext
    ///    - abbreviation
    /// ```
    #[strum(to_string = "contraction", serialize = "Contraction")]
    #[strum(serialize = "contraction")]
    Contraction = 29,

    #[strum(to_string = "converb", serialize = "Converb")]
    #[strum(serialize = "converb")]
    Converb = 30,

    #[strum(to_string = "counter", serialize = "Counter")]
    #[strum(serialize = "counter")]
    Counter = 31,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "infix", serialize = "Infix")]
    #[strum(serialize = "infix")]
    Infix = 32,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "interfix", serialize = "Interfix")]
    #[strum(serialize = "interfix")]
    Interfix = 33,

    /// Associated Tags:
    /// ```plaintext
    ///    - idiomatic
    /// ```
    #[strum(to_string = "phrase", serialize = "Phrase")]
    #[strum(serialize = "phrase")]
    #[strum(serialize = "phrases")]
    #[strum(serialize = "idiom")]
    Phrase = 34,

    #[strum(to_string = "postp", serialize = "Postp")]
    #[strum(serialize = "postposition")]
    Postp = 35,

    #[strum(to_string = "prep_phrase", serialize = "Prep_phrase")]
    #[strum(serialize = "prepositional phrase")]
    PrepPhrase = 36,

    #[strum(to_string = "preverb", serialize = "Preverb")]
    #[strum(serialize = "preverb")]
    Preverb = 37,

    #[strum(to_string = "proverb", serialize = "Proverb")]
    #[strum(serialize = "proverb")]
    Proverb = 38,

    /// Associated Tags:
    /// ```plaintext
    ///    - punctuation
    /// ```
    #[strum(to_string = "punct", serialize = "Punct")]
    #[strum(serialize = "punctuation")]
    #[strum(serialize = "punctuation mark")]
    Punct = 39,

    #[strum(to_string = "romanization", serialize = "Romanization")]
    #[strum(serialize = "romanization")]
    Romanization = 40,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "root", serialize = "Root")]
    #[strum(serialize = "root")]
    Root = 41,

    #[strum(to_string = "syllable", serialize = "Syllable")]
    #[strum(serialize = "syllable")]
    Syllable = 42,

    #[strum(to_string = "symbol", serialize = "Symbol")]
    #[strum(serialize = "symbol")]
    Symbol = 43,

}


impl_try_from_as_unpack! {
    PartOfSpeech => PartOfSpeech
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PartOfSpeech {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl Fits64 for PartOfSpeech {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        PartOfSpeech::try_from(x).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
}

impl From<ETermNoteElement> for PartOfSpeech {
    fn from(value: ETermNoteElement) -> Self {
        match value {
            ETermNoteElement::Noun => {
                Self::Noun
            }
            ETermNoteElement::Other => {
                Self::Other
            }
            ETermNoteElement::Verb => {
                Self::Verb
            }
            ETermNoteElement::ProperNoun => {
                Self::Name
            }
            ETermNoteElement::Adjective => {
                Self::Adj
            }
            ETermNoteElement::Adverb => {
                Self::Adj
            }
        }
    }
}

impl From<EPosElement> for PartOfSpeech {
    fn from(value: EPosElement) -> Self {
        match value {
            EPosElement::N => {
                Self::Noun
            }
            EPosElement::Adj => {
                Self::Adj
            }
            EPosElement::V => {
                Self::Verb
            }
            EPosElement::Adv => {
                Self::Adv
            }
            EPosElement::Int => {
                Self::Intj
            }
            EPosElement::Prep => {
                Self::Prep
            }
            EPosElement::Num => {
                Self::Num
            }
            EPosElement::Pron => {
                Self::Pron
            }
            EPosElement::Conj => {
                Self::Conj
            }
            EPosElement::Art => {
                Self::Article
            }
            EPosElement::Ptcl => {
                Self::Particle
            }
        }
    }
}














// old

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
// #[pyclass(eq, eq_int, hash, frozen)]
// #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
// #[derive(Display, EnumString, IntoStaticStr, EnumIter)]
// #[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
// #[repr(u64)]
// pub enum PartOfSpeech {
//     #[strum(to_string = "noun")]
//     Noun = 0,
//     #[strum(to_string = "adj", serialize = "adjective")]
//     Adjective = 1,
//     #[strum(to_string = "adv", serialize = "adv.")]
//     Adverb = 2,
//     #[strum(to_string = "verb", serialize = "v")]
//     Verb = 3,
//     #[strum(to_string = "conj")]
//     Conjuction = 4,
//     #[strum(to_string = "pron", serialize = "ppron")]
//     Pronoun = 5,
//     #[strum(to_string = "prep", serialize = "prp")]
//     Preposition = 6,
//     #[strum(to_string = "det")]
//     Determiner = 7,
//     #[strum(to_string = "int", serialize = "interj")]
//     Interjection = 8,
//     #[strum(to_string="pres-p")]
//     PresentParticiple = 9,
//     #[strum(to_string="past-p")]
//     PastParticiple = 10,
//     #[strum(to_string="prefix")]
//     Prefix = 11,
//     #[strum(to_string="suffix")]
//     Suffix = 12,
//     #[strum(to_string="num")]
//     Numeral = 13,
//     #[strum(to_string="art")]
//     Article = 14,
//     #[strum(to_string="ptcl", serialize = "particle", serialize = "Partikel")]
//     Particle = 15,
//     #[strum(to_string="pnoun")]
//     ProperNoun = 16,
//     #[strum(to_string="other", serialize = "misc")]
//     Other = 17,
//     ///```plaintext ```
//     #[strum(to_string="indart", serialize = "indefinite article")]
//     IndefiniteArticle = 18,
//     #[strum(to_string="pron interrog")]
//     InterrogativePronoun = 19,
//     #[strum(to_string="relativ.pron")]
//     RelativePronoun = 20,
// }