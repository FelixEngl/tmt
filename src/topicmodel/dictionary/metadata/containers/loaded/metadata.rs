

macro_rules! impl_collect_all {
    ($($ident:ident: $ty:ty),+) => {
        impl LoadedMetadata {
            $(
                paste::paste! {
                     pub fn [<collect_all_ $ident>](&self) -> std::collections::HashMap<Option<$crate::toolkit::typesafe_interner::DictionaryOriginSymbol>, tinyset::Set64<$ty>> {
                        self.iter().map(|value| {
                            (value.origin(), value.meta().$ident.clone())
                        }).collect()
                    }
                }
            )+
        }
    };
}
pub(super) use impl_collect_all;




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
        #[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, Eq, PartialEq)]
        pub struct AssociatedMetadata {
            $(
                $(#[doc=$doc])?
                #[serde(skip_serializing_if = "tinyset::Set64::is_empty", default)]
                $name: tinyset::Set64<$typ>,
            )+
        }

        impl AssociatedMetadata {
            pub fn update_with(&mut self, other: &AssociatedMetadata) {
                $(
                    self.$name.extend(other.$name.iter());
                )+
            }

            $crate::topicmodel::dictionary::metadata::loaded::metadata::impl_associated_metadata!{__is_empty $($name)+}

            $(
            fn $name(&self) -> &tinyset::Set64<$typ> {
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

        $crate::topicmodel::dictionary::metadata::loaded::metadata::impl_collect_all! {
            $($name: $typ),+
        }
    };
}


/*
todo:
- context: Feld für Context-Informationen
- Feld für original Eintrag
- region feld [auch brit-e und us-e]
- id-feld für generische ids.
 */

pub(super) use impl_associated_metadata;

macro_rules! create_metadata_impl {
    ($($tt:tt)+) => {
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Eq, PartialEq)]
        pub struct LoadedMetadata {
            #[serde(skip_serializing_if = "LazyAssociatedMetadata::is_not_init", default)]
            general_metadata: LazyAssociatedMetadata,
            #[serde(skip_serializing_if = "Vec::is_empty", default = "empty_vec")]
            associated_metadata: Vec<LazyAssociatedMetadata>,
        }

        fn empty_vec() -> Vec<LazyAssociatedMetadata> {
            Vec::with_capacity(0)
        }


        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Eq)]
        #[repr(transparent)]
        #[serde(transparent)]
        struct LazyAssociatedMetadata {
            #[serde(with = "crate::toolkit::once_serializer::OnceCellDef")]
            inner: std::cell::OnceCell<AssociatedMetadata>
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

        #[derive(Copy, Clone)]
        pub enum MetadataWithOrigin<T> {
            General(T),
            Associated($crate::toolkit::typesafe_interner::DictionaryOriginSymbol, T)
        }

        impl<T> MetadataWithOrigin<T> where T: Copy {
            pub fn origin(&self) -> Option<$crate::toolkit::typesafe_interner::DictionaryOriginSymbol> {
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
                                $crate::toolkit::typesafe_interner::DictionaryOriginSymbol::try_from_usize(idx).unwrap(),
                                meta
                            ))
                        }
                    }
                }
                self.pos = self.src.associated_metadata.len();
                None
            }
        }

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
                                $crate::toolkit::typesafe_interner::DictionaryOriginSymbol::try_from_usize(idx).unwrap(),
                                unsafe{std::mem::transmute(meta)}
                            ))
                        }
                    }
                }
                self.pos = assoc.len();
                None
            }
        }

        impl LoadedMetadata {
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

            pub fn get_associated_metadata(&self, origin: $crate::toolkit::typesafe_interner::DictionaryOriginSymbol) -> Option<&AssociatedMetadata> {
                use string_interner::Symbol;
                self.associated_metadata.get(origin.to_usize())?.get()
            }

            pub fn get_mut_associated_metadata(&mut self, origin: $crate::toolkit::typesafe_interner::DictionaryOriginSymbol) -> Option<&mut AssociatedMetadata> {
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

            pub fn get_or_create(&mut self, origin: $crate::toolkit::typesafe_interner::DictionaryOriginSymbol) -> &mut AssociatedMetadata {
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

        impl $crate::topicmodel::dictionary::metadata::Metadata for LoadedMetadata{}

        $crate::topicmodel::dictionary::metadata::loaded::metadata::impl_associated_metadata!($($tt)+);
    };
}

pub(super) use create_metadata_impl;