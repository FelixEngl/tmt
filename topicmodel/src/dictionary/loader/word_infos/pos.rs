use num_enum::*;
use pyo3::{pyclass, pymethods};
use strum::*;
use serde::*;
use tinyset::Fits64;
use ldatranslate_toolkit::register_python;
use crate::dictionary::loader::helper::gen_freedict_tei_reader::EPosElement;
use crate::dictionary::loader::helper::gen_ms_terms_reader::ETermNoteElement;
use crate::dictionary::metadata::ex::impl_try_from_as_unpack;

register_python!(
    enum PartOfSpeech;
);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumIter)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u16)]
pub enum PartOfSpeech {
    /// Associated Tags:
    /// ```plaintext
    ///    - dependent
    ///    - ideophone
    ///    - verbal
    /// ```
    #[strum(to_string = "noun", serialize = "Noun")]
    #[strum(serialize = "νoun")]
    #[strum(serialize = "nouns")]
    #[strum(serialize = "nouɲ")]
    #[strum(serialize = "dependent noun")]
    #[strum(serialize = "noun form")]
    #[strum(serialize = "ideophone")]
    #[strum(serialize = "noum")]
    #[strum(serialize = "NOUN")]
    #[strum(serialize = "verbal noun")]
    Noun = 0,

    /// Associated Tags:
    /// ```plaintext
    ///    - gerund
    ///    - perfect
    ///    - intransitive
    ///    - transitive
    ///    - infinitive
    ///    - participle
    ///    - past
    ///    - present
    /// ```
    #[strum(to_string = "verb", serialize = "Verb")]
    #[strum(serialize = "gerund")]
    #[strum(serialize = "present participle")]
    #[strum(serialize = "verbs")]
    #[strum(serialize = "intransitive verb")]
    #[strum(serialize = "past-p")]
    #[strum(serialize = "perfect expression")]
    #[strum(serialize = "perfection expression")]
    #[strum(serialize = "past participle")]
    #[strum(serialize = "pres-p")]
    #[strum(serialize = "infinitive")]
    #[strum(serialize = "v")]
    #[strum(serialize = "verb form")]
    #[strum(serialize = "participle")]
    #[strum(serialize = "instransitive verb")]
    #[strum(serialize = "perfect participle")]
    #[strum(serialize = "transitive verb")]
    #[strum(serialize = "VERB")]
    Verb = 1,

    #[strum(to_string = "adv", serialize = "Adv")]
    #[strum(serialize = "adverb")]
    #[strum(serialize = "adv.")]
    #[strum(serialize = "adverbs")]
    #[strum(serialize = "ADV")]
    Adv = 2,

    /// Associated Tags:
    /// ```plaintext
    ///    - interrogative
    ///    - prepositional
    ///    - person
    ///    - relative
    /// ```
    #[strum(to_string = "pron", serialize = "Pron")]
    #[strum(serialize = "relativ.pron")]
    #[strum(serialize = "pron interrog")]
    #[strum(serialize = "interrogative pronoun")]
    #[strum(serialize = "ppron")]
    #[strum(serialize = "prepositional pronoun")]
    #[strum(serialize = "personal pronoun")]
    #[strum(serialize = "pronoun")]
    #[strum(serialize = "PRON")]
    Pron = 3,

    /// Associated Tags:
    /// ```plaintext
    ///    - relative
    /// ```
    #[strum(to_string = "conj", serialize = "Conj")]
    #[strum(serialize = "relative")]
    #[strum(serialize = "conjuntion")]
    #[strum(serialize = "conjunction")]
    #[strum(serialize = "CONJ")]
    Conj = 4,

    #[strum(to_string = "prep", serialize = "Prep")]
    #[strum(serialize = "proposition")]
    #[strum(serialize = "prepositions")]
    #[strum(serialize = "preposition")]
    #[strum(serialize = "prp")]
    #[strum(serialize = "prepositional expressions")]
    Prep = 5,

    /// Associated Tags:
    /// ```plaintext
    ///    - possessive
    /// ```
    #[strum(to_string = "det", serialize = "Det")]
    #[strum(serialize = "possessive determiner")]
    #[strum(serialize = "possessive pronoun")]
    #[strum(serialize = "determiner")]
    #[strum(serialize = "DET")]
    Det = 6,

    #[strum(to_string = "intj", serialize = "Intj")]
    #[strum(serialize = "interjection")]
    #[strum(serialize = "interj")]
    #[strum(serialize = "int")]
    Intj = 7,

    /// Associated Tags:
    /// ```plaintext
    ///    - number
    /// ```
    #[strum(to_string = "num", serialize = "Num")]
    #[strum(serialize = "number")]
    #[strum(serialize = "numeral")]
    #[strum(serialize = "NUM")]
    Num = 8,

    /// Associated Tags:
    /// ```plaintext
    ///    - indefinite
    /// ```
    #[strum(to_string = "article", serialize = "Article")]
    #[strum(serialize = "indefinite article")]
    #[strum(serialize = "indart")]
    #[strum(serialize = "art")]
    Article = 9,

    #[strum(to_string = "name", serialize = "Name")]
    #[strum(serialize = "proper noun")]
    #[strum(serialize = "pnoun")]
    #[strum(serialize = "proper oun")]
    #[strum(serialize = "PROPN")]
    Name = 10,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "prefix", serialize = "Prefix")]
    Prefix = 11,

    /// Associated Tags:
    /// ```plaintext
    ///    - clitic
    ///    - morpheme
    /// ```
    #[strum(to_string = "suffix", serialize = "Suffix")]
    #[strum(serialize = "clitic")]
    #[strum(serialize = "enclitic particle")]
    #[strum(serialize = "adjective suffix")]
    #[strum(serialize = "enclitic")]
    #[strum(serialize = "suffix form")]
    Suffix = 12,

    /// Associated Tags:
    /// ```plaintext
    ///    - hanzi
    ///    - han
    ///    - Hanja
    ///    - letter
    ///    - diacritic
    ///    - kanji
    ///    - ligature
    /// ```
    #[strum(to_string = "character", serialize = "Character")]
    #[strum(serialize = "definitions")]
    #[strum(serialize = "hanzi")]
    #[strum(serialize = "hanja")]
    #[strum(serialize = "letter")]
    #[strum(serialize = "han character")]
    #[strum(serialize = "diacritical mark")]
    #[strum(serialize = "kanji")]
    #[strum(serialize = "han characters")]
    #[strum(serialize = "ligature")]
    Character = 13,

    #[strum(to_string = "particle", serialize = "Particle")]
    #[strum(serialize = "Partikel")]
    #[strum(serialize = "ptcl")]
    #[strum(serialize = "PRT")]
    Particle = 14,

    #[strum(to_string = "other", serialize = "Other")]
    #[strum(serialize = "misc")]
    Other = 15,

    /// Associated Tags:
    /// ```plaintext
    ///    - abbreviation
    /// ```
    #[strum(to_string = "abbrev", serialize = "Abbrev")]
    #[strum(serialize = "abbreviation")]
    #[strum(serialize = "acronym")]
    #[strum(serialize = "clipping")]
    #[strum(serialize = "initialism")]
    Abbrev = 16,

    /// Associated Tags:
    /// ```plaintext
    ///    - comparative
    ///    - predicative
    ///    - ordinal
    /// ```
    #[strum(to_string = "adj", serialize = "Adj")]
    #[strum(serialize = "ordinal number")]
    #[strum(serialize = "adjective")]
    #[strum(serialize = "adjectuve")]
    #[strum(serialize = "predicative")]
    #[strum(serialize = "comparative")]
    #[strum(serialize = "adjectives")]
    #[strum(serialize = "ADJ")]
    Adj = 17,

    #[strum(to_string = "adj_noun", serialize = "Adj_noun")]
    #[strum(serialize = "adjectival noun")]
    #[strum(serialize = "adjectival")]
    AdjNoun = 18,

    #[strum(to_string = "adj_verb", serialize = "Adj_verb")]
    #[strum(serialize = "adjectival verb")]
    AdjVerb = 19,

    #[strum(to_string = "adnominal", serialize = "Adnominal")]
    Adnominal = 20,

    #[strum(to_string = "adv_phrase", serialize = "Adv_phrase")]
    #[strum(serialize = "adverbial phrase")]
    AdvPhrase = 21,

    #[strum(to_string = "affix", serialize = "Affix")]
    Affix = 22,

    /// ADP => An adposition: either a preposition or a postposition
    #[strum(to_string = "ambiposition", serialize = "Ambiposition")]
    #[strum(serialize = "ADP")]
    Ambiposition = 23,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "circumfix", serialize = "Circumfix")]
    Circumfix = 24,

    #[strum(to_string = "circumpos", serialize = "Circumpos")]
    #[strum(serialize = "circumposition")]
    Circumpos = 25,

    #[strum(to_string = "classifier", serialize = "Classifier")]
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
    Contraction = 29,

    #[strum(to_string = "converb", serialize = "Converb")]
    Converb = 30,

    #[strum(to_string = "counter", serialize = "Counter")]
    Counter = 31,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "infix", serialize = "Infix")]
    Infix = 32,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "interfix", serialize = "Interfix")]
    Interfix = 33,

    /// Associated Tags:
    /// ```plaintext
    ///    - idiomatic
    /// ```
    #[strum(to_string = "phrase", serialize = "Phrase")]
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
    Preverb = 37,

    #[strum(to_string = "proverb", serialize = "Proverb")]
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
    Romanization = 40,

    /// Associated Tags:
    /// ```plaintext
    ///    - morpheme
    /// ```
    #[strum(to_string = "root", serialize = "Root")]
    Root = 41,

    #[strum(to_string = "syllable", serialize = "Syllable")]
    Syllable = 42,

    #[strum(to_string = "symbol", serialize = "Symbol")]
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
        PartOfSpeech::try_from(x as u16).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        (self as u16) as u64
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