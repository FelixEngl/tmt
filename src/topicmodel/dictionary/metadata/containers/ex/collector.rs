macro_rules! generate_insert {
    (set: $target_dict: ident, $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($target_dict, $value.into_iter().map(
                |value|
                match value {
                    either::Either::Left(value) => value,
                    _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                }
            ));
        }
    };
    (interned: $target_dict: ident, $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($target_dict, $value.into_iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value,
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };
    (voc: $target_dict: ident, $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($target_dict, $value.into_iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value,
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };


    (set: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name _default>]($value.into_iter().map(
                |value|
                match value {
                    either::Either::Left(value) => value,
                    _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                }
            ));
        }
    };
    (interned: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name _default>]($value.into_iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value,
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };
    (voc: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name _default>]($value.into_iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value,
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };

    (set special: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($value.into_iter().map(
                |value|
                match value {
                    either::Either::Left(value) => value,
                    _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                }
            ));
        }
    };
    (interned special: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($value.into_iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value,
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };
    (voc special: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($value.into_iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value,
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };


    (set cloning: $target_dict: ident, $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($target_dict, $value.iter().map(
                |value|
                match value {
                    either::Either::Left(value) => value.clone(),
                    _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                }
            ));
        }
    };
    (interned cloning: $target_dict: ident, $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($target_dict, $value.iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value.clone(),
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };
    (voc cloning: $target_dict: ident, $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name>]($target_dict, $value.iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value.clone(),
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };


    (set cloning: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name _default>]($value.iter().map(
                |value|
                match value {
                    either::Either::Left(value) => value.clone(),
                    _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                }
            ));
        }
    };
    (interned cloning: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name _default>]($value.iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value.clone(),
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };
    (voc cloning: $name: ident, $value: expr => $output: ident) => {
        paste::paste! {
            $output.[<add_all_to_ $name _default>]($value.iter().map(
                |value|
                match value {
                    either::Either::Right(value) => value.clone(),
                    _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                }
            ));
        }
    };
}

pub(super) use generate_insert;

macro_rules! generate_insert_builder {
    (set: $name: ident: $typ: ty | $gtyp: ty) => {
        paste::paste! {
            pub fn [<push_ $name>](&mut self, value: $typ) {
                self.$name.get_or_insert(None).get_or_insert_with(Vec::new).push(either::Either::Left(value));
            }

            pub fn [<extend_ $name>]<I: IntoIterator<Item=$typ>>(&mut self, value: I) {
                self.$name.get_or_insert(None).get_or_insert_with(Vec::new).extend(value.into_iter().map(either::Either::Left));
            }

            pub fn [<peek_ $name>](&self) -> Option<Vec<&$typ>> {
                Some(self.$name.as_ref()?.as_ref()?.iter().map(|value| {
                    match value {
                        either::Either::Left(value) => value,
                        _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                    }
                }).collect::<Vec<_>>())
            }

            pub fn [<peek_raw_ $name>](&self) -> Option<&Vec<either::Either<$typ, $gtyp>>> {
                self.$name.as_ref()?.as_ref()
            }
        }
    };
    (interned: $name: ident: $typ: ty | $gtyp: ty) => {
        paste::paste! {
            pub fn [<push_ $name>](&mut self, value: $gtyp) {
                self.$name.get_or_insert(None).get_or_insert_with(Vec::new).push(either::Either::Right(value));
            }

            pub fn [<extend_ $name>]<I: IntoIterator<Item=$gtyp>>(&mut self, value: I) {
                self.$name.get_or_insert(None).get_or_insert_with(Vec::new).extend(value.into_iter().map(either::Either::Right));
            }

            pub fn [<peek_ $name>](&self) -> Option<Vec<&$gtyp>> {
                Some(self.$name.as_ref()?.as_ref()?.iter().map(|value| {
                    match value {
                        either::Either::Right(value) => value,
                        _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                    }
                }).collect::<Vec<_>>())
            }

            pub fn [<peek_raw_ $name>](&self) -> Option<&Vec<either::Either<$typ, $gtyp>>> {
                self.$name.as_ref()?.as_ref()
            }
        }
    };
    (voc: $name: ident: $typ: ty | $gtyp: ty) => {
        paste::paste! {
            pub fn [<push_ $name>](&mut self, value: $gtyp) {
                self.$name.get_or_insert(None).get_or_insert_with(Vec::new).push(either::Either::Right(value));
            }

            pub fn [<extend_ $name>]<I: IntoIterator<Item=$gtyp>>(&mut self, value: I) {
                self.$name.get_or_insert(None).get_or_insert_with(Vec::new).extend(value.into_iter().map(either::Either::Right));
            }

            pub fn [<peek_ $name>](&self) -> Option<Vec<&$gtyp>> {
                Some(self.$name.as_ref()?.as_ref()?.iter().map(|value| {
                    match value {
                        either::Either::Right(value) => value,
                        _ => unreachable!("Failed to unpack right for {}", stringify!($name)),
                    }
                }).collect::<Vec<_>>())
            }

            pub fn [<peek_raw_ $name>](&self) -> Option<&Vec<either::Either<$typ, $gtyp>>> {
                self.$name.as_ref()?.as_ref()
            }
        }
    };
}

pub(super) use generate_insert_builder;

macro_rules! generate_getter {
    (set: $name: ident: $typ: ty | $gtyp: ty) => {
        paste::paste! {
            pub fn [<get_ $name>]<'a>(&'a self) -> Option<Vec<&'a $typ>> {
                Some(self.$name.as_ref()?.iter().map(|value| {
                    match value {
                        either::Either::Left(value) => value,
                        _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                    }
                }).collect::<Vec<_>>())
            }

            pub fn [<get_ $name _raw>](&self) -> Option<&Vec<either::Either<$typ, $gtyp>>> {
                self.$name.as_ref()
            }
        }
    };
    (interned: $name: ident: $typ: ty | $gtyp: ty) => {
        paste::paste! {
            pub fn [<get_ $name>]<'a>(&'a self) -> Option<Vec<&'a $gtyp>> {
                Some(self.$name.as_ref()?.iter().map(|value| {
                    match value {
                        either::Either::Right(value) => value,
                        _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                    }
                }).collect::<Vec<_>>())
            }

            pub fn [<get_ $name _raw>](&self) -> Option<&Vec<either::Either<$typ, $gtyp>>> {
                self.$name.as_ref()
            }
        }
    };
    (voc: $name: ident: $typ: ty | $gtyp: ty) => {
        paste::paste! {
            pub fn [<get_ $name>]<'a>(&'a self) -> Option<Vec<&'a $gtyp>> {
                Some(self.$name.as_ref()?.iter().map(|value| {
                    match value {
                        either::Either::Right(value) => value,
                        _ => unreachable!("Failed to unpack left for {}", stringify!($name)),
                    }
                }).collect::<Vec<_>>())
            }

            pub fn [<get_ $name _raw>](&self) -> Option<&Vec<either::Either<$typ, $gtyp>>> {
                self.$name.as_ref()
            }
        }
    };
}

pub(super) use generate_getter;

macro_rules! create_collector_implementation {

    ($($tt:tt: $name: ident: $collect_type: ty),+ $(,)?) => {
        #[derive(Debug, derive_builder::Builder)]
        pub struct MetadataCollection<T> {
            #[builder(setter(custom))]
            pub dictionary_name: Option<&'static str>,
            $(#[builder(setter(custom), default)]
            $name: Option<Vec<either::Either<$collect_type, T>>>,
            )+
        }

        impl<T> MetadataCollection<T> {
            $(
                crate::topicmodel::dictionary::metadata::ex::collector::generate_getter!(
                    $tt: $name: $collect_type | T
                );
            )+

            pub fn to_builder(self) -> MetadataCollectionBuilder<T> {
                MetadataCollectionBuilder {
                    dictionary_name: Some(self.dictionary_name),
                    $($name: self.$name.map(Option::Some),)+
                }
            }
        }

        impl<T: std::hash::Hash + std::clone::Clone + std::cmp::Eq> MetadataCollection<T> {
            pub fn shrink(&mut self) {
                use itertools::Itertools;
                $(
                    self.$name = self.$name.take().map(|value| value.into_iter().unique().collect());
                )+
            }
        }

        impl<T> MetadataCollection<T> {
            pub fn map<R, F>(self, mut mapping: F) -> MetadataCollection<R> where F: std::ops::FnMut(T) -> R {
                $(
                let $name = if let Some($name) = self.$name {
                    let mut new = Vec::with_capacity($name.len());
                    for x in $name {
                        new.push(match x {
                            either::Either::Right(value) => either::Either::Right(mapping(value)),
                            either::Either::Left(value) => either::Either::Left(value)
                        })
                    }
                    Some(new)
                } else {
                    None
                };
                )+
                MetadataCollection {
                    dictionary_name: self.dictionary_name,
                    $($name,)+
                }
            }
        }

        impl<T: AsRef<str>> MetadataCollection<T> {
            pub fn write_into(self, target: &mut $crate::topicmodel::dictionary::metadata::ex::MetadataMutRefEx) {
                if let Some(dictionary_name) = self.dictionary_name {
                    $(
                    if let Some(content) = self.$name {
                        crate::topicmodel::dictionary::metadata::ex::collector::generate_insert!(
                        $tt: dictionary_name, $name, content => target);
                    }
                    )+
                } else {
                    $(
                    if let Some(content) = self.$name {
                        crate::topicmodel::dictionary::metadata::ex::collector::generate_insert!(
                        $tt: $name, content => target);
                    }
                    )+
                }
            }
        }

        impl<T: AsRef<str> + Clone> MetadataCollection<T> {
            pub fn write_to(&self, target: &mut $crate::topicmodel::dictionary::metadata::ex::MetadataMutRefEx) {
                if let Some(ref dictionary_name) = self.dictionary_name {
                    $(
                    if let Some(ref content) = self.$name {
                        crate::topicmodel::dictionary::metadata::ex::collector::generate_insert!(
                        $tt cloning: dictionary_name, $name, content => target);
                    }
                    )+
                } else {
                    $(
                    if let Some(ref content) = self.$name {
                        crate::topicmodel::dictionary::metadata::ex::collector::generate_insert!(
                        $tt cloning: $name, content => target);
                    }
                    )+
                }
            }
        }

        impl<T: Clone> Clone for MetadataCollection<T> {
            fn clone(&self) -> Self {
                Self {
                    dictionary_name: self.dictionary_name.clone(),
                    $($name: self.$name.clone(),
                    )+
                }
            }
        }

        impl<'a, T: ToOwned> MetadataCollection<&'a T> {
            pub fn into_owned(self) -> MetadataCollection<<T as ToOwned>::Owned> {
                self.map(|value| value.to_owned())
            }
        }




        impl<T: std::hash::Hash + std::clone::Clone + std::cmp::Eq> MetadataCollectionBuilder<T> {
            pub fn shrink(&mut self) {
                use itertools::Itertools;
                $(
                    self.$name = self.$name.take().map(|mut value| value.take().map(|value| value.into_iter().unique().collect()));
                )+
            }
        }

        impl<T> MetadataCollectionBuilder<T> {
            pub fn map<R, F>(self, mut mapping: F) -> MetadataCollectionBuilder<R> where F: std::ops::FnMut(T) -> R {
                $(
                let $name = match self.$name {
                    None => None,
                    Some(None) => Some(None),
                    Some(Some($name)) => {
                        let mut new = Vec::with_capacity($name.len());
                        for x in $name {
                            new.push(match x {
                                either::Either::Right(value) => either::Either::Right(mapping(value)),
                                either::Either::Left(value) => either::Either::Left(value)
                            })
                        }
                        Some(Some(new))
                    }
                };
                )+
                MetadataCollectionBuilder {
                    dictionary_name: self.dictionary_name,
                    $($name,)+
                }
            }
        }

        impl<T: Clone> MetadataCollectionBuilder<T> {
            pub fn update_with(&mut self, other: &MetadataCollection<T>) {
                if self.dictionary_name != Some(other.dictionary_name) {
                    self.dictionary_name(other.dictionary_name);
                }
                self.update_fields_with(other);
            }

            /// Updates the builder but ignores the name check.
            pub fn update_fields_with(&mut self, other: &MetadataCollection<T>) {
                $(
                paste::paste! {
                    if let Some(other) = other.[<get_ $name>]() {
                        self.[<extend_ $name>](other.into_iter().cloned());
                    }
                }
                )+
            }

            pub fn update_with_other(&mut self, other: &MetadataCollectionBuilder<T>) -> Result<(), MetadataCollectionBuilderError> {
                // if self.ti != other.dictionary_name {
                //     // self.dictionary_name(other.dictionary_name)?;
                // }
                if self.dictionary_name != other.dictionary_name {
                    self.dictionary_name = self.dictionary_name.clone();
                }
                self.update_fields_with_other(other);
                Ok(())
            }

            /// Updates the builder but ignores the name check.
            pub fn update_fields_with_other(&mut self, other: &MetadataCollectionBuilder<T>) {
                $(
                paste::paste! {
                    if let Some(other) = other.[<peek_ $name>]() {
                        self.[<extend_ $name>](other.into_iter().cloned());
                    }
                }
                )+
            }
        }

        impl<T> MetadataCollectionBuilder<T> {
            pub fn new() -> Self {
                Self {
                    dictionary_name: None,
                    $($name: None,)+
                }
            }

            pub fn with_name(dictionary_name: Option<&'static str>) -> Self {
                Self {
                    dictionary_name: Some(dictionary_name),
                    $($name: None,)+
                }
            }

            /// Needs to be set!
            pub fn dictionary_name(&mut self, dictionary_name: Option<&'static str>) {
                 self.dictionary_name = Some(dictionary_name);
            }

            /// Clean up the builder for reuse
            pub fn clear(&mut self) {
                self.dictionary_name = None;
                $(if let Some($name) = &mut self.$name {
                    *$name = None;
                }
                )+
            }

            /// Returns true if the builder is clear
            pub fn is_clear(&self) -> bool {
                self.dictionary_name.is_none()
                $(&& self.$name.as_ref().is_none_or(|value| value.as_ref().is_none_or(Vec::is_empty)))+
            }



            pub fn build_and_clear(&mut self) -> Result<MetadataCollection<T>, MetadataCollectionBuilderError> {
                Ok(MetadataCollection {
                    dictionary_name: match(self.dictionary_name.take()){
                        Some(value) => value,
                        None => return Err(MetadataCollectionBuilderError::UninitializedField("dictionary_name"))
                    },
                    $($name: self.$name.take().flatten(),)+
                })
            }

            pub fn build_consuming(self) -> Result<MetadataCollection<T>, MetadataCollectionBuilderError> {
                Ok(MetadataCollection{
                    dictionary_name: match(self.dictionary_name){
                        Some(value) => value,
                        None => return Err(MetadataCollectionBuilderError::UninitializedField("dictionary_name"))
                    },
                    $($name: self.$name.flatten(),)+
                })
            }


            $(crate::topicmodel::dictionary::metadata::ex::collector::generate_insert_builder!($tt: $name: $collect_type | T);
            )+
        }
    };
}

pub(super) use create_collector_implementation;

use super::*;

impl<T> MetadataCollectionBuilder<T> {
    pub fn push_any_word_info(&mut self, word_info: AnyWordInfo) {
        match word_info {
            AnyWordInfo::Language(v) => {
                self.push_languages(v)
            }
            AnyWordInfo::Domain(v) => {
                self.push_domains(v)
            }
            AnyWordInfo::Gender(v) => {
                self.push_genders(v)
            }
            AnyWordInfo::Number(v) => {
                self.push_numbers(v)
            }
            AnyWordInfo::POS(v) => {
                self.push_pos(v)
            }
            AnyWordInfo::POSTag(v) => {
                self.push_pos_tag(v)
            }
            AnyWordInfo::Region(v) => {
                self.push_regions(v)
            }
            AnyWordInfo::Register(v) => {
                self.push_registers(v)
            }
        }
    }
    pub fn extend_any_word_info<I: IntoIterator<Item=AnyWordInfo>>(&mut self, word_info: I) {
        for value in word_info {
            self.push_any_word_info(value)
        }
    }
}