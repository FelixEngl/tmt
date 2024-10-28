use std::fmt::{Display, Formatter};
use strum::{Display, EnumString};
use crate::topicmodel::dictionary::loader::helper::gen_freedict_tei_reader::{EGenElement, ENumberElement, EPosElement, LangAttribute as FreeDictLangAttribute};
use crate::topicmodel::dictionary::loader::helper::gen_iate_tbx_reader::{LangAttribute as IateLangAttribute};
use crate::topicmodel::dictionary::loader::helper::gen_ms_terms_reader::{LangAttribute as MsTermsAttribute, ETermNoteElement};

#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Language {
    #[strum(to_string = "en", serialize = "english")]
    English,
    #[strum(to_string = "de", serialize = "german")]
    German,
    #[strum(to_string = "italian", serialize = "Ital.")]
    Italian,
    #[strum(to_string = "french", serialize = "French")]
    French,
    #[strum(to_string = "latin", serialize = "Lat.")]
    Latin
}

impl From<FreeDictLangAttribute> for Language {
    fn from(value: FreeDictLangAttribute) -> Self {
        match value {
            FreeDictLangAttribute::En => {
                Language::English
            }
            FreeDictLangAttribute::De => {
                Language::German
            }
        }
    }
}

impl From<IateLangAttribute> for Language {
    fn from(value: IateLangAttribute) -> Self {
        match value {
            IateLangAttribute::En => {
                Language::English
            }
            IateLangAttribute::De => {
                Language::German
            }
        }
    }
}

impl From<MsTermsAttribute> for Language {
    fn from(value: MsTermsAttribute) -> Self {
        match value {
            MsTermsAttribute::EnGb | MsTermsAttribute::EnUs => {
                Language::English
            }
            MsTermsAttribute::DeDe => {
                Language::German
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum WordInfo<T> {
    Type(PartOfSpeech),
    Gender(GrammaticalGender),
    Number(GrammaticalNumber),
    Other(T)
}

impl<T> From<T> for WordInfo<T> where T: AsRef<str> {
    fn from(value: T) -> Self {
        let s = value.as_ref();
        if let Ok(value) = s.parse() {
            WordInfo::Type(value)
        } else if let Ok(value) = s.parse() {
            WordInfo::Gender(value)
        } else if let Ok(value) = s.parse() {
            WordInfo::Number(value)
        } else {
            WordInfo::Other(value)
        }
    }
}

impl<T> WordInfo<T> {
    pub fn map<R, F: FnOnce(T) -> R>(self, mapper: F) -> WordInfo<R> {
        match self {
            WordInfo::Other(value) => WordInfo::Other(mapper(value)),
            WordInfo::Type(value) => WordInfo::Type(value),
            WordInfo::Gender(value) => WordInfo::Gender(value),
            WordInfo::Number(value) => WordInfo::Number(value),
        }
    }
}

impl<T> Display for WordInfo<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WordInfo::Type(value) => {
                Display::fmt(value, f)
            }
            WordInfo::Gender(value) => {
                Display::fmt(value, f)
            }
            WordInfo::Number(value) => {
                Display::fmt(value, f)
            }
            WordInfo::Other(value) => {
                Display::fmt(value, f)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq, Hash)]
pub enum PartOfSpeech {
    #[strum(to_string = "noun")]
    Noun,
    #[strum(to_string = "adj")]
    Adjective,
    #[strum(to_string = "adv")]
    Adverb,
    #[strum(to_string = "verb")]
    Verb,
    #[strum(to_string = "conj")]
    Conjuction,
    #[strum(to_string = "pron")]
    Pronoun,
    #[strum(to_string = "prep")]
    Preposition,
    #[strum(to_string = "det")]
    Determiner,
    #[strum(to_string = "int")]
    Interjection,
    #[strum(to_string="pres-p")]
    PresentParticiple,
    #[strum(to_string="past-p")]
    PastParticiple,
    #[strum(to_string="prefix")]
    Prefix,
    #[strum(to_string="suffix")]
    Suffix,
    #[strum(to_string="num")]
    Numeral,
    #[strum(to_string="art")]
    Article,
    #[strum(to_string="ptcl")]
    Particle,
    #[strum(to_string="pnoun")]
    ProperNoun,
    #[strum(to_string="other", serialize = "misc")]
    Other
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
                Self::ProperNoun
            }
            ETermNoteElement::Adjective => {
                Self::Adjective
            }
            ETermNoteElement::Adverb => {
                Self::Adverb
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
                Self::Adjective
            }
            EPosElement::V => {
                Self::Verb
            }
            EPosElement::Adv => {
                Self::Adverb
            }
            EPosElement::Int => {
                Self::Interjection
            }
            EPosElement::Prep => {
                Self::Preposition
            }
            EPosElement::Num => {
                Self::Numeral
            }
            EPosElement::Pron => {
                Self::Pronoun
            }
            EPosElement::Conj => {
                Self::Conjuction
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

#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq, Hash)]
pub enum GrammaticalGender {
    #[strum(to_string = "f", serialize = "female", serialize = "f.")]
    Feminine,
    #[strum(to_string = "m", serialize = "male", serialize = "m.")]
    Masculine,
    #[strum(to_string = "n", serialize = "neutral", serialize = "n.")]
    Neutral
}

impl From<EGenElement> for GrammaticalGender {
    fn from(value: EGenElement) -> Self {
        match value {
            EGenElement::Neut => {
                GrammaticalGender::Neutral
            }
            EGenElement::Masc => {
                GrammaticalGender::Masculine
            }
            EGenElement::Fem => {
                GrammaticalGender::Feminine
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq, Hash)]
pub enum GrammaticalNumber {
    #[strum(to_string = "sg", serialize = "sg.")]
    Singular,
    #[strum(to_string = "pl", serialize = "pl.")]
    Plural
}

impl From<ENumberElement> for GrammaticalNumber {
    fn from(value: ENumberElement) -> Self {
        match value {
            ENumberElement::Sg => {
                Self::Singular
            }
            ENumberElement::Pl => {
                Self::Plural
            }
        }
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PartialWordType {
    Prefix,
    Suffix,
}


#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq, Hash)]
pub enum Domain {
    #[strum(to_string = "bot.", serialize = "bot")]
    Bot,
    #[strum(to_string = "hist.", serialize = "hist")]
    Hist,
    #[strum(to_string = "phil.", serialize = "phil")]
    Phil,
    #[strum(to_string = "chem.", serialize = "chem")]
    Chem,
    #[strum(to_string = "arch.", serialize = "arch")]
    Arch,
    #[strum(to_string = "transp.", serialize = "transp")]
    Transp,
    #[strum(to_string = "min.", serialize = "min")]
    Min,
    #[strum(to_string = "stud.", serialize = "stud")]
    Stud,
    #[strum(to_string = "cook.", serialize = "cook")]
    Cook,
    #[strum(to_string = "auto", serialize = "auto.")]
    Auto,
    #[strum(to_string = "meteo.", serialize = "meteo")]
    Meteo,
    #[strum(to_string = "art", serialize = "art.")]
    Art,
    #[strum(to_string = "lit.", serialize = "lit")]
    Lit,
    #[strum(to_string = "geogr.", serialize = "geogr")]
    Geogr,
    #[strum(to_string = "ling.", serialize = "ling")]
    Ling,
    #[strum(to_string = "telco.", serialize = "telco")]
    Telco,
    #[strum(to_string = "pharm.", serialize = "pharm")]
    Pharm,
    #[strum(to_string = "pol.", serialize = "pol")]
    Pol,
    #[strum(to_string = "psych.", serialize = "psych")]
    Psych,
    #[strum(to_string = "agr.", serialize = "agr")]
    Agr,
    #[strum(to_string = "math.", serialize = "math")]
    Math,
    #[strum(to_string = "statist.", serialize = "statist")]
    Statist,
    #[strum(to_string = "mus.", serialize = "mus")]
    Mus,
    #[strum(to_string = "sport", serialize = "sport.")]
    Sport,
    #[strum(to_string = "anat.", serialize = "anat")]
    Anat,
    #[strum(to_string = "astrol.", serialize = "astrol")]
    Astrol,
    #[strum(to_string = "naut.", serialize = "naut")]
    Naut,
    #[strum(to_string = "photo.", serialize = "photo")]
    Photo,
    #[strum(to_string = "envir.", serialize = "envir")]
    Envir,
    #[strum(to_string = "soc.", serialize = "soc")]
    Soc,
    #[strum(to_string = "electr.", serialize = "electr")]
    Electr,
    #[strum(to_string = "biol.", serialize = "biol")]
    Biol,
    #[strum(to_string = "constr.", serialize = "constr")]
    Constr,
    #[strum(to_string = "school", serialize = "school.")]
    School,
    #[strum(to_string = "aviat.", serialize = "aviat")]
    Aviat,
    #[strum(to_string = "fin.", serialize = "fin")]
    Fin,
    #[strum(to_string = "mach.", serialize = "mach")]
    Mach,
    #[strum(to_string = "archeol.", serialize = "archeol")]
    Archeol,
    #[strum(to_string = "TV", serialize = "TV.")]
    Tv,
    #[strum(to_string = "comp.", serialize = "comp")]
    Comp,
    #[strum(to_string = "relig.", serialize = "relig")]
    Relig,
    #[strum(to_string = "astron.", serialize = "astron")]
    Astron,
    #[strum(to_string = "phys.", serialize = "phys")]
    Phys,
    #[strum(to_string = "zool.", serialize = "zool")]
    Zool,
    #[strum(to_string = "print", serialize = "print.")]
    Print,
    #[strum(to_string = "econ.", serialize = "econ")]
    Econ,
    #[strum(to_string = "textil.", serialize = "textil")]
    Textil,
    #[strum(to_string = "biochem.", serialize = "biochem")]
    Biochem,
    #[strum(to_string = "geol.", serialize = "geol")]
    Geol,
    #[strum(to_string = "ornith.", serialize = "ornith")]
    Ornith,
    #[strum(to_string = "med.", serialize = "med")]
    Med,
    #[strum(to_string = "mil.", serialize = "mil")]
    Mil,
    #[strum(to_string = "insur.", serialize = "insur")]
    Insur,
}

/// In sociolinguistics, a register is a variety of language used for a particular purpose or particular communicative situation
#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq, Hash)]
pub enum Register{
    #[strum(to_string = "humor.", serialize = "humor")]
    Humor,
    #[strum(to_string = "vulg.", serialize = "vulg")]
    Vulg,
    #[strum(to_string = "techn.", serialize = "techn")]
    Techn,
    #[strum(to_string = "coll.", serialize = "coll")]
    Coll,
    #[strum(to_string = "geh.", serialize = "geh")]
    Geh,
    #[strum(to_string = "slang", serialize = "slang.")]
    Slang,
    #[strum(to_string = "iron.", serialize = "iron")]
    Iron,
    #[strum(to_string = "ugs.", serialize = "ugs")]
    Ugs,
    #[strum(to_string = "formal", serialize = "formal.")]
    Formal,
    #[strum(to_string = "euphem.", serialize = "euphem")]
    Euphem,
    #[strum(to_string = "literary", serialize = "literary.")]
    Literary,
    #[strum(to_string = "dialect", serialize = "dialect.")]
    Dialect,
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use crate::topicmodel::dictionary::loader::word_infos::GrammaticalGender::Feminine;
    use crate::topicmodel::dictionary::loader::word_infos::WordInfo;

    #[test]
    fn can_map(){
        let other = vec![WordInfo::Other("value"), WordInfo::Gender(Feminine)];
        println!("{other:?}");
        let other = other.into_iter().map(|value| value.map(|x| x.to_string())).collect_vec();
        println!("{other:?}");
    }
}