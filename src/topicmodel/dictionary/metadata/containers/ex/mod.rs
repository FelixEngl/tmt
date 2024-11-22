mod metadata;
mod reference;
mod reference_mut;
mod solved;
mod manager;
mod collector;
mod field_denom;
mod resolved_value;
mod solved_new_arg;
mod metadata_field_holder;
mod typedef;

pub use metadata::*;
use reference::*;
pub use reference_mut::*;
// pub use solved::*;
// pub use manager::*;
// pub use collector::*;
// pub use field_denom::*;
pub use resolved_value::*;
pub use solved_new_arg::*;

use tinyset::Set64;
use crate::register_python;
use crate::topicmodel::dictionary::word_infos::*;
use crate::toolkit::typesafe_interner::*;
use crate::topicmodel::dictionary::metadata::dict_meta_topic_matrix::DomainModelIndex;
use std::ops::Deref;

register_python! {
    struct LoadedMetadataEx;
    struct MetaField;
}


macro_rules! generate_field_code {
    (
        $(
            $tt:tt {
                $doc: literal $name: ident ($lit_name:literal): $assoc_typ: ty $({
                    $interner_name: ident $(: $interner_type: ident)? => $interner_method: ident
                })?
            }
        ),+
        $(,)?
    ) => {
        $crate::topicmodel::dictionary::metadata::ex::typedef::make_storage_type_def!(
            $($tt: $name: $assoc_typ;)+
        );

        paste::paste! {
            $crate::topicmodel::dictionary::metadata::ex::reference::create_ref_implementation!(
                $($tt: $name $(, $interner_name)?: [<$name:camel StoreType>]<'a>,)+
            );
        }

        $crate::topicmodel::dictionary::metadata::ex::reference_mut::create_mut_ref_implementation!(
            $($tt: $name $(, $interner_name, $interner_method)?: $assoc_typ,)+
        );
        $crate::topicmodel::dictionary::metadata::ex::manager::create_managed_implementation!(
            $($($($interner_name: $interner_type => $interner_method: $assoc_typ,)?)?)+
        );
        $crate::topicmodel::dictionary::metadata::ex::metadata::create_metadata_impl!(
            $($tt: $doc $name: $assoc_typ,)+
        );
        $crate::topicmodel::dictionary::metadata::ex::solved::create_solved_implementation!(
            $($tt: $name $lit_name,)+
        );
        $crate::topicmodel::dictionary::metadata::ex::collector::create_collector_implementation!(
            $($tt: $name: $assoc_typ,)+
        );

        $crate::topicmodel::dictionary::metadata::ex::field_denom::generate_field_denoms! {
            $($doc $name($lit_name),)+
        }

        $crate::topicmodel::dictionary::metadata::ex::manager::update_routine!(
            $($tt: $name $(, $interner_name $(, $interner_type)?)?;)*
        );
    };
}




generate_field_code! {
    set {
        r#"Stores the languages of a word."#
        languages("languages"): Language
    },
    set {
        r#"Stores the domains of a word."#
        domains("domains"): Domain
    },
    set {
        r#"Stores the register of a word."#
        registers("registers"): Register
    },
    set {
        r#"Stores the gender of a word."#
        genders("genders"): GrammaticalGender
    },
    set {
        r#"Stores the pos of a word."#
        pos("pos"): PartOfSpeech
    },
    set {
        r#"Stores additional tags for the pos of a word."#
        pos_tag("pos_tag"): PartOfSpeechTag
    },
    set {
        r#"Stores the regions of a word."#
        regions("regions"): Region
    },
    set {
        r#"Stores the number of a word."#
        numbers("numbers"): GrammaticalNumber
    },
    set {
        r#"Stores an internal id, associating some words with each other."#
        internal_ids("internal_ids"): u64
    },
    interned {
        r#"Stores the inflected value of a word."#
        inflected("inflected"): InflectedSymbol {
            inflected_interner: InflectedStringInterner => intern_inflected
        }
    },
    interned {
        r#"Stores the abbreviations value of a word."#
        abbreviations("abbreviations"): AbbreviationSymbol {
            abbreviations_interner: AbbreviationStringInterner => intern_abbreviations
        }
    },
    interned {
        r#"Stores the unaltered vocabulary value of a word."#
        unaltered_vocabulary("unaltered_vocabulary"): UnalteredVocSymbol {
            unaltered_vocabulary_interner: UnalteredVocStringInterner => intern_unaltered_vocabulary
        }
    },
    interned {
        r#"Stores similar words"#
        look_at("look_at"): UnalteredVocSymbol {
            unaltered_vocabulary_interner => intern_unaltered_vocabulary
        }
    },
    interned {
        r#"Stores some kind of artificial id"#
        ids("ids"): AnyIdSymbol {
            ids_interner: AnyIdStringInterner => intern_ids
        }
    },
    interned {
        r#"Stores outgoing ids"#
        outgoing_ids("outgoing_ids"): AnyIdSymbol {
            ids_interner => intern_ids
        }
    },
    interned {
        r#"Stores the original entry. May contain multiple is some kind of merge action is done."#
        original_entry("original_entry"): OriginalEntrySymbol {
            original_entry_interner: OriginalEntryStringInterner => intern_original_entry
        }
    },
    interned {
        r#"Contextual information."#
        contextual_informations("contextual_informations"): ContextualInformationSymbol {
            contextual_informations_interner: ContextualInformationStringInterner => intern_contextual_informations
        }
    },
    interned {
        r#"Unclassified information."#
        unclassified("unclassified"): UnclassifiedSymbol {
            unclassified_interner: UnclassifiedStringInterner => intern_unclassified
        }
    },
    voc {
        r#"Stores the synonyms"#
        synonyms("synonyms"): usize
    },
}


impl MetadataManagerEx {
    pub fn domain_count(&self) -> DomainCounts {
        use std::sync::Arc;
        use itertools::Itertools;
        use lockfree_object_pool::LinearObjectPool;
        use rayon::prelude::*;
        use super::super::dict_meta_topic_matrix::DOMAIN_MODEL_ENTRY_MAX_SIZE;

        #[repr(transparent)]
        struct Wrap<'a>(&'a MetadataEx);
        impl<'a> Deref for Wrap<'a> {
            type Target = &'a MetadataEx;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        unsafe impl Sync for Wrap<'_> {}

        fn sum_up_meta(pool: Arc<LinearObjectPool<[u64; DOMAIN_MODEL_ENTRY_MAX_SIZE]>>, meta: &[MetadataEx]) -> [u64; DOMAIN_MODEL_ENTRY_MAX_SIZE] {
            meta.iter().map(|v| Wrap(v)).collect_vec().into_par_iter().map(|value| {
                let mut ct = pool.pull_owned();
                for value in value.iter() {
                    match value {
                        MetadataWithOrigin::General(value)
                        | MetadataWithOrigin::Associated(_, value) => {
                            if let Some(reg) = value.registers() {
                                for (k, v) in reg.iter() {
                                    ct[k.as_index()] += v.get() as u64;
                                }
                            }
                            if let Some(dom) = value.domains() {
                                for (k, v) in dom.iter() {
                                    ct[k.as_index()] += v.get() as u64;
                                }
                            }
                        }
                    }
                }
                ct
            }).reduce(
                || pool.pull_owned(),
                |mut value, value2| {
                    value.iter_mut().zip_eq(value2.into_iter()).for_each(|(a, b)| {
                        *a += b;
                    });
                    value
                }
            ).clone()
        }

        if self.changed || self.domain_count.borrow().is_none() {
            let pool = Arc::new(LinearObjectPool::new(
                || [0u64; DOMAIN_MODEL_ENTRY_MAX_SIZE],
                |value| value.fill(0)
            ));
            let a = sum_up_meta(pool.clone(), &self.meta_a);
            let b = sum_up_meta(pool.clone(), &self.meta_b);
            self.domain_count.replace(Some((a, b)));
        }

        self.domain_count.borrow().clone().unwrap()
    }
}