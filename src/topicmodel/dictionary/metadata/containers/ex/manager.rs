macro_rules! create_struct {
    ($($name: ident: $ty:ident => $method: ident: $r_typ: ty),* $(,)?) => {

        pub type DomainCounts = ([u64; $crate::topicmodel::dictionary::metadata::domain_matrix::DOMAIN_MODEL_ENTRY_MAX_SIZE], [u64; $crate::topicmodel::dictionary::metadata::domain_matrix::DOMAIN_MODEL_ENTRY_MAX_SIZE]);

        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        pub struct MetadataManagerEx {
            meta_a: Vec<$crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx>,
            meta_b: Vec<$crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx>,
            pub(in crate::topicmodel::dictionary) dictionary_interner: $crate::toolkit::typesafe_interner::DictionaryOriginStringInterner,
            #[serde(default, skip)]
            changed: bool,
            #[serde(default, skip)]
            domain_count: std::cell::RefCell<Option<DomainCounts>>,
            $(pub(in crate::topicmodel::dictionary) $name: $ty,
            )*
        }

        impl std::fmt::Debug for MetadataManagerEx {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use itertools::Itertools;
                f.debug_struct(stringify!(MetadataManagerEx))
                .field(
                    "meta_a",
                    &format!("(len: {})", self.meta_a.len())
                )
                .field(
                    "meta_b",
                    &format!("(len: {})", self.meta_b.len())
                )
                .field(
                    "dictionary_interner",
                    &format!(
                        "(len: {}, contents: [\"{}\"])",
                        self.dictionary_interner.len(),
                        self.dictionary_interner.iter().map(|v| v.1).join("\", \"")
                    )
                )
                $(
                .field(
                    stringify!($name),
                    &format!(
                        "(len: {})",
                        self.$name.len()
                    )
                )
                )+
                .finish()
            }
        }


        impl Default for MetadataManagerEx {
            fn default() -> Self {
                Self {
                    meta_a: Vec::new(),
                    meta_b: Vec::new(),
                    dictionary_interner: $crate::toolkit::typesafe_interner::DictionaryOriginStringInterner::new(),
                    changed: false,
                    domain_count: Default::default(),
                    $($name: $ty::new(),
                    )*
                }
            }
        }

        impl $crate::topicmodel::dictionary::metadata::MetadataManager for MetadataManagerEx {
            type UpdateError = $crate::topicmodel::dictionary::metadata::containers::ex::WrongResolvedValueError;
            type Metadata = $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx;
            type ResolvedMetadata = $crate::topicmodel::dictionary::metadata::containers::ex::LoadedMetadataEx;
            type Reference<'a> = $crate::topicmodel::dictionary::metadata::containers::ex::MetadataRefEx<'a> where Self: 'a;
            type MutReference<'a> = $crate::topicmodel::dictionary::metadata::containers::ex::MetadataMutRefEx<'a> where Self: 'a;

            fn meta_a(&self) -> &[Self::Metadata] {
                self.meta_a.as_slice()
            }

            fn meta_b(&self) -> &[Self::Metadata] {
                self.meta_b.as_slice()
            }

            fn switch_languages(self) -> Self {
                Self {
                    meta_a: self.meta_b,
                    meta_b: self.meta_a,
                    dictionary_interner: self.dictionary_interner,
                    changed: false,
                    domain_count: Default::default(),
                    $($name: self.$name,
                    )*
                }
            }

            fn get_meta_mut_for<'a>(
                &'a mut self,
                lang: $crate::topicmodel::dictionary::direction::LanguageKind,
                vocabulary: &'a mut dyn $crate::topicmodel::vocabulary::AnonymousVocabularyMut,
                word_id: usize
            ) -> Option<Self::MutReference<'a>> {
                let ptr = self as *mut Self;
                let value = unsafe{&mut*ptr};
                let result = if lang.is_a() {
                    value.meta_a.get_mut(word_id)
                } else {
                    value.meta_b.get_mut(word_id)
                }?;
                self.changed = true;
                let vocabulary = unsafe {
                    std::mem::transmute::<_, &'static mut dyn $crate::topicmodel::vocabulary::AnonymousVocabularyMut>(vocabulary)
                } as *mut dyn $crate::topicmodel::vocabulary::AnonymousVocabularyMut;
                Some($crate::topicmodel::dictionary::metadata::containers::ex::MetadataMutRefEx::new(
                    vocabulary,
                    ptr,
                    result
                ))
            }

            fn get_or_create_meta_for<'a>(
                &'a mut self,
                lang: $crate::topicmodel::dictionary::direction::LanguageKind,
                vocabulary: &'a mut dyn $crate::topicmodel::vocabulary::AnonymousVocabularyMut,
                word_id: usize
            ) -> Self::MutReference<'a> {
                let ptr = self as *mut Self;

                let targ = if lang.is_a() {
                    &mut self.meta_a
                } else {
                    &mut self.meta_b
                };

                if word_id >= targ.len() {
                    targ.resize_with(
                        word_id + 1,
                        $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx::default
                    );
                }

                let vocabulary = unsafe {
                    std::mem::transmute::<_, &'static mut dyn $crate::topicmodel::vocabulary::AnonymousVocabularyMut>(vocabulary)
                } as *mut dyn $crate::topicmodel::vocabulary::AnonymousVocabularyMut;

                self.changed = true;
                unsafe{
                    MetadataMutRefEx::new(
                        vocabulary,
                        ptr,
                        targ.get_unchecked_mut(word_id)
                    )
                }
            }

            fn get_meta_ref_for<'a>(
                &'a self,
                lang: $crate::topicmodel::dictionary::direction::LanguageKind,
                vocabulary: $crate::topicmodel::vocabulary::AnonymousVocabularyRef<'a>,
                word_id: usize
            ) -> Option<Self::Reference<'a>> {
                Some(MetadataRefEx::new(self.get_meta_for(lang, word_id)?, self, vocabulary))
            }

            fn resize(&mut self, meta_a: usize, meta_b: usize) {
                if meta_a > self.meta_a.len() {
                    self.changed = true;
                    self.meta_a.resize(meta_a, $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx::default());
                }

                if meta_b > self.meta_a.len() {
                    self.changed = true;
                    self.meta_b.resize(meta_b, $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx::default());
                }
            }

            fn copy_keep_vocabulary(&self) -> Self {
                Self {
                    meta_a: Default::default(),
                    meta_b: Default::default(),
                    dictionary_interner: self.dictionary_interner.clone(),
                    changed: false,
                    domain_count: Default::default(),
                    $($name: self.$name.clone(),
                    )*
                }
            }

            fn dictionaries(&self) -> Vec<&str> {
                self.dictionary_interner.iter().map(|value| value.1).collect()
            }

            fn update_ids(
                &mut self,
                update: &$crate::topicmodel::dictionary::metadata::update::WordIdUpdate
            ){
                self.update_ids_impl(update)
            }

            fn optimize(&mut self) {

            }
        }
    };
}

pub(super) use create_struct;

macro_rules! create_manager_interns {
    ($($name: ident => $method: ident: $r_typ: ty),+ $(,)?) => {
        impl MetadataManagerEx {
            $(
                paste::paste! {
                    pub fn [<$method _static>](&mut self, voc_entry: &'static str) -> $r_typ {
                        self.$name.get_or_intern_static(voc_entry)
                    }
                }
                pub fn $method(&mut self, voc_entry: impl AsRef<str>) -> $r_typ {
                    self.$name.get_or_intern(voc_entry)
                }
            )*
        }
    }
}

pub(super) use create_manager_interns;

macro_rules! create_managed_implementation {
    ($($name: ident $(: $ty:ident)? => $method: ident: $r_typ: ty),+ $(,)?) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::create_struct!(
            $($($name: $ty => $method: $r_typ,)?)+
        );
        $crate::topicmodel::dictionary::metadata::ex::manager::create_manager_interns!(
            $($name => $method: $r_typ,)+
        );
    }
}

pub(super) use create_managed_implementation;

macro_rules! update_routine_declaration_implementation {
    (interned: $field: ident, $t:ident; $($tt:tt)*) => {
        let mut $field: $t = $t::new();
        $crate::topicmodel::dictionary::metadata::ex::manager::update_routine_declaration_implementation!($($tt)*);
    };
    (interned: $field: ident; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::update_routine_declaration_implementation!($($tt)*);
    };
    ($marker:tt: ; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::update_routine_declaration_implementation!($($tt)*);
    };
    ($(;)?) => {

    };
}
pub(super) use update_routine_declaration_implementation;

macro_rules! update_routine_set_implementation {
    (interned: $TSelf:ident, $intern_name: ident, $t:ident; $($tt:tt)*) => {
        $TSelf.$intern_name = $intern_name;
        $crate::topicmodel::dictionary::metadata::ex::manager::update_routine_set_implementation!($($tt)*);
    };
    (interned: $TSelf:ident, $intern_name: ident; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::update_routine_set_implementation!($($tt)*);
    };
    ($marker:tt: $TSelf:ident; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::update_routine_set_implementation!($($tt)*);
    };
    ($(;)?) => {

    };
}
pub(super) use update_routine_set_implementation;

macro_rules! implement_filter_set {
    (interned: $TSelf:ident, $target:ident, $field_name: ident, $intern_name: ident; $($tt:tt)*) => {
        paste::paste! {
            let mut [<$field_name _value>] = tinyset::Set64::new();
            for value in $target.$field_name.iter() {
                [<$field_name _value>].insert(
                      $intern_name.get_or_intern(
                          unsafe {
                              $TSelf.$intern_name.resolve_unchecked(value)
                          }
                      )
                );
            }
            $target.$field_name = [<$field_name _value>];
        }
        $crate::topicmodel::dictionary::metadata::ex::manager::implement_filter_set!(
            $($tt)*
        );
    };

    ($marker:tt : $TSelf:ident, $target:ident, $field_name: ident, $intern_name: ident; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::implement_filter_set!(
            $($tt)*
        );
    };

    ($marker:tt: $TSelf:ident, $target:ident, $field_name: ident; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::implement_filter_set!($($tt)*);
    };
    ($(;)?) => {

    };
}
pub(super) use implement_filter_set;

macro_rules! update_routine {
    ($($marker:tt: $target_field: ident $(, $target_intern: ident $(, $ty: ident)?)?;)*) => {
        impl MetadataManagerEx {
            fn optimize_impl(&mut self) {
                $crate::topicmodel::dictionary::metadata::ex::manager::update_routine_declaration_implementation!(
                    $($marker: $($target_intern $(, $ty)?)?;)*
                );
                for value in self.meta_a.iter_mut() {
                    for value in value.iter_mut() {
                        let metadata = value.to_metadata();
                        if let Some(targ) = metadata.get_mut() {
                            $crate::topicmodel::dictionary::metadata::ex::manager::implement_filter_set!(
                                $($marker: self, targ, $target_field $(, $target_intern)?;)*
                            );
                        }
                    }
                }
                $crate::topicmodel::dictionary::metadata::ex::manager::update_routine_set_implementation!(
                    $($marker: self $(, $target_intern $(, $ty)?)?;)*
                );
            }
        }
    };
}

pub(super) use update_routine;


use crate::topicmodel::dictionary::direction::{A, B};
use super::*;



impl MetadataManagerEx {
    pub fn intern_dictionary_origin_static(&mut self, dict_origin: &'static str) -> DictionaryOriginSymbol {
        self.dictionary_interner.get_or_intern_static(dict_origin)
    }

    pub fn intern_dictionary_origin(&mut self, dict_origin: impl AsRef<str>) -> DictionaryOriginSymbol {
        self.dictionary_interner.get_or_intern(dict_origin)
    }

    pub(in super) fn update_ids_impl(
        &mut self,
        update: &crate::topicmodel::dictionary::metadata::update::WordIdUpdate
    ){
        for value in self.meta_a.iter_mut() {
            for value in value.iter_mut() {
                match value {
                    MetadataWithOrigin::General(assoc)
                    | MetadataWithOrigin::Associated(_, assoc) => {
                        if let Some(targ) = assoc.get_mut() {
                            targ.update_ids::<A>(update)
                        }
                    }
                }
            }
        }
        for value in self.meta_b.iter_mut() {
            for value in value.iter_mut() {
                match value {
                    MetadataWithOrigin::General(assoc)
                    | MetadataWithOrigin::Associated(_, assoc) => {
                        if let Some(targ) = assoc.get_mut() {
                            targ.update_ids::<B>(update)
                        }
                    }
                }
            }
        }
    }

    pub fn drop_original_words(&mut self) {
        let mut original_entry_interner: OriginalEntryStringInterner = OriginalEntryStringInterner::new();
        for value in self.meta_a.iter_mut() {
            for value in value.iter_mut() {
                let metadata = value.to_metadata();
                if let Some(targ) = metadata.get_mut() {
                    let mut original_entry_value = Set64::new();
                    for value in targ.original_entry.iter() {
                        original_entry_value.insert(
                            original_entry_interner.get_or_intern(
                                unsafe {
                                    self.original_entry_interner.resolve_unchecked(value)
                                }
                            )
                        );
                    }
                    targ.original_entry = original_entry_value;
                }
            }
        }
        self.original_entry_interner = original_entry_interner;
    }
}
