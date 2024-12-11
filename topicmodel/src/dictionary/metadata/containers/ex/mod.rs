mod metadata;
mod reference;
mod reference_mut;
mod solved;
mod manager;
mod collector;
mod field_denom;
mod resolved_value;
mod solved_new_arg;
mod typedef;

use std::collections::{HashMap, HashSet};
use std::fmt::{Display};
pub use metadata::*;
use reference::*;
pub use reference_mut::*;
pub use resolved_value::*;
pub use solved_new_arg::*;

use crate::interners::*;

use tinyset::Set64;
use crate::dictionary::word_infos::*;
use crate::dictionary::metadata::dict_meta_topic_matrix::{DictionaryMetaIndex, DictMetaTagIndex, META_DICT_ARRAY_LENTH};
use std::ops::{Deref, DerefMut};
use pretty::*;
use pyo3::{pyclass, pymethods};
use ldatranslate_toolkit::register_python;
use crate::dictionary::metadata::MetadataManager;

register_python! {
    struct LoadedMetadataEx;
    struct MetaField;
    struct DictMetaCount;
    struct DictMetaCounts;
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
        $crate::dictionary::metadata::ex::typedef::make_storage_type_def!(
            $($tt: $name: $assoc_typ;)+
        );

        paste::paste! {
            $crate::dictionary::metadata::ex::reference::create_ref_implementation!(
                $($tt: $name $(, $interner_name)?: [<$name:camel StoreType>]<'a>,)+
            );
        }

        $crate::dictionary::metadata::ex::reference_mut::create_mut_ref_implementation!(
            $($tt: $name $(, $interner_name, $interner_method)?: $assoc_typ,)+
        );
        $crate::dictionary::metadata::ex::manager::create_managed_implementation!(
            $($($($interner_name: $interner_type => $interner_method: $assoc_typ,)?)?)+
        );
        $crate::dictionary::metadata::ex::metadata::create_metadata_impl!(
            $($tt: $doc $name: $assoc_typ,)+
        );
        $crate::dictionary::metadata::ex::solved::create_solved_implementation!(
            $($tt: $name $lit_name,)+
        );
        $crate::dictionary::metadata::ex::collector::create_collector_implementation!(
            $($tt: $name: $assoc_typ,)+
        );

        $crate::dictionary::metadata::ex::field_denom::generate_field_denoms! {
            $($doc $name($lit_name),)+
        }

        $crate::dictionary::metadata::ex::manager::update_routine!(
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


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct DictMetaCount {
    counts: [u64; META_DICT_ARRAY_LENTH]
}

impl DictMetaCount {

    const EMPTY: DictMetaCount = DictMetaCount {
        counts: [0; META_DICT_ARRAY_LENTH]
    };

    pub const fn new(counts: [u64; META_DICT_ARRAY_LENTH]) -> Self {
        Self {counts}
    }

    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub fn into_inner(self) -> [u64; META_DICT_ARRAY_LENTH] {
        self.counts
    }

    pub fn get(&self, index: impl Into<DictMetaTagIndex>) -> u64 {
        self.counts[index.into().as_index()]
    }

    pub fn get_mut(&mut self, index: impl Into<DictMetaTagIndex>) -> &mut u64 {
        &mut self.counts[index.into().as_index()]
    }

    pub fn from_meta(meta: &LoadedMetadataEx) -> Self {
        let mut new = Self::empty();
        {
            let SolvedMetadataField(a, b) = meta.domains();
            if let Some(a) = a {
                for x in a {
                    let (domain, count): (Domain, u32) = x.clone().try_into().unwrap();
                    *new.get_mut(domain) += count as u64;
                }
            }
            if let Some(b) = b {
                for x in b.values().flat_map(|value| value.iter().cloned()) {
                    let (domain, count): (Domain, u32) = x.clone().try_into().unwrap();
                    *new.get_mut(domain) += count as u64;
                }
            }
        }
        {
            let SolvedMetadataField(a, b) = meta.registers();
            if let Some(a) = a {
                for x in a {
                    let (register, count): (Register, u32) = x.clone().try_into().unwrap();
                    *new.get_mut(register) += count as u64;
                }
            }
            if let Some(b) = b {
                for x in b.values().flat_map(|value| value.iter().cloned()) {
                    let (register, count): (Register, u32) = x.clone().try_into().unwrap();
                    *new.get_mut(register) += count as u64;
                }
            }
        }
        new
    }


}

impl AsRef<[u64; META_DICT_ARRAY_LENTH]> for DictMetaCount {
    fn as_ref(&self) -> &[u64; META_DICT_ARRAY_LENTH] {
        &self.counts
    }
}

impl Deref for DictMetaCount {
    type Target = [u64; META_DICT_ARRAY_LENTH];
    fn deref(&self) -> &Self::Target {
        &self.counts
    }
}

impl DerefMut for DictMetaCount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.counts
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl DictMetaCount {
    fn full(&self) -> HashMap<DictMetaTagIndex, u64> {
        self.counts.iter().enumerate().map(|(i, value)| {
            (DictMetaTagIndex::from_index(i).unwrap(), *value)
        }).collect()
    }

    fn exists(&self) -> HashSet<DictMetaTagIndex> {
        self.counts.iter().enumerate().filter_map(|(i, value)| {
            (*value > 0).then(|| DictMetaTagIndex::from_index(i).unwrap())
        }).collect()
    }

    fn __len__(&self) -> usize {
        META_DICT_ARRAY_LENTH
    }

    fn __getitem__(&self, index: DictMetaTagIndex) -> u64 {
        let idx = index.as_index();
        self.counts[idx]
    }

    pub fn sum(&self) -> u64 {
        self.counts.iter().sum()
    }


    fn __str__(&self) -> String {
        self.to_string()
    }

    /// Converts the count to a normal dict
    fn as_dict(&self) -> HashMap<DictMetaTagIndex, u64> {
        self.counts.iter().copied().enumerate().map(|(i, value)| {
            (
                DictMetaTagIndex::from_index(i).unwrap(),
                value
            )
        }).collect()
    }
}

impl<'a, 'b, D, A> Pretty<'a, D, A> for &'b DictMetaCount
where
    A: 'a + Clone,
    D: DocAllocator<'a, A>,
    D::Doc: Clone,
{
    fn pretty(self, alloc: &'a D) -> DocBuilder<'a, D, A> {
        alloc.intersperse(
            self.counts.iter().enumerate().map(|(id, ct)| {
                alloc.text(format!("{}: {}", DictMetaTagIndex::from_index(id).unwrap(), ct))
            }),
            alloc.text(",").append(alloc.hardline())
        )
    }
}

impl Display for DictMetaCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        RcDoc::<()>::nil().append(self).render_fmt(80, f)
    }
}


#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Copy, Clone)]
pub struct DictMetaCounts {
    counts_a: DictMetaCount,
    counts_b: DictMetaCount,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl DictMetaCounts {
    pub fn a(&self) -> DictMetaCount {
        self.counts_a
    }

    pub fn b(&self) -> DictMetaCount {
        self.counts_b
    }

    fn __len__(&self) -> usize {
        META_DICT_ARRAY_LENTH
    }

    fn __getitem__(&self, index: DictMetaTagIndex) -> (u64, u64) {
        let idx = index.as_index();
        (self.counts_a.counts[idx], self.counts_b.counts[idx])
    }

    fn sum(&self) -> (u64, u64) {
        (self.counts_a.counts.iter().sum(), self.counts_b.counts.iter().sum())
    }


    fn __str__(&self) -> String {
        self.to_string()
    }

    /// Returns a tuple of dicts, representing (a, b) as hashmaps.
    fn as_dicts(&self) -> (HashMap<DictMetaTagIndex, u64>, HashMap<DictMetaTagIndex, u64>) {
        (
            self.counts_a.as_dict(),
            self.counts_b.as_dict()
        )
    }
}

impl DictMetaCounts {
    pub fn new(counts_a: [u64; META_DICT_ARRAY_LENTH], counts_b: [u64; META_DICT_ARRAY_LENTH]) -> Self {
        Self { counts_a: DictMetaCount::new(counts_a), counts_b: DictMetaCount::new(counts_b) }
    }

    pub fn ref_a(&self) -> &DictMetaCount {
        &self.counts_a
    }

    pub fn ref_b(&self) -> &DictMetaCount {
        &self.counts_b
    }
}

impl Display for DictMetaCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        RcDoc::<()>::nil().append(self).render_fmt(80, f)
    }
}



impl<'a, 'b, D, A> Pretty<'a, D, A> for &'b DictMetaCounts
where
    A: 'a + Clone,
    D: DocAllocator<'a, A>,
    D::Doc: Clone,
{
    fn pretty(self, alloc: &'a D) -> DocBuilder<'a, D, A> {
        let (a, b) = self.sum();
        alloc.text("DomainCounts: ")
            .append(
                alloc.hardline().append(
                    alloc.hardline()
                        .append(
                            alloc
                                .text(format!("a ({a}): "))
                                .append(alloc.hardline()
                                    .append(
                                        self.counts_a.pretty(alloc).indent(2)
                                    ).append(alloc.hardline()).brackets()
                                )
                                .indent(2)
                        )
                        .append(alloc.hardline())
                        .append(
                            alloc
                                .text(format!("b ({b}): "))
                                .append(alloc.hardline()
                                    .append(
                                        self.counts_b.pretty(alloc).indent(2)
                                    ).append(alloc.hardline()).brackets()
                                )
                                .indent(2)
                        )
                        .append(alloc.hardline())
                        .brackets()
                ).braces()
            )

    }
}



impl MetadataManagerEx {

    /// Returns the domain counts for A and B
    pub fn dict_meta_counts(&self) -> DictMetaCounts {
        use std::sync::Arc;
        use itertools::Itertools;
        use lockfree_object_pool::LinearObjectPool;
        use rayon::prelude::*;
        use super::super::dict_meta_topic_matrix::META_DICT_ARRAY_LENTH;

        #[repr(transparent)]
        struct Wrap<'a>(&'a MetadataEx);
        impl<'a> Deref for Wrap<'a> {
            type Target = &'a MetadataEx;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        unsafe impl Sync for Wrap<'_> {}


        fn sum_up_meta(pool: Arc<LinearObjectPool<[u64; META_DICT_ARRAY_LENTH]>>, meta: &[MetadataEx]) -> [u64; META_DICT_ARRAY_LENTH] {
            let result = meta.iter().map(|v| Wrap(v)).collect_vec().into_par_iter().map(|value| {
                let mut ct_full = pool.pull_owned();
                for value in value.iter() {
                    match value {
                        MetadataWithOrigin::General(value)
                        | MetadataWithOrigin::Associated(_, value) => {
                            if let Some(reg) = value.registers() {
                                for (k, v) in reg.iter() {
                                    ct_full[k.as_index()] += v.get() as u64;
                                }
                            }
                            if let Some(dom) = value.domains() {
                                for (k, v) in dom.iter() {
                                    ct_full[k.as_index()] += v.get() as u64;
                                }
                            }
                        }
                    }
                }
                ct_full
            }).reduce(
                || pool.pull_owned(),
                |mut value, value2| {
                    value.iter_mut().zip_eq(value2.into_iter()).for_each(|(a, b)| {
                        *a += b;
                    });
                    value
                }
            );
            result.clone()
        }

        let pool = Arc::new(LinearObjectPool::new(
            || [0u64; META_DICT_ARRAY_LENTH],
            |value| value.fill(0)
        ));

        let a = sum_up_meta(pool.clone(), &self.meta_a());
        let b = sum_up_meta(pool.clone(), &self.meta_b());
        DictMetaCounts::new(a, b)
    }
}