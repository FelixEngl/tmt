use serde::{Deserialize, Serialize};
use string_interner::Symbol;
use crate::toolkit::typesafe_interner::{DefaultAbbreviation, DefaultDictionaryOrigin, DefaultInflected, DefaultSynonym, DefaultUnalteredVoc};
use crate::topicmodel::dictionary::metadata::Metadata;
use crate::topicmodel::dictionary::word_infos::{Domain, GrammaticalGender, GrammaticalNumber, Language, PartOfSpeech, Register};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct LoadedMetadata {
    #[serde(skip_serializing_if = "AssociatedMetadata::is_empty", default)]
    general_metadata: AssociatedMetadata,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "empty_vec")]
    associated_metadata: Vec<AssociatedMetadata>
}

fn empty_vec() -> Vec<AssociatedMetadata> {
    Vec::with_capacity(0)
}

impl Default for LoadedMetadata {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl LoadedMetadata {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            general_metadata: AssociatedMetadata::default(),
            associated_metadata: Vec::with_capacity(capacity)
        }
    }

    pub fn get_general_metadata(&self) -> &AssociatedMetadata {
        &self.general_metadata
    }

    pub fn get_mut_general_metadata(&mut self) -> &mut AssociatedMetadata {
        &mut self.general_metadata
    }

    pub fn get_associated_metadata(&self, origin: DefaultDictionaryOrigin) -> Option<&AssociatedMetadata> {
        self.associated_metadata.get(origin.to_usize())
    }

    pub fn get_mut_associated_metadata(&mut self, origin: DefaultDictionaryOrigin) -> Option<&mut AssociatedMetadata> {
        self.associated_metadata.get_mut(origin.to_usize())
    }

    #[inline(always)]
    fn get_or_create_impl(&mut self, origin: usize) -> &mut AssociatedMetadata {
        if self.associated_metadata.len() <= origin {
            self.associated_metadata.resize_with(origin + 1, AssociatedMetadata::default);
        }
        unsafe {self.associated_metadata.get_unchecked_mut(origin)}
    }

    pub fn get_or_create(&mut self, origin: DefaultDictionaryOrigin) -> &mut AssociatedMetadata {
        self.get_or_create_impl(origin.to_usize())
    }

    pub fn update_with(&mut self, other: &LoadedMetadata) {
        self.general_metadata.update_with(&other.general_metadata);
        for (pos, value) in other.associated_metadata.iter().enumerate() {
            self.get_or_create_impl(pos).update_with(value)
        }
    }
}

macro_rules! impl_collect_all {
    (tinyset: $($ident:ident: $ty:ty),+) => {
        impl LoadedMetadata {
            $(
                paste::paste! {
                    pub fn [<collect_all_ $ident>](&self) -> tinyset::Set64<$ty> {
                        let mut result = self.associated_metadata.iter().flat_map(|value| {
                            value.$ident.iter()
                        }).collect::<tinyset::Set64<_>>();
                        result.extend(self.general_metadata.$ident.iter());
                        result
                    }
                }
            )+
        }
    };
}


impl_collect_all! {
    tinyset:
    languages: Language,
    domains: Domain,
    registers: Register,
    gender: GrammaticalGender,
    pos: PartOfSpeech,
    number: GrammaticalNumber,
    inflected: DefaultInflected,
    abbreviations: DefaultAbbreviation,
    unaltered_vocabulary: DefaultUnalteredVoc,
    synonyms: DefaultSynonym
}

impl Metadata for LoadedMetadata{}


#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct AssociatedMetadata {
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    languages: tinyset::Set64<Language>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    domains: tinyset::Set64<Domain>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    registers: tinyset::Set64<Register>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    gender: tinyset::Set64<GrammaticalGender>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    pos: tinyset::Set64<PartOfSpeech>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    number: tinyset::Set64<GrammaticalNumber>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    inflected: tinyset::Set64<DefaultInflected>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    abbreviations: tinyset::Set64<DefaultAbbreviation>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    unaltered_vocabulary: tinyset::Set64<DefaultUnalteredVoc>,
    #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
    synonyms: tinyset::Set64<DefaultSynonym>,
}

impl AssociatedMetadata {
    pub fn update_with(&mut self, other: &AssociatedMetadata) {
        self.languages.extend(other.languages.iter());
        self.domains.extend(other.domains.iter());
        self.registers.extend(other.registers.iter());
        self.gender.extend(other.gender.iter());
        self.pos.extend(other.pos.iter());
        self.number.extend(other.number.iter());
        self.inflected.extend(other.inflected.iter());
        self.abbreviations.extend(other.abbreviations.iter());
        self.unaltered_vocabulary.extend(other.unaltered_vocabulary.iter());
        self.synonyms.extend(other.synonyms.iter());
    }

    pub fn is_empty(&self) -> bool {
        self.languages.is_empty() &&
            self.domains.is_empty() &&
            self.registers.is_empty() &&
            self.gender.is_empty() &&
            self.pos.is_empty() &&
            self.number.is_empty() &&
            self.inflected.is_empty() &&
            self.abbreviations.is_empty() &&
            self.unaltered_vocabulary.is_empty() &&
            self.synonyms.is_empty()
    }

    pub fn languages(&self) -> &tinyset::Set64<Language> {
        &self.languages
    }

    pub fn domains(&self) -> &tinyset::Set64<Domain> {
        &self.domains
    }

    pub fn registers(&self) -> &tinyset::Set64<Register> {
        &self.registers
    }

    pub fn gender(&self) -> &tinyset::Set64<GrammaticalGender> {
        &self.gender
    }

    pub fn pos(&self) -> &tinyset::Set64<PartOfSpeech> {
        &self.pos
    }

    pub fn number(&self) -> &tinyset::Set64<GrammaticalNumber> {
        &self.number
    }

    pub fn inflected(&self) -> &tinyset::Set64<DefaultInflected> {
        &self.inflected
    }

    pub fn abbreviations(&self) -> &tinyset::Set64<DefaultAbbreviation> {
        &self.abbreviations
    }

    pub fn unaltered_vocabulary(&self) -> &tinyset::Set64<DefaultUnalteredVoc> {
        &self.unaltered_vocabulary
    }

    pub fn synonyms(&self) -> &tinyset::Set64<DefaultSynonym> {
        &self.synonyms
    }
}

macro_rules! create_tinyset_impl {
    ($($ident: ident: $ty: ty),+) => {
        impl AssociatedMetadata {
            $(
                paste::paste! {
                    pub fn [<add_single_to_ $ident>](&mut self, value: $ty) {
                        self.$ident.insert(value);
                    }
                    pub fn [<add_all_to_ $ident>]<I: IntoIterator<Item=$ty>>(&mut self, values: I) {
                        self.$ident.extend(values);
                    }
                }
            )+

        }
    };
}

create_tinyset_impl! {
    languages: Language,
    domains: Domain,
    registers: Register,
    gender: GrammaticalGender,
    pos: PartOfSpeech,
    number: GrammaticalNumber,
    inflected: DefaultInflected,
    abbreviations: DefaultAbbreviation,
    unaltered_vocabulary: DefaultUnalteredVoc,
    synonyms: DefaultSynonym
}


