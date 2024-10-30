use std::fmt::{Display, Formatter};
use itertools::Itertools;
use tinyset::Set64;
use crate::topicmodel::dictionary::metadata::loaded::reference::LoadedMetadataRef;
use crate::topicmodel::dictionary::word_infos::{Domain, GrammaticalGender, GrammaticalNumber, Language, PartOfSpeech, Register};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SolvedLoadedMetadata {
    unaltered_vocabulary: Vec<String>,
    inflected: Vec<String>,
    abbreviation: Vec<String>,
    synonyms: Vec<String>,
    languages: Set64<Language>,
    domains: Set64<Domain>,
    registers: Set64<Register>,
    gender: Set64<GrammaticalGender>,
    pos: Set64<PartOfSpeech>,
    number: Set64<GrammaticalNumber>,
}

impl SolvedLoadedMetadata {
    pub fn unaltered_vocabulary(&self) -> &Vec<String> {
        &self.unaltered_vocabulary
    }

    pub fn inflected(&self) -> &Vec<String> {
        &self.inflected
    }

    pub fn abbreviation(&self) -> &Vec<String> {
        &self.abbreviation
    }

    pub fn languages(&self) -> &Set64<Language> {
        &self.languages
    }

    pub fn domains(&self) -> &Set64<Domain> {
        &self.domains
    }

    pub fn registers(&self) -> &Set64<Register> {
        &self.registers
    }

    pub fn gender(&self) -> &Set64<GrammaticalGender> {
        &self.gender
    }

    pub fn pos(&self) -> &Set64<PartOfSpeech> {
        &self.pos
    }

    pub fn number(&self) -> &Set64<GrammaticalNumber> {
        &self.number
    }

    pub fn synonyms(&self) -> &Vec<String> {
        &self.synonyms
    }
}

impl<'a> From<LoadedMetadataRef<'a>> for SolvedLoadedMetadata {
    fn from(value: LoadedMetadataRef<'a>) -> Self {
        let unaltered_vocabulary = value.get_unaltered_vocabulary().iter().map(|value| value.to_string()).collect_vec();
        let inflected = value.get_inflected().iter().map(|value| value.to_string()).collect_vec();
        let abbreviation = value.get_abbreviation().iter().map(|value| value.to_string()).collect_vec();
        let synonyms = value.get_abbreviation().iter().map(|value| value.to_string()).collect_vec();
        Self {
            inflected,
            abbreviation,
            unaltered_vocabulary,
            synonyms,
            languages: value.get_languages().clone(),
            domains: value.get_domains().clone(),
            registers: value.get_registers().clone(),
            gender: value.get_gender().clone(),
            pos: value.get_pos().clone(),
            number: value.get_number().clone(),
        }
    }
}

impl Display for SolvedLoadedMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(number: {}, " , self.number.iter().join(", "))?;
        write!(f, "pos: {}, " , self.pos.iter().join(", "))?;
        write!(f, "gender: {}, " , self.gender.iter().join(", "))?;
        write!(f, "registers: {}, " , self.registers.iter().join(", "))?;
        write!(f, "domains: {}, " , self.domains.iter().join(", "))?;
        write!(f, "languages: {}, " , self.languages.iter().join(", "))?;
        write!(f, "abbreviation: \"{}\", " , self.abbreviation.iter().join("\", \""))?;
        write!(f, "inflected: \"{}\", " , self.inflected.iter().join("\", \""))?;
        write!(f, "unaltered_vocabulary: \"{}\")" , self.unaltered_vocabulary.iter().join("\", \""))?;
        write!(f, "synonyms: \"{}\")" , self.synonyms.iter().join("\", \""))
    }
}