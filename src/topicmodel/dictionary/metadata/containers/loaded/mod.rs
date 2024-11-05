pub mod metadata;
pub mod reference;
pub mod reference_mut;
pub mod solved;
pub mod manager;
pub mod collector;
pub mod field_denom;
mod resolved_value;
mod solved_new_arg;

pub use resolved_value::*;
pub use solved_new_arg::*;

use tinyset::Set64;
use crate::register_python;
use crate::topicmodel::dictionary::word_infos::*;
use crate::toolkit::typesafe_interner::*;


register_python! {
    struct SolvedLoadedMetadata;
    struct MetaField;
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyo3::pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Test {
    A,
    B,
    C
}


macro_rules! generate_field_code {
    (
        $(
            $tt:tt {
                $doc: literal $name: ident ($lit_name:literal): $assoc_typ: ty | $cache_typ: ty $({
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
            $($tt: $name $(, $interner_name, $interner_method)?: $assoc_typ,)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::manager::create_managed_implementation!(
            $($($($interner_name: $interner_type => $interner_method: $assoc_typ,)?)?)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::metadata::create_metadata_impl!(
            $($doc $name: $assoc_typ,)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::solved::create_solved_implementation!(
            $($tt: $name $lit_name,)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::collector::create_collector_implementation!(
            $($tt: $name: $assoc_typ,)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::field_denom::generate_field_denoms!(
            $($name($lit_name),)+
        );
    };
}




generate_field_code! {
    set {
        r#"Stores the languages of a word."#
        languages("languages"): Language | Set64<Language>
    },
    set {
        r#"Stores the domains of a word."#
        domains("domains"): Domain | Set64<Domain>
    },
    set {
        r#"Stores the register of a word."#
        registers("registers"): Register | Set64<Register>
    },
    set {
        r#"Stores the gender of a word."#
        genders("genders"): GrammaticalGender | Set64<GrammaticalGender>
    },
    set {
        r#"Stores the pos of a word."#
        pos("pos"): PartOfSpeech | Set64<PartOfSpeech>
    },
    set {
        r#"Stores the regions of a word."#
        regions("regions"): Region | Set64<Region>
    },
    set {
        r#"Stores the number of a word."#
        numbers("numbers"): GrammaticalNumber | Set64<GrammaticalNumber>
    },
    set {
        r#"Stores an internal id, associating some words with each other."#
        internal_ids("internal_ids"): u64 | Set64<u64>
    },
    interned {
        r#"Stores the inflected value of a word."#
        inflected("inflected"): InflectedSymbol | (Set64<InflectedSymbol>, Vec<&'a str>) {
            inflected_interner: DefaultInflectedStringInterner => intern_inflected
        }
    },
    interned {
        r#"Stores the abbreviations value of a word."#
        abbreviations("abbreviations"): AbbreviationSymbol | (Set64<AbbreviationSymbol>, Vec<&'a str>) {
            abbreviations_interner: DefaultAbbreviationStringInterner => intern_abbreviations
        }
    },
    interned {
        r#"Stores the unaltered vocabulary value of a word."#
        unaltered_vocabulary("unaltered_vocabulary"): UnalteredVocSymbol | (Set64<UnalteredVocSymbol>, Vec<&'a str>) {
            unaltered_vocabulary_interner: DefaultUnalteredVocStringInterner => intern_unaltered_vocabulary
        }
    },
    interned {
        r#"Stores the synonyms"#
        synonyms("synonyms"): UnalteredVocSymbol | (Set64<UnalteredVocSymbol>, Vec<&'a str>)  {
            unaltered_vocabulary_interner => intern_unaltered_vocabulary
        }
    },
    interned {
        r#"Stores similar words"#
        look_at("look_at"): UnalteredVocSymbol | (Set64<UnalteredVocSymbol>, Vec<&'a str>) {
            unaltered_vocabulary_interner => intern_unaltered_vocabulary
        }
    },
    interned {
        r#"Stores some kind of artificial id"#
        ids("ids"): AnyIdSymbol | (Set64<AnyIdSymbol>, Vec<&'a str>) {
            ids_interner: DefaultAnyIdStringInterner => intern_ids
        }
    },
    interned {
        r#"Stores outgoing ids"#
        outgoing_ids("outgoing_ids"): AnyIdSymbol | (Set64<AnyIdSymbol>, Vec<&'a str>) {
            ids_interner => intern_ids
        }
    },
    interned {
        r#"Stores the original entry. May contain multiple is some kind of merge action is done."#
        original_entry("original_entry"): OriginalEntrySymbol | (Set64<OriginalEntrySymbol>, Vec<&'a str>) {
            original_entry_interner: DefaultOriginalEntryStringInterner => intern_original_entry
        }
    },
    interned {
        r#"Contextual information."#
        contextual_informations("contextual_informations"): ContextualInformationSymbol | (Set64<ContextualInformationSymbol>, Vec<&'a str>) {
            contextual_informations_interner: DefaultContextualInformationStringInterner => intern_contextual_informations
        }
    },
    interned {
        r#"Unclassified information."#
        unclassified("unclassified"): UnclassifiedSymbol | (Set64<UnclassifiedSymbol>, Vec<&'a str>) {
            unclassified_interner: DefaultUnclassifiedStringInterner => intern_unclassified
        }
    },
}
