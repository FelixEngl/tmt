
macro_rules! implement_update {
    ($targ: ident, $update: ident, $L: ident => voc: $name: ident, $($tt:tt)*) => {
        {
            if let Some(value) = $targ.synonyms_mut() {
                let update = $update.create_update::<$L, _, _>(value.iter_counts());
                unsafe {
                    value.apply_iterable_as_update(update.into_iter(), true)
                }
            }
        }
        $crate::topicmodel::dictionary::metadata::ex::metadata::implement_update!($targ, $update, $L => $($tt)*);
    };
    ($targ: ident, $update: ident, $L: ident => $marker:tt: $name: ident, $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::metadata::implement_update!($targ, $update, $L => $($tt)*);
    };
    ($targ: ident, $update: ident, $L: ident => $(,)?) => {}
}


pub(super) use implement_update;

macro_rules! implement_id_collection {
    ($targ: ident, $collector: ident => voc: $name: ident, $($tt:tt)*) => {
        $collector.extend($targ.$name().map(|value| value.iter_keys()).into_iter().flatten());
        $crate::topicmodel::dictionary::metadata::ex::metadata::implement_id_collection!($targ, $collector => $($tt)*);
    };
    ($targ: ident, $collector: ident => $marker:tt: $name: ident, $($tt:tt)*) => {
        $crate::topicmodel::dictionary::metadata::ex::metadata::implement_id_collection!($targ, $collector => $($tt)*);
    };
    ($targ: ident, $collector: ident => $(,)?) => {}
}

pub(super) use implement_id_collection;

macro_rules! impl_associated_metadata {

    ($($tt: tt: $($doc: literal)? $name: ident: $typ: ty),+ $(,)?) => {


        paste::paste! {
            $crate::topicmodel::dictionary::metadata::ex::metadata::impl_general_metadata!(
                $($name, [<$name:camel>], $typ;)+
            );
            $crate::topicmodel::dictionary::metadata::ex::metadata::impl_keys!(
                $($name, [<$name:camel>], $typ;)+
            );
        }


        impl AssociatedMetadata {
            $(
            /// Get the values of the field
            #[inline(always)]
            pub fn $name(&self) -> Option<&$crate::topicmodel::dictionary::metadata::containers::ex::MetadataContainerValueGeneric<$typ>> {
                self.inner.get()?.$name()
            }

            paste::paste! {
                /// Get the values of the field
                #[inline(always)]
                pub fn [<$name _mut>](&mut self) -> Option<&mut $crate::topicmodel::dictionary::metadata::containers::ex::MetadataContainerValueGeneric<$typ>> {
                    self.inner.get_mut()?.[<$name _mut>]()
                }

                /// Get the values of the field
                #[inline(always)]
                pub fn [<get_or_init_ $name>](&mut self) -> &mut $crate::topicmodel::dictionary::metadata::containers::ex::MetadataContainerValueGeneric<$typ> {
                    self.get_mut_or_init().[<get_or_init_ $name>]()
                }

                /// Adds a single value to the specified field
                #[inline(always)]
                pub fn [<add_single_to_ $name>](&mut self, value: $typ) {
                    self.get_mut_or_init().[<add_single_to_ $name>](value);
                }

                /// Adds all values to the specified field
                #[inline(always)]
                pub fn [<add_all_to_ $name>]<I: IntoIterator<Item=$typ>>(&mut self, values: I) {
                    self.get_mut_or_init().[<add_all_to_ $name>](values);
                }

                /// Adds a single value to the specified field
                #[inline(always)]
                pub fn [<write_single_to_ $name>](&mut self, value: $typ, count: u32, add_only_associated_count: bool) {
                    self.get_mut_or_init().[<write_single_to_ $name>](value, count, add_only_associated_count);
                }

                /// Adds all values to the specified field
                #[inline(always)]
                pub fn [<write_all_to_ $name>]<I: IntoIterator<Item=($typ, u32)>>(&mut self, values: I, add_only_associated_count: bool) {
                    self.get_mut_or_init().[<write_all_to_ $name>](values, add_only_associated_count);
                }

            }
            )+

            #[inline(always)]
            pub fn add_single_generic(&mut self, value: $crate::topicmodel::dictionary::metadata::containers::ex::GenericMetadataValue) {
                self.get_mut_or_init().add_single_generic(value);
            }
        }




        impl AssociatedMetadataImpl {

            $(
            pub fn $name(&self) -> Option<&$crate::topicmodel::dictionary::metadata::containers::ex::MetadataContainerValueGeneric<$typ>> {
                use $crate::topicmodel::dictionary::metadata::containers::ex::MetaField;
                Some(
                    unsafe {
                        paste::paste! {
                            self.get(MetaField::[<$name:camel>])?.[<as_ref_unchecked_ $name>]()
                        }
                    }
                )
            }



            paste::paste! {
                pub fn [<$name _mut>](&mut self) -> Option<&mut $crate::topicmodel::dictionary::metadata::containers::ex::MetadataContainerValueGeneric<$typ>> {
                    use $crate::topicmodel::dictionary::metadata::containers::ex::MetaField;
                    Some(
                        unsafe {
                            paste::paste! {
                                self.get_mut(MetaField::[<$name:camel>])?.[<as_mut_unchecked_ $name>]()
                            }
                        }
                    )
                }

                pub fn [<get_or_init_ $name>](&mut self) -> &mut $crate::topicmodel::dictionary::metadata::containers::ex::MetadataContainerValueGeneric<$typ> {
                    use $crate::topicmodel::dictionary::metadata::containers::ex::MetaField;
                    unsafe {
                        paste::paste! {
                            self.get_or_insert(MetaField::[<$name:camel>]).[<as_mut_unchecked_ $name>]()
                        }
                    }
                }

                pub fn [<add_single_to_ $name>](&mut self, value: $typ) {
                    unsafe {
                        self
                            .get_or_insert(MetaField::[<$name:camel>])
                            .[<as_mut_unchecked_ $name>]()
                            .insert(value);
                    }
                }
                pub fn [<add_all_to_ $name>]<I: IntoIterator<Item=$typ>>(&mut self, values: I) {
                    unsafe {
                        self
                            .get_or_insert(MetaField::[<$name:camel>])
                            .[<as_mut_unchecked_ $name>]()
                            .extend(values);
                    }
                }

                pub fn [<write_single_to_ $name>](&mut self, value: $typ, count: u32, add_only_associated_count: bool) {
                    unsafe {
                        self
                            .get_or_insert(MetaField::[<$name:camel>])
                            .[<as_mut_unchecked_ $name>]()
                            .insert_direct(value, count, add_only_associated_count);
                    }
                }
                pub fn [<write_all_to_ $name>]<I: IntoIterator<Item=($typ, u32)>>(&mut self, values: I, add_only_associated_count: bool) {
                    unsafe {
                        let targ = self
                            .get_or_insert(MetaField::[<$name:camel>])
                            .[<as_mut_unchecked_ $name>]();
                        for (k, v) in values {
                            targ.insert_direct(k, v, add_only_associated_count);
                        }
                    }
                }
            }
            )+

            pub fn add_single_generic(&mut self, value: $crate::topicmodel::dictionary::metadata::containers::ex::GenericMetadataValue) {
                use $crate::topicmodel::dictionary::metadata::containers::ex::GenericMetadataValue;
                paste::paste! {
                    match value {
                        $(GenericMetadataValue::[<$name:camel>](value) => self.[<add_single_to_ $name>](value),
                        )+
                    }
                }
            }

            pub fn update_ids<L: $crate::topicmodel::dictionary::direction::Language>(
                &mut self,
                update: &$crate::topicmodel::dictionary::metadata::update::WordIdUpdate
            ) {
                $crate::topicmodel::dictionary::metadata::ex::metadata::implement_update!(
                    self, update, L => $($tt: $name,)*
                );
            }

            pub fn collect_all_known_ids(&self) -> tinyset::Set64<usize> {
                let mut collector = tinyset::Set64::new();
                $crate::topicmodel::dictionary::metadata::ex::metadata::implement_id_collection!(
                    self, collector => $($tt: $name,)*
                );
                collector
            }

        }

        impl MetadataEx {
            $(
                paste::paste! {
                    /// Get all values of a specific field.
                    pub fn [<all_raw_ $name>]<'a>(&'a self) -> (Option<&'a MetadataContainerValueGeneric<$typ>>, Vec<Option<&'a MetadataContainerValueGeneric<$typ>>>) {
                         let a = self.general_metadata.get().and_then(|value| value.$name());
                         let b = self.associated_metadata.iter().map(|value| value.get().and_then(|value| value.$name())).collect();
                         (a, b)
                    }
                }
            )+

        }
    };
}

pub(super) use impl_associated_metadata;


macro_rules! impl_keys {
    (
        $($normal_name:ident, $enum_var_name: ident, $typ: ty);+ $(;)?
    ) => {
        /// Allows to store a count in association to a value
        #[derive(Clone, Debug)]
        pub enum GenericMetadataValue {
            $($enum_var_name($typ),
            )+
        }

        impl GenericMetadataValue {
            $(
            #[inline(always)]
            pub fn $normal_name(value: $typ) -> GenericMetadataValue {
                GenericMetadataValue::$enum_var_name(value)
            }
            )+

            pub fn associated_key(&self) -> $crate::topicmodel::dictionary::metadata::containers::ex::MetaField {
                match self {
                    $(
                    Self::$enum_var_name(_) => $crate::topicmodel::dictionary::metadata::containers::ex::MetaField::$enum_var_name,
                    )+
                }
            }
        }


        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub enum MetadataContainerValue {
            $($enum_var_name($crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$typ>),
            )+
        }

        impl std::fmt::Display for MetadataContainerValue {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                    Self::$enum_var_name(value) => value.fmt(f),
                    )+
                }
            }
        }


        const _: () = {
            use $crate::topicmodel::dictionary::metadata::containers::ex::MetaField;
            use $crate::topicmodel::dictionary::metadata::containers::ex::MetadataContainerValueGeneric;
            impl MetadataContainerValue {

                pub fn create_for_key(key: MetaField) -> Self {
                    match key {
                        $(
                        MetaField::$enum_var_name => Self::$enum_var_name(MetadataContainerValueGeneric::new()),
                        )+
                    }
                }

                pub fn is_empty(&self) -> bool {
                    match self {
                        $(
                        MetadataContainerValue::$enum_var_name(slf) => slf.is_empty(),
                        )+
                    }
                }

                pub fn update(&mut self, other: &MetadataContainerValue, add_only_associated_count: bool) {
                    match (self, other) {
                        $(
                        (MetadataContainerValue::$enum_var_name(slf), MetadataContainerValue::$enum_var_name(othr)) => slf.update(othr, add_only_associated_count),
                        )+
                        _ => {}
                    }
                }

                paste::paste! {
                    $(
                    pub fn [<as_ref_ $normal_name>](&self) -> Option<&MetadataContainerValueGeneric<$typ>> {
                        match self {
                            Self::$enum_var_name(resolved_value) => {
                                Some(resolved_value)
                            }
                            _ => None
                        }
                    }

                    pub fn [<as_mut_ $normal_name>](&mut self) -> Option<&mut MetadataContainerValueGeneric<$typ>> {
                        match self {
                            Self::$enum_var_name(resolved_value) => {
                                Some(resolved_value)
                            }
                            _ => None
                        }
                    }

                    pub unsafe fn [<as_ref_unchecked_ $normal_name>](&self) -> &MetadataContainerValueGeneric<$typ> {
                        match self {
                            Self::$enum_var_name(resolved_value) => {
                                resolved_value
                            }
                            _ => panic!("Illegal conversion for {}", stringify!($typ))
                        }
                    }

                    pub unsafe fn [<as_mut_unchecked_ $normal_name>](&mut self) -> &mut MetadataContainerValueGeneric<$typ> {
                        match self {
                            Self::$enum_var_name(resolved_value) => {
                                resolved_value
                            }
                            _ => panic!("Illegal conversion for {}", stringify!($typ))
                        }
                    }
                    )+
                }
            }
        };


    };
}
pub(super) use impl_keys;

macro_rules! impl_general_metadata {
    ($($normal_name:ident, $enum_var_name: ident, $typ: ty);+ $(;)?) => {

        #[derive(Clone, Eq, PartialEq, Debug)]
        pub enum GeneralMetadataEntry<'a> {
            $($enum_var_name(std::borrow::Cow<'a, std::collections::HashMap<$typ, u32>>),
            )+
        }

        #[derive(Clone, Eq, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
        pub enum GeneralMetadata {
            $($enum_var_name(Option<std::collections::HashMap<$typ, u32>>, Vec<Option<std::collections::HashMap<$typ, u32>>>),
            )+
        }

        impl GeneralMetadata {
            $(
            fn $normal_name(default: Option<std::collections::HashMap<$typ, u32>>, dicts: Vec<Option<std::collections::HashMap<$typ, u32>>>) -> GeneralMetadata {
                GeneralMetadata::$enum_var_name(default, dicts)
            }
            )+

            pub fn get_default(&self) -> Option<GeneralMetadataEntry> {
                match self {
                    $(
                    GeneralMetadata::$enum_var_name(Some(default), _) => {
                        Some(GeneralMetadataEntry::$enum_var_name(std::borrow::Cow::Borrowed(default)))
                    }
                    )+
                    _ => None
                }
            }

            pub fn get_dicts(&self) -> Vec<Option<GeneralMetadataEntry>> {
                match self {
                    $(
                    GeneralMetadata::$enum_var_name(_, value) => {
                        value
                        .iter()
                        .map(|value| value.as_ref().map(|value| GeneralMetadataEntry::$enum_var_name(std::borrow::Cow::Borrowed(value))))
                        .collect()
                    }
                    )+
                }
            }

            pub fn get_for_dict<S: string_interner::Symbol>(&self, idx: S) -> Option<GeneralMetadataEntry> {
                match self {
                    $(
                    GeneralMetadata::$enum_var_name(_, value) => {
                        value
                        .get(idx.to_usize())?
                        .as_ref()
                        .map(|value| GeneralMetadataEntry::$enum_var_name(std::borrow::Cow::Borrowed(value)))
                    }
                    )+
                }
            }
        }

        impl MetadataEx {

            pub fn all_fields(&self) -> enum_map::EnumMap<MetaField, GeneralMetadata> {
                enum_map::enum_map! {
                    $(
                        MetaField::$enum_var_name => GeneralMetadata::$normal_name(
                            self.general_metadata.get().and_then(|value| {
                                value.$normal_name().map(|value| {
                                    value.counts().into_owned()
                                })
                            }),
                            self.associated_metadata.iter().map(|value| {
                                value.get().and_then(|value| {
                                    value.$normal_name().map(|value| {
                                        value.counts().into_owned()
                                    })
                                })
                            }).collect()
                        ),
                    )+
                }
            }

            pub fn field(&self, field: MetaField) -> GeneralMetadata {
                match field {
                    $(
                        MetaField::$enum_var_name => GeneralMetadata::$normal_name(
                            self.general_metadata.get().and_then(|value| {
                                value.$normal_name().map(|value| {
                                    value.counts().into_owned()
                                })
                            }),
                            self.associated_metadata.iter().map(|value| {
                                value.get().and_then(|value| {
                                    value.$normal_name().map(|value| {
                                        value.counts().into_owned()
                                    })
                                })
                            }).collect()
                        ),
                    )+
                }
            }

            /// set dict to None for default
            pub fn single_field_value<S: string_interner::Symbol>(&self, field: MetaField, dict: Option<S>) -> Option<GeneralMetadataEntry> {
                match field {
                    $(
                        MetaField::$enum_var_name => {
                            if let Some(dict) = dict {
                                self
                                .associated_metadata
                                .get(dict.to_usize())?
                                .get()?
                                .$normal_name()
                                .map(|value| GeneralMetadataEntry::$enum_var_name(value.counts()))
                            } else {
                                self
                                .general_metadata
                                .get()?
                                .$normal_name()
                                .map(|value| GeneralMetadataEntry::$enum_var_name(value.counts()))
                            }
                        }
                    )+
                }
            }
        }

    };
}

pub(super) use impl_general_metadata;

macro_rules! create_metadata_impl {
    ($($tt:tt)+) => {
        $crate::topicmodel::dictionary::metadata::ex::metadata::impl_associated_metadata!($($tt)+);
    };
}

pub(super) use create_metadata_impl;

use std::borrow::{Borrow, Cow};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::iter::Map;
use std::num::NonZeroU32;
use std::ops::Range;
use either::Either;
use itertools::{Itertools, Position};
use strum::EnumIs;
use thiserror::Error;
use tinyset::Fits64;
use super::*;


#[derive(Debug, Error)]
#[error("Expected the field name {actual} but got {expected}!")]
pub struct UnexpectedFieldKeyError<T>{
    pub actual: MetaField,
    pub expected: MetaField,
    pub value: T
}

impl<T> UnexpectedFieldKeyError<T> {
    pub fn new(actual: MetaField, expected: MetaField, value: T) -> Self {
        Self { actual, expected, value }
    }
}

#[derive(Copy, Clone)]
pub enum MetadataWithOrigin<T> {
    General(T),
    Associated(DictionaryOriginSymbol, T)
}

impl<T> MetadataWithOrigin<T> {
    pub fn to_metadata(self) -> T {
        match self {
            MetadataWithOrigin::General(v) => {
                v
            }
            MetadataWithOrigin::Associated(_, v) => {
                v
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, EnumIs)]
enum InnerMetadataContainerValueGeneric<T> where T: Fits64 + Eq + Hash {
    Small(Set64<T>),
    Big(HashMap<T, u32>),
}

impl<T> Display for InnerMetadataContainerValueGeneric<T> where T: Fits64 + Eq + Hash + Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (p, (v, k)) in self.iter_counts().with_position() {
            match p {
                Position::First | Position::Middle => {
                    write!(f, "{v}: {k}, ")?;
                }
                Position::Last | Position::Only => {
                    write!(f, "{v}: {k}")?;
                }
            }
        }
        write!(f, ")")
    }
}

impl<T> InnerMetadataContainerValueGeneric<T> where T: Fits64 + Eq + Hash {

    pub fn entries(&self) -> Cow<Set64<T>> {
        match self {
            InnerMetadataContainerValueGeneric::Small(value) => {
                Cow::Borrowed(value)
            }
            InnerMetadataContainerValueGeneric::Big(value) => {
                Cow::Owned(value.keys().cloned().collect())
            }
        }
    }

    pub fn counts(&self) -> Cow<HashMap<T, u32>> {
        match self {
            InnerMetadataContainerValueGeneric::Small(value) => {
                Cow::Owned(value.iter().map(|v| (v, 1)).collect())
            }
            InnerMetadataContainerValueGeneric::Big(value) => {
                Cow::Borrowed(value)
            }
        }
    }

    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=T> + 'a> {
        match self {
            InnerMetadataContainerValueGeneric::Small(value) => {
                Box::new(value.iter())
            }
            InnerMetadataContainerValueGeneric::Big(value) => {
                Box::new(value.keys().copied())
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            InnerMetadataContainerValueGeneric::Small(value) => value.is_empty(),
            InnerMetadataContainerValueGeneric::Big(value) => value.is_empty()
        }
    }


    pub fn len(&self) -> usize {
        match self {
            InnerMetadataContainerValueGeneric::Small(resolved_value) => {
                resolved_value.len()
            }
            InnerMetadataContainerValueGeneric::Big(resolved_value) => {
                resolved_value.len()
            }
        }
    }

    pub fn unwrap_simple(self) -> Set64<T> {
        match self {
            InnerMetadataContainerValueGeneric::Small(resolved_value) => {
                resolved_value
            }
            _ => panic!("Not a {}!", std::any::type_name::<Set64<T>>())
        }
    }

    pub fn unwrap_counting(self) -> HashMap<T, u32> {
        match self {
            InnerMetadataContainerValueGeneric::Big(resolved_value) => {
                resolved_value
            }
            _ => panic!("Not a {}!", std::any::type_name::<Set64<T>>())
        }
    }

    pub fn as_ref(&self) -> Either<&Set64<T>, &HashMap<T, u32>> {
        match self {
            InnerMetadataContainerValueGeneric::Small(resolved_value) => {
                Either::Left(resolved_value)
            }
            InnerMetadataContainerValueGeneric::Big(resolved_value) => {
                Either::Right(resolved_value)
            }
        }
    }

    pub fn as_ref_mut(&mut self) -> Either<&mut Set64<T>, &mut HashMap<T, u32>> {
        match self {
            InnerMetadataContainerValueGeneric::Small(resolved_value) => {
                Either::Left(resolved_value)
            }
            InnerMetadataContainerValueGeneric::Big(resolved_value) => {
                Either::Right(resolved_value)
            }
        }
    }

    pub fn contains<R: Borrow<T>>(&self, value: R) -> bool {
        match self {
            InnerMetadataContainerValueGeneric::Small(v) => {
                v.contains(value)
            }
            InnerMetadataContainerValueGeneric::Big(v) => {
                v.contains_key(value.borrow())
            }
        }
    }

    /// Returns the count for [value].
    /// this value is between 0..n
    pub fn count_of<R: Borrow<T>>(&self, value: R) -> u32 {
        match self {
            InnerMetadataContainerValueGeneric::Small(val) => {
                val.contains(value) as u32
            }
            InnerMetadataContainerValueGeneric::Big(val) => {
                val.get(value.borrow()).copied().unwrap_or(0)
            }
        }
    }

    pub fn iter_counts<'a>(&'a self) -> Box<dyn Iterator<Item=(T, NonZeroU32)> + 'a> {
        match self {
            InnerMetadataContainerValueGeneric::Small(resolved_value) => {
                Box::new(resolved_value.iter().map(|value| (value, unsafe{NonZeroU32::new_unchecked(1)})))
            }
            InnerMetadataContainerValueGeneric::Big(resolved_value) => {
                Box::new(resolved_value.iter().map(|(k, v)| (k.clone(), unsafe{NonZeroU32::new_unchecked(*v)})))
            }
        }
    }

    pub fn insert_no_count(&mut self, value: T) {
        match self {
            InnerMetadataContainerValueGeneric::Small(v) => {
                v.insert(value);
            }
            InnerMetadataContainerValueGeneric::Big(v) => {
                v.entry(value).or_insert(1);
            }
        }
    }

    pub fn extend_no_count<I: IntoIterator<Item=T>>(&mut self, values: I) {
        match self {
            InnerMetadataContainerValueGeneric::Small(v) => {
                v.extend(values);
            }
            InnerMetadataContainerValueGeneric::Big(v) => {
                for value in values {
                    v.entry(value).or_insert(1);
                }
            }
        }
    }
}

impl<T> Default for InnerMetadataContainerValueGeneric<T> where T: Fits64 + Eq + Hash {
    fn default() -> Self {
        Self::Small(Set64::default())
    }
}


#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[repr(transparent)]
pub struct MetadataContainerValueGeneric<T> where T: Fits64 + Eq + Hash {
    inner: InnerMetadataContainerValueGeneric<T>
}

impl<T> Display for MetadataContainerValueGeneric<T> where T: Fits64 + Eq + Hash + Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> MetadataContainerValueGeneric<T> where T: Fits64 + Eq + Hash {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub(super) unsafe fn apply_new_as_update(&mut self, update: Self) {
        self.inner = update.inner;
    }

    pub(super) unsafe fn apply_iterable_as_update<C: Into<u32>, I: IntoIterator<Item=(T, C)>>(&mut self, update: I, add_only_associated_count: bool) {
        let mut new = MetadataContainerValueGeneric::new();
        for (k, v) in update {
            unsafe {
                new.insert_direct(k, v.into(), add_only_associated_count);
            }
        }
        self.apply_new_as_update(new)
    }

    /// add_only_associated_count indicates, that we have to remove a single count from add_only_associated_count
    pub unsafe fn insert_direct(&mut self, value: T, count: u32, add_only_associated_count: bool) {
        match (count, add_only_associated_count) {
            (1, true) | (0, _) => {
                self.inner.insert_no_count(value);
            },
            (1, _) => {
                self.insert_and_count(value);
            }
            (_, true) => {
                self.get_or_init_count().insert(value, count.saturating_sub(1));
            }
            _ => {
                self.get_or_init_count().insert(value, count);
            }
        }
    }

    fn get_or_init_count(&mut self) -> &mut HashMap<T, u32> {
        if self.inner.is_small() {
            let len = self.inner.len();
            let left = std::mem::replace(&mut self.inner, InnerMetadataContainerValueGeneric::Big(HashMap::with_capacity(len))).unwrap_simple();
            let result = match self.inner.as_ref_mut() {
                Either::Right(value) => value,
                _ => unreachable!()
            };
            result.extend(left.iter().map(|value| (value, 1)));
            result
        } else {
            match self.inner.as_ref_mut() {
                Either::Right(value) => value,
                _ => unreachable!()
            }
        }
    }

    /// starts at zero
    fn get_count_ref_to(&mut self, value: T) -> &mut u32 {
        self.get_or_init_count().entry(value).or_insert(1)
    }

    #[inline(always)]
    pub fn iter_keys<'a>(&'a self) -> Box<dyn Iterator<Item=T> + 'a> {
        self.inner.iter()
    }

    #[inline(always)]
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=(T, NonZeroU32)> + 'a> {
        self.inner.iter_counts()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }


    /// Insert without count
    pub fn insert(&mut self, value: T) {
        self.inner.insert_no_count(value);
    }

    /// Insert without count
    pub fn extend<I: IntoIterator<Item=T>>(&mut self, values: I) {

        values.into_iter().for_each(|v| self.inner.insert_no_count(v));
    }

    /// Insert and inc count.
    pub fn insert_and_count(&mut self, value: T) {
        match &mut self.inner {
            InnerMetadataContainerValueGeneric::Small(targ) => {
                if targ.insert(value) {
                    return;
                }
            }
            _ => {}
        }
        *self.get_count_ref_to(value) += 1;
    }

    #[inline(always)]
    pub fn counts(&self) -> Cow<HashMap<T, u32>> {
        self.inner.counts()
    }

    #[inline(always)]
    pub fn entries(&self) -> Cow<Set64<T>> {
        self.inner.entries()
    }

    #[inline(always)]
    pub fn contains<R: Borrow<T>>(&self, value: R) -> bool {
        self.inner.contains(value)
    }

    /// Returns the count for [value].
    /// this value is between 0..n
    #[inline(always)]
    pub fn count_of<R: Borrow<T>>(&self, value: R) -> u32 {
        self.inner.count_of(value)
    }

    /// Returns the value of asssociated counts, this value is between 0..n
    pub fn associated_count_of<R: Borrow<T>>(&self, value: R) -> u32 {
        self.count_of(value).saturating_sub(1)
    }

    /// Returns a value and the counts of it.
    /// Returns at least one.
    ///
    /// If you only want the associated counts (and not the associated + 1)
    /// use [iter_associated_counts]
    pub fn iter_counts<'a>(&'a self) -> Box<dyn Iterator<Item=(T, NonZeroU32)> + 'a> {
        self.inner.iter_counts()
    }

    pub fn iter_associated_counts<'a>(&'a self) -> Map<Box<dyn Iterator<Item=(T, NonZeroU32)> + 'a>, fn((T, NonZeroU32)) -> (T, u32)> {
        self.inner.iter_counts().map(|(k, v)| (k, v.get().saturating_sub(1)))
    }

    /// An update for this element. If the add_only_associated_count is set, the value is only set, but not counted.
    /// The three possible update strategies are:
    ///
    /// ```python
    /// target: T = ...;
    /// add_only_associated_count = true;
    ///
    /// x = self.count_of(target);
    /// n = other.count_of(target);
    ///
    /// if add_only_associated_count {
    ///     if n == 1 {
    ///         self[target] = x
    ///     } else if n > 1 {
    ///         self[target] = x + n - 1
    ///     }
    /// } else {
    ///     if n == 1 {
    ///         self[target] = x + 1
    ///     } else if n > 1 {
    ///         self[target] = x + n - 1
    ///     }
    /// }
    /// ```
    pub fn update(&mut self, other: &Self, add_only_associated_count: bool) {
        if add_only_associated_count {
            for (targ, value) in other.iter_counts() {
                if value.get() == 1 {
                    self.inner.insert_no_count(targ)
                } else {
                    *self.get_count_ref_to(targ) += value.get() - 1;
                }
            }
        } else {
            for (targ, value) in other.iter_counts() {
                if value.get() == 1 {
                    self.insert_and_count(targ);
                } else {
                    *self.get_count_ref_to(targ) += value.get();
                }
            }
        }
    }

}


#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, Eq, PartialEq)]
pub struct AssociatedMetadataImpl {
    inner: HashMap<MetaField, MetadataContainerValue>
}

impl AssociatedMetadataImpl {

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
            || self.inner.values().all(|value| value.is_empty())
    }

    pub fn field_is_empty(&self, key: MetaField) -> bool {
        self.inner.get(&key).map_or(true, |value| value.is_empty())
    }

    pub fn update_with(&mut self, other: &AssociatedMetadataImpl, add_only_associated_count: bool) {
        for (k, v) in other.inner.iter() {
            match self.inner.entry(k.clone()) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().update(v, add_only_associated_count);
                }
                Entry::Vacant(entry) => {
                    entry.insert(v.clone());
                }
            }
        }
    }

    pub fn get_or_insert(&mut self, key: MetaField) -> &mut MetadataContainerValue {
        self.inner.entry(key).or_insert_with(|| MetadataContainerValue::create_for_key(key))
    }

    pub fn get(&self, key: MetaField) -> Option<&MetadataContainerValue> {
        self.inner.get(&key)
    }

    pub fn get_mut(&mut self, key: MetaField) -> Option<&mut MetadataContainerValue> {
        self.inner.get_mut(&key)
    }

    /// Returns true if there was a non-empty field dropped
    pub fn drop_field(&mut self, key: MetaField) -> bool {
        self.inner.remove(&key).is_some_and(|value| !value.is_empty())
    }
}

impl Display for AssociatedMetadataImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (p, (k, v)) in self.inner.iter().sorted_unstable_by_key(|(k, _)| **k).with_position() {
            match p {
                Position::First | Position::Middle => {
                    write!(f, "{}: {}, ", k, v)?;
                }
                Position::Last | Position::Only => {
                    write!(f, "{}: {}", k, v)?;
                }
            }

        }
        write!(f, "}}")?;
        Ok(())
    }
}


/// The metadata for an entry
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Eq, PartialEq)]
pub struct MetadataEx {
    pub(super) general_metadata: LazyAssociatedMetadata,
    pub(super) associated_metadata: Vec<LazyAssociatedMetadata>,
}

impl MetadataEx {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            general_metadata: LazyAssociatedMetadata::new(),
            associated_metadata: Vec::with_capacity(capacity),
        }
    }

    /// Returns true if a field is empty.
    pub fn field_is_empty(&self, key: MetaField) -> bool {
        self.general_metadata.get()
            .and_then(|value| value.get())
            .map_or(true, |value| value.field_is_empty(key))
    }
    
    /// Drops a single field from all the metadata.
    /// Returns true when some kind of data was dropped.
    pub fn drop_field(&mut self, key: MetaField) -> bool {
        self.general_metadata.drop_field(key) 
            | self.associated_metadata.iter_mut().map(|m| m.drop_field(key)).any(|x| x)
    }

    /// Drops all field in the metadata.
    /// Returns true when some kind of data was dropped.
    pub fn drop_all_fields(&mut self) -> bool {
        self.general_metadata.drop_all_fields()
            | self.associated_metadata.iter_mut().map(|m| m.drop_all_fields()).any(|x| x)
    }

    pub fn get_general_metadata(&self) -> Option<&AssociatedMetadata> {
        self.general_metadata.get()
    }

    pub fn get_init_general_metadata(&mut self) -> Option<&mut AssociatedMetadata> {
        self.general_metadata.get_mut()
    }

    pub fn get_or_init_general_metadata(&self) -> &AssociatedMetadata {
        self.general_metadata.get_or_init()
    }

    pub fn get_mut_or_init_general_metadata(&mut self) -> &mut AssociatedMetadata {
        self.general_metadata.get_mut_or_init()
    }

    pub fn get_associated_metadata(&self, origin: DictionaryOriginSymbol) -> Option<&AssociatedMetadata> {
        use string_interner::Symbol;
        self.associated_metadata.get(origin.to_usize())?.get()
    }

    pub fn get_mut_associated_metadata(&mut self, origin: DictionaryOriginSymbol) -> Option<&mut AssociatedMetadata> {
        use string_interner::Symbol;
        self.associated_metadata.get_mut(origin.to_usize())?.get_mut()
    }

    #[inline(always)]
    fn get_or_create_impl(&mut self, origin: usize) -> &mut AssociatedMetadata {
        if self.associated_metadata.len() <= origin {
            self.associated_metadata.resize_with(origin + 1, LazyAssociatedMetadata::new);
        }
        unsafe {self.associated_metadata.get_unchecked_mut(origin)}.get_mut_or_init()
    }
    
    /// Gets or creates the metadata for a dictionary.
    pub fn get_or_create(&mut self, origin: DictionaryOriginSymbol) -> &mut AssociatedMetadata {
        use string_interner::Symbol;
        self.get_or_create_impl(origin.to_usize())
    }

    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut::new(self)
    }


    pub fn update_with(&mut self, other: &MetadataEx, add_only_associated_count: bool) {
        if let Some(targ) = other.general_metadata.get() {
            self.general_metadata.get_mut_or_init().update_with(targ, add_only_associated_count);
        }
        for (origin, value) in other.associated_metadata.iter().enumerate() {
            if let Some(value) = value.get() {
                self.get_or_create_impl(origin).update_with(value, add_only_associated_count)
            }
        }
    }

    pub fn collect_all_associated_word_ids(&self) -> Option<Set64<usize>> {
        let mut collection = Set64::new();
        for value in self.iter() {
            if let Some(v) = value.to_metadata().collect_all_known_ids() {
                collection.extend(v);
            }
        }
        if collection.is_empty() {
            None
        } else {
            Some(collection)
        }
    }
}

impl Display for MetadataEx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ general: [{}]", self.general_metadata)?;
        for (k, v) in self.associated_metadata.iter().enumerate() {
            write!(f, ", dict_{k}: [{v}]")?;
        }
        write!(f, " }}")
    }
}

impl crate::topicmodel::dictionary::metadata::Metadata for MetadataEx{}


/// Static extensions for MetadataWithOrigin
impl<T> MetadataWithOrigin<T> where T: Copy {
    pub fn origin(&self) -> Option<crate::toolkit::typesafe_interner::DictionaryOriginSymbol> {
        match self {
            MetadataWithOrigin::General(_) => {
                None
            }
            MetadataWithOrigin::Associated(value, _) => {
                Some(*value)
            }
        }
    }

    pub fn meta(&self) -> T {
        match self {
            MetadataWithOrigin::General(value) => {
                *value
            }
            MetadataWithOrigin::Associated(_, value) => {
                *value
            }
        }
    }
}


/// An iterator for MetadataEx
pub struct Iter<'a> {
    src: &'a MetadataEx,
    general_metadata: bool,
    pos: Range<usize>
}

impl<'a> Iter<'a> {
    pub fn new(src: &'a MetadataEx) -> Self {
        Self { src, general_metadata: false, pos: 0..src.associated_metadata.len() }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = MetadataWithOrigin<&'a AssociatedMetadata>;

    fn next(&mut self) -> Option<Self::Item> {
        use string_interner::Symbol;

        if self.general_metadata && self.pos.is_empty() {
            return None
        }
        if !self.general_metadata {
            self.general_metadata = true;
            if let Some(meta) = self.src.general_metadata.get() {
                return Some(MetadataWithOrigin::General(meta))
            }
        }
        let idx = self.pos.next()?;
        let targ = self.src.associated_metadata.get(idx)?;
        let meta = targ.get()?;
        Some(MetadataWithOrigin::Associated(
            DictionaryOriginSymbol::try_from_usize(idx).unwrap(),
            meta
        ))
    }
}


/// A mutable iterator over MetadataEx
pub struct IterMut<'a> {
    src: &'a mut MetadataEx,
    general_metadata: bool,
    pos: Range<usize>
}

impl<'a> IterMut<'a> {
    pub fn new(src: &'a mut MetadataEx) -> Self {
        let pos = 0..src.associated_metadata.len();
        Self { src, general_metadata: false, pos }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = MetadataWithOrigin<&'a mut AssociatedMetadata>;

    fn next(&mut self) -> Option<Self::Item> {
        use string_interner::Symbol;
        if self.general_metadata && self.pos.is_empty() {
            return None
        }
        if !self.general_metadata {
            self.general_metadata = true;
            if let Some(meta) = self.src.general_metadata.get_mut() {
                return Some(MetadataWithOrigin::General(unsafe{std::mem::transmute(meta)}))
            }
        }
        let idx = self.pos.next()?;
        let targ = self.src.associated_metadata.get_mut(idx)?;
        let meta = targ.get_mut()?;
        Some(MetadataWithOrigin::Associated(
            DictionaryOriginSymbol::try_from_usize(idx).unwrap(),
            unsafe{std::mem::transmute(meta)}
        ))
    }
}


/// A lazy loading structure for associated metadata.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Eq)]
#[repr(transparent)]
// #[serde(transparent)]
pub(super) struct LazyAssociatedMetadata {
    #[serde(with = "crate::toolkit::once_serializer::OnceCellDef")]
    pub(super) inner: std::cell::OnceCell<AssociatedMetadata>
}

impl Display for LazyAssociatedMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(inner) = self.get() {
            inner.fmt(f)
        } else {
            write!(f, "_Unset_")
        }
    }
}

impl Default for LazyAssociatedMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl LazyAssociatedMetadata {
    pub fn new() -> Self {
        Self {
            inner: std::cell::OnceCell::new()
        }
    }

    /// returns true when something was dropped.
    pub fn drop_field(&mut self, field_name: MetaField) -> bool {
        self.get_mut().is_some_and(|value| value.drop_field(field_name))
    }
    
    /// Returns true if some kind of content was dropped.
    pub fn drop_all_fields(&mut self) -> bool {
        self.get_mut().is_some_and(|value| value.drop_all_fields())
    }

    pub fn into_inner(self) -> std::cell::OnceCell<AssociatedMetadata> {
        self.inner
    }

    #[inline(always)]
    pub fn is_not_init(&self) -> bool {
        self.inner.get().is_none()
    }

    #[inline(always)]
    pub fn get_or_init(&self) -> &AssociatedMetadata {
        self.inner.get_or_init(AssociatedMetadata::default)
    }

    #[inline(always)]
    pub fn get_mut_or_init(&mut self) -> &mut AssociatedMetadata {
        self.inner.get_or_init(AssociatedMetadata::default);
        unsafe {self.inner.get_mut().unwrap_unchecked()}
    }

    #[inline(always)]
    pub fn get(&self) -> Option<&AssociatedMetadata> {
        self.inner.get()
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> Option<&mut AssociatedMetadata> {
        self.inner.get_mut()
    }

    #[inline(always)]
    pub fn collect_all_known_ids(&self) -> Option<Set64<usize>> {
        self.get()?.collect_all_known_ids()
    }
}

impl PartialEq for LazyAssociatedMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.inner.get() == other.inner.get()
    }
}

impl Default for MetadataEx {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}


/// The associated metadata
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, Eq)]
#[repr(transparent)]
// #[serde(transparent)]
pub struct AssociatedMetadata {
    #[serde(with = "crate::toolkit::once_serializer::OnceCellDef")]
    pub(super) inner: std::cell::OnceCell<AssociatedMetadataImpl>
}

impl Display for AssociatedMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(inner) = self.inner.get() {
            write!(f, "{inner}")
        } else {
            write!(f, "_Empty_")
        }
    }
}

impl PartialEq for AssociatedMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.inner.get() == other.inner.get()
    }
}

impl AssociatedMetadata {
    #[inline(always)]
    pub(super) fn get_or_init(&self) -> &AssociatedMetadataImpl {
        self.inner.get_or_init(AssociatedMetadataImpl::default)
    }

    #[inline(always)]
    pub(super) fn get_mut_or_init(&mut self) -> &mut AssociatedMetadataImpl {
        self.inner.get_or_init(AssociatedMetadataImpl::default);
        unsafe {self.inner.get_mut().unwrap_unchecked()}
    }

    #[inline(always)]
    pub fn get(&self) -> Option<&AssociatedMetadataImpl> {
        self.inner.get()
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> Option<&mut AssociatedMetadataImpl> {
        self.inner.get_mut()
    }

    // Returns true if the underlying element is either unintitialized or it is initialized
    // but does not conatin any values.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.get().is_none_or(|value| value.is_empty())
    }

    #[inline(always)]
    pub fn update_with(&mut self, other: &AssociatedMetadata, add_only_associated_count: bool) {
        if let Some(other) = other.get() {
            self.get_mut_or_init().update_with(other, add_only_associated_count)
        }
    }

    #[inline(always)]
    pub fn collect_all_known_ids(&self) -> Option<Set64<usize>> {
        Some(self.get()?.collect_all_known_ids())
    }

    pub fn drop_field(&mut self, field_name: MetaField) -> bool {
        self.get_mut().is_some_and(|value| value.drop_field(field_name))
    }

    pub fn drop_all_fields(&mut self) -> bool {
        self.inner.take().is_some_and(|value| !value.is_empty())
    }
}


#[cfg(test)]
mod test {
    use tinyset::{Fits64, Set64};
    use crate::toolkit::typesafe_interner::AnyIdSymbol;
    use crate::topicmodel::dictionary::metadata::ex::{AssociatedMetadataImpl, MetadataEx};
    use crate::topicmodel::dictionary::word_infos::PartOfSpeech;


    /// The associated metadata
    #[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
    #[repr(transparent)]
    #[serde(transparent)]
    pub struct Data {
        #[serde(with = "crate::toolkit::once_serializer::OnceCellDef")]
        pub(super) inner: std::cell::OnceCell<Set64<PartOfSpeech>>
    }


    unsafe fn add_data(write: &mut AssociatedMetadataImpl) {
        write.pos_mut().unwrap().insert(PartOfSpeech::Noun);
        write.pos_mut().unwrap().insert(PartOfSpeech::Num);
        write.ids_mut().unwrap().insert(AnyIdSymbol::from_u64(12));
        write.ids_mut().unwrap().insert(AnyIdSymbol::from_u64(3));
    }

    #[test]
    fn can_deser(){
        let mut meta = MetadataEx::default();
        {
           unsafe {
               add_data(meta.general_metadata.get_mut_or_init().get_mut_or_init());
               for value in meta.associated_metadata.iter_mut() {
                   add_data(value.get_mut_or_init().get_mut_or_init());
               }
           }
        }

        // let ser = bincode::serialize(&meta).unwrap();

        // let deser: MetadataEx = bincode::deserialize(&ser).unwrap();

        let mut x = Data::default();
        x.inner.get_or_init(Set64::new);
        x.inner.get_mut().unwrap().insert(PartOfSpeech::Num);
        x.inner.get_mut().unwrap().insert(PartOfSpeech::Noun);

        let ser = bincode::serialize(&x).unwrap();

        let deser: Data = bincode::deserialize(&ser).unwrap();

        for value in deser.inner.get().unwrap().iter() {
            println!("{value}")
        }

        let mut x: Set64<PartOfSpeech> = tinyset::Set64::new();
        x.insert(PartOfSpeech::Num);
        x.insert(PartOfSpeech::Noun);



        let ser = bincode::serialize(&x).unwrap();

        let deser: Set64<PartOfSpeech> = bincode::deserialize(&ser).unwrap();
        for value in deser {
            println!("{value}")
        }
    }
}