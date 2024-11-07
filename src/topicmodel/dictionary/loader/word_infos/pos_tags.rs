use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use tinyset::Fits64;
use crate::register_python;
use crate::topicmodel::dictionary::metadata::loaded::impl_try_from_as_unpack;

register_python!(
    enum PartOfSpeechTag;
);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumIter)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum PartOfSpeechTag {
    /// Associated Pos:
    /// ```plaintext
    ///    - combining_form
    ///    - infix
    ///    - circumfix
    ///    - root
    ///    - prefix
    ///    - suffix
    ///    - interfix
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - combining form
    ///    - suffix form
    ///    - infix
    ///    - circumfix
    ///    - root
    ///    - prefix
    ///    - suffix
    ///    - interfix
    /// ```
    #[strum(to_string = "morpheme", serialize = "Morpheme")]
    Morpheme = 0,

    /// Associated Pos:
    /// ```plaintext
    ///    - verb
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - present participle
    ///    - perfect participle
    ///    - past-p
    ///    - past participle
    ///    - participle
    ///    - pres-p
    ///    - gerund
    /// ```
    #[strum(to_string = "participle", serialize = "Participle")]
    Participle = 1,

    /// Associated Pos:
    /// ```plaintext
    ///    - verb
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - present participle
    ///    - pres-p
    /// ```
    #[strum(to_string = "present", serialize = "Present")]
    Present = 2,

    /// Associated Pos:
    /// ```plaintext
    ///    - verb
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - transitive verb
    /// ```
    #[strum(to_string = "transitive", serialize = "Transitive")]
    Transitive = 3,

    /// Associated Pos:
    /// ```plaintext
    ///    - noun
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - ideophone
    /// ```
    #[strum(to_string = "ideophone", serialize = "Ideophone")]
    Ideophone = 4,

    /// Associated Pos:
    /// ```plaintext
    ///    - verb
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - perfect participle
    /// ```
    #[strum(to_string = "perfect", serialize = "Perfect")]
    Perfect = 5,

    /// Associated Pos:
    /// ```plaintext
    ///    - noun
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - verbal noun
    /// ```
    #[strum(to_string = "verbal", serialize = "Verbal")]
    Verbal = 6,

    /// Associated Pos:
    /// ```plaintext
    ///    - verb
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - gerund
    /// ```
    #[strum(to_string = "gerund", serialize = "Gerund")]
    Gerund = 7,

    /// Associated Pos:
    /// ```plaintext
    ///    - verb
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - infinitive
    /// ```
    #[strum(to_string = "infinitive", serialize = "Infinitive")]
    Infinitive = 8,

    /// Associated Pos:
    /// ```plaintext
    ///    - pron
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - pron interrog
    ///    - interrogative pronoun
    /// ```
    #[strum(to_string = "interrogative", serialize = "Interrogative")]
    Interrogative = 9,

    /// Associated Pos:
    /// ```plaintext
    ///    - verb
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - instransitive verb
    ///    - intransitive verb
    /// ```
    #[strum(to_string = "intransitive", serialize = "Intransitive")]
    Intransitive = 10,

    /// Associated Pos:
    /// ```plaintext
    ///    - verb
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - past-p
    ///    - past participle
    /// ```
    #[strum(to_string = "past", serialize = "Past")]
    Past = 11,

    /// Associated Pos:
    /// ```plaintext
    ///    - pron
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - personal pronoun
    /// ```
    #[strum(to_string = "person", serialize = "Person")]
    Person = 12,

    /// Associated Pos:
    /// ```plaintext
    ///    - pron
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - prepositional pronoun
    /// ```
    #[strum(to_string = "prepositional", serialize = "Prepositional")]
    Prepositional = 13,

    /// Associated Pos:
    /// ```plaintext
    ///    - conj
    ///    - pron
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - relativ.pron
    ///    - relative
    /// ```
    #[strum(to_string = "relative", serialize = "Relative")]
    Relative = 14,

    /// Associated Pos:
    /// ```plaintext
    ///    - character
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - hanja
    /// ```
    #[strum(to_string = "hanja", serialize = "Hanja")]
    Hanja = 15,

    /// Associated Pos:
    /// ```plaintext
    ///    - abbrev
    ///    - contraction
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - initialism
    ///    - contraction
    ///    - abbreviation
    ///    - clipping
    ///    - acronym
    /// ```
    #[strum(to_string = "abbreviation", serialize = "Abbreviation")]
    Abbreviation = 16,

    /// Associated Pos:
    /// ```plaintext
    ///    - suffix
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - enclitic
    ///    - clitic
    ///    - enclitic particle
    /// ```
    #[strum(to_string = "clitic", serialize = "Clitic")]
    Clitic = 17,

    /// Associated Pos:
    /// ```plaintext
    ///    - adj
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - comparative
    /// ```
    #[strum(to_string = "comparative", serialize = "Comparative")]
    Comparative = 18,

    /// Associated Pos:
    /// ```plaintext
    ///    - noun
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - dependent noun
    /// ```
    #[strum(to_string = "dependent", serialize = "Dependent")]
    Dependent = 19,

    /// Associated Pos:
    /// ```plaintext
    ///    - character
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - diacritical mark
    /// ```
    #[strum(to_string = "diacritic", serialize = "Diacritic")]
    Diacritic = 20,

    /// Associated Pos:
    /// ```plaintext
    ///    - character
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - han characters
    ///    - han character
    /// ```
    #[strum(to_string = "han", serialize = "Han")]
    Han = 21,

    /// Associated Pos:
    /// ```plaintext
    ///    - character
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - hanzi
    /// ```
    #[strum(to_string = "hanzi", serialize = "Hanzi")]
    Hanzi = 22,

    /// Associated Pos:
    /// ```plaintext
    ///    - phrase
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - idiom
    /// ```
    #[strum(to_string = "idiomatic", serialize = "Idiomatic")]
    Idiomatic = 23,

    /// Associated Pos:
    /// ```plaintext
    ///    - article
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - indart
    ///    - indefinite article
    /// ```
    #[strum(to_string = "indefinite", serialize = "Indefinite")]
    Indefinite = 24,

    /// Associated Pos:
    /// ```plaintext
    ///    - character
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - kanji
    /// ```
    #[strum(to_string = "kanji", serialize = "Kanji")]
    Kanji = 25,

    /// Associated Pos:
    /// ```plaintext
    ///    - character
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - letter
    /// ```
    #[strum(to_string = "letter", serialize = "Letter")]
    Letter = 26,

    /// Associated Pos:
    /// ```plaintext
    ///    - character
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - ligature
    /// ```
    #[strum(to_string = "ligature", serialize = "Ligature")]
    Ligature = 27,

    /// Associated Pos:
    /// ```plaintext
    ///    - num
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - number
    /// ```
    #[strum(to_string = "number", serialize = "Number")]
    Number = 28,

    /// Associated Pos:
    /// ```plaintext
    ///    - adj
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - ordinal number
    /// ```
    #[strum(to_string = "ordinal", serialize = "Ordinal")]
    Ordinal = 29,

    /// Associated Pos:
    /// ```plaintext
    ///    - det
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - possessive determiner
    ///    - possessive pronoun
    /// ```
    #[strum(to_string = "possessive", serialize = "Possessive")]
    Possessive = 30,

    /// Associated Pos:
    /// ```plaintext
    ///    - adj
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - predicative
    /// ```
    #[strum(to_string = "predicative", serialize = "Predicative")]
    Predicative = 31,

    /// Associated Pos:
    /// ```plaintext
    ///    - punct
    /// ```
    /// Associated Targets:
    /// ```plaintext
    ///    - punctuation
    ///    - punctuation mark
    /// ```
    #[strum(to_string = "punctuation", serialize = "Punctuation")]
    Punctuation = 32,

}
impl PartOfSpeechTag {
    pub fn get_tags(target: &str) -> Option<&'static [PartOfSpeechTag]> {
        match target {
            "abbreviation" => Some(&Self::ABBREVIATION),
            "acronym" => Some(&Self::ABBREVIATION),
            "circumfix" => Some(&Self::MORPHEME),
            "clipping" => Some(&Self::ABBREVIATION),
            "clitic" => Some(&Self::CLITIC),
            "combining form" => Some(&Self::MORPHEME),
            "comparative" => Some(&Self::COMPARATIVE),
            "contraction" => Some(&Self::ABBREVIATION),
            "dependent noun" => Some(&Self::DEPENDENT),
            "diacritical mark" => Some(&Self::DIACRITIC),
            "enclitic" => Some(&Self::CLITIC),
            "enclitic particle" => Some(&Self::CLITIC),
            "gerund" => Some(&Self::GERUND_PARTICIPLE),
            "han character" => Some(&Self::HAN),
            "han characters" => Some(&Self::HAN),
            "hanja" => Some(&Self::HANJA),
            "hanzi" => Some(&Self::HANZI),
            "ideophone" => Some(&Self::IDEOPHONE),
            "idiom" => Some(&Self::IDIOMATIC),
            "infix" => Some(&Self::MORPHEME),
            "infinitive" => Some(&Self::INFINITIVE),
            "initialism" => Some(&Self::ABBREVIATION),
            "interfix" => Some(&Self::MORPHEME),
            "interrogative pronoun" => Some(&Self::INTERROGATIVE),
            "intransitive verb" => Some(&Self::INTRANSITIVE),
            "instransitive verb" => Some(&Self::INTRANSITIVE),
            "kanji" => Some(&Self::KANJI),
            "letter" => Some(&Self::LETTER),
            "ligature" => Some(&Self::LIGATURE),
            "number" => Some(&Self::NUMBER),
            "ordinal number" => Some(&Self::ORDINAL),
            "participle" => Some(&Self::PARTICIPLE),
            "past participle" => Some(&Self::PARTICIPLE_PAST),
            "perfect participle" => Some(&Self::PARTICIPLE_PERFECT),
            "personal pronoun" => Some(&Self::PERSON),
            "possessive determiner" => Some(&Self::POSSESSIVE),
            "possessive pronoun" => Some(&Self::POSSESSIVE),
            "predicative" => Some(&Self::PREDICATIVE),
            "prefix" => Some(&Self::MORPHEME),
            "prepositional pronoun" => Some(&Self::PREPOSITIONAL),
            "present participle" => Some(&Self::PARTICIPLE_PRESENT),
            "punctuation mark" => Some(&Self::PUNCTUATION),
            "punctuation" => Some(&Self::PUNCTUATION),
            "relative" => Some(&Self::RELATIVE),
            "root" => Some(&Self::MORPHEME),
            "suffix" => Some(&Self::MORPHEME),
            "suffix form" => Some(&Self::MORPHEME),
            "transitive verb" => Some(&Self::TRANSITIVE),
            "verbal noun" => Some(&Self::VERBAL),
            "pres-p" => Some(&Self::PARTICIPLE_PRESENT),
            "past-p" => Some(&Self::PARTICIPLE_PAST),
            "indart" => Some(&Self::INDEFINITE),
            "indefinite article" => Some(&Self::INDEFINITE),
            "pron interrog" => Some(&Self::INTERROGATIVE),
            "relativ.pron" => Some(&Self::RELATIVE),
            _ => None
        }
    }


    pub const ABBREVIATION: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Abbreviation];
    pub const MORPHEME: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Morpheme];
    pub const CLITIC: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Clitic];
    pub const COMPARATIVE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Comparative];
    pub const DEPENDENT: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Dependent];
    pub const DIACRITIC: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Diacritic];
    pub const GERUND_PARTICIPLE: [PartOfSpeechTag; 2] = [PartOfSpeechTag::Gerund, PartOfSpeechTag::Participle];
    pub const HAN: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Han];
    pub const HANJA: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Hanja];
    pub const HANZI: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Hanzi];
    pub const IDEOPHONE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Ideophone];
    pub const IDIOMATIC: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Idiomatic];
    pub const INFINITIVE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Infinitive];
    pub const INTERROGATIVE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Interrogative];
    pub const INTRANSITIVE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Intransitive];
    pub const KANJI: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Kanji];
    pub const LETTER: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Letter];
    pub const LIGATURE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Ligature];
    pub const NUMBER: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Number];
    pub const ORDINAL: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Ordinal];
    pub const PARTICIPLE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Participle];
    pub const PARTICIPLE_PAST: [PartOfSpeechTag; 2] = [PartOfSpeechTag::Participle, PartOfSpeechTag::Past];
    pub const PARTICIPLE_PERFECT: [PartOfSpeechTag; 2] = [PartOfSpeechTag::Participle, PartOfSpeechTag::Perfect];
    pub const PERSON: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Person];
    pub const POSSESSIVE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Possessive];
    pub const PREDICATIVE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Predicative];
    pub const PREPOSITIONAL: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Prepositional];
    pub const PARTICIPLE_PRESENT: [PartOfSpeechTag; 2] = [PartOfSpeechTag::Participle, PartOfSpeechTag::Present];
    pub const PUNCTUATION: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Punctuation];
    pub const RELATIVE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Relative];
    pub const TRANSITIVE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Transitive];
    pub const VERBAL: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Verbal];
    pub const INDEFINITE: [PartOfSpeechTag; 1] = [PartOfSpeechTag::Indefinite];
}

impl_try_from_as_unpack! {
        PartOfSpeechTag => PartOfSpeechTag
}


#[pymethods]
impl PartOfSpeechTag {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl Fits64 for PartOfSpeechTag {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        PartOfSpeechTag::try_from(x).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
}