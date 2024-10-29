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
    languages: Set64<Language>,
    domains: Set64<Domain>,
    registers: Set64<Register>,
    gender: Set64<GrammaticalGender>,
    pos: Set64<PartOfSpeech>,
    number: Set64<GrammaticalNumber>,
}

impl<'a> From<LoadedMetadataRef<'a>> for SolvedLoadedMetadata {
    fn from(value: LoadedMetadataRef<'a>) -> Self {
        let unaltered_vocabulary = value.get_unaltered_vocabulary().iter().map(|value| value.to_string()).collect_vec();
        let inflected = value.get_inflected().iter().map(|value| value.to_string()).collect_vec();
        let abbreviation = value.get_abbreviation().iter().map(|value| value.to_string()).collect_vec();
        Self {
            inflected,
            abbreviation,
            unaltered_vocabulary,
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
        write!(f, "unaltered_vocabulary: \"{}\")" , self.unaltered_vocabulary.iter().join("\", \""))
    }
}