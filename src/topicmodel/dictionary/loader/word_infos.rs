use std::fmt::{Display, Formatter};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumString, IntoStaticStr};
use tinyset::Fits64;
use crate::register_python;
use crate::topicmodel::dictionary::loader::helper::gen_freedict_tei_reader::{EGenElement, ENumberElement, EPosElement, LangAttribute as FreeDictLangAttribute};
use crate::topicmodel::dictionary::loader::helper::gen_iate_tbx_reader::{LangAttribute as IateLangAttribute};
use crate::topicmodel::dictionary::loader::iate_reader::{AdministrativeStatus};
use crate::topicmodel::dictionary::loader::helper::gen_ms_terms_reader::{LangAttribute as MsTermsAttribute, ETermNoteElement};
use crate::topicmodel::domain_matrix::TopicMatrixIndex;
use crate::topicmodel::dictionary::metadata::loaded::impl_try_from_as_unpack;

register_python! {
    enum Language;
    enum Region;
    enum PartOfSpeech;
    enum GrammaticalGender;
    enum GrammaticalNumber;
    enum Domain;
    enum Register;
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(name = "DictionaryLanguage", eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum Language {
    #[strum(to_string = "en", serialize = "english")]
    English = 0,
    #[strum(to_string = "de", serialize = "german", serialize = "Dt.")]
    German = 1,
    #[strum(to_string = "italian", serialize = "Ital.")]
    Italian = 2,
    #[strum(to_string = "french", serialize = "French", serialize = "from French")]
    French = 3,
    #[strum(to_string = "latin", serialize = "Lat.", serialize = "lat.")]
    Latin = 4
}

impl_try_from_as_unpack! {
    Language => Language
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Language {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl Fits64 for Language {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Language::try_from(x).unwrap()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum Region {
    #[strum(to_string = "BE", serialize = "eBr.", serialize = "Br.", serialize = "BR.", serialize = "Br,")]
    #[strum(serialize = "also Br.")]
    BritishEnglish = 0,
    #[strum(to_string = "AE", serialize = "eAm.", serialize = "Am.", serialize = "AM.", serialize = "Am .")]
    #[strum(serialize = "mainly Am.")]
    AmericanEnglish = 1,
    #[strum(to_string = "Aus.", serialize = "Austr.")]
    AustralianEnglish = 2,
    #[strum(to_string = "NZ")]
    NewZealandEnglish = 3,
    #[strum(to_string = "Can.")]
    CanadianEnglish = 4,
    #[strum(to_string = "Irish", serialize = "Ir.", serialize = "Irl.")]
    IrishEnglish = 5,
    #[strum(to_string = "Ind.")]
    IndianEnglish = 6,
    #[strum(to_string = "S.Afr.", serialize = "South Africa")]
    SouthAfricanEnglish = 7,
    #[strum(to_string = "Scot.", serialize = "Sc.")]
    ScottishEnglish = 8,
    #[strum(to_string = "österr.", serialize = "Ös.")]
    AustrianGerman = 9,
    #[strum(to_string = "südd.", serialize = "Süddt.")]
    SouthGerman = 10,
    #[strum(to_string = "nordd.", serialize = "Norddt.")]
    NorthGerman = 11,
    #[strum(to_string = "ostd.", serialize = "Ostdt.")]
    EastGerman = 12,
    #[strum(to_string = "schweiz.", serialize = "Schw.")]
    SwissGerman = 13,
    #[strum(to_string = "regional")]
    Regional = 14,
    #[strum(to_string = "Mittelwestdt.")]
    MiddleWestGerman = 15,
    #[strum(to_string = "Südwestdt.")]
    SouthWestGerman = 16,
    #[strum(to_string = "Nordwestdt.")]
    NorthWestGerman = 17,
    #[strum(to_string = "BW", serialize = "Württemberg", serialize = "BW.")]
    BadenWuerttembergGerman = 18,
    #[strum(to_string = "Mittelostdt.")]
    MiddleEastGerman = 19,
    #[strum(to_string = "Südostdt.")]
    SouthEastGerman = 20,
    #[strum(to_string = "Nordostdt.")]
    NorthEastGerman = 21,
    #[strum(to_string = "Mitteldt.")]
    MiddleGerman = 22,
    #[strum(to_string = "Bayr.")]
    BavarianGerman = 23,
    #[strum(to_string = "Northern Irish")]
    NorthernIrish = 24,
    #[strum(to_string = "Oberdt.")]
    UpperGerman = 25,
    #[strum(to_string = "Ostös.")]
    EastAustrianGerman = 26,
    #[strum(to_string = "Berlin")]
    BerlinGerman = 27,
    #[strum(to_string = "Schwäb.", serialize = "Sachsen")]
    SwabianGerman = 28,
    #[strum(to_string = "Westös.")]
    WestAustrianGerman = 29,
    #[strum(to_string = "Wien")]
    ViennaGerman = 30,
    #[strum(to_string = "Tirol")]
    TyrolGerman = 31,
    #[strum(to_string = "Northern English")]
    NorthEnglish = 32,
    #[strum(to_string = "DDR")]
    DDRGerman = 33,
    #[strum(to_string = "Pfalz")]
    PfalzGerman = 34,
    #[strum(to_string = "Südtirol")]
    SouthTyrolGerman = 35,
    #[strum(to_string = "Ostmitteldt.")]
    EastMiddleGerman = 36,
    #[strum(to_string = "SE Asia")]
    SouthEastAsianEnglish = 37,
    #[strum(to_string = "Hessen")]
    HesseGerman = 38,
    #[strum(to_string = "Lux.")]
    LuxenbourgGerman = 39,
    #[strum(to_string = "Welch")]
    WelchEnglish = 40,
    #[strum(to_string = "Rheinl.")]
    RhinelandPalatinateGerman = 41,
    #[strum(to_string = "Sächs.")]
    SaxonyGerman = 42,
    #[strum(to_string = "Westdt.")]
    WestGerman = 43,
    #[strum(to_string = "Lie.")]
    LiechtensteinGerman = 44,
    #[strum(to_string = "Westfalen")]
    WestphaliaGerman = 45,
    #[strum(to_string = "Südostös.")]
    SouthEastAustrianGerman = 46,
    #[strum(to_string = "Nordostös.")]
    NorthEastAustrianGerman = 47,
    #[strum(to_string = "Nordwestös.")]
    NorthWestAustrianGerman = 48,
    #[strum(to_string = "Südwestös.")]
    SouthWestAustrianGerman = 49,
    #[strum(to_string = "Westschw.")]
    WestSwissGerman = 50,
    #[strum(to_string = "Nordirl.")]
    NorthIrishEnglish = 51,
    #[strum(to_string = "Mittelös.")]
    MiddleAustrianGerman = 52,
    #[strum(to_string = "Franken")]
    FranconianGerman = 53,
    #[strum(to_string = "Ostschw.")]
    EastSwissGerman = 54
}

impl_try_from_as_unpack! {
    Region => Region
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Region {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}


impl TryFrom<MsTermsAttribute> for Region {
    type Error = ();

    fn try_from(value: MsTermsAttribute) -> Result<Self, Self::Error> {
        match value {
            MsTermsAttribute::EnGb => {
                Ok(Region::BritishEnglish)
            }
            MsTermsAttribute::EnUs => {
                Ok(Region::AmericanEnglish)
            }
            _ => Err(())
        }
    }
}


impl Fits64 for Region {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Region::try_from(x).unwrap()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum PartOfSpeech {
    #[strum(to_string = "noun")]
    Noun = 0,
    #[strum(to_string = "adj")]
    Adjective = 1,
    #[strum(to_string = "adv", serialize = "adv.")]
    Adverb = 2,
    #[strum(to_string = "verb", serialize = "v")]
    Verb = 3,
    #[strum(to_string = "conj")]
    Conjuction = 4,
    #[strum(to_string = "pron", serialize = "ppron")]
    Pronoun = 5,
    #[strum(to_string = "prep", serialize = "prp")]
    Preposition = 6,
    #[strum(to_string = "det")]
    Determiner = 7,
    #[strum(to_string = "int", serialize = "interj")]
    Interjection = 8,
    #[strum(to_string="pres-p")]
    PresentParticiple = 9,
    #[strum(to_string="past-p")]
    PastParticiple = 10,
    #[strum(to_string="prefix")]
    Prefix = 11,
    #[strum(to_string="suffix")]
    Suffix = 12,
    #[strum(to_string="num")]
    Numeral = 13,
    #[strum(to_string="art")]
    Article = 14,
    #[strum(to_string="ptcl", serialize = "particle", serialize = "Partikel")]
    Particle = 15,
    #[strum(to_string="pnoun")]
    ProperNoun = 16,
    #[strum(to_string="other", serialize = "misc")]
    Other = 17,
    #[strum(to_string="indart", serialize = "indefinite article")]
    IndefiniteArticle = 18,
    #[strum(to_string="pron interrog")]
    InterrogativePronoun = 19,
    #[strum(to_string="relativ.pron")]
    RelativePronoun = 20
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum GrammaticalGender {
    #[strum(to_string = "f", serialize = "female", serialize = "f.")]
    Feminine = 0,
    #[strum(to_string = "m", serialize = "male", serialize = "m.")]
    Masculine = 1,
    #[strum(to_string = "n", serialize = "neutral", serialize = "n.")]
    Neutral = 2,
    #[strum(to_string = "not f")]
    NotFeminine = 3
}

impl_try_from_as_unpack! {
    GrammaticalGender => GrammaticalGender
}


// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl GrammaticalGender {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl Fits64 for GrammaticalGender {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        GrammaticalGender::try_from(x).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
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

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum GrammaticalNumber {
    #[strum(to_string = "sg", serialize = "sg.")]
    Singular = 0,
    #[strum(to_string = "pl", serialize = "pl.", serialize = "plural", serialize = "Pluralwort")]
    #[strum(serialize = "Pluralw")]
    Plural = 1,
    #[strum(to_string = "kein Singular", serialize = "no singular")]
    NoSingular = 2,
    #[strum(to_string = "kein Plural", serialize = "no plural", serialize = "keine Mehrzahl")]
    NoPlural = 3,
    #[strum(to_string = "usually pl")]
    UsuallyPlural = 4,
    #[strum(to_string = "sg only")]
    SingularOnly = 5,
    #[strum(to_string = "only plural")]
    PluralOnly = 6,
}

impl_try_from_as_unpack! {
    GrammaticalNumber => GrammaticalNumber
}


// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl GrammaticalNumber {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl Fits64 for GrammaticalNumber {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        GrammaticalNumber::try_from(x).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
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


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumCount)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum Domain {
    /// Academic Disciplines / Wissenschaft
    #[strum(to_string = "acad.", serialize = "ACAD.", serialize = "ACAD", serialize = "acad", serialize = "academic")]
    Acad = 0,
    /// Accounting / Buchführung
    #[strum(to_string = "acc.", serialize = "ACC", serialize = "ACC.", serialize = "acc")]
    Acc = 1,
    /// (Public) Administration / (Öffentliche) Verwaltung
    #[strum(to_string = "admin.", serialize = "ADMIN.", serialize = "ADMIN", serialize = "admin")]
    Admin = 2,
    /// Agriculture, Aquaculture / Agrarwirtschaft, Land- und Gewässerbewirtschaftung
    #[strum(to_string = "agr.", serialize = "AGR.", serialize = "agr", serialize = "AGR")]
    Agr = 3,
    /// Human Anatomy / Humananatomie
    #[strum(to_string = "anat.", serialize = "ANAT", serialize = "ANAT.", serialize = "anat")]
    Anat = 4,
    /// Archaeology / Archäologie
    #[strum(to_string = "archaeo.", serialize = "archeol", serialize = "ARCHEOL.", serialize = "ARCHAEO.", serialize = "ARCHAEO", serialize = "ARCHEOL", serialize = "archaeo", serialize = "archeol.")]
    Archaeo = 5,
    /// Architecture / Architektur
    #[strum(to_string = "archi.", serialize = "ARCHI", serialize = "arch", serialize = "ARCH.", serialize = "archi", serialize = "ARCHI.", serialize = "arch.", serialize = "ARCH")]
    Archi = 6,
    /// Historic Armour / Rüstungen, historische Schutzbekleidung
    #[strum(to_string = "armour", serialize = "ARMOUR", serialize = "armour.", serialize = "ARMOUR.")]
    Armour = 7,
    /// Art / Kunst
    #[strum(to_string = "art", serialize = "art.", serialize = "ART.", serialize = "ART")]
    Art = 8,
    /// Astrology / Astrologie
    #[strum(to_string = "astrol.", serialize = "ASTROL", serialize = "ASTROL.", serialize = "astrol")]
    Astrol = 9,
    /// Astronomy / Astronomie
    #[strum(to_string = "astron.", serialize = "ASTRON.", serialize = "astron", serialize = "ASTRON")]
    Astron = 10,
    /// Astronautics / Astronautik, Raumfahrt
    #[strum(to_string = "astronau", serialize = "astronau.", serialize = "ASTRONAU.", serialize = "ASTRONAU")]
    Astronau = 11,
    /// Audiology / Audiologie, Akustik
    #[strum(to_string = "audio", serialize = "AUDIO.", serialize = "AUDIO", serialize = "audio.")]
    Audio = 12,
    /// Automotive Engineering / Automobil- und Fahrzeugtechnik
    #[strum(to_string = "automot.", serialize = "AUTO.", serialize = "AUTO", serialize = "auto", serialize = "AUTOMOT", serialize = "AUTOMOT.", serialize = "automot", serialize = "auto.")]
    Automot = 13,
    /// Aviation / Luftfahrt, Flugwesen
    #[strum(to_string = "aviat.", serialize = "AVIAT.", serialize = "aviat", serialize = "AVIAT")]
    Aviat = 14,
    /// Biblical / Biblisch
    #[strum(to_string = "bibl.", serialize = "bibl", serialize = "BIBL.", serialize = "BIBL")]
    Bibl = 15,
    /// Bicycle / Fahrrad
    #[strum(to_string = "bike", serialize = "BIKE", serialize = "bike.", serialize = "BIKE.")]
    Bike = 16,
    /// Biochemistry / Biochemie
    #[strum(to_string = "biochem.", serialize = "BIOCHEM.", serialize = "biochem", serialize = "BIOCHEM")]
    Biochem = 17,
    /// Biology / Biologie
    #[strum(to_string = "biol.", serialize = "BIOL.", serialize = "BIOL", serialize = "biol")]
    Biol = 18,
    /// Biotechnology / Biotechnologie
    #[strum(to_string = "biotech.", serialize = "biotech", serialize = "BIOTECH.", serialize = "BIOTECH")]
    Biotech = 19,
    /// Botany, Plants / Botanik, Pflanzen
    #[strum(to_string = "bot.", serialize = "bot", serialize = "BOT.", serialize = "BOT")]
    Bot = 20,
    /// Brewing / Brauwesen
    #[strum(to_string = "brew", serialize = "BREW.", serialize = "brew.", serialize = "BREW")]
    Brew = 21,
    /// Chemistry / Chemie
    #[strum(to_string = "chem.", serialize = "chem", serialize = "CHEM", serialize = "CHEM.")]
    Chem = 22,
    /// Climbing, Mountaineering / Bergsteigerei
    #[strum(to_string = "climbing", serialize = "CLIMBING.", serialize = "CLIMBING", serialize = "climbing.")]
    Climbing = 23,
    /// Clothing, Fashion / Bekleidung, Mode
    #[strum(to_string = "cloth.", serialize = "cloth", serialize = "CLOTH", serialize = "CLOTH.")]
    Cloth = 24,
    /// Comics and Animated Cartoons / Comics und Zeichentrickfilme
    #[strum(to_string = "comics", serialize = "comics.", serialize = "COMICS", serialize = "COMICS.")]
    Comics = 25,
    /// Commerce / Handel
    #[strum(to_string = "comm.", serialize = "comm", serialize = "COMM", serialize = "COMM.")]
    Comm = 26,
    /// Computer Sciences / Informatik, IT
    #[strum(to_string = "comp.", serialize = "COMP.", serialize = "COMP", serialize = "comp")]
    Comp = 27,
    /// Construction / Bauwesen
    #[strum(to_string = "constr.", serialize = "CONSTR.", serialize = "constr", serialize = "CONSTR")]
    Constr = 28,
    /// Cooking
    #[strum(to_string = "cook.", serialize = "COOK.", serialize = "COOK", serialize = "cook")]
    Cook = 29,
    /// Cosmetics & Body Care / Kosmetik und Körperpflege
    #[strum(to_string = "cosmet.", serialize = "COSMET.", serialize = "COSMET", serialize = "cosmet")]
    Cosmet = 30,
    /// Currencies / Währungen
    #[strum(to_string = "curr.", serialize = "CURR.", serialize = "curr", serialize = "CURR")]
    Curr = 31,
    /// Dance / Tanz
    #[strum(to_string = "dance", serialize = "DANCE.", serialize = "dance.", serialize = "DANCE")]
    Dance = 32,
    /// Dental Medicine / Zahnmedizin
    #[strum(to_string = "dent.", serialize = "DENT.", serialize = "DENT", serialize = "dent")]
    Dent = 33,
    /// Drugs / Drogen
    #[strum(to_string = "drugs", serialize = "DRUGS.", serialize = "DRUGS", serialize = "drugs.")]
    Drugs = 34,
    /// Ecology, Environment / Ökologie, Umwelt
    #[strum(to_string = "ecol.", serialize = "envir", serialize = "ENVIR.", serialize = "ecol", serialize = "ECOL", serialize = "ENVIR", serialize = "envir.", serialize = "ECOL.")]
    Ecol = 35,
    /// Economy / Wirtschaft, Ökonomie
    #[strum(to_string = "econ.", serialize = "econ", serialize = "ECON.", serialize = "ECON")]
    Econ = 36,
    /// Education / Ausbildung
    #[strum(to_string = "educ.", serialize = "EDUC", serialize = "EDUC.", serialize = "educ")]
    Educ = 37,
    /// Electrical Engin., Electronics / Elektrotechnik, Elektronik
    #[strum(to_string = "electr.", serialize = "ELECTR.", serialize = "ELECTR", serialize = "electr")]
    #[strum(serialize = "elect.")]
    Electr = 38,
    /// Engineering / Ingenieurwissenschaften
    #[strum(to_string = "engin.", serialize = "ENGIN.", serialize = "engin", serialize = "ENGIN")]
    Engin = 39,
    /// Entomology / Entomologie, Insektenkunde
    #[strum(to_string = "entom.", serialize = "entom", serialize = "ENTOM.", serialize = "ENTOM")]
    Entom = 40,
    /// Equestrianism, Horses / Reitsport, Pferde
    #[strum(to_string = "equest.", serialize = "EQUEST.", serialize = "equest", serialize = "EQUEST")]
    Equest = 41,
    /// Esotericism / Esoterik
    #[strum(to_string = "esot.", serialize = "ESOT.", serialize = "esot", serialize = "ESOT")]
    Esot = 42,
    /// Ethnology / Ethnologie
    #[strum(to_string = "ethn.", serialize = "ETHN.", serialize = "ETHN", serialize = "ethn")]
    Ethn = 43,
    /// European Union / Europäische Union
    #[strum(to_string = "EU", serialize = "eu", serialize = "EU.", serialize = "eu.")]
    Eu = 44,
    /// Fiction: Names and Titles in Literature, Film, TV, Arts / Fiktion: Namen und Titel in Literatur, Film, TV, Kunst
    #[strum(to_string = "F", serialize = "F.", serialize = "f.", serialize = "f")]
    F = 45,
    /// Film / Film
    #[strum(to_string = "film", serialize = "FILM", serialize = "film.", serialize = "FILM.")]
    Film = 46,
    /// Finance / Finanzwesen
    #[strum(to_string = "fin.", serialize = "FIN.", serialize = "fin", serialize = "FIN")]
    Fin = 47,
    /// Firefighting & Rescue / Feuerwehr & Rettungsdienst
    #[strum(to_string = "FireResc", serialize = "FIRERESC", serialize = "FireResc.", serialize = "FIRERESC.", serialize = "fireresc", serialize = "fireresc.")]
    FireResc = 48,
    /// Ichthyology, fish, fishing / Fischkunde, Fischen, Angelsport
    #[strum(to_string = "fish", serialize = "fish.", serialize = "FISH.", serialize = "FISH")]
    Fish = 49,
    /// Foodstuffs Industry / Lebensmittelindustrie
    #[strum(to_string = "FoodInd.", serialize = "FOODIND", serialize = "FOODIND.", serialize = "foodind.", serialize = "FoodInd", serialize = "foodind")]
    FoodInd = 50,
    /// Forestry / Forstwissenschaft, Forstwirtschaft
    #[strum(to_string = "for.", serialize = "FOR.", serialize = "for", serialize = "FOR")]
    For = 51,
    /// Furniture / Möbel
    #[strum(to_string = "furn.", serialize = "FURN", serialize = "furn", serialize = "FURN.")]
    Furn = 52,
    /// Games / Spiele
    #[strum(to_string = "games", serialize = "GAMES.", serialize = "GAMES", serialize = "games.")]
    Games = 53,
    /// Gastronomy, Cooking / Gastronomie, Kochen
    #[strum(to_string = "gastr.", serialize = "gastr", serialize = "GASTR", serialize = "GASTR.")]
    Gastr = 54,
    /// Geography / Geografie
    #[strum(to_string = "geogr.", serialize = "geogr", serialize = "GEOGR", serialize = "GEOGR.")]
    Geogr = 55,
    /// Geology / Geologie
    #[strum(to_string = "geol.", serialize = "GEOL.", serialize = "geol", serialize = "GEOL")]
    Geol = 56,
    /// Heraldry / Heraldik
    #[strum(to_string = "herald.", serialize = "HERALD.", serialize = "herald", serialize = "HERALD")]
    Herald = 57,
    /// History / Historische Begriffe, Geschichte
    #[strum(to_string = "hist.", serialize = "HIST", serialize = "HIST.", serialize = "hist")]
    Hist = 58,
    /// Horticulture / Gartenbau
    #[strum(to_string = "hort.", serialize = "HORT.", serialize = "hort", serialize = "HORT")]
    Hort = 59,
    /// Hunting / Jagd
    #[strum(to_string = "hunting", serialize = "HUNTING", serialize = "hunting.", serialize = "HUNTING.")]
    #[strum(serialize = "Jägersprache", serialize = "hunter's parlance", serialize = "hunters' parlance")]
    Hunting = 60,
    /// Hydrology & Hydrogeology / Hydrologie & Hydrogeologie
    #[strum(to_string = "hydro.", serialize = "HYDRO", serialize = "hydro", serialize = "HYDRO.")]
    Hydro = 61,
    /// Idiom / Idiom, Redewendung
    #[strum(to_string = "idiom", serialize = "idiom.", serialize = "IDIOM", serialize = "IDIOM.", serialize = "Redewendung")]
    #[strum(serialize = "Sprw.")]
    Idiom = 62,
    /// Industry / Industrie
    #[strum(to_string = "ind.", serialize = "IND.", serialize = "ind", serialize = "IND")]
    Ind = 63,
    /// Insurance / Versicherungswesen
    #[strum(to_string = "insur.", serialize = "INSUR", serialize = "insur", serialize = "INSUR.")]
    Insur = 64,
    /// Internet / Internet
    #[strum(to_string = "Internet", serialize = "internet.", serialize = "Internet.", serialize = "INTERNET", serialize = "internet", serialize = "INTERNET.")]
    Internet = 65,
    /// Jobs, Employment Market / Berufe, Arbeitsmarkt
    #[strum(to_string = "jobs", serialize = "JOBS.", serialize = "JOBS", serialize = "jobs.")]
    Jobs = 66,
    /// Journalism / Journalismus
    #[strum(to_string = "journ.", serialize = "journ", serialize = "JOURN", serialize = "JOURN.")]
    Journ = 67,
    /// Law / Jura, Rechtswesen
    #[strum(to_string = "law", serialize = "law.", serialize = "LAW.", serialize = "LAW", serialize = "jur.")]
    Law = 68,
    /// Library Science / Bibliothekswissenschaft
    #[strum(to_string = "libr.", serialize = "LIBR.", serialize = "LIBR", serialize = "libr")]
    Libr = 69,
    /// Linguistics / Linguistik, Sprachwissenschaft
    #[strum(to_string = "ling.", serialize = "LING.", serialize = "ling", serialize = "LING")]
    Ling = 70,
    /// Literature / Literatur
    #[strum(to_string = "lit.", serialize = "LIT.", serialize = "LIT", serialize = "lit")]
    Lit = 71,
    /// Machines
    #[strum(to_string = "mach.", serialize = "MACH.", serialize = "mach", serialize = "MACH")]
    Mach = 72,
    /// Marketing, Advertising / Marketing, Werbung, Vertrieb und Handelswesen
    #[strum(to_string = "market.", serialize = "MARKET", serialize = "market", serialize = "MARKET.")]
    Market = 73,
    /// Materials Science / Materialwissenschaft, Werkstoffkunde
    #[strum(to_string = "material", serialize = "MATERIAL", serialize = "MATERIAL.", serialize = "material.")]
    Material = 74,
    /// Mathematics / Mathematik
    #[strum(to_string = "math.", serialize = "MATH.", serialize = "math", serialize = "MATH")]
    Math = 75,
    /// Medicine / Medizin
    #[strum(to_string = "med.", serialize = "MED.", serialize = "med", serialize = "MED")]
    Med = 76,
    /// Medical Engineering & Imaging / Medizintechnik
    #[strum(to_string = "MedTech.", serialize = "MEDTECH.", serialize = "MEDTECH", serialize = "medtech", serialize = "medtech.", serialize = "MedTech")]
    MedTech = 77,
    /// Meteorology / Meteorologie
    #[strum(to_string = "meteo.", serialize = "METEO.", serialize = "meteo", serialize = "METEO")]
    Meteo = 78,
    /// Military / Militärwesen
    #[strum(to_string = "mil.", serialize = "MIL", serialize = "MIL.", serialize = "mil", serialize = "Soldatensprache")]
    #[strum(serialize = "milit.")]
    Mil = 79,
    /// Mineralogy / Mineralogie
    #[strum(to_string = "mineral.", serialize = "mineral", serialize = "MINERAL.", serialize = "MINERAL")]
    Mineral = 80,
    /// Mining & Drilling / Bergbau & Bohrtechnik
    #[strum(to_string = "mining", serialize = "min.", serialize = "MIN.", serialize = "MINING", serialize = "MIN", serialize = "mining.", serialize = "min", serialize = "MINING.")]
    Mining = 81,
    /// Music / Musik
    #[strum(to_string = "mus.", serialize = "MUS.", serialize = "MUS", serialize = "mus")]
    Mus = 82,
    /// Mycology / Mykologie, Pilze
    #[strum(to_string = "mycol.", serialize = "MYCOL.", serialize = "MYCOL", serialize = "mycol")]
    #[strum(serialize = "myc.")]
    Mycol = 83,
    /// Mythology / Mythologie
    #[strum(to_string = "myth.", serialize = "MYTH.", serialize = "myth", serialize = "MYTH")]
    Myth = 84,
    /// Names of Persons / Namenkunde (nur Personennamen)
    #[strum(to_string = "name", serialize = "NAME", serialize = "name.", serialize = "NAME.")]
    Name = 85,
    /// Nautical Science / Nautik, Schifffahrtskunde
    #[strum(to_string = "naut.", serialize = "NAUT", serialize = "naut", serialize = "NAUT.")]
    Naut = 86,
    /// Neologisms / Neologismen (Wortneubildungen)
    #[strum(to_string = "neol.", serialize = "neol", serialize = "NEOL.", serialize = "NEOL")]
    Neol = 87,
    /// Nuclear Engineering / Nukleartechnik
    #[strum(to_string = "nucl.", serialize = "NUCL", serialize = "nucl", serialize = "NUCL.")]
    Nucl = 88,
    /// Oenology / Önologie, Lehre vom Wein
    #[strum(to_string = "oenol.", serialize = "OENOL.", serialize = "oenol", serialize = "OENOL")]
    Oenol = 89,
    /// Optics / Optik
    #[strum(to_string = "optics", serialize = "OPTICS.", serialize = "optics.", serialize = "OPTICS")]
    #[strum(serialize = "optical")]
    Optics = 90,
    /// Ornithology / Ornithologie, Vogelkunde
    #[strum(to_string = "orn.", serialize = "orn", serialize = "ORN.", serialize = "ORN")]
    #[strum(serialize = "ornith.", serialize = "ORNITH.", serialize = "ornith", serialize = "ORNITH")]
    Orn = 91,
    /// Pharmacy / Pharmazie
    #[strum(to_string = "pharm.", serialize = "PHARM.", serialize = "pharm", serialize = "PHARM")]
    Pharm = 92,
    /// Philately / Philatelie, Briefmarkenkunde
    #[strum(to_string = "philat.", serialize = "philat", serialize = "PHILAT", serialize = "PHILAT.")]
    Philat = 93,
    /// Philosophy / Philosophie
    #[strum(to_string = "philos.", serialize = "phil.", serialize = "PHILOS", serialize = "phil", serialize = "PHIL", serialize = "PHIL.", serialize = "philos", serialize = "PHILOS.")]
    Philos = 94,
    /// Phonetics / Phonetik
    #[strum(to_string = "phonet.", serialize = "PHONET.", serialize = "PHONET", serialize = "phonet")]
    Phonet = 95,
    /// Photography / Fotografie
    #[strum(to_string = "photo.", serialize = "PHOTO", serialize = "photo", serialize = "PHOTO.")]
    Photo = 96,
    /// Physics / Physik
    #[strum(to_string = "phys.", serialize = "PHYS.", serialize = "phys", serialize = "PHYS")]
    Phys = 97,
    /// Politics / Politik
    #[strum(to_string = "pol.", serialize = "POL", serialize = "POL.", serialize = "pol")]
    Pol = 98,
    /// Print, Typography, Layout / Druck, Typografie, Layout
    #[strum(to_string = "print", serialize = "print.", serialize = "PRINT.", serialize = "PRINT")]
    Print = 99,
    /// Proverb / Sprichwort
    #[strum(to_string = "proverb", serialize = "PROVERB", serialize = "PROVERB.", serialize = "proverb.")]
    #[strum(serialize = "prov.")]
    Proverb = 100,
    /// Psychology / Psychologie
    #[strum(to_string = "psych.", serialize = "PSYCH.", serialize = "psych", serialize = "PSYCH")]
    Psych = 101,
    /// Publishing / Verlagswesen
    #[strum(to_string = "publ.", serialize = "publ", serialize = "PUBL", serialize = "PUBL.")]
    Publ = 102,
    /// Quality Management / Qualitätsmanagement
    #[strum(to_string = "QM", serialize = "qm.", serialize = "QM.", serialize = "qm")]
    Qm = 103,
    /// Quotation / Zitat
    #[strum(to_string = "quote", serialize = "QUOTE", serialize = "QUOTE.", serialize = "quote.")]
    Quote = 104,
    /// Radio and Television / Radio und Fernsehen
    #[strum(to_string = "RadioTV", serialize = "RADIOTV", serialize = "tv", serialize = "TV.", serialize = "RadioTV.", serialize = "RADIOTV.", serialize = "tv.", serialize = "radiotv", serialize = "TV", serialize = "radiotv.")]
    RadioTv = 105,
    /// Rail / Eisenbahn
    #[strum(to_string = "rail", serialize = "RAIL.", serialize = "RAIL", serialize = "rail.")]
    Rail = 106,
    /// Real Estate / Immobilien
    #[strum(to_string = "RealEst.", serialize = "REALEST.", serialize = "RealEst", serialize = "realest.", serialize = "realest", serialize = "REALEST")]
    RealEst = 107,
    /// Religion / Religion
    #[strum(to_string = "relig.", serialize = "relig", serialize = "RELIG", serialize = "RELIG.")]
    Relig = 108,
    /// Rhetoric / Rhetorik
    #[strum(to_string = "rhet.", serialize = "rhet", serialize = "RHET.", serialize = "RHET")]
    Rhet = 109,
    /// School/Schule
    #[strum(to_string = "school", serialize = "SCHOOL.", serialize = "SCHOOL", serialize = "school.")]
    School = 110,
    /// Sociology / Soziologie
    #[strum(to_string = "sociol.", serialize = "SOC", serialize = "SOC.", serialize = "SOCIOL.", serialize = "sociol", serialize = "soc", serialize = "SOCIOL", serialize = "soc.")]
    Sociol = 111,
    /// Specialized Term / Fachsprachlicher Ausdruck
    #[strum(to_string = "spec.", serialize = "spec", serialize = "SPEC", serialize = "SPEC.")]
    Spec = 112,
    /// Sports / Sport
    #[strum(to_string = "sports", serialize = "sport", serialize = "SPORT.", serialize = "SPORTS.", serialize = "SPORTS", serialize = "SPORT", serialize = "sport.", serialize = "sports.")]
    Sports = 113,
    /// Statistics / Statistik
    #[strum(to_string = "stat.", serialize = "STAT", serialize = "STATIST.", serialize = "STAT.", serialize = "stat", serialize = "STATIST", serialize = "statist", serialize = "statist.")]
    Stat = 114,
    /// Stock Exchange / Börsenwesen
    #[strum(to_string = "stocks", serialize = "STOCKS", serialize = "stocks.", serialize = "STOCKS.")]
    Stocks = 115,
    /// Studium
    #[strum(to_string = "stud.", serialize = "STUD", serialize = "stud", serialize = "STUD.")]
    Stud = 116,
    /// Taxonomic terms for animals, plants and fungi (incl. varieties and breeds) / Taxonomische Bezeichnungen für Tiere, Pflanzen und Pilze (inkl. Zuchtformen und Rassen)
    #[strum(to_string = "T", serialize = "t", serialize = "t.", serialize = "T.")]
    T = 117,
    /// Technology / Technik
    #[strum(to_string = "tech.", serialize = "TECH", serialize = "tech", serialize = "TECH.")]
    Tech = 118,
    /// Telecommunications / Telekommunikation
    #[strum(to_string = "telecom.", serialize = "TELCO", serialize = "TELECOM.", serialize = "TELECOM", serialize = "TELCO.", serialize = "telco", serialize = "telecom", serialize = "telco.")]
    Telecom = 119,
    /// Textiles, Textile Industry / Textilien, Textilindustrie
    #[strum(to_string = "textil.", serialize = "TEXTIL", serialize = "textil", serialize = "TEXTIL.")]
    Textil = 120,
    /// Theatre / Theater
    #[strum(to_string = "theatre", serialize = "THEATRE.", serialize = "theatre.", serialize = "THEATRE")]
    Theatre = 121,
    /// Tools / Werkzeuge
    #[strum(to_string = "tools", serialize = "TOOLS.", serialize = "tools.", serialize = "TOOLS")]
    Tools = 122,
    /// Toys / Spielzeug
    #[strum(to_string = "toys", serialize = "TOYS", serialize = "toys.", serialize = "TOYS.")]
    Toys = 123,
    /// Travellers vocabulary / Reise-Wortschatz
    #[strum(to_string = "TrVocab.", serialize = "TrVocab", serialize = "trvocab.", serialize = "trvocab", serialize = "TRVOCAB", serialize = "TRVOCAB.")]
    TrVocab = 124,
    /// Traffic / Verkehrswesen
    #[strum(to_string = "traffic", serialize = "TRAFFIC", serialize = "TRAFFIC.", serialize = "traffic.")]
    Traffic = 125,
    /// Transportation (Land Transport) / Transportwesen (Landtransport)
    #[strum(to_string = "transp.", serialize = "TRANSP.", serialize = "TRANSP", serialize = "transp")]
    Transp = 126,
    /// Travel Industry / Touristik
    #[strum(to_string = "travel", serialize = "travel.", serialize = "TRAVEL", serialize = "TRAVEL.")]
    Travel = 127,
    /// Units, Measures, Weights / Einheiten, Maße, Gewichte
    #[strum(to_string = "unit", serialize = "UNIT", serialize = "UNIT.", serialize = "unit.")]
    Unit = 128,
    /// Urban Planning / Urbanistik, Stadtplanung
    #[strum(to_string = "urban", serialize = "URBAN", serialize = "URBAN.", serialize = "urban.")]
    Urban = 129,
    /// UNESCO World Heritage / UNESCO-Welterbe
    #[strum(to_string = "UWH", serialize = "uwh.", serialize = "uwh", serialize = "UWH.")]
    Uwh = 130,
    /// Veterinary Medicine / Veterinärmedizin
    #[strum(to_string = "VetMed.", serialize = "vetmed.", serialize = "VetMed", serialize = "vetmed", serialize = "VETMED.", serialize = "VETMED")]
    VetMed = 131,
    /// Watches, Clocks / Uhren
    #[strum(to_string = "watches", serialize = "WATCHES.", serialize = "WATCHES", serialize = "watches.")]
    Watches = 132,
    /// Weapons / Waffen
    #[strum(to_string = "weapons", serialize = "weapons.", serialize = "WEAPONS", serialize = "WEAPONS.")]
    Weapons = 133,
    /// Zoology, Animals / Zoologie, Tierkunde
    #[strum(to_string = "zool.", serialize = "ZOOL.", serialize = "ZOOL", serialize = "zool")]
    Zool = 134,
    /// Kindersprache
    #[strum(to_string = "children's speech", serialize = "Kindersprache")]
    Child = 135,
    /// Kindersprache
    #[strum(to_string = "youth speech", serialize = "Jugendsprache")]
    Youth = 136,
    /// Wissenschaft
    #[strum(to_string = "sci.")]
    Science = 137,
    /// Wissenschaft
    #[strum(to_string = "poet.")]
    Poetry = 138,
    /// Wissenschaft
    #[strum(to_string = "currency")]
    Currency = 139
}

impl_try_from_as_unpack! {
    Domain => Domain
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Domain {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl TopicMatrixIndex for Domain {
    #[inline(always)]
    fn get(self) -> usize {
        (self as u64) as usize
    }
}

impl Fits64 for Domain {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Domain::try_from(x).unwrap()
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
}


/// In sociolinguistics, a register is a variety of language used for a particular purpose or particular communicative situation
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Display, EnumString, IntoStaticStr, EnumCount)]
#[derive(TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u64)]
pub enum Register {
    #[strum(to_string = "humor.", serialize = "humor", serialize = "hum.", serialize = "hum")]
    Humor = 0,
    #[strum(to_string = "vulg.", serialize = "vulg", serialize = "derb")]
    Vulg = 1,
    #[strum(to_string = "techn.", serialize = "techn")]
    Techn = 2,
    #[strum(to_string = "coll.", serialize = "coll", serialize = "ugs", serialize = "ugs.")]
    #[strum(serialize = "ug")]
    Coll = 3,
    /// Gehoben
    #[strum(to_string = "geh.", serialize = "geh")]
    Geh = 4,
    #[strum(to_string = "slang", serialize = "slang.", serialize = "sl.", serialize = "jargon")]
    Slang = 5,
    #[strum(to_string = "iron.", serialize = "iron")]
    Iron = 6,
    #[strum(to_string = "formal", serialize = "formal.", serialize = "formell")]
    Formal = 7,
    #[strum(to_string = "euphem.", serialize = "euphem")]
    Euphem = 8,
    #[strum(to_string = "literary", serialize = "literary.")]
    Literary = 9,
    #[strum(to_string = "dialect", serialize = "dialect.")]
    Dialect = 10,
    /// DictCC
    #[strum(to_string = "archaic", serialize = "veraltet", serialize = "veraltend")]
    #[strum(serialize = "dated", serialize = "alt", serialize = "obs.")]
    #[strum(serialize = "altertümlich", serialize = "veraltentd", serialize = "frühere Bezeichnung")]
    #[strum(serialize = "ancient name", serialize = "becoming dated")]
    #[strum(serialize = "altertümelnd", serialize = "slightly dated")]
    Archaic = 11,
    /// DictCC
    #[strum(to_string = "rare", serialize = "selten", serialize = "very rare")]
    Rare = 12,
    /// DictCC -
    #[strum(to_string = "pej.")]
    Pejorativ = 13,
    /// DictCC - also figurative
    #[strum(to_string = "fig.")]
    Figurative = 14,
    #[strum(to_string = "also fig.", serialize = "auch fig.")]
    AlsoFigurative = 15,
    /// spelling variant (less common)
    #[strum(to_string = "spv.", serialize = "Rsv.")]
    SpellingVariant = 16,
    /// official language; administration
    #[strum(to_string = "adm.")]
    Admin = 17,
    /// Übertragen: giftig -> virulently
    #[strum(to_string = "übtr.")]
    Transfer = 18,
    /// Netzjargon
    #[strum(to_string = "Chat-Jargon", serialize = "internet slang", serialize = "chat jargon")]
    NetJargon = 19,
    /// Informal
    #[strum(to_string = "informell")]
    Informal = 20,
    /// Quantity
    #[strum(to_string = "Mengenangabe")]
    QuantityInformation = 21,
    #[strum(to_string = "IATEPreferred")]
    IATEPreferred = 22
}

impl_try_from_as_unpack! {
    Register => Register
}

// #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Register {
    fn __str__(&self) -> &'static str {
        self.into()
    }

    fn __repr__(&self) -> &'static str {
        self.into()
    }
}

impl TopicMatrixIndex for Register {
    #[inline(always)]
    fn get(self) -> usize {
        Domain::COUNT + (self as u64) as usize
    }
}

impl Fits64 for Register {
    #[inline(always)]
    unsafe fn from_u64(x: u64) -> Self {
        Register::try_from(x).unwrap()
    }

    #[inline(always)]
    fn to_u64(self) -> u64 {
        self.into()
    }
}

impl TryFrom<AdministrativeStatus> for Register {
    type Error = AdministrativeStatus;

    fn try_from(value: AdministrativeStatus) -> Result<Self, AdministrativeStatus> {
        match value {
            AdministrativeStatus::Obsolete | AdministrativeStatus::Deprecated => {
                Ok(Self::Archaic)
            }
            AdministrativeStatus::Preferred => {
                Ok(Self::IATEPreferred)
            }
            other => {
                Err(other)
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum PartialWordType {
    Prefix,
    Suffix,
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