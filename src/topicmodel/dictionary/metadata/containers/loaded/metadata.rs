

macro_rules! impl_associated_metadata {

    (__is_empty $name: ident $($name2: ident)*) => {
        pub fn is_empty(&self) -> bool {
            self.$name.is_empty()
            $(
                && self.$name2.is_empty()
            )*
        }
    };

    ($($($doc: literal)? $name: ident: $typ: ty),+ $(,)?) => {


        paste::paste! {
            $crate::topicmodel::dictionary::metadata::loaded::metadata::impl_general_metadata!(
                $($name, [<$name:camel>], $typ;)+
            );
        }

        
        impl AssociatedMetadata {
            $(
            /// Get the values of the field
            #[inline(always)]
            pub fn $name(&self) -> Option<&tinyset::Set64<$typ>> {
                Some(&self.inner.get()?.$name)
            }

            paste::paste! {
                /// Adds a single value to the specified field
                #[inline(always)]
                pub fn [<add_single_to_ $name>](&mut self, value: $typ) {
                    self.get_mut_or_init().$name.insert(value);
                }

                /// Adds all values to the specified field
                #[inline(always)]
                pub fn [<add_all_to_ $name>]<I: IntoIterator<Item=$typ>>(&mut self, values: I) {
                    self.get_mut_or_init().$name.extend(values);
                }
            }
            )+
        }

        #[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, Eq, PartialEq)]
        pub struct AssociatedMetadataImpl {
            /// Allows to store various associated words based on type.
            #[serde(skip_serializing_if = "enum_map::EnumMap::is_empty", default)]
            pub(super) associated_words: $crate::topicmodel::dictionary::metadata::loaded::WordAssociationMap,
            $(
                $(#[doc=$doc])?
                #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
                $name: tinyset::Set64<$typ>,
            )+
        }

        impl AssociatedMetadataImpl {
            pub fn update_with(&mut self, other: &AssociatedMetadataImpl) {
                $(
                    self.$name.extend(other.$name.iter());
                )+
            }

            $crate::topicmodel::dictionary::metadata::loaded::metadata::impl_associated_metadata!{__is_empty $($name)+}

            $(
            pub fn $name(&self) -> &tinyset::Set64<$typ> {
                &self.$name
            }

            paste::paste! {
                pub fn [<add_single_to_ $name>](&mut self, value: $typ) {
                    self.$name.insert(value);
                }
                pub fn [<add_all_to_ $name>]<I: IntoIterator<Item=$typ>>(&mut self, values: I) {
                    self.$name.extend(values);
                }
            }
            )+

        }
        
        impl LoadedMetadata {
            $(
                paste::paste! {
                    /// Get all values of a specific field.
                    pub fn [<all_raw_ $name>](&self) -> (Option<tinyset::Set64<$typ>>, Vec<Option<tinyset::Set64<$typ>>>) {
                         let a = self.general_metadata.get().map(|value| value.$name().cloned()).flatten();
                         let b = self.associated_metadata.iter().map(|value| value.get().map(|value| value.$name().cloned()).flatten()).collect();
                         (a, b)
                    }
                }
            )+

        }
    };
}

use std::ops::IndexMut;
pub(super) use impl_associated_metadata;

macro_rules! impl_general_metadata {
    ($($normal_name:ident, $enum_var_name: ident, $typ: ty);+ $(;)?) => {

        #[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
        pub enum GeneralMetadataEntry<'a> {
            WordAssociation(&'a $crate::topicmodel::dictionary::metadata::loaded::WordAssociationMap),
            $($enum_var_name(&'a tinyset::Set64<$typ>),
            )+
        }

        #[derive(Clone, Hash, Eq, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
        pub enum GeneralMetadata {
            WordAssociation(Option<$crate::topicmodel::dictionary::metadata::loaded::WordAssociationMap>, Vec<Option<$crate::topicmodel::dictionary::metadata::loaded::WordAssociationMap>>),
            $($enum_var_name(Option<tinyset::Set64<$typ>>, Vec<Option<tinyset::Set64<$typ>>>),
            )+
        }

        impl GeneralMetadata {

            $(
            fn $normal_name(default: Option<tinyset::Set64<$typ>>, dicts: Vec<Option<tinyset::Set64<$typ>>>) -> GeneralMetadata {
                GeneralMetadata::$enum_var_name(default, dicts)
            }
            )+

            pub fn get_default(&self) -> Option<GeneralMetadataEntry> {
                match self {
                    GeneralMetadata::WordAssociation(Some(default), _) => {
                        Some(GeneralMetadataEntry::WordAssociation(default))
                    }
                    $(
                    GeneralMetadata::$enum_var_name(Some(default), _) => {
                        Some(GeneralMetadataEntry::$enum_var_name(default))
                    }
                    )+
                    _ => None
                }
            }

            pub fn get_dicts(&self) -> Vec<Option<GeneralMetadataEntry>> {
                match self {
                    GeneralMetadata::WordAssociation(_, value) => {
                        value
                        .iter()
                        .map(|value| value.as_ref().map(GeneralMetadataEntry::WordAssociation))
                        .collect()
                    }
                    $(
                    GeneralMetadata::$enum_var_name(_, value) => {
                        value
                        .iter()
                        .map(|value| value.as_ref().map(GeneralMetadataEntry::$enum_var_name))
                        .collect()
                    }
                    )+
                }
            }

            pub fn get_for_dict<S: string_interner::Symbol>(&self, idx: S) -> Option<GeneralMetadataEntry> {
                match self {
                    GeneralMetadata::WordAssociation(_, value) => {
                        value
                        .get(idx.to_usize())?
                        .as_ref()
                        .map(GeneralMetadataEntry::WordAssociation)
                    }
                    $(
                    GeneralMetadata::$enum_var_name(_, value) => {
                        value
                        .get(idx.to_usize())?
                        .as_ref()
                        .map(GeneralMetadataEntry::$enum_var_name)
                    }
                    )+
                }
            }
        }

        impl LoadedMetadata {
            pub fn all_fields(&self) -> enum_map::EnumMap<MetaField, GeneralMetadata> {
                enum_map::enum_map! {
                    MetaField::WordAssociation => GeneralMetadata::WordAssociation(
                        self.general_metadata.get().map(|value| value.associated_words().cloned()).flatten(),
                        self.associated_metadata.iter().map(|value| value.get().map(|value| value.associated_words().cloned()).flatten()).collect()
                    ),
                    $(
                        MetaField::$enum_var_name => GeneralMetadata::$normal_name(
                            self.general_metadata.get().map(|value| value.$normal_name().cloned()).flatten(),
                            self.associated_metadata.iter().map(|value| value.get().map(|value| value.$normal_name().cloned()).flatten()).collect()
                        ),
                    )+
                }
            }

            pub fn field(&self, field: MetaField) -> GeneralMetadata {
                match field {
                    MetaField::WordAssociation => GeneralMetadata::WordAssociation(
                        self.general_metadata.get().map(|value| value.associated_words().cloned()).flatten(),
                        self.associated_metadata.iter().map(|value| value.get().map(|value| value.associated_words().cloned()).flatten()).collect()
                    ),
                    $(
                        MetaField::$enum_var_name => GeneralMetadata::$normal_name(
                            self.general_metadata.get().map(|value| value.$normal_name().cloned()).flatten(),
                            self.associated_metadata.iter().map(|value| value.get().map(|value| value.$normal_name().cloned()).flatten()).collect()
                        ),
                    )+
                }
            }

            /// set dict to None for default
            pub fn single_field_value<S: string_interner::Symbol>(&self, field: MetaField, dict: Option<S>) -> Option<GeneralMetadataEntry> {
                match field {
                    MetaField::WordAssociation => {
                        if let Some(dict) = dict {
                            self
                            .associated_metadata
                            .get(dict.to_usize())?
                            .get()
                            .map(
                                |value|
                                value
                                .associated_words()
                                .map(GeneralMetadataEntry::WordAssociation)
                            )
                            .flatten()
                        } else {
                            self.general_metadata.get()?.associated_words().map(GeneralMetadataEntry::WordAssociation)
                        }
                    }
                    $(
                        MetaField::$enum_var_name => {
                            if let Some(dict) = dict {
                                self
                                .associated_metadata
                                .get(dict.to_usize())?
                                .get()
                                .map(
                                    |value|
                                    value
                                    .$normal_name()
                                    .map(GeneralMetadataEntry::$enum_var_name)
                                )
                                .flatten()
                            } else {
                                self.general_metadata.get()?.$normal_name().map(GeneralMetadataEntry::$enum_var_name)
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
        #[derive(Copy, Clone)]
        pub enum MetadataWithOrigin<T> {
            General(T),
            Associated($crate::toolkit::typesafe_interner::DictionaryOriginSymbol, T)
        }

        $crate::topicmodel::dictionary::metadata::loaded::metadata::impl_associated_metadata!($($tt)+);
    };
}

pub(super) use create_metadata_impl;


use super::*;

/// The metadata for an entry
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Eq, PartialEq)]
pub struct LoadedMetadata {
    #[serde(skip_serializing_if = "LazyAssociatedMetadata::is_not_init", default)]
    pub(super) general_metadata: LazyAssociatedMetadata,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "empty_vec")]
    pub(super) associated_metadata: Vec<LazyAssociatedMetadata>,
}

fn empty_vec() -> Vec<LazyAssociatedMetadata> {
    Vec::with_capacity(0)
}

impl LoadedMetadata {
    pub fn all_raw_associated_words(&self) -> (Option<WordAssociationMap>, Vec<Option<WordAssociationMap>>) {
        let a = self.general_metadata.get().map(|value| value.associated_words().cloned()).flatten();
        let b = self.associated_metadata.iter().map(|value| value.get().map(|value| value.associated_words().cloned()).flatten()).collect();
        (a, b)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            general_metadata: LazyAssociatedMetadata::new(),
            associated_metadata: Vec::with_capacity(capacity),
        }
    }

    pub fn get_general_metadata(&self) -> &AssociatedMetadata {
        self.general_metadata.get_or_init()
    }

    pub fn get_mut_general_metadata(&mut self) -> &mut AssociatedMetadata {
        self.general_metadata.get_mut_or_init()
    }

    pub fn get_associated_metadata(&self, origin: crate::toolkit::typesafe_interner::DictionaryOriginSymbol) -> Option<&AssociatedMetadata> {
        use string_interner::Symbol;
        self.associated_metadata.get(origin.to_usize())?.get()
    }

    pub fn get_mut_associated_metadata(&mut self, origin: crate::toolkit::typesafe_interner::DictionaryOriginSymbol) -> Option<&mut AssociatedMetadata> {
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

    pub fn get_or_create(&mut self, origin: crate::toolkit::typesafe_interner::DictionaryOriginSymbol) -> &mut AssociatedMetadata {
        use string_interner::Symbol;
        self.get_or_create_impl(origin.to_usize())
    }

    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut::new(self)
    }


    pub fn update_with(&mut self, other: &LoadedMetadata) {
        if let Some(targ) = other.general_metadata.get() {
            self.general_metadata.get_mut_or_init().update_with(targ);
        }
        for (origin, value) in other.associated_metadata.iter().enumerate() {
            if let Some(value) = value.get() {
                self.get_or_create_impl(origin).update_with(value)
            }
        }
    }
}

impl crate::topicmodel::dictionary::metadata::Metadata for LoadedMetadata{}


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


/// An iterator for LoadedMetadata
pub struct Iter<'a> {
    src: &'a LoadedMetadata,
    general_metadata: bool,
    pos: usize
}

impl<'a> Iter<'a> {
    pub fn new(src: &'a LoadedMetadata) -> Self {
        Self { src, general_metadata: false, pos: 0 }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = MetadataWithOrigin<&'a AssociatedMetadata>;

    fn next(&mut self) -> Option<Self::Item> {
        use string_interner::Symbol;

        if self.general_metadata && self.pos == self.src.associated_metadata.len() {
            return None
        }
        if !self.general_metadata {
            self.general_metadata = true;
            if let Some(meta) = self.src.general_metadata.get() {
                return Some(MetadataWithOrigin::General(meta))
            }
        }
        for idx in self.pos..self.src.associated_metadata.len() {
            if let Some(targ) = self.src.associated_metadata.get(idx) {
                if let Some(meta) = targ.get() {
                    self.pos = idx;
                    return Some(MetadataWithOrigin::Associated(
                        crate::toolkit::typesafe_interner::DictionaryOriginSymbol::try_from_usize(idx).unwrap(),
                        meta
                    ))
                }
            }
        }
        self.pos = self.src.associated_metadata.len();
        None
    }
}


/// A mutable iterator over LoadedMetadata
pub struct IterMut<'a> {
    src: &'a mut LoadedMetadata,
    general_metadata: bool,
    pos: usize
}

impl<'a> IterMut<'a> {
    pub fn new(src: &'a mut LoadedMetadata) -> Self {
        Self { src, general_metadata: false, pos: 0 }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = MetadataWithOrigin<&'a mut AssociatedMetadata>;

    fn next(&mut self) -> Option<Self::Item> {
        use string_interner::Symbol;
        if self.general_metadata && self.pos == self.src.associated_metadata.len() {
            return None
        }
        if !self.general_metadata {
            self.general_metadata = true;
            if let Some(meta) = self.src.general_metadata.get_mut() {
                return Some(MetadataWithOrigin::General(unsafe{std::mem::transmute(meta)}))
            }
        }
        let assoc: &'static mut Vec<LazyAssociatedMetadata> = unsafe{std::mem::transmute(&mut self.src.associated_metadata)};
        for idx in self.pos..assoc.len() {
            if let Some(targ) = assoc.get_mut(idx) {
                if let Some(meta) = targ.get_mut() {
                    self.pos = idx;
                    return Some(MetadataWithOrigin::Associated(
                        crate::toolkit::typesafe_interner::DictionaryOriginSymbol::try_from_usize(idx).unwrap(),
                        unsafe{std::mem::transmute(meta)}
                    ))
                }
            }
        }
        self.pos = assoc.len();
        None
    }
}


/// A lazy loading structure for associated metadata.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Eq)]
#[repr(transparent)]
#[serde(transparent)]
pub(super) struct LazyAssociatedMetadata {
    #[serde(with = "crate::toolkit::once_serializer::OnceCellDef")]
    pub(super) inner: std::cell::OnceCell<AssociatedMetadata>
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
}

impl PartialEq for LazyAssociatedMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.inner.get() == other.inner.get()
    }
}

impl Default for LoadedMetadata {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}


/// The associated metadata
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, Eq)]
#[repr(transparent)]
#[serde(transparent)]
pub struct AssociatedMetadata {
    #[serde(with = "crate::toolkit::once_serializer::OnceCellDef")]
    pub(super) inner: std::cell::OnceCell<AssociatedMetadataImpl>
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
        self.inner.get().is_none_or(|value| value.is_empty())
    }

    #[inline(always)]
    pub fn update_with(&mut self, other: &AssociatedMetadata) {
        if let Some(targ) = self.inner.get_mut() {
            if let Some(other) = other.inner.get() {
                targ.update_with(other)
            }
        }
    }

    pub fn associated_words(&self) -> Option<&WordAssociationMap> {
        Some(&self.inner.get()?.associated_words)
    }

    pub fn add_single_to_associated_words(&mut self, association: WordAssociation, word_id: usize) {
        self.get_mut_or_init().associated_words.index_mut(association).insert(word_id);
    }

    pub fn add_all_to_associated_words<I: IntoIterator<Item=(WordAssociation, usize)>>(&mut self, values: I) {
        self.get_mut_or_init().add_all_to_associated_words(values);
    }
}


impl AssociatedMetadataImpl {
    pub fn add_single_to_associated_words(&mut self, association: WordAssociation, word_id: usize) {
        self.associated_words.index_mut(association).insert(word_id);
    }

    pub fn add_all_to_associated_words<I: IntoIterator<Item=(WordAssociation, usize)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.add_single_to_associated_words(k, v)
        }
    }

    pub fn associated_words(&self) -> &WordAssociationMap {
        &self.associated_words
    }
}