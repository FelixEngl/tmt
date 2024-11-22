macro_rules! create_struct {
    ($($name: ident: $ty:ident => $method: ident: $r_typ: ty),* $(,)?) => {

        pub type DomainCounts = ([u64; $crate::topicmodel::dictionary::metadata::dict_meta_topic_matrix::DOMAIN_MODEL_ENTRY_MAX_SIZE], [u64; $crate::topicmodel::dictionary::metadata::dict_meta_topic_matrix::DOMAIN_MODEL_ENTRY_MAX_SIZE]);

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
            type FieldName = $crate::topicmodel::dictionary::metadata::containers::ex::MetaField;
            type FieldValue = $crate::topicmodel::dictionary::metadata::containers::ex::ResolvableValue;
            type BoundFieldValue = $crate::topicmodel::dictionary::metadata::containers::ex::GenericMetadataValue;
            type UpdateError = $crate::topicmodel::dictionary::metadata::containers::ex::WrongResolvedValueError<$crate::topicmodel::dictionary::metadata::containers::ex::ResolvedValue>;
            type Metadata = $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx;
            type ResolvedMetadata = $crate::topicmodel::dictionary::metadata::containers::ex::LoadedMetadataEx;
            type Reference<'a> = $crate::topicmodel::dictionary::metadata::containers::ex::MetadataRefEx<'a> where Self: 'a;
            type MutReference<'a> = $crate::topicmodel::dictionary::metadata::containers::ex::MetadataMutRefEx<'a> where Self: 'a;

            fn unprocessed_field() -> Option<Self::FieldName> {
                Some($crate::topicmodel::dictionary::metadata::containers::ex::MetaField::UnalteredVocabulary)
            }

            fn drop_field(&mut self, field: Self::FieldName) -> bool {
                let a = self.meta_a.iter_mut().map(|value| {
                    value.drop_field(field)
                }).any(|value| value);

                let b = self.meta_b.iter_mut().map(|value| {
                    value.drop_field(field)
                }).any(|value| value);

                let had_drop = a || b;
                if had_drop {
                   self.drop_if_unused();
                }
                had_drop
            }

            fn drop_all_fields(&mut self) -> bool {
                let a = self.meta_a.iter_mut().map(|value| value.drop_all_fields()).any(|value| value);
                let b = self.meta_b.iter_mut().map(|value| value.drop_all_fields()).any(|value| value);
                $(
                    if !self.$name.is_empty() {
                        self.$name = $ty::new();
                    }
                )*
                a || b
            }

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

            // fn resize(&mut self, meta_a: usize, meta_b: usize) {
            //     if meta_a > self.meta_a.len() {
            //         self.changed = true;
            //         self.meta_a.resize(meta_a, $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx::default());
            //     }
            //
            //     if meta_b > self.meta_a.len() {
            //         self.changed = true;
            //         self.meta_b.resize(meta_b, $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx::default());
            //     }
            // }



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
                self.optimize_impl()
            }

            #[inline(always)]
            fn convert_to_bound_value<T: Into<Self::FieldValue>>(&mut self, field: Self::FieldName, value: T) -> Result<Self::BoundFieldValue, (Self::FieldName, Self::FieldValue)> {
                self.convert_value(field, value)
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

// for value in $target.$field_name.iter() {
//     [<$field_name _value>].insert(
//           $intern_name.get_or_intern(
//               unsafe {
//                   $TSelf.$intern_name.resolve_unchecked(value)
//               }
//           )
//     );
// }
// $target.$field_name = [<$field_name _value>];

macro_rules! implement_filter_set {
    (interned: $TSelf:ident, $target:ident, $field_name: ident, $intern_name: ident; $($tt:tt)*) => {
        paste::paste! {
            if let Some(to_iter) = $target.[<$field_name _mut>]() {
                let mut result = $crate::topicmodel::dictionary::metadata::ex::metadata::MetadataContainerValueGeneric::new();
                for (value, count) in to_iter.iter() {
                    unsafe { result.insert_direct(
                        $intern_name.get_or_intern(
                            $TSelf.$intern_name.resolve_unchecked(value)
                        ),
                        count.get(),
                        true
                    ) }
                }
                unsafe{
                    to_iter.apply_new_as_update(result);
                }
            }
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


macro_rules! clean_routine_declaration_implementation {
    (interned: $field: ident, $t:ident; $($tt:tt)*) => {
        let mut $field: usize = 0;
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_declaration_implementation!($($tt)*);
    };
    (interned: $field: ident; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_declaration_implementation!($($tt)*);
    };
    ($marker:tt: ; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_declaration_implementation!($($tt)*);
    };
    ($(;)?) => {

    };
}
pub(super) use clean_routine_declaration_implementation;

macro_rules! clean_routine_counts_implementation {
    (interned: $field: ident; $($tt:tt)*) => {
        $field += 2;
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_counts_implementation!($($tt)*);
    };
    ($marker:tt: ; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_counts_implementation!($($tt)*);
    };
    ($(;)?) => {

    };
}
pub(super) use clean_routine_counts_implementation;

macro_rules! clean_routine_check_empty_implementation {
    (interned: $var: ident, $name: ident, $field_name: ident; $($tt:tt)*) => {
        paste::paste! {
            if $var.field_is_empty($crate::topicmodel::dictionary::metadata::ex::MetaField::[<$name:camel>]) {
                $field_name -= 1;
            }
        }
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_check_empty_implementation!($($tt)*);
    };
    ($marker:tt: $var: ident, $name: ident ; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_check_empty_implementation!($($tt)*);
    };
    ($(;)?) => {

    };
}
pub(super) use clean_routine_check_empty_implementation;


macro_rules! clean_routine_clear_empty_implementation {
    (interned: $TSelf:ident, $field_name: ident, $t:ident; $($tt:tt)*) => {
        if $field_name == 0 && !$TSelf.$field_name.is_empty() {
            $TSelf.$field_name = $t::new();
        }
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_clear_empty_implementation!($($tt)*);
    };
    (interned: $TSelf:ident, $field_name: ident ; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_clear_empty_implementation!($($tt)*);
    };
    ($marker:tt: $TSelf:ident; $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_clear_empty_implementation!($($tt)*);
    };
    ($(;)?) => {

    };
}
pub(super) use clean_routine_clear_empty_implementation;

macro_rules! general_conversion_right_impl {
    (interned: $TSelf: ident, $value: ident, $field_enum: ident, $target_intern: ident) => {
        {
            use $crate::topicmodel::dictionary::metadata::containers::ex::{GenericMetadataValue, MetaField, WrongResolvedValueError, ResolvableValue};
            Ok(
                GenericMetadataValue::$field_enum(
                    $value
                        .resolve_with_interner(&mut $TSelf.$target_intern)
                        .map_err(|value: WrongResolvedValueError<ResolvableValue>| (MetaField::$field_enum, value.1))?
                )
            )
        }
    };
    (set: $TSelf: ident, $value: ident, $field_enum: ident) => {
        {
            use $crate::topicmodel::dictionary::metadata::containers::ex::{GenericMetadataValue, MetaField, WrongResolvedValueError};
            Ok(
                GenericMetadataValue::$field_enum(
                    $value
                        .try_into()
                        .map_err(|value: WrongResolvedValueError<ResolvableValue>| (MetaField::$field_enum, value.1))?
                )
            )
        }
    };
    (voc: $TSelf: ident, $value: ident, $field_enum: ident) => {
        {
            use $crate::topicmodel::dictionary::metadata::containers::ex::{GenericMetadataValue, MetaField, WrongResolvedValueError};
            Ok(
                GenericMetadataValue::$field_enum(
                    $value
                        .try_into()
                        .map_err(|value: WrongResolvedValueError<ResolvableValue>| (MetaField::$field_enum, value.1))?
                )
            )
        }
    };
}

pub(super) use general_conversion_right_impl;


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

                for value in self.meta_b.iter_mut() {
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

            fn drop_if_unused(&mut self) {
                $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_declaration_implementation!(
                    $($marker: $($target_intern $(, $ty)?)?;)*
                );

                $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_counts_implementation!(
                    $($marker: $($target_intern)?;)*
                );

                for value in self.meta_a.iter().chain(self.meta_b.iter()) {
                    $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_check_empty_implementation!(
                        $($marker: value, $target_field $(, $target_intern)?;)*
                    );
                }

                $crate::topicmodel::dictionary::metadata::ex::manager::clean_routine_clear_empty_implementation!(
                    $($marker: self $(, $target_intern $(, $ty)?)?;)*
                );
            }

            fn convert_value<T: Into<<Self as $crate::topicmodel::dictionary::metadata::MetadataManager>::FieldValue>>(
                &mut self,
                field: <Self as $crate::topicmodel::dictionary::metadata::MetadataManager>::FieldName,
                value: T
            ) -> Result<<Self as $crate::topicmodel::dictionary::metadata::MetadataManager>::BoundFieldValue, (<Self as $crate::topicmodel::dictionary::metadata::MetadataManager>::FieldName, $crate::topicmodel::dictionary::metadata::ex::ResolvableValue)> {
                use $crate::topicmodel::dictionary::metadata::ex::MetaField;
                let value: <Self as $crate::topicmodel::dictionary::metadata::MetadataManager>::FieldValue = value.into();
                paste::paste! {
                    match field {
                        $(
                            MetaField::[<$target_field:camel>] => {
                                $crate::topicmodel::dictionary::metadata::ex::manager::general_conversion_right_impl!(
                                    $marker: self, value, [<$target_field:camel>] $(, $target_intern)?
                                )
                            }
                        )*
                    }
                }
            }
        }
    };
}

pub(super) use update_routine;



use crate::topicmodel::dictionary::direction::{A, B};
use crate::topicmodel::dictionary::metadata::MetadataManager;
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

    pub(super) fn insert_into_a_impl(&mut self, word_id: usize, dictionary: Option<impl AsRef<str>>, value: <Self as MetadataManager>::BoundFieldValue) -> bool {
        let dictionary = if let Some(dictionary) = dictionary {
            Some(self.dictionary_interner.get_or_intern(dictionary))
        } else {
            None
        };
        self.meta_a.get_mut(word_id).map_or(false, |meta| {
            match dictionary {
                None => {
                    meta.get_mut_or_init_general_metadata().add_single_generic(
                        value
                    );
                }
                Some(resolved_value) => {
                    meta.get_or_create(resolved_value).add_single_generic(
                        value
                    );
                }
            }
            true
        })
    }

    pub(super) fn insert_into_b_impl(&mut self, word_id: usize, dictionary: Option<impl AsRef<str>>, value: <Self as MetadataManager>::BoundFieldValue) -> bool {
        let dictionary = if let Some(dictionary) = dictionary {
            Some(self.dictionary_interner.get_or_intern(dictionary))
        } else {
            None
        };
        self.meta_b.get_mut(word_id).map_or(false, |meta| {
            match dictionary {
                None => {
                    meta.get_mut_or_init_general_metadata().add_single_generic(
                        value
                    );
                }
                Some(resolved_value) => {
                    meta.get_or_create(resolved_value).add_single_generic(
                        value
                    );
                }
            }
            true
        })
    }
}
