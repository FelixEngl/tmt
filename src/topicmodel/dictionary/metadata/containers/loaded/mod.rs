pub mod metadata;
pub mod reference;
pub mod reference_mut;
pub mod solved;
pub mod manager;
pub mod collector;

use std::collections::HashMap;
use tinyset::Set64;
use crate::toolkit::typesafe_interner::*;
use crate::topicmodel::dictionary::word_infos::*;


macro_rules! generate_field_code {
    (
        $(
            $tt:tt {
                $doc: literal $name: ident: $assoc_typ: ty | $cache_typ: ty | $resolved_typ: ty $( as $marker:tt {
                    $interner_name: ident $(: $interner_type: ident)? => $interner_method: ident
                })?
            }
        ),+
        $(,)?
    ) => {
        $crate::topicmodel::dictionary::metadata::loaded::reference::create_ref_implementation!(
            $($tt: $name $(, $interner_name)?: $cache_typ | $assoc_typ,)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::reference_mut::create_mut_ref_implementation!(
            $($tt: $name $(, $interner_method)?: $assoc_typ,)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::manager::create_managed_implementation!(
            $($($($interner_name: $interner_type => $interner_method: $assoc_typ,)?)?)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::metadata::create_metadata_impl!(
            $($doc $name: $assoc_typ,)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::solved::create_solved_implementation!(
            $($tt $(as $marker)?: $name: $resolved_typ,)+
        );
        crate::topicmodel::dictionary::metadata::loaded::collector::create_collector_implementation!(
            $($tt: $name: $assoc_typ,)+
        );
    };
}

generate_field_code! {
    set {
        r#"Stores the languages of a word."#
        languages: Language | Set64<Language> | Set64<Language>
    },
    set {
        r#"Stores the domains of a word."#
        domains: Domain | Set64<Domain> | Set64<Domain>
    },
    set {
        r#"Stores the register of a word."#
        registers: Register | Set64<Register> | Set64<Register>
    },
    set {
        r#"Stores the gender of a word."#
        genders: GrammaticalGender | Set64<GrammaticalGender> | Set64<GrammaticalGender>
    },
    set {
        r#"Stores the pos of a word."#
        pos: PartOfSpeech | Set64<PartOfSpeech> | Set64<PartOfSpeech>
    },
    set {
        r#"Stores the regions of a word."#
        regions: Region | Set64<Region> | Set64<Region>
    },
    set {
        r#"Stores the number of a word."#
        numbers: GrammaticalNumber | Set64<GrammaticalNumber> | Set64<GrammaticalNumber>
    },
    set {
        r#"Stores an internal id, associating some words with each other."#
        internal_ids: u64 | Set64<u64> | Set64<u64>
    },
    interned {
        r#"Stores the inflected value of a word."#
        inflected: InflectedSymbol | (Set64<InflectedSymbol>, Vec<&'a str>) | Vec<String> as interned {
            inflected_interner: DefaultInflectedStringInterner => intern_inflected
        }
    },
    interned {
        r#"Stores the abbreviations value of a word."#
        abbreviations: AbbreviationSymbol | (Set64<AbbreviationSymbol>, Vec<&'a str>) | Vec<String> as interned {
            abbreviations_interner: DefaultAbbreviationStringInterner => intern_abbreviations
        }
    },
    interned {
        r#"Stores the unaltered vocabulary value of a word."#
        unaltered_vocabulary: UnalteredVocSymbol | (Set64<UnalteredVocSymbol>, Vec<&'a str>) | Vec<String> as interned {
            unaltered_vocabulary_interner: DefaultUnalteredVocStringInterner => intern_unaltered_vocabulary
        }
    },
    interned {
        r#"Stores the synonyms"#
        synonyms: UnalteredVocSymbol | (Set64<UnalteredVocSymbol>, Vec<&'a str>) | Vec<String> as interned {
            unaltered_vocabulary_interner => intern_unaltered_vocabulary
        }
    },
    interned {
        r#"Stores similar words"#
        look_at: UnalteredVocSymbol | (Set64<UnalteredVocSymbol>, Vec<&'a str>) | Vec<String> as interned {
            unaltered_vocabulary_interner => intern_unaltered_vocabulary
        }
    },
    interned {
        r#"Stores some kind of artificial id"#
        ids: AnyIdSymbol | (Set64<AnyIdSymbol>, Vec<&'a str>) | Set64<AnyIdSymbol> as set {
            ids_interner: DefaultAnyIdStringInterner => intern_ids
        }
    },
    interned {
        r#"Stores outgoing ids"#
        outgoing_ids: AnyIdSymbol | (Set64<AnyIdSymbol>, Vec<&'a str>) | Set64<AnyIdSymbol> as set {
            ids_interner => intern_ids
        }
    },
    interned {
        r#"Stores the original entry. May contain multiple is some kind of merge action is done."#
        original_entry: OriginalEntrySymbol | (Set64<OriginalEntrySymbol>, Vec<&'a str>) | Vec<String> as interned {
            original_entry_interner: DefaultOriginalEntryStringInterner => intern_original_entry
        }
    },
    interned {
        r#"Contextual information."#
        contextual_informations: ContextualInformationSymbol | (Set64<ContextualInformationSymbol>, Vec<&'a str>) | Vec<String> as interned {
            contextual_informations_interner: DefaultContextualInformationStringInterner => intern_contextual_informations
        }
    },
    interned {
        r#"Unclassified information."#
        unclassified: UnclassifiedSymbol | (Set64<UnclassifiedSymbol>, Vec<&'a str>) | Vec<String> as interned {
            unclassified_interner: DefaultUnclassifiedStringInterner => intern_unclassified
        }
    },
}

impl LoadedMetadata {
    pub fn lead_x(&self) -> HashMap<Option<DictionaryOriginSymbol>, tinyset::Set64<u64>> {
        self.iter().map(|value| {
            (value.origin(), value.meta().internal_ids.clone())
        }).collect()
    }
}